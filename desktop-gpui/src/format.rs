//! What the Electron renderer's `format.ts` prints: an instant as `Intl.DateTimeFormat`
//! prints it with `dateStyle: "medium"` and `timeStyle: "short"` in English, in this
//! computer's time zone ("Nov 4, 2026, 6:31 PM"). Intl puts a narrow no-break space before
//! the meridiem, which the app's font draws without width, so this prints a space.

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Seconds since the Unix epoch of an RFC 3339 instant, such as the core's
/// `2026-11-04T21:31:12.123Z` or `2026-11-04T18:31:12-03:00`.
pub fn parse_rfc3339(value: &str) -> Option<i64> {
    instant_millis(value).map(|millis| millis.div_euclid(1000))
}

/// Milliseconds since the Unix epoch of an RFC 3339 instant, as `Date.parse` reads it: the
/// fraction's first three digits count.
pub fn instant_millis(value: &str) -> Option<i64> {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() < 19 || !matches!(bytes[10], b'T' | b't' | b' ') {
        return None;
    }
    let number = |range: std::ops::Range<usize>| -> Option<i64> { value.get(range)?.parse().ok() };
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }
    // Fractional seconds, then the offset.
    let mut rest = &value[19..];
    let mut millis = 0;
    if let Some(fraction) = rest.strip_prefix('.') {
        let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
        millis = fraction[..digits]
            .bytes()
            .chain(std::iter::repeat(b'0'))
            .take(3)
            .fold(0, |sum, digit| sum * 10 + i64::from(digit - b'0'));
        rest = &fraction[digits..];
    }
    let offset = match rest {
        "Z" | "z" | "" => 0,
        _ => {
            let sign = match rest.as_bytes().first()? {
                b'+' => 1,
                b'-' => -1,
                _ => return None,
            };
            let hours: i64 = rest.get(1..3)?.parse().ok()?;
            let minutes: i64 = rest.get(4..6)?.parse().ok()?;
            sign * (hours * 3600 + minutes * 60)
        }
    };
    let days = days_from_civil(year, month, day);
    Some((days * 86_400 + hour * 3600 + minute * 60 + second - offset) * 1000 + millis)
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_index = (month + 9) % 12;
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The local wall clock of an instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Wall {
    pub year: i64,
    /// 0 for January.
    pub month: usize,
    pub day: i64,
    pub hour: i64,
    pub minute: i64,
    pub second: i64,
    /// 0 for Sunday.
    pub weekday: usize,
}

/// The local wall clock of `seconds` since the epoch.
pub fn wall(seconds: i64) -> Option<Wall> {
    // `time_t` is 64-bit on every target the app builds for.
    let time: libc::time_t = seconds;
    // SAFETY: `localtime_r` writes only into the `tm` it is given.
    let tm = unsafe {
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&time, &mut tm).is_null() {
            return None;
        }
        tm
    };
    Some(Wall {
        year: i64::from(tm.tm_year) + 1900,
        month: usize::try_from(tm.tm_mon).ok()?,
        day: i64::from(tm.tm_mday),
        hour: i64::from(tm.tm_hour),
        minute: i64::from(tm.tm_min),
        second: i64::from(tm.tm_sec),
        weekday: usize::try_from(tm.tm_wday).ok()?,
    })
}

/// The local wall clock of `seconds`: year, month (0-11), day, hour (0-23), minute.
fn local_time(seconds: i64) -> Option<(i64, usize, i64, i64, i64)> {
    wall(seconds).map(|wall| (wall.year, wall.month, wall.day, wall.hour, wall.minute))
}

/// The hour on a 12-hour clock and its meridiem.
fn twelve(hour: i64) -> (i64, &'static str) {
    match hour {
        0 => (12, "AM"),
        1..=11 => (hour, "AM"),
        12 => (12, "PM"),
        _ => (hour - 12, "PM"),
    }
}

/// `toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" })`: "6:32 PM", or the
/// text itself when it does not parse ("—" for none).
pub fn time_of_day(value: &str) -> String {
    if value.trim().is_empty() {
        return "\u{2014}".into();
    }
    match parse_rfc3339(value).and_then(wall) {
        Some(wall) => {
            let (hour, meridiem) = twelve(wall.hour);
            format!("{hour}:{:02} {meridiem}", wall.minute)
        }
        None => value.to_string(),
    }
}

