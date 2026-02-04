//! Date parsing: absolute, relative, and weekday formats.

use crate::error::TaskCtlError;
use chrono::{Datelike, NaiveDate, Weekday};

/// Parse a due date string relative to `today`.
pub fn parse_due(input: &str, today: NaiveDate) -> Result<NaiveDate, TaskCtlError> {
    let input = input.trim().to_lowercase();

    // Absolute: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
        return Ok(date);
    }

    // Relative keywords
    match input.as_str() {
        "today" => return Ok(today),
        "tomorrow" => return Ok(today + chrono::Duration::days(1)),
        _ => {}
    }

    // Relative: +Nd or +Nw
    if let Some(rest) = input.strip_prefix('+') {
        if let Some(days_str) = rest.strip_suffix('d') {
            let days: i64 = days_str.parse().map_err(|_| {
                TaskCtlError::InvalidArgument(format!("Invalid relative date: {input}"))
            })?;
            return Ok(today + chrono::Duration::days(days));
        }
        if let Some(weeks_str) = rest.strip_suffix('w') {
            let weeks: i64 = weeks_str.parse().map_err(|_| {
                TaskCtlError::InvalidArgument(format!("Invalid relative date: {input}"))
            })?;
            return Ok(today + chrono::Duration::weeks(weeks));
        }
    }

    // Weekday: monday, tuesday, ...
    if let Some(weekday) = parse_weekday(&input) {
        return Ok(next_weekday(today, weekday));
    }

    Err(TaskCtlError::InvalidArgument(format!(
        "Cannot parse date: {input}"
    )))
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s {
        "monday" | "mon" => Some(Weekday::Mon),
        "tuesday" | "tue" => Some(Weekday::Tue),
        "wednesday" | "wed" => Some(Weekday::Wed),
        "thursday" | "thu" => Some(Weekday::Thu),
        "friday" | "fri" => Some(Weekday::Fri),
        "saturday" | "sat" => Some(Weekday::Sat),
        "sunday" | "sun" => Some(Weekday::Sun),
        _ => None,
    }
}

fn next_weekday(from: NaiveDate, target: Weekday) -> NaiveDate {
    let current = from.weekday().num_days_from_monday();
    let target_day = target.num_days_from_monday();
    let days_ahead = if target_day <= current {
        7 - (current - target_day)
    } else {
        target_day - current
    };
    from + chrono::Duration::days(i64::from(days_ahead))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2025, 2, 5).unwrap() // Wednesday
    }

    #[test]
    fn absolute_date() {
        assert_eq!(
            parse_due("2025-03-15", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 3, 15).unwrap()
        );
    }

    #[test]
    fn today_keyword() {
        assert_eq!(parse_due("today", today()).unwrap(), today());
    }

    #[test]
    fn tomorrow_keyword() {
        assert_eq!(
            parse_due("tomorrow", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 6).unwrap()
        );
    }

    #[test]
    fn relative_days() {
        assert_eq!(
            parse_due("+3d", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 8).unwrap()
        );
    }

    #[test]
    fn relative_weeks() {
        assert_eq!(
            parse_due("+1w", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 12).unwrap()
        );
    }

    #[test]
    fn weekday_friday() {
        // Wednesday -> next Friday = +2 days
        assert_eq!(
            parse_due("friday", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 7).unwrap()
        );
    }

    #[test]
    fn weekday_monday() {
        // Wednesday -> next Monday = +5 days
        assert_eq!(
            parse_due("monday", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 10).unwrap()
        );
    }

    #[test]
    fn weekday_wednesday_from_wednesday() {
        // Same day -> next week
        assert_eq!(
            parse_due("wednesday", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 12).unwrap()
        );
    }

    #[test]
    fn abbreviated_weekday() {
        assert_eq!(
            parse_due("fri", today()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 2, 7).unwrap()
        );
    }

    #[test]
    fn invalid_input() {
        assert!(parse_due("not-a-date", today()).is_err());
        assert!(parse_due("+xd", today()).is_err());
        assert!(parse_due("", today()).is_err());
    }
}
