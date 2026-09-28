//! Message templates stored in the database (`message_templates`).
//! Placeholders are written as `{name}`.

use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Templates(HashMap<String, String>);

impl Templates {
    pub fn new(entries: impl IntoIterator<Item = (String, String)>) -> Self {
        Self(entries.into_iter().collect())
    }

    /// Renders `key` with `args`. A missing template renders as an empty
    /// string (and is logged) rather than leaking the key to users.
    pub fn render(&self, key: &str, args: &[(&str, &str)]) -> String {
        let Some(body) = self.0.get(key) else {
            tracing::warn!(template = key, "missing message template");
            return String::new();
        };
        let mut out = body.clone();
        for (name, value) in args {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        out
    }
}

#[cfg(test)]
pub fn test_templates() -> Templates {
    Templates::new(
        [
            ("mission.course", "امروز {minutes} دقیقه روی درس {course} کار کن."),
            ("mission.general", "امروز {minutes} دقیقه مفاهیم دانشنامه را مرور کن."),
            ("checkin.review.title", "مرور مفهوم {term}"),
            ("checkin.review.body", "مرور"),
            ("checkin.review_any.title", "مرور یک مفهوم"),
            ("checkin.review_any.body", "مرور آزاد"),
            ("checkin.lesson.title", "مطالعه‌ی «{lesson}»"),
            ("checkin.lesson.body", "درس بعدی {course}"),
            ("checkin.practice.title", "مطالعه و تمرین «{lesson}»"),
            ("checkin.practice.body", "تمرین {course}"),
            ("advisor.nudge", "شروع دوباره"),
            ("advisor.start", "شروع مسیر"),
            ("advisor.focus", "در {strong} خوبی؛ روی {weak} تمرکز کن."),
            ("advisor.steady", "درس‌های {weak} را جلو ببر."),
            ("advisor.celebrate", "آفرین"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_placeholders() {
        let t = test_templates();
        assert_eq!(t.render("mission.course", &[("minutes", "۳۰"), ("course", "معماری تمیز")]), "امروز ۳۰ دقیقه روی درس معماری تمیز کار کن.");
    }

    #[test]
    fn missing_template_is_empty() {
        assert_eq!(Templates::default().render("nope", &[]), "");
    }
}
