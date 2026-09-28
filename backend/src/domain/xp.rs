//! Experience-point rules. Awards are recorded in the `xp_events` ledger and
//! keyed by (reason, ref) so the same accomplishment is never paid twice.

use super::CourseKind;

pub const LESSON_XP: i32 = 25;
pub const DAILY_MISSION_XP: i32 = 50;
pub const YEAR_PROJECT_XP: i32 = 150;
pub const FINAL_PROJECT_XP: i32 = 1000;
pub const FINAL_YEAR: i16 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XpAward {
    pub reason: &'static str,
    pub reference: String,
    pub amount: i32,
}

pub fn for_lesson(lesson_id: i32) -> XpAward {
    XpAward { reason: "lesson_completed", reference: format!("lesson:{lesson_id}"), amount: LESSON_XP }
}

pub fn for_mission(mission_id: i32) -> XpAward {
    XpAward { reason: "daily_mission", reference: format!("mission:{mission_id}"), amount: DAILY_MISSION_XP }
}

/// Bonus for finishing a course. Only projects earn one; the graduation
/// project (year 6) earns the special reward.
pub fn for_course_completion(course_id: i32, kind: CourseKind, year: i16) -> Option<XpAward> {
    match kind {
        CourseKind::Course => None,
        CourseKind::Project if year == FINAL_YEAR => Some(XpAward {
            reason: "final_project",
            reference: format!("course:{course_id}"),
            amount: FINAL_PROJECT_XP,
        }),
        CourseKind::Project => Some(XpAward {
            reason: "year_project",
            reference: format!("course:{course_id}"),
            amount: YEAR_PROJECT_XP,
        }),
    }
}

/// Sums a ledger. Negative totals are clamped (the ledger never debits today,
/// but the column is constrained to be non-negative).
pub fn total(awards: &[i32]) -> i32 {
    awards.iter().sum::<i32>().max(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lesson_is_25() {
        assert_eq!(for_lesson(7).amount, 25);
        assert_eq!(for_lesson(7).reference, "lesson:7");
    }

    #[test]
    fn mission_is_50() {
        assert_eq!(for_mission(3).amount, 50);
    }

    #[test]
    fn only_projects_earn_completion_bonus() {
        assert_eq!(for_course_completion(1, CourseKind::Course, 3), None);
        assert_eq!(for_course_completion(1, CourseKind::Project, 2).unwrap().amount, YEAR_PROJECT_XP);
    }

    #[test]
    fn final_project_earns_special_reward() {
        let award = for_course_completion(9, CourseKind::Project, 6).unwrap();
        assert_eq!(award.reason, "final_project");
        assert_eq!(award.amount, FINAL_PROJECT_XP);
    }

    #[test]
    fn total_sums_ledger() {
        assert_eq!(total(&[25, 25, 50]), 100);
        assert_eq!(total(&[]), 0);
    }
}
