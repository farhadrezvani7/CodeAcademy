//! Gregorian → Jalali (Persian) calendar conversion.

use chrono::{Datelike, NaiveDate};

/// Returns (year, month, day) in the Jalali calendar.
pub fn from_gregorian(date: NaiveDate) -> (i32, u32, u32) {
    let (gy, gm, gd) = (date.year(), date.month() as i32, date.day() as i32);
    const G_DAYS: [i32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let gy2 = if gm > 2 { gy + 1 } else { gy };
    let mut days = 355666 + 365 * gy + (gy2 + 3) / 4 - (gy2 + 99) / 100 + (gy2 + 399) / 400 + gd + G_DAYS[(gm - 1) as usize];
    let mut jy = -1595 + 33 * (days / 12053);
    days %= 12053;
    jy += 4 * (days / 1461);
    days %= 1461;
    if days > 365 {
        jy += (days - 1) / 365;
        days = (days - 1) % 365;
    }
    let (jm, jd) = if days < 186 { (1 + days / 31, 1 + days % 31) } else { (7 + (days - 186) / 30, 1 + (days - 186) % 30) };
    (jy, jm as u32, jd as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn known_dates() {
        assert_eq!(from_gregorian(d(2026, 9, 28)), (1405, 7, 6));
        assert_eq!(from_gregorian(d(2025, 3, 21)), (1404, 1, 1));
        assert_eq!(from_gregorian(d(2025, 3, 20)), (1403, 12, 30));
        assert_eq!(from_gregorian(d(2024, 3, 20)), (1403, 1, 1));
    }
}
