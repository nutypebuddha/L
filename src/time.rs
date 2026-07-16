//! Timezone-safe datetime resolution for chart commands.
//!
//! Part 1.1 — the "gyroscope upgrade" begins with not silently corrupting
//! deterministic calculation. A birth-chart datetime must be unambiguous:
//! either a UTC instant (`--datetime-utc`) or a local time paired with an
//! explicit IANA timezone (`--datetime` + `--tz`). A bare local time is
//! rejected — silent UTC assumption is exactly the bug that produced a wrong
//! lagna for a CDT birth cast as literal UTC.

use crate::ephemeris::julian_day;
use chrono::{DateTime, Datelike, NaiveDateTime, Offset, TimeZone, Timelike, Utc};
use chrono_tz::Tz;

/// Typed failures for chart datetime input. These are machine-readable so an
/// LLM orchestration loop (Part 2.3) can react to them, not just a human.
#[derive(Debug, thiserror::Error)]
pub enum TimezoneError {
    #[error(
        "missing timezone: a chart datetime must be given as --datetime-utc (UTC instant) \
         OR a local --datetime together with an explicit --tz IANA timezone. \
         Bare local time is rejected because a silent UTC assumption corrupts the \
         sidereal (lagna/ayanamsa) computation."
    )]
    MissingTimezone,

    #[error("invalid timezone identifier '{0}': expected an IANA zone such as 'America/Chicago'")]
    InvalidTimezone(String),

    #[error("invalid datetime '{0}': expected format 'YYYY-MM-DD HH:MM[:SS]' (or ...T... with a trailing Z for UTC)")]
    InvalidDatetime(String),

    #[error(
        "conflicting datetime inputs: provide --datetime-utc OR (--datetime + --tz), not both"
    )]
    ConflictingInputs,
}

/// The unambiguous result of resolving a chart datetime to UTC.
#[derive(Debug, Clone)]
pub struct ResolvedInstant {
    /// Julian day (UTC) fed into the ephemeris math.
    pub jd_utc: f64,
    /// RFC3339 UTC string actually used (self-documenting proof object field).
    pub utc_iso: String,
    /// Applied offset in seconds (local = UTC + offset). 0 for explicit UTC input.
    pub offset_seconds: i32,
    /// The local wall-clock string as supplied, for echo / regression diffing.
    pub local_iso: String,
}

fn parse_naive(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    for fmt in [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
    ] {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
    }
    None
}

fn julian_day_from_utc(dt: &DateTime<Utc>) -> f64 {
    let hour = dt.hour() as f64
        + dt.minute() as f64 / 60.0
        + dt.second() as f64 / 3600.0
        + dt.nanosecond() as f64 / 3.6e12;
    julian_day(dt.year() as i16, dt.month() as u8, dt.day() as u8, hour)
}

/// Resolve a chart datetime to a UTC `ResolvedInstant`.
///
/// - `--datetime-utc "1996-06-28T22:35:00Z"` → parsed as UTC, offset 0.
/// - `--datetime "1996-06-28 17:35:00" --tz "America/Chicago"` → resolved through
///   the IANA database to the correct UTC instant; `offset_seconds` recorded.
/// - bare `--datetime` with no `--tz` → `MissingTimezone`.
/// - both `--datetime-utc` and `--datetime` → `ConflictingInputs`.
pub fn resolve_chart_datetime(
    datetime_utc: Option<&str>,
    datetime_local: Option<&str>,
    tz: Option<&str>,
) -> Result<ResolvedInstant, TimezoneError> {
    match (datetime_utc, datetime_local) {
        (Some(utc), None) => {
            let s = utc.trim().trim_end_matches('Z').trim();
            let naive =
                parse_naive(s).ok_or_else(|| TimezoneError::InvalidDatetime(utc.to_string()))?;
            let dt_utc: DateTime<Utc> = Utc.from_utc_datetime(&naive);
            Ok(to_resolved(&dt_utc, 0, Some(utc.trim().to_string())))
        }
        (None, Some(local)) => {
            let tz_name = tz.ok_or(TimezoneError::MissingTimezone)?;
            let tz: Tz = tz_name
                .parse()
                .map_err(|_| TimezoneError::InvalidTimezone(tz_name.to_string()))?;
            let naive = parse_naive(local)
                .ok_or_else(|| TimezoneError::InvalidDatetime(local.to_string()))?;
            let local_dt = tz
                .from_local_datetime(&naive)
                .single()
                .ok_or_else(|| TimezoneError::InvalidDatetime(local.to_string()))?;
            let dt_utc = local_dt.with_timezone(&Utc);
            Ok(to_resolved(
                &dt_utc,
                local_dt.offset().fix().local_minus_utc(),
                Some(local.to_string()),
            ))
        }
        (Some(_), Some(_)) => Err(TimezoneError::ConflictingInputs),
        (None, None) => Err(TimezoneError::MissingTimezone),
    }
}

fn to_resolved(dt_utc: &DateTime<Utc>, offset: i32, local_echo: Option<String>) -> ResolvedInstant {
    ResolvedInstant {
        jd_utc: julian_day_from_utc(dt_utc),
        utc_iso: dt_utc.to_rfc3339(),
        offset_seconds: offset,
        local_iso: local_echo.unwrap_or_else(|| dt_utc.format("%Y-%m-%d %H:%M:%S").to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bare_local_time() {
        assert!(matches!(
            resolve_chart_datetime(None, Some("1996-06-28 17:35:00"), None),
            Err(TimezoneError::MissingTimezone)
        ));
    }

    #[test]
    fn utc_input_has_zero_offset() {
        let r = resolve_chart_datetime(Some("1996-06-28T22:35:00Z"), None, None).unwrap();
        assert_eq!(r.offset_seconds, 0);
        assert!(r.utc_iso.starts_with("1996-06-28T22:35:00"));
    }

    #[test]
    fn cdt_resolves_to_correct_utc() {
        // 1996-06-28 17:35:00 America/Chicago (CDT = UTC-5) -> 22:35 UTC.
        let r = resolve_chart_datetime(None, Some("1996-06-28 17:35:00"), Some("America/Chicago"))
            .unwrap();
        assert_eq!(r.offset_seconds, -5 * 3600);
        assert!(r.utc_iso.starts_with("1996-06-28T22:35:00"));
    }
}
