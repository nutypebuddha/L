//! # Ephemeris — deterministic planetary positions (Jyotisha-grade)
//!
//! Pure functions from Julian Day to graha longitudes. No randomness, no
//! network, no data files, no FFI — the same input always produces the
//! bit-identical output, satisfying Athena's determinism contract.
//!
//! Sources (all permissively licensed, pure Rust):
//! - Planets: VSOP87D heliocentric theory (`vsop87` crate, MIT/Apache-2.0),
//!   converted to geocentric ecliptic-of-date coordinates here.
//! - Moon: ELP-2000/82 truncation from Meeus ch. 47 (`astro` crate, MIT).
//! - Rahu/Ketu: Meeus mean lunar node polynomial (`astro::lunar::mn_ascend_node`).
//!   Mean node (not true node) is the traditional Jyotisha choice.
//!
//! Swiss Ephemeris was evaluated and rejected: every Rust binding is
//! AGPL-3.0 (or the Swiss dual license), incompatible with Athena's
//! MIT/Apache-2.0 licensing and the `cargo deny` allowlist. VSOP87D gives
//! geometric geocentric longitudes accurate to ~1 arcsecond for the inner
//! planets (a few arcseconds for Jupiter/Saturn over ±2000 years) — a
//! nakshatra pada spans 3°20′, so the placement error budget is ~4 orders
//! of magnitude wider than the computation error.
//!
//! Positions are **geometric** (no light-time, aberration, or nutation
//! corrections). Those effects are below ~0.01°, irrelevant for rashi,
//! nakshatra, and pada placement.

use serde::Serialize;

use crate::astrology::{Graha, Nakshatra, Rashi};

/// Degrees per nakshatra: 360° / 27.
const NAKSHATRA_ARC: f64 = 360.0 / 27.0;
/// Degrees per pada: nakshatra arc / 4 = 3°20′.
const PADA_ARC: f64 = NAKSHATRA_ARC / 4.0;

/// Lahiri (Chitrapaksha) ayanamsa at J2000.0, in degrees (23°51′11.5″).
const LAHIRI_J2000: f64 = 23.85320;
/// Mean accumulation rate of the ayanamsa (general precession), °/Julian year.
const LAHIRI_RATE_DEG_PER_YEAR: f64 = 50.2719 / 3600.0;

/// Position of one graha at a moment in time. All longitudes in degrees.
#[derive(Debug, Clone, Serialize)]
pub struct GrahaPosition {
    pub graha: Graha,
    /// Geocentric ecliptic longitude, tropical (equinox of date), 0–360°.
    pub tropical: f64,
    /// Sidereal longitude (tropical − Lahiri ayanamsa), 0–360°.
    pub sidereal: f64,
    /// Sidereal rashi (sign), 30° each.
    pub rashi: Rashi,
    /// Sidereal nakshatra (lunar mansion), 13°20′ each.
    pub nakshatra: Nakshatra,
    /// Pada (quarter) within the nakshatra, 1–4.
    pub pada: u8,
}

