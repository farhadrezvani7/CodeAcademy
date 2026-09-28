//! A consistent in-memory view of the catalog plus one student's state.
//! The catalog is small (tens of courses), so loading it per request keeps
//! every rule computation simple and free of N+1 queries.

use crate::domain::{
    gpa, percent,
    progression::{self, CourseProgress, PromotionEvaluation, RuleDefinition},
    templates::Templates,
    CourseKind, EnrollmentStatus, NodeState,
};
use crate::repo::content::{self, AdvisorRow};
use crate::repo::{
    catalog::{self, CourseRow, DepartmentRow, LessonSummaryRow, LevelRow, SkillRow},
    student::{self, CompletionRow, EnrollmentRow, UserRow},
};
use crate::error::{AppError, AppResult};
use sqlx::PgConnection;
use std::collections::{HashMap, HashSet};

pub struct Catalog {
    pub levels: Vec<LevelRow>,
    pub departments: Vec<DepartmentRow>,
    pub courses: Vec<CourseRow>,
    pub prerequisites: HashMap<i32, Vec<i32>>,
    pub lessons: Vec<LessonSummaryRow>,
    pub skills: Vec<SkillRow>,
    pub rules: Vec<RuleDefinition>,
    pub score_options: Vec<(f64, String)>,
    pub templates: Templates,
    pub term_names: HashMap<String, String>,
    pub advisors: Vec<AdvisorRow>,
}

impl Catalog {
    pub async fn load(db: &mut PgConnection) -> sqlx::Result<Self> {
        let levels = catalog::levels(&mut *db).await?;
        let departments = catalog::departments(&mut *db).await?;
        let courses = catalog::courses(&mut *db).await?;
        let mut prerequisites: HashMap<i32, Vec<i32>> = HashMap::new();
        for (course, pre) in catalog::prerequisites(&mut *db).await? {
            prerequisites.entry(course).or_default().push(pre);
        }
        let lessons = catalog::lesson_summaries(&mut *db).await?;
        let skills = catalog::skills(&mut *db).await?;
        let rules = content::promotion_rules(&mut *db)
            .await?
            .into_iter()
            .map(|(key, title, description)| RuleDefinition { key, title, description })
            .collect();
        let score_options = content::score_options(&mut *db).await?;
        let templates = Templates::new(content::templates(&mut *db).await?);
        let term_names = content::term_names(&mut *db).await?.into_iter().collect();
        let advisors = content::advisors(&mut *db).await?;
        Ok(Self { levels, departments, courses, prerequisites, lessons, skills, rules, score_options, templates, term_names, advisors })
    }

    pub fn advisor_for(&self, department_id: i32) -> Option<&AdvisorRow> {
        self.advisors.iter().find(|a| a.department_id == department_id).or_else(|| self.advisors.first())
    }

    pub fn course(&self, id: i32) -> Option<&CourseRow> {
        self.courses.iter().find(|c| c.id == id)
    }

    pub fn course_by_slug(&self, slug: &str) -> Option<&CourseRow> {
        self.courses.iter().find(|c| c.slug == slug)
    }

    pub fn level_by_year(&self, year: i16) -> Option<&LevelRow> {
        self.levels.iter().find(|l| l.year == year)
    }

    pub fn department(&self, id: i32) -> Option<&DepartmentRow> {
        self.departments.iter().find(|d| d.id == id)
    }

    pub fn department_by_slug(&self, slug: &str) -> Option<&DepartmentRow> {
        self.departments.iter().find(|d| d.slug == slug)
    }

    pub fn lessons_of(&self, course_id: i32) -> impl Iterator<Item = &LessonSummaryRow> {
        self.lessons.iter().filter(move |l| l.course_id == course_id)
    }

    pub fn prerequisites_of(&self, course_id: i32) -> Vec<&CourseRow> {
        self.prerequisites
            .get(&course_id)
            .map(|ids| ids.iter().filter_map(|id| self.course(*id)).collect())
            .unwrap_or_default()
    }

    pub fn courses_of_year(&self, year: i16) -> impl Iterator<Item = &CourseRow> {
        self.courses.iter().filter(move |c| c.year == year)
    }

    pub fn total_credits(&self) -> i64 {
        self.courses.iter().map(|c| c.credits as i64).sum()
    }

    pub fn year_project(&self, year: i16) -> Option<&CourseRow> {
        self.courses_of_year(year).find(|c| c.kind == "project")
    }
}

pub struct Student {
    pub user: UserRow,
    pub enrollments: HashMap<i32, EnrollmentRow>,
    pub completions: Vec<CompletionRow>,
}

impl Student {
    pub async fn load(db: &mut PgConnection, user_id: i32) -> AppResult<Self> {
        let user = student::user(&mut *db, user_id).await?.ok_or(AppError::NotFound("دانشجو"))?;
        let enrollments = student::enrollments(&mut *db, user_id).await?.into_iter().map(|e| (e.course_id, e)).collect();
        let completions = student::completions(&mut *db, user_id).await?;
        Ok(Self { user, enrollments, completions })
    }

    pub fn status(&self, course_id: i32) -> EnrollmentStatus {
        self.enrollments
            .get(&course_id)
            .and_then(|e| EnrollmentStatus::parse(&e.status))
            .unwrap_or(EnrollmentStatus::Locked)
    }

    pub fn grade(&self, course_id: i32) -> Option<f64> {
        self.enrollments.get(&course_id).and_then(|e| e.grade)
    }

    pub fn completion(&self, lesson_id: i32) -> Option<&CompletionRow> {
        self.completions.iter().find(|c| c.lesson_id == lesson_id)
    }

