//! Rule-based academic advisor: compares departments by grades and recent
//! study time and suggests where to focus this week.

use super::templates::Templates;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct DepartmentSignal {
    pub slug: String,
    pub name: String,
    /// Weighted average of finished courses, if any.
    pub average_grade: Option<f64>,
    /// Study minutes in the last 14 days.
    pub recent_minutes: i32,
    /// Whether the student currently has open courses here.
    pub has_open_courses: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Advice {
    pub message: String,
    pub focus_department: Option<String>,
    pub tone: &'static str,
}

/// `has_history` is false for a brand-new student (no activity at all yet).
pub fn advise(signals: &[DepartmentSignal], streak: u32, studied_today: bool, has_history: bool, t: &Templates) -> Advice {
    if !has_history {
        return Advice { message: t.render("advisor.start", &[]), focus_department: None, tone: "start" };
    }
    if streak == 0 && !studied_today {
        return Advice { message: t.render("advisor.nudge", &[]), focus_department: None, tone: "nudge" };
    }

    let graded: Vec<&DepartmentSignal> = signals.iter().filter(|s| s.average_grade.is_some()).collect();
    let strongest = graded
        .iter()
        .max_by(|a, b| a.average_grade.partial_cmp(&b.average_grade).unwrap())
        .copied();

    // Weakest: among departments with open courses, lowest grade first, then least recent time.
    let weakest = signals
        .iter()
        .filter(|s| s.has_open_courses)
        .min_by(|a, b| {
            let ga = a.average_grade.unwrap_or(20.0);
            let gb = b.average_grade.unwrap_or(20.0);
            ga.partial_cmp(&gb).unwrap().then(a.recent_minutes.cmp(&b.recent_minutes))
        });

    match (strongest, weakest) {
        (Some(strong), Some(weak)) if strong.slug != weak.slug => Advice {
            message: t.render("advisor.focus", &[("strong", &strong.name), ("weak", &weak.name)]),
            focus_department: Some(weak.slug.clone()),
            tone: "focus",
        },
        (_, Some(weak)) => Advice {
            message: t.render("advisor.steady", &[("weak", &weak.name)]),
            focus_department: Some(weak.slug.clone()),
            tone: "steady",
        },
        _ => Advice {
            message: t.render("advisor.celebrate", &[]),
            focus_department: None,
            tone: "celebrate",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::templates::test_templates;

    fn s(slug: &str, grade: Option<f64>, minutes: i32, open: bool) -> DepartmentSignal {
        DepartmentSignal { slug: slug.into(), name: slug.to_uppercase(), average_grade: grade, recent_minutes: minutes, has_open_courses: open }
    }

    #[test]
    fn recommends_weakest_open_department() {
        let signals = vec![s("se", Some(18.0), 200, true), s("flutter", Some(15.5), 60, true), s("dart", Some(17.0), 90, true)];
        let advice = advise(&signals, 10, true, true, &test_templates());
        assert_eq!(advice.focus_department.as_deref(), Some("flutter"));
        assert_eq!(advice.message, "در SE خوبی؛ روی FLUTTER تمرکز کن.");
    }

    #[test]
    fn nudges_when_streak_is_broken() {
        let advice = advise(&[s("se", Some(18.0), 0, true)], 0, false, true, &test_templates());
        assert_eq!(advice.tone, "nudge");
    }

    #[test]
    fn ties_on_grade_break_by_recent_time() {
        let signals = vec![s("a", None, 100, true), s("b", None, 20, true), s("c", Some(19.0), 0, false)];
        assert_eq!(advise(&signals, 3, true, true, &test_templates()).focus_department.as_deref(), Some("b"));
    }

    #[test]
    fn welcomes_new_students() {
        assert_eq!(advise(&[], 0, false, false, &test_templates()).tone, "start");
    }

    #[test]
    fn celebrates_when_nothing_is_open() {
        assert_eq!(advise(&[s("a", Some(18.0), 0, false)], 5, true, true, &test_templates()).tone, "celebrate");
    }
}
