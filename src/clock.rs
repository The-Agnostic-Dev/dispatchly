//! Time helpers. The business runs on Lagos time (WAT, UTC+1, no daylight saving);
//! timestamps are stored in the database as UTC RFC 3339 strings.

use chrono::{DateTime, Duration, FixedOffset, NaiveDate, SecondsFormat, TimeZone, Utc};

pub fn lagos() -> FixedOffset {
    FixedOffset::east_opt(3600).expect("valid offset")
}

pub fn now() -> DateTime<Utc> {
    Utc::now()
}

/// Today's date in Lagos.
pub fn today() -> NaiveDate {
    now().with_timezone(&lagos()).date_naive()
}

pub fn to_db(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn from_db(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| DateTime::<Utc>::UNIX_EPOCH)
}

/// A Lagos wall-clock time on `day`, as a UTC instant.
pub fn at_local(day: NaiveDate, hour: u32, minute: u32) -> DateTime<Utc> {
    let naive = day.and_hms_opt(hour, minute, 0).expect("valid time");
    lagos()
        .from_local_datetime(&naive)
        .single()
        .expect("Lagos has no gaps")
        .with_timezone(&Utc)
}

/// Start and end (exclusive) of a business day, in the stored timestamp format.
pub fn day_bounds(day: NaiveDate) -> (String, String) {
    let start = day.and_hms_opt(0, 0, 0).expect("midnight").and_utc();
    let end = start + Duration::days(1);
    (to_db(start), to_db(end))
}

/// "14:05"
pub fn time_label(stored: &str) -> String {
    from_db(stored).with_timezone(&lagos()).format("%H:%M").to_string()
}

/// "3 Nov, 14:05"
pub fn datetime_label(stored: &str) -> String {
    from_db(stored).with_timezone(&lagos()).format("%-d %b, %H:%M").to_string()
}

/// "2026-11-03T14:05", used in data attributes and exports.
pub fn local_iso(stored: &str) -> String {
    from_db(stored).with_timezone(&lagos()).format("%Y-%m-%dT%H:%M").to_string()
}

/// "Tuesday 3 November 2026"
pub fn day_label(day: NaiveDate) -> String {
    day.format("%A %-d %B %Y").to_string()
}

pub fn parse_day(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").ok()
}
