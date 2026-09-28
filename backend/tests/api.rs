//! API integration tests. Each test gets a fresh database from `#[sqlx::test]`
//! (requires DATABASE_URL pointing at a server where the user may create databases).

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use chrono::{DateTime, Utc};
use codeacademy_api::{config::Config, router, seed, AppState, Clock};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;

fn clock() -> Clock {
    let at: DateTime<Utc> = "2026-09-28T09:00:00Z".parse().unwrap();
    Clock::fixed(chrono_tz::Asia::Tehran, at)
}

async fn app(pool: PgPool) -> Router {
    assert!(seed::sync(&pool).await.unwrap());
    let config = Config {
        database_url: String::new(),
        bind_addr: String::new(),
        cors_origins: vec!["http://localhost:5173".into()],
        cookie_secure: false,
        timezone: chrono_tz::Asia::Tehran,
    };
    router(AppState { pool, config: Arc::new(config), clock: clock() })
}

struct Client {
    app: Router,
    cookie: Option<String>,
}

impl Client {
    fn anonymous(app: &Router) -> Self {
        Self { app: app.clone(), cookie: None }
    }

    /// Registers a fresh student and keeps its session cookie.
    async fn student(app: &Router, email: &str) -> Self {
        let mut c = Self::anonymous(app);
        let (status, body) = c.post("/api/v1/auth/register", json!({ "name": "سارا محمدی", "email": email, "password": "tehran1405" })).await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        assert!(c.cookie.is_some());
        c
    }

    async fn call(&mut self, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
        let mut req = Request::builder().method(method).uri(uri);
        if let Some(cookie) = &self.cookie {
            req = req.header(header::COOKIE, cookie);
        }
        let body = match body {
            Some(v) => {
                req = req.header(header::CONTENT_TYPE, "application/json");
                Body::from(v.to_string())
            }
            None => Body::empty(),
        };
        let res = self.app.clone().oneshot(req.body(body).unwrap()).await.unwrap();
        if let Some(set) = res.headers().get(header::SET_COOKIE) {
            let pair = set.to_str().unwrap().split(';').next().unwrap().to_string();
            self.cookie = if pair.ends_with('=') { None } else { Some(pair) };
        }
        let status = res.status();
        let bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json = if bytes.is_empty() { Value::Null } else { serde_json::from_slice(&bytes).unwrap() };
        (status, json)
    }

    async fn get(&mut self, uri: &str) -> Value {
        let (status, body) = self.call("GET", uri, None).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
        body
    }

    async fn post(&mut self, uri: &str, body: Value) -> (StatusCode, Value) {
        self.call("POST", uri, Some(body)).await
    }

    /// Completes every remaining lesson of a course with the given score.
    async fn finish_course(&mut self, slug: &str, score: f64) -> Value {
        let course = self.get(&format!("/api/v1/courses/{slug}")).await;
        let mut last = Value::Null;
        for lesson in course["lessons"].as_array().unwrap() {
            if !lesson["completed"].as_bool().unwrap() {
                let (status, body) = self.post(&format!("/api/v1/lessons/{}/complete", lesson["id"]), json!({ "selfScore": score })).await;
                assert_eq!(status, StatusCode::OK, "{slug}: {body}");
                last = body;
            }
        }
        last
    }

    async fn rescore_course(&mut self, slug: &str, score: f64) -> Value {
        let course = self.get(&format!("/api/v1/courses/{slug}")).await;
        let mut last = Value::Null;
        for lesson in course["lessons"].as_array().unwrap() {
            let (status, body) = self.post(&format!("/api/v1/lessons/{}/complete", lesson["id"]), json!({ "selfScore": score })).await;
            assert_eq!(status, StatusCode::OK, "{body}");
            last = body;
        }
        last
    }
}

