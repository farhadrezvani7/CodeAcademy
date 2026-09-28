use sqlx::{FromRow, PgExecutor};

#[derive(Debug, Clone, FromRow)]
pub struct LevelRow {
    pub id: i32,
    pub year: i16,
    pub slug: String,
    pub name_fa: String,
    pub name_en: String,
    pub year_blurb: String,
    pub description: String,
    pub required_credits: i32,
    pub course_count_label: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct DepartmentRow {
    pub id: i32,
    pub slug: String,
    pub name_fa: String,
    pub name_en: String,
    pub code_prefix: String,
    pub accent: String,
    pub description: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct CourseRow {
    pub id: i32,
    pub slug: String,
    pub code: String,
    pub title: String,
    pub summary: String,
    pub department_id: i32,
    pub level_id: i32,
    pub year: i16,
    pub credits: i32,
    pub term_in_year: i16,
    pub kind: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct LessonSummaryRow {
    pub id: i32,
    pub course_id: i32,
    pub sort_order: i32,
    pub title: String,
    pub estimated_minutes: i32,
    pub related_terms: Vec<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct LessonRow {
    pub id: i32,
    pub course_id: i32,
    pub sort_order: i32,
    pub title: String,
    pub estimated_minutes: i32,
    pub what: String,
    pub why: String,
    pub simple_example: serde_json::Value,
    pub real_example: serde_json::Value,
    pub mistakes: Vec<String>,
    pub best_practices: Vec<String>,
    pub related_terms: Vec<String>,
    pub try_it: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct SkillRow {
    pub id: i32,
    pub course_id: i32,
    pub name: String,
}

pub async fn levels(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<LevelRow>> {
    sqlx::query_as("SELECT * FROM levels ORDER BY year").fetch_all(db).await
}

pub async fn departments(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<DepartmentRow>> {
    sqlx::query_as("SELECT id, slug, name_fa, name_en, code_prefix, accent, description FROM departments ORDER BY sort_order")
        .fetch_all(db)
        .await
}

const COURSE_COLUMNS: &str = "c.id, c.slug, c.code, c.title, c.summary, c.department_id, c.level_id, l.year, \
     c.credits, c.term_in_year, c.kind, c.sort_order";

pub async fn courses(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<CourseRow>> {
    sqlx::query_as(&format!(
        "SELECT {COURSE_COLUMNS} FROM courses c JOIN levels l ON l.id = c.level_id \
         ORDER BY l.year, c.term_in_year, c.sort_order"
    ))
    .fetch_all(db)
    .await
}

/// (course_id, prerequisite_id) pairs.
pub async fn prerequisites(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<(i32, i32)>> {
    sqlx::query_as("SELECT course_id, prerequisite_id FROM course_prerequisites").fetch_all(db).await
}

pub async fn lesson_summaries(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<LessonSummaryRow>> {
    sqlx::query_as(
        "SELECT id, course_id, sort_order, title, estimated_minutes, related_terms FROM lessons ORDER BY course_id, sort_order",
    )
    .fetch_all(db)
    .await
}

pub async fn lesson(db: impl PgExecutor<'_>, id: i32) -> sqlx::Result<Option<LessonRow>> {
    sqlx::query_as("SELECT * FROM lessons WHERE id = $1").bind(id).fetch_optional(db).await
}

pub async fn skills(db: impl PgExecutor<'_>) -> sqlx::Result<Vec<SkillRow>> {
    sqlx::query_as("SELECT id, course_id, name FROM skills ORDER BY course_id, sort_order").fetch_all(db).await
}

#[derive(Debug, Clone, FromRow)]
pub struct CatalogCounts {
    pub levels: i64,
    pub departments: i64,
    pub courses: i64,
    pub projects: i64,
    pub lessons: i64,
    pub credits: i64,
    pub terms: i64,
    pub students: i64,
}

pub async fn counts(db: impl PgExecutor<'_>) -> sqlx::Result<CatalogCounts> {
    sqlx::query_as(
        "SELECT (SELECT count(*) FROM levels) AS levels,
                (SELECT count(*) FROM departments) AS departments,
                (SELECT count(*) FROM courses WHERE kind = 'course') AS courses,
                (SELECT count(*) FROM courses WHERE kind = 'project') AS projects,
                (SELECT count(*) FROM lessons) AS lessons,
                (SELECT COALESCE(sum(credits), 0)::bigint FROM courses) AS credits,
                (SELECT count(*) FROM semesters) AS terms,
                (SELECT count(*) FROM users) AS students",
    )
    .fetch_one(db)
    .await
}
