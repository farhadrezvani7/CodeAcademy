//! Daily streaks from activity dates.
//!
//! A streak counts consecutive active days ending today. Today is still "open",
//! so if the student has not studied yet today the streak ending yesterday is
//! kept; once a full calendar day passes with no activity it resets to 0.

use chrono::{Duration, NaiveDate};
use std::collections::BTreeSet;

pub fn current_streak(active_days: &[NaiveDate], today: NaiveDate) -> u32 {
    let days: BTreeSet<NaiveDate> = active_days.iter().copied().filter(|d| *d <= today).collect();
    let yesterday = today - Duration::days(1);
    let mut cursor = if days.contains(&today) {
        today
    } else if days.contains(&yesterday) {
        yesterday
    } else {
        return 0;
    };
    let mut count = 0;
    while days.contains(&cursor) {
        count += 1;
        cursor -= Duration::days(1);
    }
    count
}

pub fn longest_streak(active_days: &[NaiveDate]) -> u32 {
    let days: BTreeSet<NaiveDate> = active_days.iter().copied().collect();
    let mut best = 0;
    let mut run = 0;
    let mut prev: Option<NaiveDate> = None;
    for d in days {
        run = match prev {
            Some(p) if d - p == Duration::days(1) => run + 1,
            _ => 1,
        };
        best = best.max(run);
        prev = Some(d);
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    fn range(end: &str, len: i64) -> Vec<NaiveDate> {
        (0..len).map(|i| d(end) - Duration::days(i)).collect()
    }

    #[test]
    fn counts_consecutive_days_including_today() {
        assert_eq!(current_streak(&range("2026-09-28", 5), d("2026-09-28")), 5);
    }

    #[test]
    fn today_not_yet_active_keeps_yesterdays_streak() {
        assert_eq!(current_streak(&range("2026-09-27", 4), d("2026-09-28")), 4);
    }

    #[test]
    fn a_missed_day_resets_to_zero() {
        assert_eq!(current_streak(&range("2026-09-26", 10), d("2026-09-28")), 0);
    }

    #[test]
    fn gap_breaks_the_run() {
        let mut days = range("2026-09-28", 3);
        days.extend(range("2026-09-20", 10));
        assert_eq!(current_streak(&days, d("2026-09-28")), 3);
        assert_eq!(longest_streak(&days), 10);
    }

    #[test]
    fn duplicates_and_order_do_not_matter() {
        let days = vec![d("2026-09-28"), d("2026-09-27"), d("2026-09-28"), d("2026-09-26")];
        assert_eq!(current_streak(&days, d("2026-09-28")), 3);
    }

    #[test]
    fn empty_is_zero() {
        assert_eq!(current_streak(&[], d("2026-09-28")), 0);
        assert_eq!(longest_streak(&[]), 0);
    }
}
