//! Lesson reading and completion, including the course/level progression
//! that a completion can trigger.

use super::{engagement, snapshot::Snapshot};
use crate::domain::{
    course_status::{self, Unlockable},
    progression, xp, CourseKind, EnrollmentStatus,
};
use crate::error::{AppError, AppResult};
use crate::repo::{
    activity::{self, NewActivity},
    catalog, content, student,
};
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use sqlx::PgConnection;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TermRef {
    pub slug: String,
    pub name_en: String,
    pub name_fa: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonNav {
    pub id: i32,
    pub title: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreOption {
    pub score: f64,
    pub label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonDto {
    pub id: i32,
    pub order: i32,
    pub title: String,
    pub estimated_minutes: i32,
    pub what: String,
    pub why: String,
    pub simple_example: serde_json::Value,
    pub real_example: serde_json::Value,
    pub mistakes: Vec<String>,
    pub best_practices: Vec<String>,
    pub related_terms: Vec<TermRef>,
    pub try_it: String,
    pub course: CourseRef,
    pub lesson_count: usize,
    pub previous: Option<LessonNav>,
    pub next: Option<LessonNav>,
    pub locked: bool,
    pub completed: bool,
    pub self_score: Option<f64>,
    pub score_options: Vec<ScoreOption>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseRef {
    pub slug: String,
    pub code: String,
    pub title: String,
    pub department_slug: String,
    pub department_name: String,
    pub status: EnrollmentStatus,
}

pub async fn get(state: &AppState, user_id: i32, lesson_id: i32) -> AppResult<LessonDto> {
    let mut conn = state.pool.acquire().await?;
    let lesson = catalog::lesson(&mut *conn, lesson_id).await?.ok_or(AppError::NotFound("درس"))?;
    let snapshot = Snapshot::load(&mut conn, user_id).await?;
    let course = snapshot.catalog.course(lesson.course_id).ok_or(AppError::NotFound("درس"))?;
    let dept = snapshot.catalog.department(course.department_id).ok_or(AppError::NotFound("دانشکده"))?;

    let mut terms = content::terms_by_slugs(&mut *conn, &lesson.related_terms).await?;
    terms.sort_by_key(|t| lesson.related_terms.iter().position(|s| *s == t.slug));

    let siblings: Vec<_> = snapshot.catalog.lessons_of(course.id).collect();
    let pos = siblings.iter().position(|l| l.id == lesson.id).unwrap_or(0);
    let nav = |l: &&crate::repo::catalog::LessonSummaryRow| LessonNav { id: l.id, title: l.title.clone() };
    let status = snapshot.student.status(course.id);
    let completion = snapshot.student.completion(lesson.id);

    Ok(LessonDto {
        id: lesson.id,
        order: lesson.sort_order,
        title: lesson.title,
        estimated_minutes: lesson.estimated_minutes,
        what: lesson.what,
        why: lesson.why,
        simple_example: lesson.simple_example,
        real_example: lesson.real_example,
        mistakes: lesson.mistakes,
        best_practices: lesson.best_practices,
        related_terms: terms.into_iter().map(|t| TermRef { slug: t.slug, name_en: t.name_en, name_fa: t.name_fa }).collect(),
        try_it: lesson.try_it,
        course: CourseRef {
            slug: course.slug.clone(),
            code: course.code.clone(),
            title: course.title.clone(),
            department_slug: dept.slug.clone(),
            department_name: dept.name_fa.clone(),
            status,
        },
        lesson_count: siblings.len(),
        previous: pos.checked_sub(1).and_then(|i| siblings.get(i)).map(nav),
        next: siblings.get(pos + 1).map(nav),
        locked: status == EnrollmentStatus::Locked,
        completed: completion.is_some(),
        self_score: completion.map(|c| c.self_score),
        score_options: snapshot.catalog.score_options.iter().map(|(score, label)| ScoreOption { score: *score, label: label.clone() }).collect(),
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRequest {
    pub self_score: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteResponse {
    pub first_completion: bool,
    pub xp_gained: i32,
    pub total_xp: i32,
    pub course_status: EnrollmentStatus,
    pub course_grade: Option<f64>,
    pub unlocked_courses: Vec<String>,
    pub promoted_to: Option<String>,
    pub graduated: bool,
    pub streak_days: i32,
    pub mission_completed: bool,
    pub new_achievements: Vec<String>,
    pub new_certificates: Vec<String>,
}

pub async fn complete(state: &AppState, user_id: i32, lesson_id: i32, req: CompleteRequest) -> AppResult<CompleteResponse> {
    let now = state.clock.now();
    let today = state.clock.today();
    let mut tx = state.pool.begin().await?;

    let lesson = catalog::lesson(&mut *tx, lesson_id).await?.ok_or(AppError::NotFound("درس"))?;
    let before = Snapshot::load(&mut tx, user_id).await?;
    let options: Vec<f64> = before.catalog.score_options.iter().map(|(s, _)| *s).collect();
    if !course_status::is_valid_self_score(req.self_score, &options) {
        return Err(AppError::Validation("نمره‌ی خودارزیابی نامعتبر است.".into()));
    }
    let course = before.catalog.course(lesson.course_id).ok_or(AppError::NotFound("درس"))?.clone();
    let previous_status = before.student.status(course.id);
    if previous_status == EnrollmentStatus::Locked {
        return Err(AppError::Conflict("این درس هنوز برای تو باز نشده است؛ ابتدا پیش‌نیازها را بگذران.".into()));
    }
    let xp_before = before.student.user.total_xp;

    let first = student::upsert_completion(&mut *tx, user_id, lesson.id, req.self_score, now).await?;
    if first {
        let award = xp::for_lesson(lesson.id);
        student::award_xp(&mut tx, user_id, award.reason, &award.reference, award.amount, now).await?;
        let topic = before.catalog.department(course.department_id).map(|d| d.slug.clone()).unwrap_or_default();
        activity::insert(
            &mut *tx,
            NewActivity {
                user_id,
                date: today,
                minutes: lesson.estimated_minutes,
                topic_tag: &topic,
                course_id: Some(course.id),
                source: "lesson",
                at: now,
            },
        )
        .await?;
    }

    // Course status and grade from all lesson scores.
    let lesson_ids: Vec<i32> = before.catalog.lessons_of(course.id).map(|l| l.id).collect();
    let scores: Vec<f64> = lesson_ids
        .iter()
        .filter_map(|id| {
            if *id == lesson.id {
                Some(req.self_score)
            } else {
                before.student.completion(*id).map(|c| c.self_score)
            }
        })
        .collect();
    let status = course_status::status_for_progress(scores.len(), lesson_ids.len());
    let grade = (status == EnrollmentStatus::Done).then(|| course_status::course_grade(&scores)).flatten();
    let semester = before.student.enrollments.get(&course.id).and_then(|e| e.semester_number).unwrap_or(before.student.user.current_term);
    let semester = (status == EnrollmentStatus::Done).then_some(semester);
    student::upsert_enrollment(&mut *tx, user_id, course.id, status.as_str(), grade, semester, now).await?;

    if status == EnrollmentStatus::Done && previous_status != EnrollmentStatus::Done {
        if let Some(award) = xp::for_course_completion(course.id, CourseKind::parse(&course.kind), course.year) {
            student::award_xp(&mut tx, user_id, award.reason, &award.reference, award.amount, now).await?;
        }
    }

    let progression = advance(&mut tx, user_id, now).await?;
    let sync = engagement::sync(&mut tx, &state.clock, user_id).await?;
    let after = student::user(&mut *tx, user_id).await?.ok_or(AppError::NotFound("دانشجو"))?;
    tx.commit().await?;

    Ok(CompleteResponse {
        first_completion: first,
        xp_gained: after.total_xp - xp_before,
        total_xp: after.total_xp,
        course_status: status,
        course_grade: grade,
        unlocked_courses: progression.unlocked,
        promoted_to: progression.promoted_to,
        graduated: progression.graduated,
        streak_days: sync.streak_days,
        mission_completed: sync.mission_completed_now,
        new_achievements: sync.new_achievements,
        new_certificates: sync.new_certificates,
    })
}

#[derive(Debug, Default)]
pub struct Advancement {
    pub unlocked: Vec<String>,
    pub promoted_to: Option<String>,
    pub graduated: bool,
}

/// Unlocks courses whose prerequisites are done, moves the term forward and
/// promotes the student when the year's promotion rules are all met.
pub async fn advance(db: &mut PgConnection, user_id: i32, now: chrono::DateTime<chrono::Utc>) -> AppResult<Advancement> {
    let mut out = Advancement::default();
    let snapshot = Snapshot::load(&mut *db, user_id).await?;
    let mut year = snapshot.year();

    out.unlocked = unlock(&mut *db, &snapshot, user_id, year, now).await?;
    let snapshot = Snapshot::load(&mut *db, user_id).await?;
    let term = progression::current_term(year, snapshot.student.user.current_term, &snapshot.progress_up_to(year));
    student::set_standing(&mut *db, user_id, year, term).await?;

    let snapshot = Snapshot::load(&mut *db, user_id).await?;
    if snapshot.promotion(year).eligible && !snapshot.student.is_graduated() {
        if year < progression::TOTAL_YEARS {
            year += 1;
            student::set_standing(&mut *db, user_id, year, progression::expected_term(year, 1)).await?;
            let snapshot = Snapshot::load(&mut *db, user_id).await?;
            out.unlocked.extend(unlock(&mut *db, &snapshot, user_id, year, now).await?);
            out.promoted_to = snapshot.catalog.level_by_year(year).map(|l| l.name_fa.clone());
        } else {
            student::set_graduated(&mut *db, user_id, now).await?;
            out.graduated = true;
        }
    }
    Ok(out)
}

async fn unlock(db: &mut PgConnection, snapshot: &Snapshot, user_id: i32, year: i16, now: chrono::DateTime<chrono::Utc>) -> AppResult<Vec<String>> {
    let candidates: Vec<Unlockable> = snapshot
        .catalog
        .courses
        .iter()
        .map(|c| Unlockable {
            course_id: c.id,
            year: c.year,
            status: snapshot.student.status(c.id),
            passed: snapshot.is_passed(c.id),
            prerequisites: snapshot.catalog.prerequisites.get(&c.id).cloned().unwrap_or_default(),
        })
        .collect();
    let mut unlocked = Vec::new();
    for id in course_status::newly_unlocked(&candidates, year) {
        student::upsert_enrollment(&mut *db, user_id, id, EnrollmentStatus::Enrolled.as_str(), None, None, now).await?;
        if let Some(c) = snapshot.catalog.course(id) {
            unlocked.push(c.code.clone());
        }
    }
    Ok(unlocked)
}
