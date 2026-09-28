use chrono::{DateTime, NaiveDate, Utc};
use sqlx::{FromRow, PgExecutor};

pub struct NewActivity<'a> {
    pub user_id: i32,
    pub date: NaiveDate,
    pub minutes: i32,
    pub topic_tag: &'a str,
    pub course_id: Option<i32>,
    pub source: &'a str,
    pub at: DateTime<Utc>,
}

pub async fn insert(db: impl PgExecutor<'_>, a: NewActivity<'_>) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO activity_logs (user_id, date, minutes_spent, topic_tag, course_id, source, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(a.user_id)
    .bind(a.date)
    .bind(a.minutes)
    .bind(a.topic_tag)
    .bind(a.course_id)
    .bind(a.source)
    .bind(a.at)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn active_dates(db: impl PgExecutor<'_>, user_id: i32) -> sqlx::Result<Vec<NaiveDate>> {
    sqlx::query_scalar("SELECT DISTINCT date FROM activity_logs WHERE user_id = $1 ORDER BY date")
        .bind(user_id)
        .fetch_all(db)
        .await
}

pub async fn minutes_per_day(db: impl PgExecutor<'_>, user_id: i32, from: NaiveDate, to: NaiveDate) -> sqlx::Result<Vec<(NaiveDate, i32)>> {
    sqlx::query_as(
        "SELECT date, SUM(minutes_spent)::int FROM activity_logs
         WHERE user_id = $1 AND date BETWEEN $2 AND $3 GROUP BY date ORDER BY date",
    )
    .bind(user_id)
    .bind(from)
    .bind(to)
    .fetch_all(db)
    .await
}

/// Minutes per topic tag since `from` (inclusive).
pub async fn minutes_by_topic(db: impl PgExecutor<'_>, user_id: i32, from: NaiveDate) -> sqlx::Result<Vec<(String, i32)>> {
    sqlx::query_as(
        "SELECT topic_tag, SUM(minutes_spent)::int FROM activity_logs
         WHERE user_id = $1 AND date >= $2 GROUP BY topic_tag",
    )
    .bind(user_id)
    .bind(from)
    .fetch_all(db)
    .await
}

/// Minutes on a date, optionally only for one course.
pub async fn minutes_on(db: impl PgExecutor<'_>, user_id: i32, date: NaiveDate, course_id: Option<i32>) -> sqlx::Result<i32> {
    sqlx::query_scalar(
        "SELECT COALESCE(SUM(minutes_spent), 0)::int FROM activity_logs
         WHERE user_id = $1 AND date = $2 AND ($3::int IS NULL OR course_id = $3)",
    )
    .bind(user_id)
    .bind(date)
    .bind(course_id)
    .fetch_one(db)
    .await
}

pub async fn has_activity_in_hours(db: impl PgExecutor<'_>, user_id: i32, tz: &str, from_hour: i32, to_hour: i32) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM activity_logs WHERE user_id = $1
                AND EXTRACT(HOUR FROM created_at AT TIME ZONE $2) >= $3
                AND EXTRACT(HOUR FROM created_at AT TIME ZONE $2) < $4)",
    )
    .bind(user_id)
    .bind(tz)
    .bind(from_hour)
    .bind(to_hour)
    .fetch_one(db)
    .await
}

#[derive(Debug, Clone, FromRow)]
pub struct RecentActivityRow {
    pub date: NaiveDate,
    pub minutes_spent: i32,
    pub topic_tag: String,
    pub source: String,
    pub course_title: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub async fn recent(db: impl PgExecutor<'_>, user_id: i32, limit: i64) -> sqlx::Result<Vec<RecentActivityRow>> {
    sqlx::query_as(
        "SELECT a.date, a.minutes_spent, a.topic_tag, a.source, c.title AS course_title, a.created_at
         FROM activity_logs a LEFT JOIN courses c ON c.id = a.course_id
         WHERE a.user_id = $1 ORDER BY a.created_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone, FromRow)]
pub struct MissionRow {
    pub id: i32,
    pub date: NaiveDate,
    pub description: String,
    pub course_id: Option<i32>,
    pub target_minutes: i32,
    pub progress_minutes: i32,
    pub xp_reward: i32,
    pub completed_at: Option<DateTime<Utc>>,
}

pub async fn mission(db: impl PgExecutor<'_>, user_id: i32, date: NaiveDate) -> sqlx::Result<Option<MissionRow>> {
    sqlx::query_as(
        "SELECT id, date, description, course_id, target_minutes, progress_minutes, xp_reward, completed_at
         FROM daily_missions WHERE user_id = $1 AND date = $2",
    )
    .bind(user_id)
    .bind(date)
    .fetch_optional(db)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_mission(
    db: impl PgExecutor<'_>,
    user_id: i32,
    date: NaiveDate,
    description: &str,
    course_id: Option<i32>,
    target_minutes: i32,
    xp_reward: i32,
) -> sqlx::Result<MissionRow> {
    // ON CONFLICT keeps concurrent first requests of the day from failing.
    sqlx::query_as(
        "INSERT INTO daily_missions (user_id, date, description, course_id, target_minutes, xp_reward)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (user_id, date) DO UPDATE SET date = EXCLUDED.date
         RETURNING id, date, description, course_id, target_minutes, progress_minutes, xp_reward, completed_at",
    )
    .bind(user_id)
    .bind(date)
    .bind(description)
    .bind(course_id)
    .bind(target_minutes)
    .bind(xp_reward)
    .fetch_one(db)
    .await
}

pub async fn update_mission_progress(db: impl PgExecutor<'_>, id: i32, minutes: i32, completed_at: Option<DateTime<Utc>>) -> sqlx::Result<()> {
    sqlx::query("UPDATE daily_missions SET progress_minutes = $2, completed_at = COALESCE(completed_at, $3) WHERE id = $1")
        .bind(id)
        .bind(minutes)
        .bind(completed_at)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn upsert_check_in(db: impl PgExecutor<'_>, user_id: i32, date: NaiveDate, minutes: i32, kind: &str) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO check_ins (user_id, date, minutes_available, suggestion_kind) VALUES ($1, $2, $3, $4)
         ON CONFLICT (user_id, date) DO UPDATE SET minutes_available = EXCLUDED.minutes_available,
             suggestion_kind = EXCLUDED.suggestion_kind, created_at = now()",
    )
    .bind(user_id)
    .bind(date)
    .bind(minutes)
    .bind(kind)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn check_in_minutes(db: impl PgExecutor<'_>, user_id: i32, date: NaiveDate) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar("SELECT minutes_available FROM check_ins WHERE user_id = $1 AND date = $2")
        .bind(user_id)
        .bind(date)
        .fetch_optional(db)
        .await
}
