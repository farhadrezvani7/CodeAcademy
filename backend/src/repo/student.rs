use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgConnection, PgExecutor};

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: i32,
    pub name: String,
    pub student_id: String,
    pub current_level_id: i32,
    pub year: i16,
    pub current_term: i16,
    pub total_xp: i32,
    pub streak_days: i32,
    pub longest_streak: i32,
    pub joined_at: DateTime<Utc>,
    pub graduated_at: Option<DateTime<Utc>>,
}

pub async fn user(db: impl PgExecutor<'_>, id: i32) -> sqlx::Result<Option<UserRow>> {
    sqlx::query_as(
        "SELECT u.id, u.name, u.student_id, u.current_level_id, l.year, u.current_term, u.total_xp, u.streak_days,
                u.longest_streak, u.joined_at, u.graduated_at
         FROM users u JOIN levels l ON l.id = u.current_level_id WHERE u.id = $1",
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn set_streak(db: impl PgExecutor<'_>, user_id: i32, current: i32, longest: i32) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET streak_days = $2, longest_streak = GREATEST(longest_streak, $3) WHERE id = $1")
        .bind(user_id)
        .bind(current)
        .bind(longest)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_standing(db: impl PgExecutor<'_>, user_id: i32, level_year: i16, term: i16) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET current_level_id = (SELECT id FROM levels WHERE year = $2), current_term = $3 WHERE id = $1")
        .bind(user_id)
        .bind(level_year)
        .bind(term)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_graduated(db: impl PgExecutor<'_>, user_id: i32, at: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET graduated_at = COALESCE(graduated_at, $2) WHERE id = $1")
        .bind(user_id)
        .bind(at)
        .execute(db)
        .await?;
    Ok(())
}

/// Records an XP award once. Returns true when it was new (and the total updated).
pub async fn award_xp(
    db: &mut PgConnection,
    user_id: i32,
    reason: &str,
    reference: &str,
    amount: i32,
    at: DateTime<Utc>,
) -> sqlx::Result<bool> {
    let inserted = sqlx::query(
        "INSERT INTO xp_events (user_id, amount, reason, ref, created_at) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, reason, ref) DO NOTHING",
    )
    .bind(user_id)
    .bind(amount)
    .bind(reason)
    .bind(reference)
    .bind(at)
    .execute(&mut *db)
    .await?
    .rows_affected()
        > 0;
    if inserted {
        sqlx::query("UPDATE users SET total_xp = total_xp + $2 WHERE id = $1")
            .bind(user_id)
            .bind(amount)
            .execute(&mut *db)
            .await?;
    }
    Ok(inserted)
}

#[derive(Debug, Clone, FromRow)]
pub struct EnrollmentRow {
    pub course_id: i32,
    pub status: String,
    pub grade: Option<f64>,
    pub semester_number: Option<i16>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub async fn enrollments(db: impl PgExecutor<'_>, user_id: i32) -> sqlx::Result<Vec<EnrollmentRow>> {
    sqlx::query_as(
        "SELECT e.course_id, e.status, e.grade, s.number AS semester_number, e.completed_at
         FROM enrollments e LEFT JOIN semesters s ON s.id = e.semester_id WHERE e.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn upsert_enrollment(
    db: impl PgExecutor<'_>,
    user_id: i32,
    course_id: i32,
    status: &str,
    grade: Option<f64>,
    semester_number: Option<i16>,
    at: DateTime<Utc>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO enrollments (user_id, course_id, status, grade, semester_id, started_at, completed_at)
         VALUES ($1, $2, $3, $4, (SELECT id FROM semesters WHERE number = $5),
                 CASE WHEN $3 IN ('in_progress', 'done') THEN $6 END,
                 CASE WHEN $3 = 'done' THEN $6 END)
         ON CONFLICT (user_id, course_id) DO UPDATE SET
             status = EXCLUDED.status,
             grade = EXCLUDED.grade,
             semester_id = COALESCE(EXCLUDED.semester_id, enrollments.semester_id),
             started_at = COALESCE(enrollments.started_at, EXCLUDED.started_at),
             completed_at = CASE WHEN EXCLUDED.status = 'done'
                                 THEN COALESCE(enrollments.completed_at, EXCLUDED.completed_at) END",
    )
    .bind(user_id)
    .bind(course_id)
    .bind(status)
    .bind(grade)
    .bind(semester_number)
    .bind(at)
    .execute(db)
    .await?;
    Ok(())
}

#[derive(Debug, Clone, FromRow)]
pub struct CompletionRow {
    pub lesson_id: i32,
    pub course_id: i32,
    pub self_score: f64,
    pub completed_at: DateTime<Utc>,
}

pub async fn completions(db: impl PgExecutor<'_>, user_id: i32) -> sqlx::Result<Vec<CompletionRow>> {
    sqlx::query_as(
        "SELECT lc.lesson_id, l.course_id, lc.self_score, lc.completed_at
         FROM lesson_completions lc JOIN lessons l ON l.id = lc.lesson_id
         WHERE lc.user_id = $1 ORDER BY lc.completed_at",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

/// Inserts or re-scores a completion. Returns true on first completion.
pub async fn upsert_completion(
    db: impl PgExecutor<'_>,
    user_id: i32,
    lesson_id: i32,
    score: f64,
    at: DateTime<Utc>,
) -> sqlx::Result<bool> {
    let first: bool = sqlx::query_scalar(
        "INSERT INTO lesson_completions (user_id, lesson_id, self_score, completed_at) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, lesson_id) DO UPDATE SET self_score = EXCLUDED.self_score
         RETURNING (xmax = 0)",
    )
    .bind(user_id)
    .bind(lesson_id)
    .bind(score)
    .bind(at)
    .fetch_one(db)
    .await?;
    Ok(first)
}

#[derive(Debug, Clone, FromRow)]
pub struct PeerRow {
    pub id: i32,
    pub name: String,
    pub total_xp: i32,
}

pub async fn peers_on_level(db: impl PgExecutor<'_>, level_id: i32) -> sqlx::Result<Vec<PeerRow>> {
    sqlx::query_as("SELECT id, name, total_xp FROM users WHERE current_level_id = $1")
        .bind(level_id)
        .fetch_all(db)
        .await
}

#[derive(Debug, Clone, FromRow)]
pub struct TranscriptRow {
    pub semester_number: i16,
    pub semester_title: String,
    pub course_slug: String,
    pub course_code: String,
    pub course_title: String,
    pub course_kind: String,
    pub credits: i32,
    pub grade: Option<f64>,
    pub status: String,
}

pub async fn transcript(db: impl PgExecutor<'_>, user_id: i32) -> sqlx::Result<Vec<TranscriptRow>> {
    sqlx::query_as(
        "SELECT semester_number, semester_title, course_slug, course_code, course_title, course_kind, credits, grade, status
         FROM transcript_entries WHERE user_id = $1 ORDER BY semester_number, course_code",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}