/// `toLocaleString()` in English: "11/4/2026, 6:32:00 PM" ("—" for none).
pub fn date_time(value: &str) -> String {
    if value.trim().is_empty() {
        return "\u{2014}".into();
    }
    match parse_rfc3339(value).and_then(wall) {
        Some(wall) => {
            let (hour, meridiem) = twelve(wall.hour);
            format!(
                "{}/{}/{}, {hour}:{:02}:{:02} {meridiem}",
                wall.month + 1,
                wall.day,
                wall.year,
                wall.minute,
                wall.second
            )
        }
        None => value.to_string(),
    }
}

/// `formatAgo`: "Just now", "5m ago", "3h ago", "2d ago", then the date ("11/4/2026").
pub fn ago(value: &str, now: i64) -> String {
    let Some(at) = parse_rfc3339(value) else {
        return if value.is_empty() {
            "\u{2014}".into()
        } else {
            value.to_string()
        };
    };
    let minutes = ((now - at) / 60).max(0);
    if minutes < 1 {
        return "Just now".into();
    }
    if minutes < 60 {
        return format!("{minutes}m ago");
    }
    let hours = minutes / 60;
    if hours < 24 {
        return format!("{hours}h ago");
    }
    let days = hours / 24;
    if days < 7 {
        return format!("{days}d ago");
    }
    wall(at).map_or_else(
        || value.to_string(),
        |wall| format!("{}/{}/{}", wall.month + 1, wall.day, wall.year),
    )
}

/// `DocumentActivity`'s `ago`: "just now", "5m ago", "3h ago", "2d ago", then the date as
/// `toLocaleDateString` prints it ("Nov 4, 2026"); nothing for an instant that does not parse.
pub fn document_ago(value: &str, now: i64) -> String {
    let Some(at) = parse_rfc3339(value) else {
        return String::new();
    };
    let minutes = ((now - at) / 60).max(0);
    match minutes {
        0 => "just now".into(),
        1..60 => format!("{minutes}m ago"),
        60..1440 => format!("{}h ago", minutes / 60),
        1440..10080 => format!("{}d ago", minutes / 1440),
        _ => wall(at).map_or_else(String::new, |wall| {
            let month = MONTHS.get(wall.month).copied().unwrap_or("Jan");
            format!("{month} {}, {}", wall.day, wall.year)
        }),
    }
}

/// `activityDayId`: the local day of an instant as (year, month 1-12, day), none when it
/// does not parse.
/// A calendar day: year, month (1 to 12), and day of the month.
pub type Day = (i64, i64, i64);

pub fn day_of(value: &str) -> Option<Day> {
    let wall = parse_rfc3339(value).and_then(wall)?;
    Some((wall.year, wall.month as i64 + 1, wall.day))
}

const MONTH_NAMES: [&str; 12] = [
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
const WEEKDAYS: [&str; 7] = [
    "Sunday",
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
];

/// `activityDayLabel`: "Today", "Yesterday", the weekday within the last week, then the
/// date ("November 4"), with its year once it is not this year's. `today` is the local day.
pub fn day_label(day: Day, today: Day) -> String {
    let (year, month, date) = day;
    let days = days_from_civil(today.0, today.1, today.2) - days_from_civil(year, month, date);
    match days {
        0 => "Today".into(),
        1 => "Yesterday".into(),
        2..=6 => {
            // 1970-01-01 was a Thursday.
            let weekday = (days_from_civil(year, month, date) + 4).rem_euclid(7) as usize;
            WEEKDAYS[weekday].into()
        }
        _ => {
            let name = MONTH_NAMES
                .get((month - 1).clamp(0, 11) as usize)
                .copied()
                .unwrap_or("January");
            if year == today.0 {
                format!("{name} {date}")
            } else {
                format!("{name} {date}, {year}")
            }
        }
    }
}

/// The local day now.
pub fn today() -> Day {
    let now = now_seconds();
    wall(now).map_or((1970, 1, 1), |wall| {
        (wall.year, wall.month as i64 + 1, wall.day)
    })
}

/// Seconds since the epoch now.
pub fn now_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}

/// "Nov 4, 2026, 6:31 PM" from the wall clock's parts.
pub fn medium_short(year: i64, month: usize, day: i64, hour: i64, minute: i64) -> String {
    let (hour12, meridiem) = twelve(hour);
    let month = MONTHS.get(month).copied().unwrap_or("Jan");
    format!("{month} {day}, {year}, {hour12}:{minute:02} {meridiem}")
}

