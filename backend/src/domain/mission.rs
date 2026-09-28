//! Daily mission planning and progress.

use super::{fa, percent, templates::Templates, xp};
use chrono::{Datelike, NaiveDate, Weekday};

#[derive(Debug, Clone, PartialEq)]
pub struct MissionPlan {
    pub description: String,
    pub course_id: Option<i32>,
    pub target_minutes: i32,
    pub xp_reward: i32,
}

#[derive(Debug, Clone)]
pub struct FocusCourse {
    pub id: i32,
    pub title: String,
}

/// Friday is the Iranian weekend, so the mission is a little lighter.
pub fn target_minutes(date: NaiveDate) -> i32 {
    if date.weekday() == Weekday::Fri {
        20
    } else {
        30
    }
}

/// Plans today's mission around the course the student is actively taking.
pub fn plan(focus: Option<&FocusCourse>, date: NaiveDate, t: &Templates) -> MissionPlan {
    let minutes = target_minutes(date);
    let m = fa::digits(minutes.to_string());
    match focus {
        Some(course) => MissionPlan {
            description: t.render("mission.course", &[("minutes", &m), ("course", &course.title)]),
            course_id: Some(course.id),
            target_minutes: minutes,
            xp_reward: xp::DAILY_MISSION_XP,
        },
        None => MissionPlan {
            description: t.render("mission.general", &[("minutes", &m)]),
            course_id: None,
            target_minutes: minutes,
            xp_reward: xp::DAILY_MISSION_XP,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub minutes: i32,
    pub percent: i32,
    pub completed: bool,
}

pub fn progress(target_minutes: i32, minutes_spent: i32) -> Progress {
    let minutes = minutes_spent.clamp(0, target_minutes.max(0));
    Progress {
        minutes,
        percent: percent(minutes as i64, target_minutes as i64),
        completed: target_minutes > 0 && minutes_spent >= target_minutes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn plans_around_focus_course() {
        let focus = FocusCourse { id: 4, title: "معماری تمیز".into() };
        let plan = plan(Some(&focus), d("2026-09-28"), &crate::domain::templates::test_templates()); // Monday
        assert_eq!(plan.description, "امروز ۳۰ دقیقه روی درس معماری تمیز کار کن.");
        assert_eq!(plan.course_id, Some(4));
        assert_eq!(plan.xp_reward, 50);
    }

    #[test]
    fn friday_is_lighter_and_fallback_without_course() {
        let plan = plan(None, d("2026-10-02"), &crate::domain::templates::test_templates()); // Friday
        assert_eq!(plan.target_minutes, 20);
        assert_eq!(plan.course_id, None);
        assert!(plan.description.contains("۲۰"));
    }

    #[test]
    fn progress_caps_and_completes() {
        assert_eq!(progress(30, 15), Progress { minutes: 15, percent: 50, completed: false });
        assert_eq!(progress(30, 45), Progress { minutes: 30, percent: 100, completed: true });
        assert_eq!(progress(30, 0).percent, 0);
    }
}
