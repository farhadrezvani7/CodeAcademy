use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};

#[derive(Debug, Clone, FromRow)]
pub struct Credentials {
    pub id: i32,
    pub password_hash: String,
}

pub async fn credentials_by_email(db: impl PgExecutor<'_>, email: &str) -> sqlx::Result<Option<Credentials>> {
    sqlx::query_as("SELECT id, password_hash FROM users WHERE email = $1").bind(email).fetch_optional(db).await
}

pub async fn credentials_by_id(db: impl PgExecutor<'_>, id: i32) -> sqlx::Result<Option<Credentials>> {
    sqlx::query_as("SELECT id, password_hash FROM users WHERE id = $1").bind(id).fetch_optional(db).await
}

pub async fn email_taken(db: impl PgExecutor<'_>, email: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)").bind(email).fetch_one(db).await
}

pub async fn next_student_number(db: impl PgExecutor<'_>) -> sqlx::Result<i64> {
    sqlx::query_scalar("SELECT nextval('student_number_seq')").fetch_one(db).await
}

/// Creates a first-year student. Returns the new user id, or None if the
/// email was taken concurrently.
pub async fn create_user(
    db: impl PgExecutor<'_>,
    name: &str,
    email: &str,
    password_hash: &str,
    student_id: &str,
    at: DateTime<Utc>,
) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar(
        "INSERT INTO users (name, email, password_hash, student_id, current_level_id, current_term, joined_at)
         VALUES ($1, $2, $3, $4, (SELECT id FROM levels WHERE year = 1), 1, $5)
         ON CONFLICT (email) DO NOTHING RETURNING id",
    )
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .bind(student_id)
    .bind(at)
    .fetch_optional(db)
    .await
}

pub async fn set_name(db: impl PgExecutor<'_>, user_id: i32, name: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET name = $2 WHERE id = $1").bind(user_id).bind(name).execute(db).await?;
    Ok(())
}

pub async fn set_password(db: impl PgExecutor<'_>, user_id: i32, hash: &str) -> sqlx::Result<()> {
    sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1").bind(user_id).bind(hash).execute(db).await?;
    Ok(())
}

pub async fn create_session(db: impl PgExecutor<'_>, user_id: i32, token_hash: &str, expires_at: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO sessions (user_id, token_hash, expires_at) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(db)
        .await?;
    Ok(())
}

/// Resolves a live session and refreshes its last-seen time.
pub async fn session_user(db: impl PgExecutor<'_>, token_hash: &str, now: DateTime<Utc>) -> sqlx::Result<Option<i32>> {
    sqlx::query_scalar(
        "UPDATE sessions SET last_seen_at = $2 WHERE token_hash = $1 AND expires_at > $2 RETURNING user_id",
    )
    .bind(token_hash)
    .bind(now)
    .fetch_optional(db)
    .await
}

pub async fn delete_session(db: impl PgExecutor<'_>, token_hash: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE token_hash = $1").bind(token_hash).execute(db).await?;
    Ok(())
}

/// Signs the user out everywhere except the given session.
pub async fn delete_other_sessions(db: impl PgExecutor<'_>, user_id: i32, keep_hash: &str) -> sqlx::Result<()> {
    sqlx::query("DELETE FROM sessions WHERE user_id = $1 AND token_hash <> $2").bind(user_id).bind(keep_hash).execute(db).await?;
    Ok(())
}

pub async fn delete_expired_sessions(db: impl PgExecutor<'_>, now: DateTime<Utc>) -> sqlx::Result<u64> {
    Ok(sqlx::query("DELETE FROM sessions WHERE expires_at <= $1").bind(now).execute(db).await?.rows_affected())
}

pub async fn record_login(db: impl PgExecutor<'_>, email: &str, success: bool, at: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO login_attempts (email, success, created_at) VALUES ($1, $2, $3)")
        .bind(email)
        .bind(success)
        .bind(at)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn recent_failures(db: impl PgExecutor<'_>, email: &str, since: DateTime<Utc>) -> sqlx::Result<i64> {
    sqlx::query_scalar(
        "SELECT count(*) FROM login_attempts WHERE email = $1 AND NOT success AND created_at > $2
           AND created_at > COALESCE((SELECT max(created_at) FROM login_attempts WHERE email = $1 AND success), '-infinity')",
    )
    .bind(email)
    .bind(since)
    .fetch_one(db)
    .await
}
