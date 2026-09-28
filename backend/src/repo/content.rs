use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgExecutor};

#[derive(Debug, Clone, FromRow)]
pub struct TermRow {
    pub slug: String,
    pub name_en: String,
    pub name_fa: String,
    pub category: String,
    pub definition: String,
    pub example: String,
    pub related: Vec<String>,
}

const TERM_COLUMNS: &str = "slug, name_en, name_fa, category, definition, example, related";

/// Case-insensitive search over English name, Persian name and definition.
/// The query is bound as a parameter and LIKE wildcards are escaped.
pub async fn search_terms(db: impl PgExecutor<'_>, query: Option<&str>) -> sqlx::Result<Vec<TermRow>> {
    let pattern = query.map(|q| {
        let escaped = q.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        format!("%{escaped}%")
    });
    sqlx::query_as(&format!(
        "SELECT {TERM_COLUMNS} FROM dictionary_terms
         WHERE $1::text IS NULL OR name_en ILIKE $1 OR name_fa ILIKE $1 OR definition ILIKE $1 OR slug ILIKE $1
         ORDER BY (CASE WHEN $1::text IS NOT NULL AND (name_en ILIKE $1 OR name_fa ILIKE $1) THEN 0 ELSE 1 END), lower(name_en)"
    ))
    .bind(pattern)
    .fetch_all(db)
    .await
}

pub async fn term(db: impl PgExecutor<'_>, slug: &str) -> sqlx::Result<Option<TermRow>> {
    sqlx::query_as(&format!("SELECT {TERM_COLUMNS} FROM dictionary_terms WHERE slug = $1"))
        .bind(slug)
        .fetch_optional(db)
        .await
}

pub async fn terms_by_slugs(db: impl PgExecutor<'_>, slugs: &[String]) -> sqlx::Result<Vec<TermRow>> {
    sqlx::query_as(&format!("SELECT {TERM_COLUMNS} FROM dictionary_terms WHERE slug = ANY($1)"))
        .bind(slugs)
        .fetch_all(db)
        .await
}

#[derive(Debug, Clone, FromRow)]
pub struct AwardRow {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub unlocked_at: Option<DateTime<Utc>>,
}

pub async fn achievements(db: impl PgExecutor<'_>, user_id: i32) -> sqlx::Result<Vec<AwardRow>> {
    sqlx::query_as(
        "SELECT a.id, a.slug, a.title, a.description, a.icon, ua.unlocked_at
         FROM achievements a LEFT JOIN user_achievements ua ON ua.achievement_id = a.id AND ua.user_id = $1
         ORDER BY a.sort_order",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn unlock_achievement(db: impl PgExecutor<'_>, user_id: i32, achievement_id: i32, at: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO user_achievements (user_id, achievement_id, unlocked_at) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(achievement_id)
        .bind(at)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, FromRow)]
pub struct CertificateRow {
    pub id: i32,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub serial_no: Option<String>,
    pub issued_at: Option<DateTime<Utc>>,
}

pub async fn certificates(db: impl PgExecutor<'_>, user_id: i32) -> sqlx::Result<Vec<CertificateRow>> {
    sqlx::query_as(
        "SELECT c.id, c.slug, c.title, c.description, uc.serial_no, uc.issued_at
         FROM certificates c LEFT JOIN user_certificates uc ON uc.certificate_id = c.id AND uc.user_id = $1
         ORDER BY c.sort_order",
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn issue_certificate(db: impl PgExecutor<'_>, user_id: i32, certificate_id: i32, serial: &str, at: DateTime<Utc>) -> sqlx::Result<()> {
    sqlx::query("INSERT INTO user_certificates (user_id, certificate_id, serial_no, issued_at) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING")
        .bind(user_id)
        .bind(certificate_id)
        .bind(serial)
        .bind(at)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, FromRow)]
pub struct AnnouncementRow {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub priority: String,
    pub published_at: DateTime<Utc>,
}

pub async fn announcements(db: impl PgExecutor<'_>, limit: i64) -> sqlx::Result<Vec<AnnouncementRow>> {
    sqlx::query_as("SELECT id, title, body, priority, published_at FROM announcements ORDER BY published_at DESC LIMIT $1")
        .bind(limit)
        .fetch_all(db)
        .await
}

#[derive(Debug, Clone, FromRow)]
pub struct AdvisorRow {
    pub id: i32,
    pub name: String,
    pub title: String,
    pub department_id: i32,
    pub bio: String,
}

pub async fn advisors(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<AdvisorRow>> {
    sqlx::query_as("SELECT id, name, title, department_id, bio FROM advisors ORDER BY id").fetch_all(db).await
}

pub async fn promotion_rules(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<(String, String, String)>> {
    sqlx::query_as("SELECT key, title, description FROM promotion_rules ORDER BY sort_order").fetch_all(db).await
}

pub async fn score_options(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<(f64, String)>> {
    sqlx::query_as("SELECT score, label FROM score_options ORDER BY sort_order").fetch_all(db).await
}

pub async fn templates(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<(String, String)>> {
    sqlx::query_as("SELECT key, body FROM message_templates").fetch_all(db).await
}

pub async fn term_names(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<(String, String)>> {
    sqlx::query_as("SELECT slug, name_en FROM dictionary_terms").fetch_all(db).await
}

#[derive(Debug, Clone, FromRow)]
pub struct HighlightRow {
    pub section: String,
    pub icon: String,
    pub title: String,
    pub body: String,
}

pub async fn highlights(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<HighlightRow>> {
    sqlx::query_as("SELECT section, icon, title, body FROM site_highlights ORDER BY section, sort_order").fetch_all(db).await
}