const YEAR_ONE: [&str; 6] = ["se-101", "dart-101", "pro-100", "se-110", "flutter-101", "dart-190"];

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn content_sync_matches_curriculum_and_is_idempotent(pool: PgPool) {
    let app = app(pool.clone()).await;
    assert!(!seed::sync(&pool).await.unwrap(), "second sync is a no-op");
    let mut anon = Client::anonymous(&app);
    let overview = anon.get("/api/v1/overview").await;
    assert_eq!(overview["stats"]["years"], 6);
    assert_eq!(overview["stats"]["departments"], 4);
    assert_eq!(overview["stats"]["credits"], 99);
    assert_eq!(overview["stats"]["students"], 0, "no sample students are seeded");
    assert!(!overview["highlights"].as_array().unwrap().is_empty());
    assert!(anon.get("/api/v1/dictionary").await.as_array().unwrap().len() >= 12);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn protected_endpoints_require_a_session(pool: PgPool) {
    let app = app(pool).await;
    let mut anon = Client::anonymous(&app);
    for uri in ["/api/v1/me", "/api/v1/dashboard", "/api/v1/transcript", "/api/v1/passport", "/api/v1/courses/se-101", "/api/v1/lessons/1"] {
        let (status, body) = anon.call("GET", uri, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{uri}");
        assert_eq!(body["error"]["code"], "unauthorized");
    }
    let (status, _) = anon.post("/api/v1/check-in", json!({ "minutes": 10 })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    anon.cookie = Some(format!("ca_session={}", "a".repeat(64)));
    let (status, _) = anon.call("GET", "/api/v1/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "forged token is rejected");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn register_login_logout(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "Sara@Example.ir").await;
    let me = s.get("/api/v1/me").await;
    assert_eq!(me["email"], "sara@example.ir");
    assert_eq!(me["studentId"], "CA-1405-00001");
    assert_eq!(me["year"], 1);

    let (status, _) = s.post("/api/v1/auth/logout", json!({})).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(s.cookie.is_none());

    let mut other = Client::anonymous(&app);
    let (status, body) = other.post("/api/v1/auth/login", json!({ "email": "sara@example.ir", "password": "wrong-pass1" })).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"]["code"], "invalid_credentials");
    let (status, body) = other.post("/api/v1/auth/login", json!({ "email": " SARA@example.ir", "password": "tehran1405" })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(other.get("/api/v1/me").await["name"], "سارا محمدی");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn registration_is_validated(pool: PgPool) {
    let app = app(pool).await;
    Client::student(&app, "ali@example.ir").await;
    let mut anon = Client::anonymous(&app);
    let cases = [
        (json!({ "name": "علی", "email": "ali@example.ir", "password": "tehran1405" }), StatusCode::CONFLICT),
        (json!({ "name": "علی", "email": "not-an-email", "password": "tehran1405" }), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({ "name": "علی", "email": "a@b.ir", "password": "short1" }), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({ "name": "علی", "email": "a@b.ir", "password": "noDigitsHere" }), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({ "name": "ع", "email": "a@b.ir", "password": "tehran1405" }), StatusCode::UNPROCESSABLE_ENTITY),
        (json!({ "email": "a@b.ir" }), StatusCode::UNPROCESSABLE_ENTITY),
    ];
    for (body, expected) in cases {
        let (status, res) = anon.post("/api/v1/auth/register", body.clone()).await;
        assert_eq!(status, expected, "{body} → {res}");
        assert!(anon.cookie.is_none());
    }
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn repeated_failed_logins_are_throttled(pool: PgPool) {
    let app = app(pool).await;
    Client::student(&app, "reza@example.ir").await;
    let mut anon = Client::anonymous(&app);
    for _ in 0..5 {
        let (status, _) = anon.post("/api/v1/auth/login", json!({ "email": "reza@example.ir", "password": "guess12345" })).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let (status, body) = anon.post("/api/v1/auth/login", json!({ "email": "reza@example.ir", "password": "tehran1405" })).await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS, "even the right password is blocked during lockout");
    assert_eq!(body["error"]["code"], "too_many_requests");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn password_change_signs_out_other_devices(pool: PgPool) {
    let app = app(pool).await;
    let mut laptop = Client::student(&app, "maryam@example.ir").await;
    let mut phone = Client::anonymous(&app);
    phone.post("/api/v1/auth/login", json!({ "email": "maryam@example.ir", "password": "tehran1405" })).await;

    let (status, _) = laptop.post("/api/v1/me/password", json!({ "currentPassword": "wrong1234", "newPassword": "isfahan1405" })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = laptop.post("/api/v1/me/password", json!({ "currentPassword": "tehran1405", "newPassword": "isfahan1405" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    laptop.get("/api/v1/me").await;
    let (status, _) = phone.call("GET", "/api/v1/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, me) = laptop.call("PUT", "/api/v1/me", Some(json!({ "name": "  مریم   احمدی " }))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["name"], "مریم احمدی");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn new_student_starts_at_year_one(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "new@example.ir").await;
    let d = s.get("/api/v1/dashboard").await;
    assert_eq!(d["student"]["year"], 1);
    assert_eq!(d["student"]["currentTerm"], 1);
    assert_eq!(d["student"]["totalXp"], 0);
    assert_eq!(d["student"]["streakDays"], 0);
    assert_eq!(d["progress"]["creditsPassed"], 0);
    assert_eq!(d["advisor"]["advice"]["tone"], "start");
    assert!(d["recentActivity"].as_array().unwrap().is_empty());
    assert!(d["currentCourse"]["slug"].is_string());
    // First-term courses without prerequisites are open; the rest are locked.
    for slug in ["se-101", "dart-101", "pro-100"] {
        assert_eq!(s.get(&format!("/api/v1/courses/{slug}")).await["status"], "enrolled", "{slug}");
    }
    for slug in ["se-110", "flutter-101", "dart-201"] {
        assert_eq!(s.get(&format!("/api/v1/courses/{slug}")).await["status"], "locked", "{slug}");
    }
    let board = s.get("/api/v1/leaderboard").await;
    assert_eq!(board["of"], 1, "only real students are ranked");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn completing_a_lesson_awards_xp_once_and_starts_streak(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "xp@example.ir").await;
    let id = s.get("/api/v1/courses/se-101").await["nextLessonId"].as_i64().unwrap();

    let (status, first) = s.post(&format!("/api/v1/lessons/{id}/complete"), json!({ "selfScore": 17 })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["firstCompletion"], true);
    assert!(first["xpGained"].as_i64().unwrap() >= 25);
    assert_eq!(first["streakDays"], 1);
    assert!(first["newAchievements"].as_array().unwrap().iter().any(|a| a == "اولین قدم"));

    let (_, again) = s.post(&format!("/api/v1/lessons/{id}/complete"), json!({ "selfScore": 20 })).await;
    assert_eq!(again["firstCompletion"], false);
    assert_eq!(again["xpGained"], 0);
    assert_eq!(s.get("/api/v1/me").await["totalXp"], first["totalXp"]);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn passing_a_course_unlocks_dependents_but_failing_does_not(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "unlock@example.ir").await;

    let result = s.finish_course("se-101", 10.0).await;
    assert_eq!(result["courseStatus"], "done");
    assert_eq!(result["courseGrade"], 10.0);
    assert!(result["unlockedCourses"].as_array().unwrap().is_empty());
    assert_eq!(s.get("/api/v1/courses/se-110").await["status"], "locked", "failed prerequisite keeps it locked");
    let course = s.get("/api/v1/courses/se-101").await;
    assert_eq!(course["needsRetake"], true);
    assert!(course["nextLessonId"].is_number(), "a failed course points to a lesson to redo");

    let result = s.rescore_course("se-101", 17.0).await;
    assert_eq!(result["courseGrade"], 17.0);
    // The dependent unlocks as soon as the average reaches 14.
    assert_eq!(s.get("/api/v1/courses/se-110").await["status"], "enrolled");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn passing_year_one_promotes_to_junior_plus(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "promo@example.ir").await;
    let mut last = Value::Null;
    for slug in YEAR_ONE {
        last = s.finish_course(slug, 20.0).await;
    }
    assert_eq!(last["promotedTo"], "جونیور پلاس");
    let me = s.get("/api/v1/me").await;
    assert_eq!(me["year"], 2);
    assert_eq!(me["currentTerm"], 3);
    assert_eq!(s.get("/api/v1/courses/dart-201").await["status"], "enrolled");

    let t = s.get("/api/v1/transcript").await;
    assert_eq!(t["creditsPassed"], 15);
    assert_eq!(t["creditsRemaining"], 84);
    assert_eq!(t["cumulativeGpa"], 20.0);
    let p = s.get("/api/v1/passport").await;
    assert_eq!(p["stamps"][0]["passed"], true);
    assert_eq!(p["stamps"][1]["passed"], false);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn transcript_gpa_is_credit_weighted(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "gpa@example.ir").await;
    s.finish_course("se-101", 20.0).await; // 3 credits
    s.finish_course("pro-100", 14.0).await; // 2 credits
    let t = s.get("/api/v1/transcript").await;
    // (20×3 + 14×2) / 5 = 17.6 (a plain mean would be 17)
    assert_eq!(t["cumulativeGpa"], 17.6);
    assert_eq!(t["semesters"][0]["gpa"], 17.6);
    assert_eq!(t["creditsPassed"], 5);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn rejects_invalid_input(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "input@example.ir").await;
    let (status, body) = s.post("/api/v1/check-in", json!({ "minutes": 45 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"]["code"], "validation_error");
    let id = s.get("/api/v1/courses/se-101").await["nextLessonId"].as_i64().unwrap();
    let (status, _) = s.post(&format!("/api/v1/lessons/{id}/complete"), json!({ "selfScore": 19 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = s.post("/api/v1/activity", json!({ "minutes": 999 })).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = s.call("GET", "/api/v1/lessons/not-a-number", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = s.call("GET", &format!("/api/v1/dictionary?q={}", "a".repeat(80)), None).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn locked_lessons_cannot_be_completed(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "locked@example.ir").await;
    let id = s.get("/api/v1/courses/se-510").await["lessons"][0]["id"].as_i64().unwrap();
    let (status, body) = s.post(&format!("/api/v1/lessons/{id}/complete"), json!({ "selfScore": 20 })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "conflict");
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn check_in_suggests_by_available_time(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "checkin@example.ir").await;
    let (_, thirty) = s.post("/api/v1/check-in", json!({ "minutes": 30 })).await;
    assert_eq!(thirty["suggestion"]["kind"], "study_lesson");
    assert!(thirty["suggestion"]["lessonId"].is_number());
    assert!(!thirty["suggestion"]["title"].as_str().unwrap().is_empty());

    let id = thirty["suggestion"]["lessonId"].as_i64().unwrap();
    s.post(&format!("/api/v1/lessons/{id}/complete"), json!({ "selfScore": 20 })).await;
    let (_, ten) = s.post("/api/v1/check-in", json!({ "minutes": 10 })).await;
    assert_eq!(ten["suggestion"]["kind"], "review_concept");
    let title = ten["suggestion"]["title"].as_str().unwrap();
    assert!(!title.contains('-'), "review suggestion shows the term name, not its slug: {title}");
    assert_eq!(s.get("/api/v1/dashboard").await["checkIn"]["minutes"], 10);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn study_session_completes_daily_mission(pool: PgPool) {
    let app = app(pool).await;
    let mut s = Client::student(&app, "mission@example.ir").await;
    let mission = s.get("/api/v1/missions/today").await;
    assert_eq!(mission["completed"], false);
    assert!(mission["description"].as_str().unwrap().contains(mission["courseTitle"].as_str().unwrap()));
    let (status, outcome) =
        s.post("/api/v1/activity", json!({ "minutes": mission["targetMinutes"], "courseSlug": mission["courseSlug"] })).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(outcome["missionCompletedNow"], true);
    assert_eq!(outcome["streakDays"], 1);
    let after = s.get("/api/v1/missions/today").await;
    assert_eq!(after["completed"], true);
    assert_eq!(s.get("/api/v1/me").await["totalXp"], 50);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn dictionary_search_matches_english_and_persian(pool: PgPool) {
    let app = app(pool).await;
    let mut anon = Client::anonymous(&app);
    let hits = anon.get("/api/v1/dictionary?q=stream").await;
    assert!(hits.as_array().unwrap().iter().any(|t| t["slug"] == "stream"));
    let hits = anon.get("/api/v1/dictionary?q=%25").await; // a literal %, not a wildcard
    assert!(hits.as_array().unwrap().is_empty());
    let (status, _) = anon.call("GET", "/api/v1/dictionary/does-not-exist", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrator = "codeacademy_api::MIGRATOR")]
async fn ranking_is_among_real_students_of_the_same_level(pool: PgPool) {
    let app = app(pool).await;
    let mut a = Client::student(&app, "a@example.ir").await;
    let mut b = Client::student(&app, "b@example.ir").await;
    a.finish_course("se-101", 20.0).await;
    let board = b.get("/api/v1/leaderboard").await;
    assert_eq!(board["of"], 2);
    assert_eq!(board["rank"], 2);
    let entries = board["entries"].as_array().unwrap();
    assert_eq!(entries.iter().filter(|e| e["isMe"] == true).count(), 1);
    assert_eq!(a.get("/api/v1/passport").await["rank"]["rank"], 1);
}
