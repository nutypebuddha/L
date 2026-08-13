use crate::julian::JdUT1;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
/// Calendar system used for date-to-JD and JD-to-date conversions.
#[derive(Default)]
pub enum CalendarSystem {
    /// Gregorian calendar extended before 1582 (proleptic).
    #[default]
    ProlepticGregorian,
    /// Julian calendar extended forward indefinitely (proleptic).
    ProlepticJulian,
    /// Julian calendar before the given JD, Gregorian after.
    JulianWithCutover { cutover_jd: f64 },
}

/// Convert a calendar date (year, month, day, fractional hour) to a Julian Date.
///
/// # Time scale (approximation)
/// The `hour` is civil clock time (effectively UTC once a tz offset has been
/// removed upstream). The returned [`JdUT1`] LABELS this as UT1, but UTC and UT1
/// are NOT identical: they differ by DUT1 (kept within ±0.9 s by leap-second
/// insertion) plus the leap-second count itself. We treat **UTC ≈ UT1 to within
/// ±0.9 s** and do not apply a DUT1 correction here. For sub-second epoch
/// work feed an already-UT1 instant. (≤0.9 s of UT1 error ≈ ≤13.5″ of Earth
/// rotation; negligible for chart-scale work, material for occultation timing.)
pub fn calendar_to_jd(
    year: i32,
    month: u32,
    day: u32,
    hour: f64,
    calendar: CalendarSystem,
) -> JdUT1 {
    match calendar {
        CalendarSystem::ProlepticGregorian => JdUT1(gregorian_to_jd(year, month, day, hour)),
        CalendarSystem::ProlepticJulian => JdUT1(julian_to_jd(year, month, day, hour)),
        CalendarSystem::JulianWithCutover { cutover_jd } => {
            let jd_greg = gregorian_to_jd(year, month, day, hour);
            let jd_jul = julian_to_jd(year, month, day, hour);
            // The reform skips a block of calendar dates (for the historical
            // 1582 cutover: the ten days 1582-10-05 … 1582-10-14, which never
            // existed — Julian 1582-10-04 was followed directly by Gregorian
            // 1582-10-15). Such a "gap" date is one that the Julian rule places
            // ON OR AFTER the cutover (so it should be Gregorian) while the
            // Gregorian rule places it BEFORE (so it should be Julian) — the two
            // interpretations disagree about which side of the cutover it lies
            // on, which is exactly the signature of a nonexistent date.
            //
            // The old code branched solely on `jd_greg < cutover_jd`, silently
            // running the Julian formula on these gap dates and ALIASING them
            // onto valid post-reform JDs (e.g. 1582-10-05 → the same JD as the
            // first real Gregorian day 1582-10-15). We instead clamp every gap
            // date CONSISTENTLY to the first valid post-cutover instant
            // (`cutover_jd`), so no nonexistent input ever produces a plausible
            // but wrong JD. (`f64`-returning signature is preserved for the
            // public API; callers needing strict rejection can pre-validate.)
            let is_gap = jd_jul >= cutover_jd && jd_greg < cutover_jd;
            if is_gap {
                JdUT1(cutover_jd)
            } else if jd_greg < cutover_jd {
                JdUT1(jd_jul)
            } else {
                JdUT1(jd_greg)
            }
        }
    }
}

/// Smallest / largest Julian Date `jd_to_calendar` will process. Outside this
/// range the year would not fit an `i32` (or the input is non-finite) and the
/// `jd.floor() as i64` cast in `jd_to_gregorian` / `jd_to_julian` would silently
/// saturate (NaN→0, +∞→i64::MAX) producing nonsense dates. The bounds span
/// roughly years −5_000_000 … +5_000_000, comfortably beyond any astronomical
/// use while keeping every downstream integer cast sound.
const JD_TO_CALENDAR_MIN: f64 = -1_825_000_000.0;
const JD_TO_CALENDAR_MAX: f64 = 1_830_000_000.0;