/// `formatArchiveDate`: the local date in short ("Nov 5, 2026"), "—" for none, the text itself
/// when it does not parse.
pub fn archive_date(value: &str) -> String {
    if value.trim().is_empty() {
        return "\u{2014}".into();
    }
    match parse_rfc3339(value).and_then(wall) {
        Some(wall) => {
            let month = MONTHS.get(wall.month).copied().unwrap_or("Jan");
            format!("{month} {}, {}", wall.day, wall.year)
        }
        None => value.to_string(),
    }
}

/// `toLocaleString("en-US", { month: "short", day: "numeric", hour: "numeric", minute:
/// "2-digit" })`: "Nov 4, 6:31 PM", the text itself when it does not parse.
pub fn month_day_time(value: &str) -> String {
    match parse_rfc3339(value).and_then(local_time) {
        Some((_, month, day, hour, minute)) => {
            let (hour12, meridiem) = twelve(hour);
            let month = MONTHS.get(month).copied().unwrap_or("Jan");
            format!("{month} {day}, {hour12}:{minute:02} {meridiem}")
        }
        None => value.to_string(),
    }
}

/// `formatInstant`: "—" for no instant, the text itself when it does not parse.
pub fn format_instant(value: &str) -> String {
    if value.trim().is_empty() {
        return "\u{2014}".into();
    }
    match parse_rfc3339(value).and_then(local_time) {
        Some((year, month, day, hour, minute)) => medium_short(year, month, day, hour, minute),
        None => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ago, archive_date, day_label, format_instant, instant_millis, medium_short, parse_rfc3339,
    };

    #[test]
    fn an_archive_date_is_the_short_local_date() {
        assert_eq!(archive_date(""), "\u{2014}");
        assert_eq!(archive_date("yesterday"), "yesterday");
        // Noon UTC is the same day in every zone the machine may be in.
        assert_eq!(archive_date("2026-11-05T12:00:00Z"), "Nov 5, 2026");
    }

    #[test]
    fn an_instant_keeps_its_milliseconds_as_date_parse_does() {
        assert_eq!(
            instant_millis("2026-11-04T21:31:12.123456Z"),
            Some(1_793_827_872_123)
        );
        assert_eq!(
            instant_millis("2026-11-04T21:31:12.5Z"),
            Some(1_793_827_872_500)
        );
        assert_eq!(
            instant_millis("2026-11-04T18:31:12-03:00"),
            Some(1_793_827_872_000)
        );
        assert_eq!(
            parse_rfc3339("2026-11-04T21:31:12.999Z"),
            Some(1_793_827_872)
        );
        assert_eq!(instant_millis("not a time"), None);
    }

    #[test]
    fn instants_parse_with_fractions_and_offsets() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_rfc3339("2026-11-04T21:31:12.123Z"),
            Some(1_793_827_872)
        );
        assert_eq!(
            parse_rfc3339("2026-11-04T18:31:12-03:00"),
            parse_rfc3339("2026-11-04T21:31:12Z")
        );
        assert_eq!(parse_rfc3339("yesterday"), None);
    }

    #[test]
    fn the_wall_clock_reads_as_intl_medium_short() {
        assert_eq!(medium_short(2026, 10, 4, 18, 31), "Nov 4, 2026, 6:31 PM");
        assert_eq!(medium_short(2026, 0, 12, 0, 5), "Jan 12, 2026, 12:05 AM");
        assert_eq!(format_instant(""), "\u{2014}");
        assert_eq!(format_instant("soon"), "soon");
    }

    #[test]
    fn days_read_as_the_activity_feed_labels_them() {
        let today = (2026, 11, 4);
        assert_eq!(day_label((2026, 11, 4), today), "Today");
        assert_eq!(day_label((2026, 11, 3), today), "Yesterday");
        assert_eq!(day_label((2026, 11, 1), today), "Sunday");
        assert_eq!(day_label((2026, 10, 20), today), "October 20");
        assert_eq!(day_label((2025, 12, 31), today), "December 31, 2025");
    }

    #[test]
    fn ago_steps_from_minutes_to_days() {
        let at = parse_rfc3339("2026-11-04T18:00:00Z").unwrap();
        assert_eq!(ago("2026-11-04T18:00:00Z", at + 30), "Just now");
        assert_eq!(ago("2026-11-04T18:00:00Z", at + 5 * 60), "5m ago");
        assert_eq!(ago("2026-11-04T18:00:00Z", at + 3 * 3600 + 5), "3h ago");
        assert_eq!(ago("2026-11-04T18:00:00Z", at + 2 * 86_400), "2d ago");
    }
}