/// Normalize an angle in degrees to [0, 360).
fn norm360(deg: f64) -> f64 {
    let d = deg % 360.0;
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

/// Julian Day for a Gregorian calendar date and UT time-of-day.
///
/// `hour` is decimal hours (e.g. 18.5 for 18:30 UT). Deterministic wrapper
/// over Meeus ch. 7 via the `astro` crate.
pub fn julian_day(year: i16, month: u8, day: u8, hour: f64) -> f64 {
    let date = astro::time::Date {
        year,
        month,
        decimal_day: day as f64 + hour / 24.0,
        cal_type: astro::time::CalType::Gregorian,
    };
    astro::time::julian_day(&date)
}

/// Lahiri ayanamsa in degrees at the given Julian Day.
///
/// Linear approximation: J2000 value + general-precession rate. Agrees with
/// the official Lahiri (Chitrapaksha) ayanamsa to ~0.01° over 1900–2100
/// (the nutation wobble and rate curvature it ignores are below that).
pub fn lahiri_ayanamsa(jd: f64) -> f64 {
    LAHIRI_J2000 + LAHIRI_RATE_DEG_PER_YEAR * (jd - 2451545.0) / 365.25
}

/// Geocentric ecliptic-of-date longitude of a VSOP87D planet, in degrees.
///
/// Converts heliocentric spherical coordinates of the planet and Earth to
/// rectangular, differences them, and reads the geocentric longitude.
fn planet_geocentric_longitude(planet: fn(f64) -> vsop87::SphericalCoordinates, jd: f64) -> f64 {
    let rect = |s: vsop87::SphericalCoordinates| {
        let (l, b, r) = (s.longitude(), s.latitude(), s.distance());
        (r * b.cos() * l.cos(), r * b.cos() * l.sin(), r * b.sin())
    };
    let (px, py, _pz) = rect(planet(jd));
    let (ex, ey, _ez) = rect(vsop87::vsop87d::earth(jd));
    norm360((py - ey).atan2(px - ex).to_degrees())
}

/// Geocentric tropical longitude of the Sun: the Earth's heliocentric
/// position reflected through the origin (λ☉ = λ⊕ + 180°).
fn sun_longitude(jd: f64) -> f64 {
    let earth = vsop87::vsop87d::earth(jd);
    norm360(earth.longitude().to_degrees() + 180.0)
}

/// Geocentric tropical longitude of the Moon (ELP-2000/82 truncation).
fn moon_longitude(jd: f64) -> f64 {
    let (point, _dist) = astro::lunar::geocent_ecl_pos(jd);
    norm360(point.long.to_degrees())
}

/// Mean lunar ascending node (Rahu), tropical longitude in degrees.
fn rahu_longitude(jd: f64) -> f64 {
    let jc = (jd - 2451545.0) / 36525.0;
    norm360(astro::lunar::mn_ascend_node(jc).to_degrees())
}

/// Tropical geocentric longitude of any graha at the given Julian Day.
pub fn tropical_longitude(graha: Graha, jd: f64) -> f64 {
    match graha {
        Graha::Surya => sun_longitude(jd),
        Graha::Chandra => moon_longitude(jd),
        Graha::Mangala => planet_geocentric_longitude(vsop87::vsop87d::mars, jd),
        Graha::Budha => planet_geocentric_longitude(vsop87::vsop87d::mercury, jd),
        Graha::Brihaspati => planet_geocentric_longitude(vsop87::vsop87d::jupiter, jd),
        Graha::Shukra => planet_geocentric_longitude(vsop87::vsop87d::venus, jd),
        Graha::Shani => planet_geocentric_longitude(vsop87::vsop87d::saturn, jd),
        Graha::Rahu => rahu_longitude(jd),
        Graha::Ketu => norm360(rahu_longitude(jd) + 180.0),
    }
}

/// Full position (tropical, sidereal, rashi, nakshatra, pada) of one graha.
pub fn graha_position(graha: Graha, jd: f64) -> GrahaPosition {
    let tropical = tropical_longitude(graha, jd);
    let sidereal = norm360(tropical - lahiri_ayanamsa(jd));
    GrahaPosition {
        graha,
        tropical,
        sidereal,
        rashi: Rashi::from_index((sidereal / 30.0) as usize),
        nakshatra: Nakshatra::from_index((sidereal / NAKSHATRA_ARC) as usize),
        pada: ((sidereal % NAKSHATRA_ARC) / PADA_ARC) as u8 + 1,
    }
}

/// Positions of all 9 grahas at the given Julian Day, in wheel order.
pub fn all_graha_positions(jd: f64) -> Vec<GrahaPosition> {
    (0..9)
        .map(|i| graha_position(Graha::from_index(i), jd))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_julian_day_meeus_7a() {
        // Meeus ex. 7.a: 1957 Oct 4.81 UT → JD 2436116.31
        let jd = julian_day(1957, 10, 4, 0.81 * 24.0);
        assert!((jd - 2436116.31).abs() < 1e-6, "jd = {jd}");
        // J2000.0 epoch: 2000 Jan 1.5 → JD 2451545.0
        assert_eq!(julian_day(2000, 1, 1, 12.0), 2451545.0);
    }

    #[test]
    fn test_norm360() {
        assert_eq!(norm360(370.0), 10.0);
        assert_eq!(norm360(-10.0), 350.0);
        assert_eq!(norm360(0.0), 0.0);
    }

    #[test]
    fn test_sun_longitude_meeus_25b() {
        // Meeus ex. 25.b (VSOP87): 1992 Oct 13.0 TT → geometric λ☉ = 199°54′26.18″
        let jd = 2448908.5;
        let sun = sun_longitude(jd);
        let expected = 199.0 + 54.0 / 60.0 + 26.18 / 3600.0;
        assert!(
            (sun - expected).abs() < 0.001,
            "sun = {sun}, expected {expected}"
        );
    }

    #[test]
    fn test_moon_longitude_meeus_47a() {
        // Meeus ex. 47.a: 1992 Apr 12.0 TT → λ☾ = 133.162655° (geometric)
        let jd = 2448724.5;
        let moon = moon_longitude(jd);
        assert!((moon - 133.162655).abs() < 0.01, "moon = {moon}");
    }

    #[test]
    fn test_rahu_at_j2000() {
        // Meeus mean node polynomial constant term: 125.0445479° at J2000.0
        let rahu = rahu_longitude(2451545.0);
        assert!((rahu - 125.0445479).abs() < 1e-6, "rahu = {rahu}");
        // Ketu is always the opposite point
        let ketu = tropical_longitude(Graha::Ketu, 2451545.0);
        assert!((ketu - norm360(rahu + 180.0)).abs() < 1e-9);
    }

    #[test]
    fn test_lahiri_ayanamsa_range() {
        // Published Lahiri values: ~23.85° at 2000, ~24.10° at 2018, ~24.21° at 2026
        let ay2000 = lahiri_ayanamsa(2451545.0);
        assert!((ay2000 - 23.853).abs() < 0.01, "ay2000 = {ay2000}");
        let ay2026 = lahiri_ayanamsa(julian_day(2026, 7, 7, 0.0));
        assert!((ay2026 - 24.22).abs() < 0.05, "ay2026 = {ay2026}");
    }

    #[test]
    fn test_rashi_nakshatra_pada_derivation() {
        // Sidereal 0° → Mesha / Ashwini pada 1; 13°20′ → Bharani;
        // 29.9° → Mesha; 30.1° → Vrishabha.
        let pos = |sidereal: f64| {
            let rashi = Rashi::from_index((sidereal / 30.0) as usize);
            let nak = Nakshatra::from_index((sidereal / NAKSHATRA_ARC) as usize);
            let pada = ((sidereal % NAKSHATRA_ARC) / PADA_ARC) as u8 + 1;
            (rashi, nak, pada)
        };
        assert_eq!(pos(0.0), (Rashi::Mesha, Nakshatra::Ashwini, 1));
        assert_eq!(pos(10.0), (Rashi::Mesha, Nakshatra::Ashwini, 4));
        assert_eq!(pos(NAKSHATRA_ARC + 0.1).1, Nakshatra::Bharani);
        assert_eq!(pos(29.9).0, Rashi::Mesha);
        assert_eq!(pos(30.1).0, Rashi::Vrishabha);
        assert_eq!(pos(359.9), (Rashi::Meena, Nakshatra::Revati, 4));
    }

    #[test]
    fn test_all_grahas_deterministic() {
        // Same input must produce bit-identical output — Athena's core contract.
        let jd = julian_day(2026, 7, 7, 12.0);
        let a = all_graha_positions(jd);
        let b = all_graha_positions(jd);
        assert_eq!(a.len(), 9);
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.tropical.to_bits(), y.tropical.to_bits());
            assert_eq!(x.sidereal.to_bits(), y.sidereal.to_bits());
        }
    }

    #[test]
    fn test_positions_in_range() {
        let jd = julian_day(2026, 7, 7, 0.0);
        for pos in all_graha_positions(jd) {
            assert!((0.0..360.0).contains(&pos.tropical), "{:?}", pos);
            assert!((0.0..360.0).contains(&pos.sidereal), "{:?}", pos);
            assert!((1..=4).contains(&pos.pada), "{:?}", pos);
        }
    }
}