/// Validate that `jd` is finite and within the
/// [`JD_TO_CALENDAR_MIN`]..=[`JD_TO_CALENDAR_MAX`] range before the unguarded
/// `floor() as i64` cast downstream.
/// Non-finite or out-of-range inputs are clamped to the nearest bound (Swiss
/// `swe_revjul` likewise never panics on garbage). NaN → the minimum bound.
fn sanitize_jd_for_calendar(jd: f64) -> f64 {
    if jd.is_nan() {
        return JD_TO_CALENDAR_MIN;
    }
    jd.clamp(JD_TO_CALENDAR_MIN, JD_TO_CALENDAR_MAX)
}

/// Convert a Julian Date to a calendar date (year, month, day, fractional hour).
///
/// Non-finite or astronomically-implausible `jd` values are clamped to a wide
/// representable range (see [`JD_TO_CALENDAR_MIN`]/[`JD_TO_CALENDAR_MAX`]) so the
/// internal `floor() as i64` cast can never saturate into a garbage date.
pub fn jd_to_calendar(jd: f64, calendar: CalendarSystem) -> (i32, u32, u32, f64) {
    let jd = sanitize_jd_for_calendar(jd);
    match calendar {
        CalendarSystem::ProlepticGregorian => jd_to_gregorian(jd),
        CalendarSystem::ProlepticJulian => jd_to_julian(jd),
        CalendarSystem::JulianWithCutover { cutover_jd } => {
            if jd < cutover_jd {
                jd_to_julian(jd)
            } else {
                jd_to_gregorian(jd)
            }
        }
    }
}

// Meeus algorithm — Gregorian calendar to Julian Day Number
fn gregorian_to_jd(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year as f64 - 1.0, month as f64 + 12.0)
    } else {
        (year as f64, month as f64)
    };

    let a = (y / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();

    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day as f64 + hour / 24.0 + b
        - 1524.5
}

fn julian_to_jd(year: i32, month: u32, day: u32, hour: f64) -> f64 {
    let (y, m) = if month <= 2 {
        (year as f64 - 1.0, month as f64 + 12.0)
    } else {
        (year as f64, month as f64)
    };

    (365.25 * (y + 4716.0)).floor() + (30.6001 * (m + 1.0)).floor() + day as f64 + hour / 24.0
        - 1524.5
}

fn jd_to_gregorian(jd: f64) -> (i32, u32, u32, f64) {
    let jd = jd + 0.5;
    let z = jd.floor() as i64;
    let f = jd - z as f64;

    // Proleptic Gregorian: ALWAYS apply the Gregorian correction,
    // regardless of whether z < 2299161 (Oct 15, 1582).
    // The previous code fell back to the Julian formula for dates
    // before the Gregorian adoption, which is wrong for proleptic Gregorian.
    let alpha = ((z as f64 - 1867216.25) / 36524.25).floor() as i64;
    let a = z + 1 + alpha - alpha / 4;

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as u32;
    let month = if e < 14 { e - 1 } else { e - 13 } as u32;
    let year = if month > 2 { c - 4716 } else { c - 4715 } as i32;
    let hour = f * 24.0;

    (year, month, day, hour)
}

fn jd_to_julian(jd: f64) -> (i32, u32, u32, f64) {
    let jd = jd + 0.5;
    let z = jd.floor() as i64;
    let f = jd - z as f64;

    let a = z; // No Gregorian correction
    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25).floor() as i64;
    let d = (365.25 * c as f64).floor() as i64;
    let e = ((b - d) as f64 / 30.6001).floor() as i64;

    let day = (b - d - (30.6001 * e as f64).floor() as i64) as u32;
    let month = if e < 14 { e - 1 } else { e - 13 } as u32;
    let year = if month > 2 { c - 4716 } else { c - 4715 } as i32;
    let hour = f * 24.0;

    (year, month, day, hour)
}

