//! The personalized dashboard: every widget is derived from the student's
//! real enrollments, activity and records.

use super::{
    academics::{course_detail, CourseDetail},
    engagement::{self, MissionDto},
    records::{self, AchievementDto, RankInfo},
    snapshot::Snapshot,
};
use crate::domain::{
    advisor::{self, Advice, DepartmentSignal},
    checkin::Suggestion,
    percent,
    progression::PromotionEvaluation,
    EnrollmentStatus,
};
use crate::error::AppResult;
use crate::repo::{activity, content};
use super::lessons;
use crate::state::AppState;
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StudentSummary {
    pub name: String,
    pub student_id: String,
    pub year: i16,
    pub current_term: i16,
    pub level_name: String,
    pub level_name_en: String,
    pub level_blurb: String,
    pub total_xp: i32,
    pub streak_days: i32,
    pub longest_streak: i32,
    pub gpa: Option<f64>,
    pub graduated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSummary {
    pub overall_percent: i32,
    pub credits_passed: i64,
    pub total_credits: i64,
    pub year_percent: i32,
    pub year_credits_passed: i64,
    pub year_credits: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatus {
    pub slug: String,
    pub code: String,
    pub title: String,
    pub status: EnrollmentStatus,
    pub percent: i32,
    pub lessons_done: usize,
    pub lesson_count: usize,
    pub grade: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityItem {
    pub date: NaiveDate,
    pub minutes: i32,
    pub topic: String,
    pub source: String,
    pub course_title: Option<String>,
    pub at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillStatus {
    pub department_slug: String,
    pub name_fa: String,
    pub accent: String,
    pub percent: i32,
    pub open_courses: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvisorCard {
    pub name: String,
    pub title: String,
    pub advice: Advice,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncementDto {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub important: bool,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckInState {
    pub minutes: i32,
    pub suggestion: Suggestion,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dashboard {
    pub today: NaiveDate,
    pub student: StudentSummary,
    pub progress: ProgressSummary,
    pub current_course: Option<CourseDetail>,
    pub mission: MissionDto,
    pub check_in: Option<CheckInState>,
    pub project: Option<ProjectStatus>,
    pub promotion: PromotionEvaluation,
    pub recent_achievements: Vec<AchievementDto>,
    pub recent_activity: Vec<ActivityItem>,
    pub skills: Vec<SkillStatus>,
    pub advisor: Option<AdvisorCard>,
    pub announcements: Vec<AnnouncementDto>,
    pub rank: RankInfo,
}

pub async fn dashboard(state: &AppState, user_id: i32) -> AppResult<Dashboard> {
    let today = state.clock.today();
    let mut tx = state.pool.begin().await?;
    // Keep derived state fresh: unlocks/term (e.g. after a catalog update),
    // streak decay, today's mission, achievements.
    lessons::advance(&mut tx, user_id, state.clock.now()).await?;
    engagement::sync(&mut tx, &state.clock, user_id).await?;
    let s = Snapshot::load(&mut tx, user_id).await?;
    let mission = engagement::ensure_mission(&mut tx, &s, today).await?;
    tx.commit().await?;

    let mut conn = state.pool.acquire().await?;
    let user = &s.student.user;
    let level = s.catalog.level_by_year(s.year());
    let year = s.year();

    let (year_passed, year_total) = s.year_credits(year);
    let credits_passed = s.passed_credits(|_| true);
    let total_credits = s.catalog.total_credits();

    let project = s.catalog.year_project(year).map(|p| {
        let (done, total) = s.lesson_progress(p.id);
        ProjectStatus {
            slug: p.slug.clone(),
            code: p.code.clone(),
            title: p.title.clone(),
            status: s.student.status(p.id),
            percent: percent(done as i64, total as i64),
            lessons_done: done,
            lesson_count: total,
            grade: s.student.grade(p.id),
        }
    });

    let awards = records::load_awards(&mut conn, user_id).await?;
    let mut recent_achievements: Vec<AchievementDto> = awards.achievements.into_iter().filter(|a| a.unlocked).collect();
    recent_achievements.sort_by(|a, b| b.unlocked_at.cmp(&a.unlocked_at));
    recent_achievements.truncate(3);

    let recent_activity = activity::recent(&mut *conn, user_id, 6)
        .await?
        .into_iter()
        .map(|a| ActivityItem {
            date: a.date,
            minutes: a.minutes_spent,
            topic: s.catalog.department_by_slug(&a.topic_tag).map(|d| d.name_fa.clone()).unwrap_or_else(|| "مطالعه‌ی آزاد".into()),
            source: a.source,
            course_title: a.course_title,
            at: a.created_at,
        })
        .collect();

    let open = s.open_courses();
    let skills = s
        .catalog
        .departments
        .iter()
        .map(|d| {
            let credits: i64 = s.catalog.courses.iter().filter(|c| c.department_id == d.id).map(|c| c.credits as i64).sum();
            SkillStatus {
                department_slug: d.slug.clone(),
                name_fa: d.name_fa.clone(),
                accent: d.accent.clone(),
                percent: percent(s.passed_credits(|c| c.department_id == d.id), credits),
                open_courses: open.iter().filter(|c| c.department_id == d.id).count(),
            }
        })
        .collect();

    // Advisor signals: grades per department plus the last 14 days of study time.
    let recent_topics = activity::minutes_by_topic(&mut *conn, user_id, today - Duration::days(13)).await?;
    let signals: Vec<DepartmentSignal> = s
        .catalog
        .departments
        .iter()
        .map(|d| DepartmentSignal {
            slug: d.slug.clone(),
            name: d.name_fa.clone(),
            average_grade: s.gpa(|c| c.department_id == d.id),
            recent_minutes: recent_topics.iter().find(|(t, _)| *t == d.slug).map(|(_, m)| *m).unwrap_or(0),
            has_open_courses: open.iter().any(|c| c.department_id == d.id),
        })
        .collect();
    let studied_today = activity::minutes_on(&mut *conn, user_id, today, None).await? > 0;
    let has_history = s.has_history() || !activity::active_dates(&mut *conn, user_id).await?.is_empty();
    let advice = advisor::advise(&signals, user.streak_days as u32, studied_today, has_history, &s.catalog.templates);
    // The advisor of the department the advice points to (or of the focus course).
    let advisor_dept = advice
        .focus_department
        .as_deref()
        .and_then(|slug| s.catalog.department_by_slug(slug))
        .map(|d| d.id)
        .or_else(|| s.focus_course().map(|c| c.department_id))
        .unwrap_or_default();
    let advisor = s.catalog.advisor_for(advisor_dept).map(|a| AdvisorCard { name: a.name.clone(), title: a.title.clone(), advice });

    let check_in = activity::check_in_minutes(&mut *conn, user_id, today)
        .await?
        .map(|minutes| CheckInState { minutes, suggestion: engagement::suggestion_for(&s, minutes) });

    let announcements = content::announcements(&mut *conn, 3)
        .await?
        .into_iter()
        .map(|a| AnnouncementDto { id: a.id, title: a.title, body: a.body, important: a.priority == "important", published_at: a.published_at })
        .collect();

    let (rank, _) = records::rank_of(&mut conn, &s).await?;

    Ok(Dashboard {
        today,
        student: StudentSummary {
            name: user.name.clone(),
            student_id: user.student_id.clone(),
            year,
            current_term: user.current_term,
            level_name: level.map(|l| l.name_fa.clone()).unwrap_or_default(),
            level_name_en: level.map(|l| l.name_en.clone()).unwrap_or_default(),
            level_blurb: level.map(|l| l.year_blurb.clone()).unwrap_or_default(),
            total_xp: user.total_xp,
            streak_days: user.streak_days,
            longest_streak: user.longest_streak,
            gpa: s.gpa(|_| true),
            graduated: s.student.is_graduated(),
        },
        progress: ProgressSummary {
            overall_percent: percent(credits_passed, total_credits),
            credits_passed,
            total_credits,
            year_percent: percent(year_passed, year_total),
            year_credits_passed: year_passed,
            year_credits: year_total,
        },
        current_course: s.focus_course().map(|c| course_detail(&s, c)),
        mission: engagement::mission_dto(&mission, &s),
        check_in,
        project,
        promotion: s.promotion(year),
        recent_achievements,
        recent_activity,
        skills,
        advisor,
        announcements,
        rank,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Me {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub student_id: String,
    pub year: i16,
    pub level_name: String,
    pub current_term: i16,
    pub total_xp: i32,
    pub streak_days: i32,
    pub joined_at: DateTime<Utc>,
}

pub async fn me(state: &AppState, user_id: i32) -> AppResult<Me> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let u = &s.student.user;
    let email: String = sqlx::query_scalar("SELECT email FROM users WHERE id = $1").bind(user_id).fetch_one(&mut *conn).await?;
    Ok(Me {
        id: u.id,
        name: u.name.clone(),
        email,
        student_id: u.student_id.clone(),
        year: u.year,
        level_name: s.catalog.level_by_year(u.year).map(|l| l.name_fa.clone()).unwrap_or_default(),
        current_term: u.current_term,
        total_xp: u.total_xp,
        streak_days: u.streak_days,
        joined_at: u.joined_at,
    })
}
