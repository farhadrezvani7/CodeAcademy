//! Course lifecycle: lesson progress → status and grade, and which locked
//! courses become available once prerequisites are finished.

use super::{gpa, EnrollmentStatus};
use std::collections::HashSet;

/// Status after `completed` of `total` lessons are finished.
pub fn status_for_progress(completed: usize, total: usize) -> EnrollmentStatus {
    if total > 0 && completed >= total {
        EnrollmentStatus::Done
    } else if completed > 0 {
        EnrollmentStatus::InProgress
    } else {
        EnrollmentStatus::Enrolled
    }
}

/// Course grade: mean of the lesson self-assessment scores, rounded to 2 decimals.
pub fn course_grade(lesson_scores: &[f64]) -> Option<f64> {
    if lesson_scores.is_empty() {
        None
    } else {
        Some(gpa::round2(lesson_scores.iter().sum::<f64>() / lesson_scores.len() as f64))
    }
}

/// A self-assessment score is valid only if it is one of the configured options.
pub fn is_valid_self_score(score: f64, options: &[f64]) -> bool {
    options.iter().any(|s| (*s - score).abs() < f64::EPSILON)
}

#[derive(Debug, Clone)]
pub struct Unlockable {
    pub course_id: i32,
    pub year: i16,
    pub status: EnrollmentStatus,
    /// Done with a passing grade.
    pub passed: bool,
    pub prerequisites: Vec<i32>,
}

/// Locked courses up to `current_year` whose prerequisites are all *passed*
/// (finished with a grade of at least 14).
pub fn newly_unlocked(courses: &[Unlockable], current_year: i16) -> Vec<i32> {
    let done: HashSet<i32> = courses.iter().filter(|c| c.passed).map(|c| c.course_id).collect();
    courses
        .iter()
        .filter(|c| c.status == EnrollmentStatus::Locked && c.year <= current_year)
        .filter(|c| c.prerequisites.iter().all(|p| done.contains(p)))
        .map(|c| c.course_id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_from_progress() {
        assert_eq!(status_for_progress(0, 4), EnrollmentStatus::Enrolled);
        assert_eq!(status_for_progress(2, 4), EnrollmentStatus::InProgress);
        assert_eq!(status_for_progress(4, 4), EnrollmentStatus::Done);
        assert_eq!(status_for_progress(0, 0), EnrollmentStatus::Enrolled);
    }

    #[test]
    fn grade_is_mean_of_scores() {
        assert_eq!(course_grade(&[20.0, 17.0, 17.0]), Some(18.0));
        assert_eq!(course_grade(&[20.0, 17.0, 14.0, 17.0]), Some(17.0));
        assert_eq!(course_grade(&[]), None);
    }

    #[test]
    fn only_listed_scores_are_valid() {
        let options = [20.0, 17.0, 14.0, 10.0];
        assert!(is_valid_self_score(17.0, &options));
        assert!(!is_valid_self_score(19.0, &options));
    }

    #[test]
    fn unlocks_when_prerequisites_done() {
        let courses = vec![
            Unlockable { course_id: 1, year: 3, status: EnrollmentStatus::Done, passed: true, prerequisites: vec![] },
            Unlockable { course_id: 2, year: 3, status: EnrollmentStatus::Locked, passed: false, prerequisites: vec![1] },
            Unlockable { course_id: 3, year: 3, status: EnrollmentStatus::Locked, passed: false, prerequisites: vec![1, 4] },
            Unlockable { course_id: 4, year: 3, status: EnrollmentStatus::InProgress, passed: false, prerequisites: vec![] },
            Unlockable { course_id: 5, year: 4, status: EnrollmentStatus::Locked, passed: false, prerequisites: vec![1] },
        ];
        assert_eq!(newly_unlocked(&courses, 3), vec![2]);
        assert_eq!(newly_unlocked(&courses, 4), vec![2, 5]);
    }

    #[test]
    fn failed_prerequisite_keeps_course_locked() {
        let courses = vec![
            Unlockable { course_id: 1, year: 1, status: EnrollmentStatus::Done, passed: false, prerequisites: vec![] },
            Unlockable { course_id: 2, year: 1, status: EnrollmentStatus::Locked, passed: false, prerequisites: vec![1] },
        ];
        assert!(newly_unlocked(&courses, 1).is_empty());
    }
}
