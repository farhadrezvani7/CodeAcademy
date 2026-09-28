//! Learning statistics: activity heatmap, last 7 days and time by topic.

use crate::domain::{percent, streak};
use crate::error::AppResult;
use crate::repo::{activity, catalog, student};
use crate::state::AppState;
use chrono::{Duration, NaiveDate};
use serde::Serialize;
use std::collections::HashMap;

pub const HEATMAP_DAYS: i64 = 7 * 26;

/// Lower bound for all-time aggregates (before the academy existed).
fn epoch() -> NaiveDate {
    NaiveDate::from_ymd_opt(2000, 1, 1).expect("valid date")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DayMinutes {
    pub date: NaiveDate,
    pub minutes: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicMinutes {
    pub slug: String,
    pub name_fa: String,
    pub accent: String,
    pub minutes: i32,
    pub percent: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Totals {
    pub minutes: i64,
    pub active_days: usize,
    pub average_per_active_day: i64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub lessons_completed: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub today: NaiveDate,
    pub heatmap: Vec<DayMinutes>,
    pub weekly: Vec<DayMinutes>,
    pub by_topic: Vec<TopicMinutes>,
    pub totals: Totals,
}

/// Fills missing days with zero so the client can render a dense grid.
pub fn dense_days(from: NaiveDate, to: NaiveDate, sparse: &[(NaiveDate, i32)]) -> Vec<DayMinutes> {
    let map: HashMap<NaiveDate, i32> = sparse.iter().copied().collect();
    let mut out = Vec::new();
    let mut d = from;
    while d <= to {
        out.push(DayMinutes { date: d, minutes: map.get(&d).copied().unwrap_or(0) });
        d += Duration::days(1);
    }
    out
}

pub async fn stats(state: &AppState, user_id: i32) -> AppResult<Stats> {
    let today = state.clock.today();
    let mut conn = state.pool.acquire().await?;
    let from = today - Duration::days(HEATMAP_DAYS - 1);
    let daily = activity::minutes_per_day(&mut *conn, user_id, from, today).await?;
    let heatmap = dense_days(from, today, &daily);
    let weekly = dense_days(today - Duration::days(6), today, &daily);

    let departments = catalog::departments(&mut *conn).await?;
    let topics = activity::minutes_by_topic(&mut *conn, user_id, epoch()).await?;
    let topic_total: i64 = topics.iter().map(|(_, m)| *m as i64).sum();
    let by_topic = departments
        .iter()
        .map(|d| {
            let minutes = topics.iter().find(|(t, _)| *t == d.slug).map(|(_, m)| *m).unwrap_or(0);
            TopicMinutes { slug: d.slug.clone(), name_fa: d.name_fa.clone(), accent: d.accent.clone(), minutes, percent: percent(minutes as i64, topic_total) }
        })
        .collect();

    let dates = activity::active_dates(&mut *conn, user_id).await?;
    let all_days = activity::minutes_per_day(&mut *conn, user_id, epoch(), today).await?;
    let total_minutes: i64 = all_days.iter().map(|(_, m)| *m as i64).sum();
    let lessons_completed = student::completions(&mut *conn, user_id).await?.len();
    Ok(Stats {
        today,
        heatmap,
        weekly,
        by_topic,
        totals: Totals {
            minutes: total_minutes,
            active_days: dates.len(),
            average_per_active_day: if dates.is_empty() { 0 } else { total_minutes / dates.len() as i64 },
            current_streak: streak::current_streak(&dates, today),
            longest_streak: streak::longest_streak(&dates),
            lessons_completed,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dense_days_fills_gaps() {
        let d = |s: &str| NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap();
        let out = dense_days(d("2026-09-01"), d("2026-09-04"), &[(d("2026-09-02"), 30)]);
        assert_eq!(out.iter().map(|x| x.minutes).collect::<Vec<_>>(), vec![0, 30, 0, 0]);
    }
}
