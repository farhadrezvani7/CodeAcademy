//! Streak, daily mission, achievements and check-in: everything that reacts
//! to the student being active.

use super::snapshot::Snapshot;
use crate::domain::{
    achievements::{self, StudentFacts, NIGHT_HOURS},
    checkin::{self, NextLesson, ReviewTerm, Suggestion},
    mission::{self, FocusCourse},
    streak, xp, EnrollmentStatus,
};
use crate::error::{AppError, AppResult};
use crate::repo::{
    activity::{self, MissionRow, NewActivity},
    content, student,
};
use crate::state::{AppState, Clock};
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncOutcome {
    pub streak_days: i32,
    pub mission_completed_now: bool,
    pub new_achievements: Vec<String>,
    pub new_certificates: Vec<String>,
}

/// Brings derived state up to date: streak, today's mission, achievements and
/// certificates. Idempotent, so it is safe to run on reads as well as writes.
pub async fn sync(db: &mut PgConnection, clock: &Clock, user_id: i32) -> AppResult<SyncOutcome> {
    let today = clock.today();
    let now = clock.now();
    let mut out = SyncOutcome::default();

    let dates = activity::active_dates(&mut *db, user_id).await?;
    let current = streak::current_streak(&dates, today) as i32;
    let longest = streak::longest_streak(&dates) as i32;
    student::set_streak(&mut *db, user_id, current, longest).await?;
    out.streak_days = current;

    let snapshot = Snapshot::load(&mut *db, user_id).await?;
    let mission = ensure_mission(&mut *db, &snapshot, today).await?;
    let spent = activity::minutes_on(&mut *db, user_id, today, mission.course_id).await?;
    let progress = mission::progress(mission.target_minutes, spent);
    let completing = progress.completed && mission.completed_at.is_none();
    activity::update_mission_progress(&mut *db, mission.id, progress.minutes, completing.then_some(now)).await?;
    if completing {
        let award = xp::for_mission(mission.id);
        student::award_xp(&mut *db, user_id, award.reason, &award.reference, award.amount, now).await?;
        out.mission_completed_now = true;
    }

    let total_xp = student::user(&mut *db, user_id).await?.map(|u| u.total_xp).unwrap_or(0);
    let night = activity::has_activity_in_hours(
        &mut *db,
        user_id,
        clock.tz.name(),
        NIGHT_HOURS.start as i32,
        NIGHT_HOURS.end as i32,
    )
    .await?;
    let facts = facts(&snapshot, longest.max(snapshot.student.user.longest_streak) as u32, night, total_xp);

    for a in content::achievements(&mut *db, user_id).await? {
        if a.unlocked_at.is_none() && achievements::is_unlocked(&a.slug, &facts) {
            content::unlock_achievement(&mut *db, user_id, a.id, now).await?;
            out.new_achievements.push(a.title);
        }
    }
    for c in content::certificates(&mut *db, user_id).await? {
        if c.issued_at.is_none() && achievements::is_certified(&c.slug, &facts) {
            let serial = format!("CA-{}-{:05}-{}", c.slug.to_uppercase(), user_id, now.format("%Y%m%d"));
            content::issue_certificate(&mut *db, user_id, c.id, &serial, now).await?;
            out.new_certificates.push(c.title);
        }
    }
    Ok(out)
}

pub fn facts(snapshot: &Snapshot, longest_streak: u32, has_night_activity: bool, total_xp: i32) -> StudentFacts {
    // Only passed courses count as completed for achievements.
    let completed_courses: HashSet<String> = snapshot
        .catalog
        .courses
        .iter()
        .filter(|c| snapshot.is_passed(c.id))
        .map(|c| c.slug.clone())
        .collect();
    let mut lessons_completed: HashMap<String, usize> = HashMap::new();
    for completion in &snapshot.student.completions {
        if let Some(course) = snapshot.catalog.course(completion.course_id) {
            *lessons_completed.entry(course.slug.clone()).or_default() += 1;
        }
    }
    StudentFacts {
        completed_courses,
        lessons_completed,
        longest_streak,
        has_night_activity,
        total_xp,
        passed_year: snapshot.passed_year(),
        graduated: snapshot.student.is_graduated(),
    }
}

