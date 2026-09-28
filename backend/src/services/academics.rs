//! Read models for the academic structure: levels, learning path,
//! departments, courses and the skill tree — all annotated with the
//! student's real enrollment state.

use super::snapshot::Snapshot;
use crate::domain::{
    percent,
    progression::{PromotionEvaluation, RuleDefinition},
    EnrollmentStatus, NodeState,
};
use crate::error::{AppError, AppResult};
use crate::repo::catalog::{self, CourseRow, DepartmentRow};
use crate::repo::content;
use crate::state::AppState;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseLink {
    pub slug: String,
    pub code: String,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseCard {
    pub id: i32,
    pub slug: String,
    pub code: String,
    pub title: String,
    pub summary: String,
    pub credits: i32,
    pub kind: String,
    pub year: i16,
    pub term_in_year: i16,
    pub level_name: String,
    pub department_slug: String,
    pub department_name: String,
    pub status: EnrollmentStatus,
    pub grade: Option<f64>,
    pub passed: bool,
    /// Finished with a failing grade; lessons must be redone.
    pub needs_retake: bool,
    pub lessons_done: usize,
    pub lesson_count: usize,
    pub percent: i32,
    pub prerequisites: Vec<CourseLink>,
    pub skills: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LessonItem {
    pub id: i32,
    pub order: i32,
    pub title: String,
    pub estimated_minutes: i32,
    pub completed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CourseDetail {
    #[serde(flatten)]
    pub card: CourseCard,
    pub lessons: Vec<LessonItem>,
    pub next_lesson_id: Option<i32>,
}

pub fn course_card(s: &Snapshot, c: &CourseRow) -> CourseCard {
    let dept = s.catalog.department(c.department_id);
    let (done, total) = s.lesson_progress(c.id);
    CourseCard {
        id: c.id,
        slug: c.slug.clone(),
        code: c.code.clone(),
        title: c.title.clone(),
        summary: c.summary.clone(),
        credits: c.credits,
        kind: c.kind.clone(),
        year: c.year,
        term_in_year: c.term_in_year,
        level_name: s.catalog.level_by_year(c.year).map(|l| l.name_fa.clone()).unwrap_or_default(),
        department_slug: dept.map(|d| d.slug.clone()).unwrap_or_default(),
        department_name: dept.map(|d| d.name_fa.clone()).unwrap_or_default(),
        status: s.student.status(c.id),
        grade: s.student.grade(c.id),
        passed: s.is_passed(c.id),
        needs_retake: s.needs_retake(c.id),
        lessons_done: done,
        lesson_count: total,
        percent: percent(done as i64, total as i64),
        prerequisites: s
            .catalog
            .prerequisites_of(c.id)
            .into_iter()
            .map(|p| CourseLink { slug: p.slug.clone(), code: p.code.clone(), title: p.title.clone(), done: s.is_passed(p.id) })
            .collect(),
        skills: s.catalog.skills.iter().filter(|sk| sk.course_id == c.id).map(|sk| sk.name.clone()).collect(),
    }
}

pub fn course_detail(s: &Snapshot, c: &CourseRow) -> CourseDetail {
    let done = s.student.completed_lesson_ids();
    CourseDetail {
        card: course_card(s, c),
        lessons: s
            .catalog
            .lessons_of(c.id)
            .map(|l| LessonItem { id: l.id, order: l.sort_order, title: l.title.clone(), estimated_minutes: l.estimated_minutes, completed: done.contains(&l.id) })
            .collect(),
        next_lesson_id: s.next_lesson(c.id).map(|l| l.id),
    }
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StageState {
    Passed,
    Current,
    Upcoming,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDto {
    pub year: i16,
    pub slug: String,
    pub name_fa: String,
    pub name_en: String,
    pub year_blurb: String,
    pub description: String,
    pub required_credits: i32,
    pub course_count: usize,
    pub course_count_label: String,
    pub project: Option<CourseLink>,
    pub state: StageState,
    pub credits_passed: i64,
    pub percent: i32,
    pub courses: Vec<CourseLink>,
    pub evaluation: Option<PromotionEvaluation>,
}

fn levels_for(s: &Snapshot) -> Vec<LevelDto> {
    let current = s.year();
    s.catalog
        .levels
        .iter()
        .map(|l| {
            let state = if l.year < current || s.student.is_graduated() {
                StageState::Passed
            } else if l.year == current {
                StageState::Current
            } else {
                StageState::Upcoming
            };
            let (passed, _) = s.year_credits(l.year);
            let link = |c: &CourseRow| CourseLink { slug: c.slug.clone(), code: c.code.clone(), title: c.title.clone(), done: s.is_passed(c.id) };
            LevelDto {
                year: l.year,
                slug: l.slug.clone(),
                name_fa: l.name_fa.clone(),
                name_en: l.name_en.clone(),
                year_blurb: l.year_blurb.clone(),
                description: l.description.clone(),
                required_credits: l.required_credits,
                course_count: s.catalog.courses_of_year(l.year).filter(|c| c.kind == "course").count(),
                course_count_label: l.course_count_label.clone(),
                project: s.catalog.year_project(l.year).map(link),
                state,
                credits_passed: passed,
                percent: percent(passed, l.required_credits as i64),
                courses: s.catalog.courses_of_year(l.year).filter(|c| c.kind == "course").map(link).collect(),
                evaluation: (state != StageState::Upcoming).then(|| s.promotion(l.year)),
            }
        })
        .collect()
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelsResponse {
    pub current_year: i16,
    pub rules: Vec<RuleDefinition>,
    pub levels: Vec<LevelDto>,
}

pub async fn levels(state: &AppState, user_id: i32) -> AppResult<LevelsResponse> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    Ok(LevelsResponse { current_year: s.year(), rules: s.catalog.rules.clone(), levels: levels_for(&s) })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearningPathResponse {
    pub current_year: i16,
    pub current_term: i16,
    pub graduated: bool,
    pub credits_passed: i64,
    pub total_credits: i64,
    pub percent: i32,
    pub stages: Vec<LevelDto>,
}

pub async fn learning_path(state: &AppState, user_id: i32) -> AppResult<LearningPathResponse> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let passed = s.passed_credits(|_| true);
    let total = s.catalog.total_credits();
    Ok(LearningPathResponse {
        current_year: s.year(),
        current_term: s.student.user.current_term,
        graduated: s.student.is_graduated(),
        credits_passed: passed,
        total_credits: total,
        percent: percent(passed, total),
        stages: levels_for(&s),
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentDto {
    pub slug: String,
    pub name_fa: String,
    pub name_en: String,
    pub code_prefix: String,
    pub accent: String,
    pub description: String,
    pub course_count: usize,
    pub project_count: usize,
    pub credits: i64,
    pub credits_passed: i64,
    pub percent: i32,
}

fn department_dto(s: &Snapshot, d: &DepartmentRow) -> DepartmentDto {
    let courses: Vec<&CourseRow> = s.catalog.courses.iter().filter(|c| c.department_id == d.id).collect();
    let credits: i64 = courses.iter().map(|c| c.credits as i64).sum();
    let passed = s.passed_credits(|c| c.department_id == d.id);
    DepartmentDto {
        slug: d.slug.clone(),
        name_fa: d.name_fa.clone(),
        name_en: d.name_en.clone(),
        code_prefix: d.code_prefix.clone(),
        accent: d.accent.clone(),
        description: d.description.clone(),
        course_count: courses.iter().filter(|c| c.kind == "course").count(),
        project_count: courses.iter().filter(|c| c.kind == "project").count(),
        credits,
        credits_passed: passed,
        percent: percent(passed, credits),
    }
}

pub async fn departments(state: &AppState, user_id: i32) -> AppResult<Vec<DepartmentDto>> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    Ok(s.catalog.departments.iter().map(|d| department_dto(&s, d)).collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DepartmentDetail {
    #[serde(flatten)]
    pub department: DepartmentDto,
    pub courses: Vec<CourseDetail>,
}

pub async fn department(state: &AppState, user_id: i32, slug: &str) -> AppResult<DepartmentDetail> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let d = s.catalog.department_by_slug(slug).ok_or(AppError::NotFound("دانشکده"))?;
    Ok(DepartmentDetail {
        department: department_dto(&s, d),
        courses: s.catalog.courses.iter().filter(|c| c.department_id == d.id).map(|c| course_detail(&s, c)).collect(),
    })
}

pub async fn courses(state: &AppState, user_id: i32, department: Option<&str>) -> AppResult<Vec<CourseCard>> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let dept_id = match department {
        Some(slug) => Some(s.catalog.department_by_slug(slug).ok_or(AppError::NotFound("دانشکده"))?.id),
        None => None,
    };
    Ok(s.catalog.courses.iter().filter(|c| dept_id.is_none_or(|d| c.department_id == d)).map(|c| course_card(&s, c)).collect())
}

pub async fn course(state: &AppState, user_id: i32, slug: &str) -> AppResult<CourseDetail> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    let c = s.catalog.course_by_slug(slug).ok_or(AppError::NotFound("درس"))?;
    Ok(course_detail(&s, c))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillNode {
    pub course_slug: String,
    pub code: String,
    pub title: String,
    pub year: i16,
    pub state: NodeState,
    pub grade: Option<f64>,
    pub skills: Vec<String>,
    pub prerequisites: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillBranch {
    pub department_slug: String,
    pub name_fa: String,
    pub accent: String,
    pub done: usize,
    pub total: usize,
    pub percent: i32,
    pub nodes: Vec<SkillNode>,
}

pub async fn skill_tree(state: &AppState, user_id: i32) -> AppResult<Vec<SkillBranch>> {
    let mut conn = state.pool.acquire().await?;
    let s = Snapshot::load(&mut conn, user_id).await?;
    Ok(s.catalog
        .departments
        .iter()
        .map(|d| {
            let nodes: Vec<SkillNode> = s
                .catalog
                .courses
                .iter()
                .filter(|c| c.department_id == d.id)
                .map(|c| SkillNode {
                    course_slug: c.slug.clone(),
                    code: c.code.clone(),
                    title: c.title.clone(),
                    year: c.year,
                    state: s.node_state(c.id),
                    grade: s.student.grade(c.id),
                    skills: s.catalog.skills.iter().filter(|sk| sk.course_id == c.id).map(|sk| sk.name.clone()).collect(),
                    prerequisites: s.catalog.prerequisites_of(c.id).into_iter().map(|p| p.slug.clone()).collect(),
                })
                .collect();
            let done = nodes.iter().filter(|n| n.state == NodeState::Done).count();
            SkillBranch {
                department_slug: d.slug.clone(),
                name_fa: d.name_fa.clone(),
                accent: d.accent.clone(),
                done,
                total: nodes.len(),
                percent: percent(done as i64, nodes.len() as i64),
                nodes,
            }
        })
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewLevel {
    pub year: i16,
    pub name_fa: String,
    pub name_en: String,
    pub year_blurb: String,
    pub required_credits: i32,
    pub course_count_label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewDepartment {
    pub slug: String,
    pub name_fa: String,
    pub name_en: String,
    pub accent: String,
    pub description: String,
    pub course_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewStats {
    pub years: i64,
    pub departments: i64,
    pub courses: i64,
    pub projects: i64,
    pub lessons: i64,
    pub credits: i64,
    pub terms: i64,
    pub students: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Highlight {
    pub section: String,
    pub icon: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Overview {
    pub stats: OverviewStats,
    pub highlights: Vec<Highlight>,
    pub levels: Vec<OverviewLevel>,
    pub departments: Vec<OverviewDepartment>,
}

/// Public landing-page data; needs no student.
pub async fn overview(state: &AppState) -> AppResult<Overview> {
    let mut conn = state.pool.acquire().await?;
    let counts = catalog::counts(&mut *conn).await?;
    let cat = super::snapshot::Catalog::load(&mut conn).await?;
    let highlights = content::highlights(&mut *conn)
        .await?
        .into_iter()
        .map(|h| Highlight { section: h.section, icon: h.icon, title: h.title, body: h.body })
        .collect();
    Ok(Overview {
        highlights,
        stats: OverviewStats {
            years: counts.levels,
            departments: counts.departments,
            courses: counts.courses,
            projects: counts.projects,
            lessons: counts.lessons,
            credits: counts.credits,
            terms: counts.terms,
            students: counts.students,
        },
        levels: cat
            .levels
            .iter()
            .map(|l| OverviewLevel {
                year: l.year,
                name_fa: l.name_fa.clone(),
                name_en: l.name_en.clone(),
                year_blurb: l.year_blurb.clone(),
                required_credits: l.required_credits,
                course_count_label: l.course_count_label.clone(),
            })
            .collect(),
        departments: cat
            .departments
            .iter()
            .map(|d| OverviewDepartment {
                slug: d.slug.clone(),
                name_fa: d.name_fa.clone(),
                name_en: d.name_en.clone(),
                accent: d.accent.clone(),
                description: d.description.clone(),
                course_count: cat.courses.iter().filter(|c| c.department_id == d.id && c.kind == "course").count() as i64,
            })
            .collect(),
    })
}
