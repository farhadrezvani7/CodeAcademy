//! Grade point average: credit-weighted mean of grades on a 0–20 scale.

pub const PASSING_GRADE: f64 = 14.0;

#[derive(Debug, Clone, Copy)]
pub struct GradedCourse {
    pub credits: i32,
    pub grade: f64,
}

/// `Σ(grade × credits) / Σ(credits)`, rounded to two decimals.
/// Zero-credit items (e.g. assessment-only projects) do not affect the average.
/// Returns `None` when nothing credit-bearing has been graded yet.
pub fn weighted_gpa(items: &[GradedCourse]) -> Option<f64> {
    let (weighted, credits) = items
        .iter()
        .filter(|c| c.credits > 0)
        .fold((0.0, 0i64), |(w, c), item| (w + item.grade * item.credits as f64, c + item.credits as i64));
    if credits == 0 {
        None
    } else {
        Some(round2(weighted / credits as f64))
    }
}

pub fn is_passing(grade: f64) -> bool {
    grade >= PASSING_GRADE
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g(credits: i32, grade: f64) -> GradedCourse {
        GradedCourse { credits, grade }
    }

    #[test]
    fn weighted_average_uses_credits() {
        // (20*4 + 14*2) / 6 = 18
        assert_eq!(weighted_gpa(&[g(4, 20.0), g(2, 14.0)]), Some(18.0));
    }

    #[test]
    fn differs_from_plain_mean() {
        // plain mean would be 16.5; weighted is (18*3 + 15*1)/4 = 17.25
        assert_eq!(weighted_gpa(&[g(3, 18.0), g(1, 15.0)]), Some(17.25));
    }

    #[test]
    fn rounds_to_two_decimals() {
        // (17*3 + 18*3 + 19.5*1) / 7 = 17.7857…
        assert_eq!(weighted_gpa(&[g(3, 17.0), g(3, 18.0), g(1, 19.5)]), Some(17.79));
    }

    #[test]
    fn ignores_zero_credit_items() {
        assert_eq!(weighted_gpa(&[g(3, 16.0), g(0, 5.0)]), Some(16.0));
    }

    #[test]
    fn empty_is_none() {
        assert_eq!(weighted_gpa(&[]), None);
        assert_eq!(weighted_gpa(&[g(0, 20.0)]), None);
    }

    #[test]
    fn passing_threshold_is_14() {
        assert!(is_passing(14.0));
        assert!(!is_passing(13.99));
    }
}