#[allow(dead_code)]
/// Compute the Local Mean Time offset in hours for a given longitude.
pub fn lmt_offset_hours(longitude_deg: f64) -> f64 {
    longitude_deg / 15.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn j2000_epoch() {
        let jd = calendar_to_jd(2000, 1, 1, 12.0, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd.0 - 2_451_545.0).abs() < 1e-6,
            "J2000 should be JD 2451545.0, got {}",
            jd.0
        );
    }

    #[test]
    fn gregorian_roundtrip() {
        let cases = [
            (2000, 1, 1, 12.0),
            (1990, 6, 15, 10.5),
            (1582, 10, 15, 0.0), // First Gregorian day
            (2026, 5, 24, 8.0),
        ];
        for (y, m, d, h) in cases {
            let jd = calendar_to_jd(y, m, d, h, CalendarSystem::ProlepticGregorian);
            let (y2, m2, d2, h2) = jd_to_calendar(jd.0, CalendarSystem::ProlepticGregorian);
            assert_eq!(y, y2, "Year mismatch for {y}-{m}-{d}");
            assert_eq!(m, m2, "Month mismatch for {y}-{m}-{d}");
            assert_eq!(d, d2, "Day mismatch for {y}-{m}-{d}");
            assert!(
                (h - h2).abs() < 0.001,
                "Hour mismatch for {y}-{m}-{d}: {h} vs {h2}"
            );
        }
    }

    #[test]
    fn julian_gregorian_differ() {
        // 1582-10-04 Julian = 1582-10-14 Gregorian (10-day gap)
        let jd_jul = calendar_to_jd(1582, 10, 4, 12.0, CalendarSystem::ProlepticJulian);
        let jd_greg = calendar_to_jd(1582, 10, 14, 12.0, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_jul.0 - jd_greg.0).abs() < 1.0,
            "Julian 1582-10-04 should be near Gregorian 1582-10-14"
        );
    }

    /// C18a: a non-finite or absurd JD must NOT saturate the `floor() as i64`
    /// cast into a garbage date — it is clamped to the representable range and
    /// still yields a well-formed (year, month, day, hour) with the year fitting
    /// an i32 and a valid 1..=12 / 1..=31 month/day.
    #[test]
    fn jd_to_calendar_guards_nonfinite_and_huge() {
        for &(jd, label) in &[
            (f64::NAN, "NaN"),
            (f64::INFINITY, "+inf"),
            (f64::NEG_INFINITY, "-inf"),
            (1.0e300, "huge positive"),
            (-1.0e300, "huge negative"),
        ] {
            for cal in [
                CalendarSystem::ProlepticGregorian,
                CalendarSystem::ProlepticJulian,
            ] {
                let (y, m, d, h) = jd_to_calendar(jd, cal);
                // The clamp keeps the year inside i32 (it already is, by type) and
                // produces a structurally valid date rather than e.g. month 0.
                assert!((1..=12).contains(&m), "{label}: bad month {m}");
                assert!((1..=31).contains(&d), "{label}: bad day {d}");
                assert!((0.0..24.0).contains(&h), "{label}: bad hour {h}");
                // Clamped extremes land near the documented bounds (≈ ±5e6 yr).
                assert!(
                    y.abs() > 100_000,
                    "{label}: expected clamped extreme year, got {y}"
                );
            }
        }
    }

    /// C11b: under `JulianWithCutover`, the nonexistent Gregorian-reform gap
    /// dates 1582-10-05 … 1582-10-14 must NOT silently alias onto valid JDs via
    /// the Julian formula. They are clamped consistently to the cutover instant,
    /// while the surrounding valid dates (Julian 10-04, Gregorian 10-15) keep
    /// their correct, distinct JDs.
    #[test]
    fn julian_cutover_rejects_reform_gap_dates() {
        // Standard historical cutover: first Gregorian instant = 1582-10-15 00:00.
        let cutover_jd = calendar_to_jd(1582, 10, 15, 0.0, CalendarSystem::ProlepticGregorian).0; // 2299160.5
        let cal = CalendarSystem::JulianWithCutover { cutover_jd };

        // Last valid Julian day and first valid Gregorian day bracket the gap and
        // are exactly one day apart in JD — the 10-day gap has been removed.
        let last_julian = calendar_to_jd(1582, 10, 4, 12.0, cal).0;
        let first_gregorian = calendar_to_jd(1582, 10, 15, 12.0, cal).0;
        assert!(
            (first_gregorian - last_julian - 1.0).abs() < 1e-9,
            "Julian 1582-10-04 and Gregorian 1582-10-15 must be 1 day apart, got {} and {}",
            last_julian,
            first_gregorian
        );

        // Every gap date clamps to the cutover instant (NOT to a wrong real JD).
        for d in 5..=14u32 {
            let jd = calendar_to_jd(1582, 10, d, 12.0, cal).0;
            assert!(
                (jd - cutover_jd).abs() < 1e-9,
                "gap date 1582-10-{d:02} must clamp to cutover {cutover_jd}, got {jd}"
            );
            // Critically, it must NOT equal the old aliased value (Julian formula).
            let aliased = julian_to_jd(1582, 10, d, 12.0);
            assert!(
                (jd - aliased).abs() > 0.4,
                "gap date 1582-10-{d:02} still aliases onto the old Julian JD {aliased}"
            );
        }
    }

    #[test]
    fn lmt_pune() {
        let offset = lmt_offset_hours(73.85); // Pune longitude
        assert!(
            (offset - 4.923).abs() < 0.01,
            "Pune LMT offset should be ~4.92h, got {offset}"
        );
    }

    #[test]
    fn proleptic_gregorian_roundtrip_year_1500() {
        // Year 1500 is before the Gregorian adoption (1582).
        // ProlepticGregorian must round-trip correctly at this date.
        let (y, m, d, h) = (1500, 6, 15, 12.0);
        let jd = calendar_to_jd(y, m, d, h, CalendarSystem::ProlepticGregorian);
        let (y2, m2, d2, h2) = jd_to_calendar(jd.0, CalendarSystem::ProlepticGregorian);
        assert_eq!(y, y2, "Year mismatch for proleptic Gregorian 1500-06-15");
        assert_eq!(m, m2, "Month mismatch for proleptic Gregorian 1500-06-15");
        assert_eq!(d, d2, "Day mismatch for proleptic Gregorian 1500-06-15");
        assert!(
            (h - h2).abs() < 0.001,
            "Hour mismatch for proleptic Gregorian 1500-06-15: {h} vs {h2}"
        );
    }

    #[test]
    fn proleptic_gregorian_roundtrip_at_jd_2299161_boundary() {
        // JD 2299161 = the Gregorian cutover boundary (Oct 15, 1582 Gregorian).
        // ProlepticGregorian must use the Gregorian formula on BOTH sides of this boundary.
        let boundary_jd = 2299161.0;

        // Just below the boundary
        let (y1, m1, d1, h1) =
            jd_to_calendar(boundary_jd - 1.0, CalendarSystem::ProlepticGregorian);
        let jd_back = calendar_to_jd(y1, m1, d1, h1, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_back.0 - (boundary_jd - 1.0)).abs() < 1e-6,
            "ProlepticGregorian round-trip below JD 2299161 failed: {} vs {}",
            jd_back.0,
            boundary_jd - 1.0
        );

        // Exactly at the boundary
        let (y2, m2, d2, h2) = jd_to_calendar(boundary_jd, CalendarSystem::ProlepticGregorian);
        let jd_back2 = calendar_to_jd(y2, m2, d2, h2, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_back2.0 - boundary_jd).abs() < 1e-6,
            "ProlepticGregorian round-trip at JD 2299161 failed: {} vs {}",
            jd_back2.0,
            boundary_jd
        );

        // Just above the boundary
        let (y3, m3, d3, h3) =
            jd_to_calendar(boundary_jd + 1.0, CalendarSystem::ProlepticGregorian);
        let jd_back3 = calendar_to_jd(y3, m3, d3, h3, CalendarSystem::ProlepticGregorian);
        assert!(
            (jd_back3.0 - (boundary_jd + 1.0)).abs() < 1e-6,
            "ProlepticGregorian round-trip above JD 2299161 failed: {} vs {}",
            jd_back3.0,
            boundary_jd + 1.0
        );
    }
}
