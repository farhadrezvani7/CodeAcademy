//! "How much time do you have today?" — rule-based study suggestion.

use super::templates::Templates;
use serde::Serialize;

pub const ALLOWED_MINUTES: [i32; 3] = [10, 30, 60];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionKind {
    ReviewConcept,
    StudyLesson,
    StudyAndPractice,
}

impl SuggestionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReviewConcept => "review_concept",
            Self::StudyLesson => "study_lesson",
            Self::StudyAndPractice => "study_and_practice",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NextLesson {
    pub id: i32,
    pub title: String,
    pub course_title: String,
}

#[derive(Debug, Clone)]
pub struct ReviewTerm {
    pub slug: String,
    pub name_en: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub kind: SuggestionKind,
    pub title: String,
    pub description: String,
    pub lesson_id: Option<i32>,
    pub term_slug: Option<String>,
}

pub fn is_allowed(minutes: i32) -> bool {
    ALLOWED_MINUTES.contains(&minutes)
}

pub fn suggest(minutes: i32, next: Option<&NextLesson>, review: Option<&ReviewTerm>, t: &Templates) -> Suggestion {
    let review_suggestion = || match review {
        Some(term) => Suggestion {
            kind: SuggestionKind::ReviewConcept,
            title: t.render("checkin.review.title", &[("term", &term.name_en)]),
            description: t.render("checkin.review.body", &[("term", &term.name_en)]),
            lesson_id: None,
            term_slug: Some(term.slug.clone()),
        },
        None => Suggestion {
            kind: SuggestionKind::ReviewConcept,
            title: t.render("checkin.review_any.title", &[]),
            description: t.render("checkin.review_any.body", &[]),
            lesson_id: None,
            term_slug: None,
        },
    };
    let lesson_suggestion = |kind: SuggestionKind, key: &str, lesson: &NextLesson| Suggestion {
        kind,
        title: t.render(&format!("{key}.title"), &[("lesson", &lesson.title), ("course", &lesson.course_title)]),
        description: t.render(&format!("{key}.body"), &[("lesson", &lesson.title), ("course", &lesson.course_title)]),
        lesson_id: Some(lesson.id),
        term_slug: None,
    };

    match (minutes, next) {
        (m, _) if m < 30 => review_suggestion(),
        (m, Some(lesson)) if m < 60 => lesson_suggestion(SuggestionKind::StudyLesson, "checkin.lesson", lesson),
        (_, Some(lesson)) => lesson_suggestion(SuggestionKind::StudyAndPractice, "checkin.practice", lesson),
        (_, None) => review_suggestion(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::templates::test_templates;

    fn lesson() -> NextLesson {
        NextLesson { id: 12, title: "Stream".into(), course_title: "برنامه‌نویسی ناهمزمان".into() }
    }

    #[test]
    fn ten_minutes_reviews_a_concept() {
        let term = ReviewTerm { slug: "future".into(), name_en: "Future".into() };
        let s = suggest(10, Some(&lesson()), Some(&term), &test_templates());
        assert_eq!(s.kind, SuggestionKind::ReviewConcept);
        assert_eq!(s.term_slug.as_deref(), Some("future"));
    }

    #[test]
    fn thirty_minutes_studies_a_lesson() {
        let s = suggest(30, Some(&lesson()), None, &test_templates());
        assert_eq!(s.kind, SuggestionKind::StudyLesson);
        assert_eq!(s.lesson_id, Some(12));
    }

    #[test]
    fn sixty_minutes_studies_and_practices() {
        assert_eq!(suggest(60, Some(&lesson()), None, &test_templates()).kind, SuggestionKind::StudyAndPractice);
    }

    #[test]
    fn falls_back_to_review_without_lessons() {
        assert_eq!(suggest(60, None, None, &test_templates()).kind, SuggestionKind::ReviewConcept);
    }

    #[test]
    fn only_three_options_allowed() {
        assert!(is_allowed(30));
        assert!(!is_allowed(45));
    }
}
