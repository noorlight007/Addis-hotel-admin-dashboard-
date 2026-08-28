//! Minimal calendar arithmetic.
//!
//! The dashboard has no backend and no `chrono`, but the availability calendar
//! still needs real months: the right number of days, the right weekday for
//! each one, and a "today" that follows the wall clock instead of a constant
//! baked in at build time.

/// `(year, month 1-12, day 1-31)`.
pub type Date = (i32, u32, u32);

pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Sunday-first, matching [`weekday`].
pub const WEEKDAY_ABBR: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// The browser's local date.
pub fn today() -> Date {
    let now = js_sys::Date::new_0();
    (
        now.get_full_year() as i32,
        now.get_month() + 1, // js_sys months are 0-based
        now.get_date(),
    )
}

pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 30,
    }
}

/// Days since 1970-01-01 (Howard Hinnant's `days_from_civil`).
///
/// Also serves as a stable per-date seed for the demo availability data.
pub fn days_from_epoch((y, m, d): Date) -> i64 {
    let (y, m, d) = (y as i64, m as i64, d as i64);
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `0` = Sunday .. `6` = Saturday.
pub fn weekday(date: Date) -> usize {
    // 1970-01-01 was a Thursday (index 4).
    (days_from_epoch(date).rem_euclid(7) as usize + 4) % 7
}

pub fn is_weekend(date: Date) -> bool {
    matches!(weekday(date), 0 | 6)
}

/// Steps one month forward (`+1`) or back (`-1`), rolling the year over.
pub fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let zero_based = (month as i32 - 1) + delta;
    let year = year + zero_based.div_euclid(12);
    (year, zero_based.rem_euclid(12) as u32 + 1)
}

/// `"Wed, 26 August 2026"`.
pub fn format_long(date: Date) -> String {
    let (y, m, d) = date;
    format!(
        "{}, {d} {} {y}",
        WEEKDAY_ABBR[weekday(date)],
        MONTH_NAMES[(m - 1) as usize]
    )
}

/// `"Wed 26 Aug"` — compact enough for a cell tooltip.
pub fn format_short(date: Date) -> String {
    let (_, m, d) = date;
    format!(
        "{} {d} {}",
        WEEKDAY_ABBR[weekday(date)],
        &MONTH_NAMES[(m - 1) as usize][..3]
    )
}

/// `"August 2026"`.
pub fn format_month(year: i32, month: u32) -> String {
    format!("{} {year}", MONTH_NAMES[(month - 1) as usize])
}

/// Inverse of [`days_from_epoch`] (Howard Hinnant's `civil_from_days`).
pub fn date_from_epoch(z: i64) -> Date {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

/// Moves a date by whole days, rolling months and years over.
pub fn add_days(date: Date, delta: i64) -> Date {
    date_from_epoch(days_from_epoch(date) + delta)
}

/// `"2026-08-27"` — the format every API date field uses.
pub fn to_iso((y, m, d): Date) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

/// Parses `"2026-08-27"`, tolerating a trailing time component.
pub fn parse_iso(s: &str) -> Option<Date> {
    let mut it = s.split('T').next()?.split('-');
    Some((
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    ))
}

/// Whole nights between two ISO dates, or `0` when either is unparseable.
pub fn nights_between(check_in: &str, check_out: &str) -> i64 {
    match (parse_iso(check_in), parse_iso(check_out)) {
        (Some(a), Some(b)) => (days_from_epoch(b) - days_from_epoch(a)).max(0),
        _ => 0,
    }
}
