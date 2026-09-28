//! Level promotion rules.
//!
//! A student is promoted out of a year when all of these hold:
//! 1. every course of that year is passed with at least 14/20,
//! 2. the year's final project is delivered,
//! 3. no course (of any year so far) is more than one term behind schedule.

use super::{gpa, CourseKind, EnrollmentStatus};
use serde::Serialize;

pub const TERMS_PER_YEAR: i16 = 2;
pub const TOTAL_YEARS: i16 = 6;

#[derive(Debug, Clone)]
pub struct CourseProgress {
    pub code: String,
    pub kind: CourseKind,
    pub year: i16,
    pub term_in_year: i16,
    pub status: EnrollmentStatus,
    pub grade: Option<f64>,
}

/// Rule logic is fixed in code and addressed by key; titles and descriptions
/// come from the `promotion_rules` table.
pub const RULE_MIN_GRADE: &str = "min_grade";
pub const RULE_PROJECT: &str = "project_delivered";
pub const RULE_NOT_BEHIND: &str = "not_behind";
pub const RULE_KEYS: [&str; 3] = [RULE_MIN_GRADE, RULE_PROJECT, RULE_NOT_BEHIND];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDefinition {
    pub key: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleResult {
    pub key: String,
    pub title: String,
    pub met: bool,
    /// Codes of courses blocking this rule.
    pub blocking: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionEvaluation {
    pub year: i16,
    pub eligible: bool,
    pub rules: Vec<RuleResult>,
}

/// Absolute term number (1..=12) in which a course is scheduled.
pub fn expected_term(year: i16, term_in_year: i16) -> i16 {
    (year - 1) * TERMS_PER_YEAR + term_in_year
}

pub fn is_behind(current_term: i16, course: &CourseProgress) -> bool {
    course.status != EnrollmentStatus::Done && expected_term(course.year, course.term_in_year) < current_term - 1
}

/// `courses` should contain every course the student has in years `<= year`.
pub fn evaluate(year: i16, current_term: i16, courses: &[CourseProgress], definitions: &[RuleDefinition]) -> PromotionEvaluation {
    let this_year: Vec<&CourseProgress> = courses.iter().filter(|c| c.year == year).collect();

    let grade_blockers: Vec<String> = this_year
        .iter()
        .filter(|c| c.kind == CourseKind::Course)
        .filter(|c| c.status != EnrollmentStatus::Done || !c.grade.is_some_and(gpa::is_passing))
        .map(|c| c.code.clone())
        .collect();

    let projects: Vec<&&CourseProgress> = this_year.iter().filter(|c| c.kind == CourseKind::Project).collect();
    let project_blockers: Vec<String> = projects
        .iter()
        .filter(|c| c.status != EnrollmentStatus::Done || !c.grade.is_some_and(gpa::is_passing))
        .map(|c| c.code.clone())
        .collect();
    let project_met = !projects.is_empty() && project_blockers.is_empty();

    let behind: Vec<String> = courses
        .iter()
        .filter(|c| c.year <= year && is_behind(current_term, c))
        .map(|c| c.code.clone())
        .collect();

    let title = |key: &str| definitions.iter().find(|d| d.key == key).map(|d| d.title.clone()).unwrap_or_else(|| key.to_string());
    let rules = vec![
        RuleResult { key: RULE_MIN_GRADE.into(), title: title(RULE_MIN_GRADE), met: grade_blockers.is_empty(), blocking: grade_blockers },
        RuleResult { key: RULE_PROJECT.into(), title: title(RULE_PROJECT), met: project_met, blocking: project_blockers },
        RuleResult { key: RULE_NOT_BEHIND.into(), title: title(RULE_NOT_BEHIND), met: behind.is_empty(), blocking: behind },
    ];
    PromotionEvaluation { year, eligible: rules.iter().all(|r| r.met), rules }
}

/// The term a student sits in: the scheduled term of the earliest unfinished
/// course of their current year, never moving backwards.
pub fn current_term(year: i16, previous_term: i16, courses: &[CourseProgress]) -> i16 {
    let earliest_open = courses
        .iter()
        .filter(|c| c.year == year && c.status != EnrollmentStatus::Done)
        .map(|c| expected_term(c.year, c.term_in_year))
        .min()
        .unwrap_or(expected_term(year, TERMS_PER_YEAR));
    earliest_open.max(previous_term).clamp(1, TOTAL_YEARS * TERMS_PER_YEAR)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(code: &str, kind: CourseKind, year: i16, term: i16, status: EnrollmentStatus, grade: Option<f64>) -> CourseProgress {
        CourseProgress { code: code.into(), kind, year, term_in_year: term, status, grade }
    }

    fn done(code: &str, year: i16, term: i16, grade: f64) -> CourseProgress {
        c(code, CourseKind::Course, year, term, EnrollmentStatus::Done, Some(grade))
    }

    fn project(year: i16, status: EnrollmentStatus, grade: Option<f64>) -> CourseProgress {
        c("P", CourseKind::Project, year, 2, status, grade)
    }

    #[test]
    fn expected_term_numbers() {
        assert_eq!(expected_term(1, 1), 1);
        assert_eq!(expected_term(3, 2), 6);
        assert_eq!(expected_term(6, 2), 12);
    }

    #[test]
    fn titles_come_from_definitions() {
        let defs = vec![RuleDefinition { key: RULE_PROJECT.into(), title: "تحویل پروژه".into(), description: String::new() }];
        let eval = evaluate(2, 4, &[], &defs);
        assert_eq!(eval.rules[1].title, "تحویل پروژه");
        assert_eq!(eval.rules[0].title, RULE_MIN_GRADE);
        assert!(!eval.eligible, "a year without a delivered project cannot pass");
    }

    #[test]
    fn eligible_when_all_rules_met() {
        let courses = vec![done("A", 2, 1, 15.0), done("B", 2, 2, 18.0), project(2, EnrollmentStatus::Done, Some(17.0))];
        let eval = evaluate(2, 4, &courses, &[]);
        assert!(eval.eligible);
    }

    #[test]
    fn grade_below_14_blocks() {
        let courses = vec![done("A", 2, 1, 13.5), project(2, EnrollmentStatus::Done, Some(17.0))];
        let eval = evaluate(2, 4, &courses, &[]);
        assert!(!eval.eligible);
        assert_eq!(eval.rules[0].blocking, vec!["A".to_string()]);
    }

    #[test]
    fn unfinished_course_blocks_grade_rule() {
        let courses = vec![c("A", CourseKind::Course, 2, 1, EnrollmentStatus::InProgress, None), project(2, EnrollmentStatus::Done, Some(17.0))];
        assert!(!evaluate(2, 3, &courses, &[]).rules[0].met);
    }

    #[test]
    fn project_must_be_delivered() {
        let courses = vec![done("A", 2, 1, 19.0), project(2, EnrollmentStatus::InProgress, None)];
        let eval = evaluate(2, 4, &courses, &[]);
        assert!(eval.rules[0].met);
        assert!(!eval.rules[1].met);
        assert!(!eval.eligible);
    }

    #[test]
    fn more_than_one_term_behind_blocks() {
        // A term-1 course of year 2 (term 3) still open while the student is in term 5.
        let courses = vec![
            c("OLD", CourseKind::Course, 2, 1, EnrollmentStatus::InProgress, None),
            done("A", 3, 1, 18.0),
            project(3, EnrollmentStatus::Done, Some(18.0)),
        ];
        let eval = evaluate(3, 5, &courses, &[]);
        assert!(!eval.rules[2].met);
        assert_eq!(eval.rules[2].blocking, vec!["OLD".to_string()]);
    }

    #[test]
    fn exactly_one_term_behind_is_allowed() {
        let course = c("X", CourseKind::Course, 2, 2, EnrollmentStatus::InProgress, None);
        assert!(!is_behind(5, &course));
        assert!(is_behind(6, &course));
    }

    #[test]
    fn current_term_follows_earliest_open_course() {
        let courses = vec![done("A", 3, 1, 18.0), c("B", CourseKind::Course, 3, 2, EnrollmentStatus::Enrolled, None)];
        assert_eq!(current_term(3, 5, &courses), 6);
        let courses = vec![c("A", CourseKind::Course, 3, 1, EnrollmentStatus::InProgress, None)];
        assert_eq!(current_term(3, 5, &courses), 5);
        // never goes backwards
        assert_eq!(current_term(3, 6, &courses), 6);
    }
}