    pub fn completed_lesson_ids(&self) -> HashSet<i32> {
        self.completions.iter().map(|c| c.lesson_id).collect()
    }

    pub fn is_graduated(&self) -> bool {
        self.user.graduated_at.is_some()
    }
}

pub struct Snapshot {
    pub catalog: Catalog,
    pub student: Student,
}

impl Snapshot {
    pub async fn load(db: &mut PgConnection, user_id: i32) -> AppResult<Self> {
        let catalog = Catalog::load(&mut *db).await?;
        let student = Student::load(&mut *db, user_id).await?;
        Ok(Self { catalog, student })
    }

    pub fn year(&self) -> i16 {
        self.student.user.year
    }

    pub fn is_passed(&self, course_id: i32) -> bool {
        self.student.status(course_id) == EnrollmentStatus::Done && self.student.grade(course_id).is_some_and(gpa::is_passing)
    }

    pub fn node_state(&self, course_id: i32) -> NodeState {
        NodeState::of(self.student.status(course_id), self.is_passed(course_id))
    }

    /// Finished but below the passing grade: must be redone.
    pub fn needs_retake(&self, course_id: i32) -> bool {
        self.student.status(course_id) == EnrollmentStatus::Done && !self.is_passed(course_id)
    }

    pub fn course_progress(&self, course: &CourseRow) -> CourseProgress {
        CourseProgress {
            code: course.code.clone(),
            kind: CourseKind::parse(&course.kind),
            year: course.year,
            term_in_year: course.term_in_year,
            status: self.student.status(course.id),
            grade: self.student.grade(course.id),
        }
    }

    pub fn progress_up_to(&self, year: i16) -> Vec<CourseProgress> {
        self.catalog.courses.iter().filter(|c| c.year <= year).map(|c| self.course_progress(c)).collect()
    }

    pub fn promotion(&self, year: i16) -> PromotionEvaluation {
        progression::evaluate(year, self.student.user.current_term, &self.progress_up_to(year), &self.catalog.rules)
    }

    /// (done lessons, total lessons) of a course.
    pub fn lesson_progress(&self, course_id: i32) -> (usize, usize) {
        let done = self.student.completed_lesson_ids();
        let lessons: Vec<i32> = self.catalog.lessons_of(course_id).map(|l| l.id).collect();
        (lessons.iter().filter(|id| done.contains(id)).count(), lessons.len())
    }

    pub fn course_percent(&self, course_id: i32) -> i32 {
        let (done, total) = self.lesson_progress(course_id);
        percent(done as i64, total as i64)
    }

    /// Credits of passed courses matching `filter`.
    pub fn passed_credits(&self, filter: impl Fn(&CourseRow) -> bool) -> i64 {
        self.catalog.courses.iter().filter(|c| filter(c) && self.is_passed(c.id)).map(|c| c.credits as i64).sum()
    }

    pub fn year_credits(&self, year: i16) -> (i64, i64) {
        let total = self.catalog.courses_of_year(year).map(|c| c.credits as i64).sum();
        (self.passed_credits(|c| c.year == year), total)
    }

    /// Credit-weighted GPA over finished courses matching `filter`.
    pub fn gpa(&self, filter: impl Fn(&CourseRow) -> bool) -> Option<f64> {
        let graded: Vec<gpa::GradedCourse> = self
            .catalog
            .courses
            .iter()
            .filter(|c| filter(c) && self.student.status(c.id) == EnrollmentStatus::Done)
            .filter_map(|c| self.student.grade(c.id).map(|grade| gpa::GradedCourse { credits: c.credits, grade }))
            .collect();
        gpa::weighted_gpa(&graded)
    }

    /// Highest year fully passed.
    pub fn passed_year(&self) -> i16 {
        if self.student.is_graduated() {
            progression::TOTAL_YEARS
        } else {
            self.year() - 1
        }
    }

    /// Courses the student can work on now, most urgent first.
    pub fn open_courses(&self) -> Vec<&CourseRow> {
        let mut open: Vec<&CourseRow> = self
            .catalog
            .courses
            .iter()
            .filter(|c| matches!(self.student.status(c.id), EnrollmentStatus::Enrolled | EnrollmentStatus::InProgress) || self.needs_retake(c.id))
            .collect();
        open.sort_by_key(|c| {
            let started = self.student.status(c.id) != EnrollmentStatus::Enrolled;
            (progression::expected_term(c.year, c.term_in_year), !started, c.sort_order)
        });
        open
    }

    /// The course the student should focus on: started (or failed) before merely enrolled.
    pub fn focus_course(&self) -> Option<&CourseRow> {
        let open = self.open_courses();
        open.iter()
            .find(|c| self.student.status(c.id) != EnrollmentStatus::Enrolled)
            .or_else(|| open.first())
            .copied()
    }

    /// Next unfinished lesson; for a failed course, the lowest-scored one to redo.
    pub fn next_lesson(&self, course_id: i32) -> Option<&LessonSummaryRow> {
        let done = self.student.completed_lesson_ids();
        if let Some(l) = self.catalog.lessons_of(course_id).find(|l| !done.contains(&l.id)) {
            return Some(l);
        }
        if self.needs_retake(course_id) {
            return self.catalog.lessons_of(course_id).min_by(|a, b| {
                let sa = self.student.completion(a.id).map(|c| c.self_score).unwrap_or(0.0);
                let sb = self.student.completion(b.id).map(|c| c.self_score).unwrap_or(0.0);
                sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        None
    }

    /// True once the student has done anything at all.
    pub fn has_history(&self) -> bool {
        !self.student.completions.is_empty()
    }
}
