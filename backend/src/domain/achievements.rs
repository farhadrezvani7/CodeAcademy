//! Achievement and certificate unlock rules, evaluated against a snapshot of
//! facts about the student.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct StudentFacts {
    pub completed_courses: HashSet<String>,
    /// Lessons completed per course slug.
    pub lessons_completed: HashMap<String, usize>,
    pub longest_streak: u32,
    pub has_night_activity: bool,
    pub total_xp: i32,
    /// Highest year fully passed (0 = none).
    pub passed_year: i16,
    pub graduated: bool,
}

impl StudentFacts {
    fn done(&self, slug: &str) -> bool {
        self.completed_courses.contains(slug)
    }

    fn total_lessons(&self) -> usize {
        self.lessons_completed.values().sum()
    }
}

/// Night owl window, in local hours: [00:00, 05:00).
pub const NIGHT_HOURS: std::ops::Range<u32> = 0..5;
pub const STREAK_GOAL: u32 = 21;

/// Achievement slug → unlock rule.
pub fn is_unlocked(slug: &str, f: &StudentFacts) -> bool {
    match slug {
        "first-lesson" => f.total_lessons() > 0,
        "pattern-solver" => f.done("se-310"),
        "streak-21" => f.longest_streak >= STREAK_GOAL,
        "first-async-project" => f.done("flutter-390"),
        "novice-tester" => f.lessons_completed.get("se-320").copied().unwrap_or(0) > 0,
        "night-owl" => f.has_night_activity,
        "code-reviewer" => f.done("pro-400"),
        "system-architect" => f.done("se-510"),
        "xp-1000" => f.total_xp >= 1000,
        "graduate" => f.graduated,
        _ => false,
    }
}

/// Certificate slug → issue rule.
pub fn is_certified(slug: &str, f: &StudentFacts) -> bool {
    match slug {
        "year-two" => f.passed_year >= 2,
        "graduation" => f.graduated,
        _ => false,
    }
}

/// Slugs from `candidates` that should be unlocked but are not yet.
pub fn newly_unlocked<'a>(candidates: &'a [String], already: &HashSet<String>, rule: impl Fn(&str) -> bool) -> Vec<&'a str> {
    candidates
        .iter()
        .map(String::as_str)
        .filter(|s| !already.contains(*s) && rule(s))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_based_achievements() {
        let mut f = StudentFacts::default();
        assert!(!is_unlocked("pattern-solver", &f));
        f.completed_courses.insert("se-310".into());
        assert!(is_unlocked("pattern-solver", &f));
    }

    #[test]
    fn streak_needs_21_days() {
        let mut f = StudentFacts { longest_streak: 20, ..Default::default() };
        assert!(!is_unlocked("streak-21", &f));
        f.longest_streak = 21;
        assert!(is_unlocked("streak-21", &f));
    }

    #[test]
    fn novice_tester_after_first_testing_lesson() {
        let mut f = StudentFacts::default();
        f.lessons_completed.insert("se-101".into(), 3);
        assert!(!is_unlocked("novice-tester", &f));
        assert!(is_unlocked("first-lesson", &f));
        f.lessons_completed.insert("se-320".into(), 1);
        assert!(is_unlocked("novice-tester", &f));
    }

    #[test]
    fn certificates() {
        let f = StudentFacts { passed_year: 2, ..Default::default() };
        assert!(is_certified("year-two", &f));
        assert!(!is_certified("graduation", &f));
    }

    #[test]
    fn unknown_slugs_stay_locked() {
        assert!(!is_unlocked("nope", &StudentFacts { graduated: true, ..Default::default() }));
    }

    #[test]
    fn newly_unlocked_skips_existing() {
        let f = StudentFacts { has_night_activity: true, longest_streak: 30, ..Default::default() };
        let candidates = vec!["night-owl".to_string(), "streak-21".to_string(), "graduate".to_string()];
        let already: HashSet<String> = ["night-owl".to_string()].into();
        assert_eq!(newly_unlocked(&candidates, &already, |s| is_unlocked(s, &f)), vec!["streak-21"]);
    }
}
