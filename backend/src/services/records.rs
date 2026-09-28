//! Official records: transcript, passport, achievements and ranking.

use super::snapshot::Snapshot;
use crate::domain::{
    gpa::{self, GradedCourse},
    percent,
    ranking::{self, Contender},
    EnrollmentStatus,
};
use crate::error::AppResult;
use crate::repo::{content, student};
use crate::state::AppState;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgConnection;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptCourse {
    pub slug: String,
    pub code: String,
    pub title: String,
    pub kind: String,
    pub credits: i32,
    pub grade: Option<f64>,
    pub passed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSemester {
    pub number: i16,
    pub title: String,
    pub courses: Vec<TranscriptCourse>,
    pub credits: i32,
    pub credits_passed: i32,
    pub gpa: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InProgressCourse {
    pub slug: String,
    pub code: String,
    pub title: String,
    pub credits: i32,
    pub status: EnrollmentStatus,
    pub percent: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    pub student_name: String,
    pub student_id: String,
    pub level_name: String,
    pub year: i16,
    pub current_term: i16,
    pub semesters: Vec<TranscriptSemester>,
    pub in_progress: Vec<InProgressCourse>,
    pub cumulative_gpa: Option<f64>,
    pub credits_attempted: i32,
    pub credits_passed: i32,
    pub credits_remaining: i64,
    pub total_credits: i64,
    pub issued_at: DateTime<Utc>,
}

pub async fn transcript(state: &AppState, user_id: i32) -> AppResult<Transcript> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let rows = student::transcript(&mut *conn, user_id).await?;

    let mut semesters: Vec<TranscriptSemester> = Vec::new();
    let mut all_graded = Vec::new();
    for row in rows.into_iter().filter(|r| r.status == "done") {
        let Some(grade) = row.grade else { continue };
        all_graded.push(GradedCourse { credits: row.credits, grade });
        if semesters.last().is_none_or(|sem| sem.number != row.semester_number) {
            semesters.push(TranscriptSemester {
                number: row.semester_number,
                title: row.semester_title.clone(),
                courses: Vec::new(),
                credits: 0,
                credits_passed: 0,
                gpa: None,
            });
        }
        let sem = semesters.last_mut().expect("just pushed");
        let passed = gpa::is_passing(grade);
        sem.credits += row.credits;
        if passed {
            sem.credits_passed += row.credits;
        }
        sem.courses.push(TranscriptCourse {
            slug: row.course_slug,
            code: row.course_code,
            title: row.course_title,
            kind: row.course_kind,
            credits: row.credits,
            grade: Some(grade),
            passed,
        });
    }
    for sem in &mut semesters {
        let graded: Vec<GradedCourse> =
            sem.courses.iter().filter_map(|c| c.grade.map(|g| GradedCourse { credits: c.credits, grade: g })).collect();
        sem.gpa = gpa::weighted_gpa(&graded);
    }

    let credits_attempted = all_graded.iter().map(|g| g.credits).sum();
    let credits_passed: i32 = semesters.iter().map(|s| s.credits_passed).sum();
    let total = s.catalog.total_credits();
    let in_progress = s
        .open_courses()
        .into_iter()
        .map(|c| InProgressCourse {
            slug: c.slug.clone(),
            code: c.code.clone(),
            title: c.title.clone(),
            credits: c.credits,
            status: s.student.status(c.id),
            percent: s.course_percent(c.id),
        })
        .collect();
    Ok(Transcript {
        student_name: s.student.user.name.clone(),
        student_id: s.student.user.student_id.clone(),
        level_name: s.catalog.level_by_year(s.year()).map(|l| l.name_fa.clone()).unwrap_or_default(),
        year: s.year(),
        current_term: s.student.user.current_term,
        semesters,
        in_progress,
        cumulative_gpa: gpa::weighted_gpa(&all_graded),
        credits_attempted,
        credits_passed,
        credits_remaining: (total - credits_passed as i64).max(0),
        total_credits: total,
        issued_at: state.clock.now(),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankInfo {
    pub rank: usize,
    pub of: usize,
    pub level_name: String,
}

pub async fn rank_of(db: &mut PgConnection, s: &Snapshot) -> AppResult<(RankInfo, Vec<ranking::Ranked>)> {
    let peers = student::peers_on_level(&mut *db, s.student.user.current_level_id).await?;
    let ranked = ranking::rank(peers.into_iter().map(|p| Contender { user_id: p.id, name: p.name, xp: p.total_xp }).collect());
    let me = ranked.iter().find(|r| r.user_id == s.student.user.id).map(|r| r.rank).unwrap_or(ranked.len() + 1);
    let info = RankInfo {
        rank: me,
        of: ranked.len(),
        level_name: s.catalog.level_by_year(s.year()).map(|l| l.name_fa.clone()).unwrap_or_default(),
    };
    Ok((info, ranked))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub name: String,
    pub xp: i32,
    pub is_me: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Leaderboard {
    #[serde(flatten)]
    pub me: RankInfo,
    pub entries: Vec<LeaderboardEntry>,
}

pub async fn leaderboard(state: &AppState, user_id: i32) -> AppResult<Leaderboard> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let (me, ranked) = rank_of(&mut conn, &s).await?;
    Ok(Leaderboard {
        me,
        entries: ranked.into_iter().map(|r| LeaderboardEntry { rank: r.rank, is_me: r.user_id == user_id, name: r.name, xp: r.xp }).collect(),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementDto {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlocked: bool,
    pub unlocked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateDto {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub issued: bool,
    pub serial_no: Option<String>,
    pub issued_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AchievementsResponse {
    pub unlocked: usize,
    pub total: usize,
    pub achievements: Vec<AchievementDto>,
    pub certificates: Vec<CertificateDto>,
}

pub async fn load_awards(db: &mut PgConnection, user_id: i32) -> AppResult<AchievementsResponse> {
    let achievements: Vec<AchievementDto> = content::achievements(&mut *db, user_id)
        .await?
        .into_iter()
        .map(|a| AchievementDto { unlocked: a.unlocked_at.is_some(), slug: a.slug, title: a.title, description: a.description, icon: a.icon, unlocked_at: a.unlocked_at })
        .collect();
    let certificates = content::certificates(&mut *db, user_id)
        .await?
        .into_iter()
        .map(|c| CertificateDto { issued: c.issued_at.is_some(), slug: c.slug, title: c.title, description: c.description, serial_no: c.serial_no, issued_at: c.issued_at })
        .collect();
    Ok(AchievementsResponse { unlocked: achievements.iter().filter(|a| a.unlocked).count(), total: achievements.len(), achievements, certificates })
}

pub async fn achievements(state: &AppState, user_id: i32) -> AppResult<AchievementsResponse> {
    let mut conn = state.pool.acquire().await?;
    load_awards(&mut conn, user_id).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stamp {
    pub year: i16,
    pub name_fa: String,
    pub name_en: String,
    pub passed: bool,
    pub gpa: Option<f64>,
    pub passed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillBar {
    pub department_slug: String,
    pub name_fa: String,
    pub accent: String,
    pub percent: i32,
    pub credits_passed: i64,
    pub credits: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Passport {
    pub name: String,
    pub student_id: String,
    pub joined_at: DateTime<Utc>,
    pub level_name: String,
    pub level_name_en: String,
    pub year: i16,
    pub current_term: i16,
    pub total_xp: i32,
    pub streak_days: i32,
    pub gpa: Option<f64>,
    pub graduated: bool,
    pub graduated_at: Option<DateTime<Utc>>,
    pub stamps: Vec<Stamp>,
    pub skills: Vec<SkillBar>,
    pub rank: RankInfo,
    pub certificates: Vec<CertificateDto>,
    pub achievements_unlocked: usize,
    pub achievements_total: usize,
}

pub async fn passport(state: &AppState, user_id: i32) -> AppResult<Passport> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let (rank, _) = rank_of(&mut conn, &s).await?;
    let awards = load_awards(&mut conn, user_id).await?;
    let passed_year = s.passed_year();
    let user = &s.student.user;
    let level = s.catalog.level_by_year(s.year());

    let stamps = s
        .catalog
        .levels
        .iter()
        .map(|l| {
            let passed = l.year <= passed_year;
            Stamp {
                year: l.year,
                name_fa: l.name_fa.clone(),
                name_en: l.name_en.clone(),
                passed,
                gpa: if passed { s.gpa(|c| c.year == l.year) } else { None },
                passed_at: if passed {
                    s.catalog.courses_of_year(l.year).filter_map(|c| s.student.enrollments.get(&c.id).and_then(|e| e.completed_at)).max()
                } else {
                    None
                },
            }
        })
        .collect();
    let skills = s
        .catalog
        .departments
        .iter()
        .map(|d| {
            let credits: i64 = s.catalog.courses.iter().filter(|c| c.department_id == d.id).map(|c| c.credits as i64).sum();
            let passed = s.passed_credits(|c| c.department_id == d.id);
            SkillBar { department_slug: d.slug.clone(), name_fa: d.name_fa.clone(), accent: d.accent.clone(), percent: percent(passed, credits), credits_passed: passed, credits }
        })
        .collect();

    Ok(Passport {
        name: user.name.clone(),
        student_id: user.student_id.clone(),
        joined_at: user.joined_at,
        level_name: level.map(|l| l.name_fa.clone()).unwrap_or_default(),
        level_name_en: level.map(|l| l.name_en.clone()).unwrap_or_default(),
        year: s.year(),
        current_term: user.current_term,
        total_xp: user.total_xp,
        streak_days: user.streak_days,
        gpa: s.gpa(|_| true),
        graduated: s.student.is_graduated(),
        graduated_at: user.graduated_at,
        stamps,
        skills,
        rank,
        certificates: awards.certificates,
        achievements_unlocked: awards.unlocked,
        achievements_total: awards.total,
    })
}