/// Today's mission, created from the student's focus course on first access.
pub async fn ensure_mission(db: &mut PgConnection, snapshot: &Snapshot, today: chrono::NaiveDate) -> AppResult<MissionRow> {
    let user_id = snapshot.student.user.id;
    if let Some(m) = activity::mission(&mut *db, user_id, today).await? {
        return Ok(m);
    }
    let focus = snapshot.focus_course().map(|c| FocusCourse { id: c.id, title: c.title.clone() });
    let plan = mission::plan(focus.as_ref(), today, &snapshot.catalog.templates);
    Ok(activity::insert_mission(&mut *db, user_id, today, &plan.description, plan.course_id, plan.target_minutes, plan.xp_reward).await?)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionDto {
    pub id: i32,
    pub date: chrono::NaiveDate,
    pub description: String,
    pub course_slug: Option<String>,
    pub course_title: Option<String>,
    pub target_minutes: i32,
    pub progress_minutes: i32,
    pub percent: i32,
    pub xp_reward: i32,
    pub completed: bool,
}

pub fn mission_dto(m: &MissionRow, snapshot: &Snapshot) -> MissionDto {
    let course = m.course_id.and_then(|id| snapshot.catalog.course(id));
    let p = mission::progress(m.target_minutes, m.progress_minutes);
    MissionDto {
        id: m.id,
        date: m.date,
        description: m.description.clone(),
        course_slug: course.map(|c| c.slug.clone()),
        course_title: course.map(|c| c.title.clone()),
        target_minutes: m.target_minutes,
        progress_minutes: p.minutes,
        percent: p.percent,
        xp_reward: m.xp_reward,
        completed: m.completed_at.is_some(),
    }
}

pub async fn today_mission(state: &AppState, user_id: i32) -> AppResult<MissionDto> {
    let mut tx = state.pool.begin().await?;
    sync(&mut tx, &state.clock, user_id).await?;
    let snapshot = Snapshot::load(&mut tx, user_id).await?;
    let mission = ensure_mission(&mut tx, &snapshot, state.clock.today()).await?;
    tx.commit().await?;
    Ok(mission_dto(&mission, &snapshot))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInRequest {
    pub minutes: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInResponse {
    pub minutes: i32,
    pub suggestion: Suggestion,
}

/// Suggestion for the given available time, from the student's actual progress.
pub fn suggestion_for(snapshot: &Snapshot, minutes: i32) -> Suggestion {
    let focus = snapshot.focus_course();
    let next = focus.and_then(|c| {
        snapshot.next_lesson(c.id).map(|l| NextLesson { id: l.id, title: l.title.clone(), course_title: c.title.clone() })
    });
    // Review a concept from the most recently completed lesson.
    let review = snapshot
        .student
        .completions
        .iter()
        .rev()
        .find_map(|c| snapshot.catalog.lessons.iter().find(|l| l.id == c.lesson_id))
        .and_then(|l| l.related_terms.iter().find(|slug| snapshot.catalog.term_names.contains_key(*slug)))
        .map(|slug| ReviewTerm { slug: slug.clone(), name_en: snapshot.catalog.term_names[slug].clone() });
    checkin::suggest(minutes, next.as_ref(), review.as_ref(), &snapshot.catalog.templates)
}

pub async fn check_in(state: &AppState, user_id: i32, req: CheckInRequest) -> AppResult<CheckInResponse> {
    if !checkin::is_allowed(req.minutes) {
        return Err(AppError::Validation("زمان انتخابی باید ۱۰، ۳۰ یا ۶۰ دقیقه باشد.".into()));
    }
    let mut conn = state.pool.acquire().await?;
    let snapshot = Snapshot::load(&mut conn, user_id).await?;
    let suggestion = suggestion_for(&snapshot, req.minutes);
    activity::upsert_check_in(&mut *conn, user_id, state.clock.today(), req.minutes, suggestion.kind.as_str()).await?;
    Ok(CheckInResponse { minutes: req.minutes, suggestion })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudyRequest {
    pub minutes: i32,
    pub course_slug: Option<String>,
}

pub const MAX_SESSION_MINUTES: i32 = 240;

/// Logs a study session (e.g. from the lesson page timer).
pub async fn log_study(state: &AppState, user_id: i32, req: StudyRequest) -> AppResult<SyncOutcome> {
    if !(1..=MAX_SESSION_MINUTES).contains(&req.minutes) {
        return Err(AppError::Validation("مدت مطالعه باید بین ۱ تا ۲۴۰ دقیقه باشد.".into()));
    }
    let mut tx = state.pool.begin().await?;
    let snapshot = Snapshot::load(&mut tx, user_id).await?;
    let (course_id, topic) = match req.course_slug.as_deref() {
        Some(slug) => {
            let course = snapshot.catalog.course_by_slug(slug).ok_or(AppError::NotFound("درس"))?;
            if snapshot.student.status(course.id) == EnrollmentStatus::Locked {
                return Err(AppError::Conflict("این درس هنوز برای تو باز نشده است.".into()));
            }
            let dept = snapshot.catalog.department(course.department_id).map(|d| d.slug.clone()).unwrap_or_default();
            (Some(course.id), dept)
        }
        None => (None, "general".to_string()),
    };
    activity::insert(
        &mut *tx,
        NewActivity {
            user_id,
            date: state.clock.today(),
            minutes: req.minutes,
            topic_tag: &topic,
            course_id,
            source: "study",
            at: state.clock.now(),
        },
    )
    .await?;
    let outcome = sync(&mut tx, &state.clock, user_id).await?;
    tx.commit().await?;
    Ok(outcome)
}
