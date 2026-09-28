//! Academic content: loads `seed/*.json` (embedded at build time) into the
//! database. The sync is idempotent and keyed by stable identifiers (slugs,
//! years, lesson order), so updating content never touches student data.
//! It only runs when the content bundle changed since the last sync.

use crate::domain::{course_status, progression, EnrollmentStatus};
use crate::services::snapshot::Snapshot;
use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{PgConnection, PgPool};
use std::collections::HashMap;

const CATALOG: &str = include_str!("../../seed/catalog.json");
const DICTIONARY: &str = include_str!("../../seed/dictionary.json");
const CONTENT: &str = include_str!("../../seed/content.json");
const LESSON_FILES: [&str; 5] = [
    include_str!("../../seed/lessons/se-part1.json"),
    include_str!("../../seed/lessons/se-part2.json"),
    include_str!("../../seed/lessons/dart-pro.json"),
    include_str!("../../seed/lessons/flutter-part1.json"),
    include_str!("../../seed/lessons/flutter-part2.json"),
];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogFile {
    levels: Vec<LevelSeed>,
    departments: Vec<DepartmentSeed>,
    courses: Vec<CourseSeed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LevelSeed {
    year: i16,
    slug: String,
    name_fa: String,
    name_en: String,
    required_credits: i32,
    course_count_label: String,
    year_blurb: String,
    description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DepartmentSeed {
    slug: String,
    name_fa: String,
    name_en: String,
    code_prefix: String,
    accent: String,
    description: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CourseSeed {
    slug: String,
    code: String,
    title: String,
    department: String,
    year: i16,
    term: i16,
    credits: i32,
    kind: String,
    prerequisites: Vec<String>,
    skills: Vec<String>,
    summary: String,
}

#[derive(Deserialize)]
struct LessonFile {
    courses: Vec<CourseLessons>,
}

#[derive(Deserialize)]
struct CourseLessons {
    course: String,
    lessons: Vec<LessonSeed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LessonSeed {
    title: String,
    estimated_minutes: i32,
    what: String,
    why: String,
    simple_example: serde_json::Value,
    real_example: serde_json::Value,
    mistakes: Vec<String>,
    best_practices: Vec<String>,
    related_terms: Vec<String>,
    try_it: String,
}

#[derive(Deserialize)]
struct DictionaryFile {
    terms: Vec<TermSeed>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TermSeed {
    slug: String,
    name_en: String,
    name_fa: String,
    category: String,
    definition: String,
    #[serde(default)]
    example: String,
    related: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentFile {
    advisors: Vec<AdvisorSeed>,
    promotion_rules: Vec<RuleSeed>,
    score_options: Vec<ScoreSeed>,
    templates: HashMap<String, String>,
    highlights: Vec<HighlightSeed>,
    achievements: Vec<AwardSeed>,
    certificates: Vec<CertificateSeed>,
    announcements: Vec<AnnouncementSeed>,
}

#[derive(Deserialize)]
struct AdvisorSeed {
    department: String,
    name: String,
    title: String,
    bio: String,
}

#[derive(Deserialize)]
struct RuleSeed {
    key: String,
    title: String,
    description: String,
}

#[derive(Deserialize)]
struct ScoreSeed {
    score: f64,
    label: String,
}

#[derive(Deserialize)]
struct HighlightSeed {
    section: String,
    icon: String,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct AwardSeed {
    slug: String,
    title: String,
    description: String,
    icon: String,
}

#[derive(Deserialize)]
struct CertificateSeed {
    slug: String,
    title: String,
    description: String,
}

#[derive(Deserialize)]
struct AnnouncementSeed {
    title: String,
    body: String,
    priority: String,
}

const TERM_NAMES: [&str; 2] = ["نیمسال اول", "نیمسال دوم"];
const YEAR_NAMES: [&str; 6] = ["اول", "دوم", "سوم", "چهارم", "پنجم", "ششم"];

fn bundle_hash() -> String {
    let mut h = Sha256::new();
    for part in [CATALOG, DICTIONARY, CONTENT].into_iter().chain(LESSON_FILES) {
        h.update(part.as_bytes());
        h.update([0u8]);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Applies the embedded content if it differs from what the database has.
/// Returns true when a sync happened.
pub async fn sync(pool: &PgPool) -> Result<bool> {
    let hash = bundle_hash();
    let current: Option<String> = sqlx::query_scalar("SELECT value FROM app_meta WHERE key = 'content_hash'").fetch_optional(pool).await?;
    if current.as_deref() == Some(hash.as_str()) {
        return Ok(false);
    }
    let mut tx = pool.begin().await?;
    // Serialize concurrent starts.
    sqlx::query("SELECT pg_advisory_xact_lock(73110)").execute(&mut *tx).await?;
    apply(&mut tx).await.context("applying content")?;
    reconcile_students(&mut tx).await.context("reconciling student progress")?;
    sqlx::query("INSERT INTO app_meta (key, value) VALUES ('content_hash', $1) ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value")
        .bind(&hash)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(true)
}

async fn apply(db: &mut PgConnection) -> Result<()> {
    let catalog: CatalogFile = serde_json::from_str(CATALOG).context("catalog.json")?;
    let dictionary: DictionaryFile = serde_json::from_str(DICTIONARY).context("dictionary.json")?;
    let content: ContentFile = serde_json::from_str(CONTENT).context("content.json")?;

    let mut level_ids = HashMap::new();
    for l in &catalog.levels {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO levels (year, slug, name_fa, name_en, year_blurb, description, required_credits, course_count_label)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (year) DO UPDATE SET slug = EXCLUDED.slug, name_fa = EXCLUDED.name_fa, name_en = EXCLUDED.name_en,
                 year_blurb = EXCLUDED.year_blurb, description = EXCLUDED.description,
                 required_credits = EXCLUDED.required_credits, course_count_label = EXCLUDED.course_count_label
             RETURNING id",
        )
        .bind(l.year)
        .bind(&l.slug)
        .bind(&l.name_fa)
        .bind(&l.name_en)
        .bind(&l.year_blurb)
        .bind(&l.description)
        .bind(l.required_credits)
        .bind(&l.course_count_label)
        .fetch_one(&mut *db)
        .await?;
        level_ids.insert(l.year, id);
    }

    let mut department_ids = HashMap::new();
    for (i, d) in catalog.departments.iter().enumerate() {
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO departments (slug, name_fa, name_en, code_prefix, accent, description, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (slug) DO UPDATE SET name_fa = EXCLUDED.name_fa, name_en = EXCLUDED.name_en,
                 code_prefix = EXCLUDED.code_prefix, accent = EXCLUDED.accent, description = EXCLUDED.description,
                 sort_order = EXCLUDED.sort_order
             RETURNING id",
        )
        .bind(&d.slug)
        .bind(&d.name_fa)
        .bind(&d.name_en)
        .bind(&d.code_prefix)
        .bind(&d.accent)
        .bind(&d.description)
        .bind(i as i32)
        .fetch_one(&mut *db)
        .await?;
        department_ids.insert(d.slug.clone(), id);
    }

    let mut course_ids: HashMap<String, i32> = HashMap::new();
    for (i, c) in catalog.courses.iter().enumerate() {
        let dept = *department_ids.get(&c.department).with_context(|| format!("department {}", c.department))?;
        let level = *level_ids.get(&c.year).with_context(|| format!("level {}", c.year))?;
        let id: i32 = sqlx::query_scalar(
            "INSERT INTO courses (slug, code, title, summary, department_id, level_id, credits, term_in_year, kind, sort_order)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (slug) DO UPDATE SET code = EXCLUDED.code, title = EXCLUDED.title, summary = EXCLUDED.summary,
                 department_id = EXCLUDED.department_id, level_id = EXCLUDED.level_id, credits = EXCLUDED.credits,
                 term_in_year = EXCLUDED.term_in_year, kind = EXCLUDED.kind, sort_order = EXCLUDED.sort_order
             RETURNING id",
        )
        .bind(&c.slug)
        .bind(&c.code)
        .bind(&c.title)
        .bind(&c.summary)
        .bind(dept)
        .bind(level)
        .bind(c.credits)
        .bind(c.term)
        .bind(&c.kind)
        .bind(i as i32)
        .fetch_one(&mut *db)
        .await?;
        course_ids.insert(c.slug.clone(), id);
    }

    sqlx::query("DELETE FROM course_prerequisites").execute(&mut *db).await?;
    sqlx::query("DELETE FROM skills").execute(&mut *db).await?;
    for c in &catalog.courses {
        let id = course_ids[&c.slug];
        for p in &c.prerequisites {
            let pre = *course_ids.get(p).with_context(|| format!("prerequisite {p} of {}", c.slug))?;
            sqlx::query("INSERT INTO course_prerequisites (course_id, prerequisite_id) VALUES ($1, $2)")
                .bind(id)
                .bind(pre)
                .execute(&mut *db)
                .await?;
        }
        for (j, skill) in c.skills.iter().enumerate() {
            sqlx::query("INSERT INTO skills (department_id, course_id, name, sort_order) VALUES ($1, $2, $3, $4)")
                .bind(department_ids[&c.department])
                .bind(id)
                .bind(skill)
                .bind(j as i32)
                .execute(&mut *db)
                .await?;
        }
    }

    let mut with_lessons = std::collections::HashSet::new();
    for raw in LESSON_FILES {
        let file: LessonFile = serde_json::from_str(raw).context("lesson file")?;
        for cl in file.courses {
            let course_id = *course_ids.get(&cl.course).with_context(|| format!("lessons for unknown course {}", cl.course))?;
            anyhow::ensure!(!cl.lessons.is_empty(), "course {} has no lessons", cl.course);
            with_lessons.insert(course_id);
            for (i, l) in cl.lessons.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO lessons (course_id, sort_order, title, estimated_minutes, what, why, simple_example,
                                          real_example, mistakes, best_practices, related_terms, try_it)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                     ON CONFLICT (course_id, sort_order) DO UPDATE SET title = EXCLUDED.title,
                         estimated_minutes = EXCLUDED.estimated_minutes, what = EXCLUDED.what, why = EXCLUDED.why,
                         simple_example = EXCLUDED.simple_example, real_example = EXCLUDED.real_example,
                         mistakes = EXCLUDED.mistakes, best_practices = EXCLUDED.best_practices,
                         related_terms = EXCLUDED.related_terms, try_it = EXCLUDED.try_it",
                )
                .bind(course_id)
                .bind(i as i32 + 1)
                .bind(&l.title)
                .bind(l.estimated_minutes)
                .bind(&l.what)
                .bind(&l.why)
                .bind(&l.simple_example)
                .bind(&l.real_example)
                .bind(&l.mistakes)
                .bind(&l.best_practices)
                .bind(&l.related_terms)
                .bind(&l.try_it)
                .execute(&mut *db)
                .await?;
            }
            let removed = sqlx::query("DELETE FROM lessons WHERE course_id = $1 AND sort_order > $2")
                .bind(course_id)
                .bind(cl.lessons.len() as i32)
                .execute(&mut *db)
                .await?
                .rows_affected();
            if removed > 0 {
                tracing::warn!(course = cl.course, removed, "lessons removed by content update");
            }
        }
    }
    if let Some((slug, _)) = course_ids.iter().find(|(_, id)| !with_lessons.contains(id)) {
        anyhow::bail!("course {slug} has no lessons");
    }

    let slugs: Vec<String> = dictionary.terms.iter().map(|t| t.slug.clone()).collect();
    for t in &dictionary.terms {
        sqlx::query(
            "INSERT INTO dictionary_terms (slug, name_en, name_fa, category, definition, example, related)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (slug) DO UPDATE SET name_en = EXCLUDED.name_en, name_fa = EXCLUDED.name_fa,
                 category = EXCLUDED.category, definition = EXCLUDED.definition, example = EXCLUDED.example,
                 related = EXCLUDED.related",
        )
        .bind(&t.slug)
        .bind(&t.name_en)
        .bind(&t.name_fa)
        .bind(&t.category)
        .bind(&t.definition)
        .bind(&t.example)
        .bind(&t.related)
        .execute(&mut *db)
        .await?;
    }
    sqlx::query("DELETE FROM dictionary_terms WHERE slug <> ALL($1)").bind(&slugs).execute(&mut *db).await?;

    for number in 1..=(progression::TOTAL_YEARS * progression::TERMS_PER_YEAR) {
        let year = (number - 1) / 2 + 1;
        let term = (number - 1) % 2 + 1;
        let title = format!("{} — سال {}", TERM_NAMES[(term - 1) as usize], YEAR_NAMES[(year - 1) as usize]);
        sqlx::query(
            "INSERT INTO semesters (number, year, term_in_year, title_fa) VALUES ($1, $2, $3, $4)
             ON CONFLICT (number) DO UPDATE SET title_fa = EXCLUDED.title_fa",
        )
        .bind(number)
        .bind(year)
        .bind(term)
        .bind(title)
        .execute(&mut *db)
        .await?;
    }

    for a in &content.advisors {
        let dept = *department_ids.get(&a.department).with_context(|| format!("advisor department {}", a.department))?;
        sqlx::query(
            "INSERT INTO advisors (name, title, department_id, bio) VALUES ($1, $2, $3, $4)
             ON CONFLICT (department_id) DO UPDATE SET name = EXCLUDED.name, title = EXCLUDED.title, bio = EXCLUDED.bio",
        )
        .bind(&a.name)
        .bind(&a.title)
        .bind(dept)
        .bind(&a.bio)
        .execute(&mut *db)
        .await?;
    }

    let keys: Vec<String> = content.promotion_rules.iter().map(|r| r.key.clone()).collect();
    for key in progression::RULE_KEYS {
        anyhow::ensure!(keys.iter().any(|k| k == key), "promotion rule {key} missing from content.json");
    }
    for (i, r) in content.promotion_rules.iter().enumerate() {
        sqlx::query(
            "INSERT INTO promotion_rules (key, title, description, sort_order) VALUES ($1, $2, $3, $4)
             ON CONFLICT (key) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description, sort_order = EXCLUDED.sort_order",
        )
        .bind(&r.key)
        .bind(&r.title)
        .bind(&r.description)
        .bind(i as i32)
        .execute(&mut *db)
        .await?;
    }
    sqlx::query("DELETE FROM promotion_rules WHERE key <> ALL($1)").bind(&keys).execute(&mut *db).await?;

    anyhow::ensure!(!content.score_options.is_empty(), "scoreOptions must not be empty");
    let scores: Vec<f64> = content.score_options.iter().map(|s| s.score).collect();
    for (i, s) in content.score_options.iter().enumerate() {
        sqlx::query(
            "INSERT INTO score_options (score, label, sort_order) VALUES ($1, $2, $3)
             ON CONFLICT (score) DO UPDATE SET label = EXCLUDED.label, sort_order = EXCLUDED.sort_order",
        )
        .bind(s.score)
        .bind(&s.label)
        .bind(i as i32)
        .execute(&mut *db)
        .await?;
    }
    sqlx::query("DELETE FROM score_options WHERE score <> ALL($1)").bind(&scores).execute(&mut *db).await?;

    let template_keys: Vec<String> = content.templates.keys().cloned().collect();
    for (key, body) in &content.templates {
        sqlx::query("INSERT INTO message_templates (key, body) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET body = EXCLUDED.body")
            .bind(key)
            .bind(body)
            .execute(&mut *db)
            .await?;
    }
    sqlx::query("DELETE FROM message_templates WHERE key <> ALL($1)").bind(&template_keys).execute(&mut *db).await?;

    sqlx::query("DELETE FROM site_highlights").execute(&mut *db).await?;
    for (i, h) in content.highlights.iter().enumerate() {
        sqlx::query("INSERT INTO site_highlights (section, icon, title, body, sort_order) VALUES ($1, $2, $3, $4, $5)")
            .bind(&h.section)
            .bind(&h.icon)
            .bind(&h.title)
            .bind(&h.body)
            .bind(i as i32)
            .execute(&mut *db)
            .await?;
    }

    for (i, a) in content.achievements.iter().enumerate() {
        sqlx::query(
            "INSERT INTO achievements (slug, title, description, icon, sort_order) VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (slug) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description,
                 icon = EXCLUDED.icon, sort_order = EXCLUDED.sort_order",
        )
        .bind(&a.slug)
        .bind(&a.title)
        .bind(&a.description)
        .bind(&a.icon)
        .bind(i as i32)
        .execute(&mut *db)
        .await?;
    }
    for (i, c) in content.certificates.iter().enumerate() {
        sqlx::query(
            "INSERT INTO certificates (slug, title, description, sort_order) VALUES ($1, $2, $3, $4)
             ON CONFLICT (slug) DO UPDATE SET title = EXCLUDED.title, description = EXCLUDED.description, sort_order = EXCLUDED.sort_order",
        )
        .bind(&c.slug)
        .bind(&c.title)
        .bind(&c.description)
        .bind(i as i32)
        .execute(&mut *db)
        .await?;
    }
    // Announcements are only added (never overwritten), so ones published
    // directly in the database are kept.
    for a in &content.announcements {
        sqlx::query(
            "INSERT INTO announcements (title, body, priority, published_at)
             SELECT $1, $2, $3, now() WHERE NOT EXISTS (SELECT 1 FROM announcements WHERE title = $1)",
        )
        .bind(&a.title)
        .bind(&a.body)
        .bind(&a.priority)
        .execute(&mut *db)
        .await?;
    }
    Ok(())
}

/// After lessons change, recompute each started course's status and grade
/// from the student's actual lesson completions.
async fn reconcile_students(db: &mut PgConnection) -> Result<()> {
    let users: Vec<i32> = sqlx::query_scalar("SELECT id FROM users").fetch_all(&mut *db).await?;
    for user_id in users {
        let s = Snapshot::load(&mut *db, user_id).await.map_err(|e| anyhow::anyhow!("{e}"))?;
        for c in &s.catalog.courses {
            let status = s.student.status(c.id);
            if matches!(status, EnrollmentStatus::Locked | EnrollmentStatus::Enrolled) && s.lesson_progress(c.id).0 == 0 {
                continue;
            }
            let scores: Vec<f64> = s.catalog.lessons_of(c.id).filter_map(|l| s.student.completion(l.id).map(|x| x.self_score)).collect();
            let (_, total) = s.lesson_progress(c.id);
            let new_status = course_status::status_for_progress(scores.len(), total);
            let grade = (new_status == EnrollmentStatus::Done).then(|| course_status::course_grade(&scores)).flatten();
            if new_status != status || grade != s.student.grade(c.id) {
                sqlx::query(
                    "UPDATE enrollments SET status = $3, grade = $4,
                         completed_at = CASE WHEN $3 = 'done' THEN COALESCE(completed_at, now()) END
                     WHERE user_id = $1 AND course_id = $2",
                )
                .bind(user_id)
                .bind(c.id)
                .bind(new_status.as_str())
                .bind(grade)
                .execute(&mut *db)
                .await?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_content_parses_and_is_consistent() {
        let catalog: CatalogFile = serde_json::from_str(CATALOG).unwrap();
        let content: ContentFile = serde_json::from_str(CONTENT).unwrap();
        let dictionary: DictionaryFile = serde_json::from_str(DICTIONARY).unwrap();
        let slugs: std::collections::HashSet<&str> = dictionary.terms.iter().map(|t| t.slug.as_str()).collect();
        let mut lesson_courses = std::collections::HashSet::new();
        for raw in LESSON_FILES {
            let file: LessonFile = serde_json::from_str(raw).unwrap();
            for c in file.courses {
                assert!((3..=4).contains(&c.lessons.len()), "{} lesson count", c.course);
                for l in &c.lessons {
                    for t in &l.related_terms {
                        assert!(slugs.contains(t.as_str()), "{}: unknown term {t}", c.course);
                    }
                }
                lesson_courses.insert(c.course);
            }
        }
        for c in &catalog.courses {
            assert!(lesson_courses.contains(&c.slug), "{} has no lessons", c.slug);
        }
        for year in 1..=6 {
            let credits: i32 = catalog.courses.iter().filter(|c| c.year == year).map(|c| c.credits).sum();
            let level = catalog.levels.iter().find(|l| l.year == year).unwrap();
            assert_eq!(credits, level.required_credits, "year {year} credits");
        }
        for key in progression::RULE_KEYS {
            assert!(content.promotion_rules.iter().any(|r| r.key == key));
        }
        for key in [
            "mission.course", "mission.general", "checkin.review.title", "checkin.review.body", "checkin.review_any.title",
            "checkin.review_any.body", "checkin.lesson.title", "checkin.lesson.body", "checkin.practice.title",
            "checkin.practice.body", "advisor.start", "advisor.nudge", "advisor.focus", "advisor.steady", "advisor.celebrate",
        ] {
            assert!(content.templates.contains_key(key), "template {key} missing");
        }
        assert_eq!(content.advisors.len(), catalog.departments.len());
    }
}
