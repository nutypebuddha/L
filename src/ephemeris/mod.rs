//! # Ephemeris — deterministic planetary positions (Jyotisha-grade)
//!
//! Pure functions from Julian Day to graha longitudes. No randomness, no
//! network, no data files, no FFI — the same input always produces
//! bit-identical output across CPU architectures.
//!
//! ## Sources
//!
//! - **Planets:** VSOP87D heliocentric theory (`vsop87` crate),
//!   converted to geocentric ecliptic-of-date coordinates.
//! - **Moon:** ELP-2000/82 truncation via `astro` crate.
//! - **Rahu/Ketu:** Meeus mean lunar node polynomial via `astro`.
//! - **Ayanamsa:** `xalen-ayanamsa` (IAU 2006/P03 precession).
//! - **Rashi / Nakshatra:** `xalen-vedic` from sidereal longitude.

use serde::{Deserialize, Serialize};

use crate::astrology::{Graha, Nakshatra, Rashi};

/// Which ayanamsa (tropical→sidereal precession offset) system a chart uses.
///
/// Part 1.4: the numeric ayanamsa value is meaningless without naming the
/// *method* — official Lahiri (linear from the 285 CE epoch) and "True
/// Chitrapaksha" (re-anchors Spica's real position each date) diverge by
/// 30″–60″, enough to flip a nakshatra pada or sub-lord near a cusp. Laverna
/// pins and emits the method on every chart so output is self-documenting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AyanamsaSystem {
    /// Lahiri (Chitrapaksha) — linear formula from the 285 CE zero-point.
    /// Official Indian Ephemeris & Nautical Almanac system.
    #[default]
    Lahiri,
    /// True Chitrapaksha — Spica anchored at exactly 180° sidereal of date
    /// (IAU 2006/P03 precession + IAU 2000B nutation + aberration). Diverges
    /// from Lahiri by ~30″–60″.
    TrueChitra,
}

impl AyanamsaSystem {
    /// The underlying xalen ayanamsa model.
    fn inner(self) -> xalen_ayanamsa::Ayanamsa {
        match self {
            AyanamsaSystem::Lahiri => xalen_ayanamsa::Ayanamsa::Lahiri,
            AyanamsaSystem::TrueChitra => xalen_ayanamsa::Ayanamsa::TrueChitra,
        }
    }

    /// Human-readable method name, e.g. "Lahiri (Chitrapaksha)".
    pub fn name(self) -> String {
        self.inner().to_string()
    }

    /// Parse a CLI token (`lahiri` | `true-chitra`). Defaults to Lahiri.
    pub fn from_cli(token: Option<&str>) -> Result<Self, String> {
        match token {
            None | Some("lahiri") | Some("chitrapaksha") => Ok(AyanamsaSystem::Lahiri),
            Some("true-chitra") | Some("true-chitrapaksha") => Ok(AyanamsaSystem::TrueChitra),
            Some(other) => Err(format!(
                "unknown ayanamsa system '{other}': expected 'lahiri' or 'true-chitra'"
            )),
        }
    }
}

/// Ayanamsa in degrees at the given UT1 Julian Day for a specific system.
///
/// Uses IAU 2006/P03 precession (via `xalen-ayanamsa`), converting UT1→TT
/// internally through `xalen-time` Delta-T.
pub fn ayanamsa_deg(jd_ut1: f64, system: AyanamsaSystem) -> f64 {
    let delta_t = xalen_time::delta_t(
        jd_ut1,
        &xalen_time::DeltaTModel::StephensonMorrisonHohenkerk2016,
    ) / 86400.0;
    let jd_tt = jd_ut1 + delta_t;
    system.inner().compute_deg(jd_tt)
}

/// Position of one graha at a moment in time. All longitudes in degrees.
#[derive(Debug, Clone, Deserialize)]
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

/// Quantize a longitude to 1e-9° so that identical computations on different
/// CPU architectures serialize byte-identically. 1e-9° is ~10,000× larger
/// than cross-architecture drift and far below any bucket boundary.
fn quantize_longitude(deg: f64) -> f64 {
    (deg * 1e9).round() / 1e9
}

impl Serialize for GrahaPosition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("GrahaPosition", 6)?;
        s.serialize_field("graha", &self.graha)?;
        s.serialize_field("tropical", &quantize_longitude(self.tropical))?;
        s.serialize_field("sidereal", &quantize_longitude(self.sidereal))?;
        s.serialize_field("rashi", &self.rashi)?;
        s.serialize_field("nakshatra", &self.nakshatra)?;
        s.serialize_field("pada", &self.pada)?;
        s.end()
    }
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
/// `hour` is decimal hours (e.g. 18.5 for 18:30 UT).
pub fn julian_day(year: i16, month: u8, day: u8, hour: f64) -> f64 {
    let date = astro::time::Date {
        year,
        month,
        decimal_day: day as f64 + hour / 24.0,
        cal_type: astro::time::CalType::Gregorian,
    };
    astro::time::julian_day(&date)
}

/// Julian Day number to Gregorian date components (year, month, day).
pub fn julian_day_to_date(julian_day: f64) -> (i32, u8, u8) {
    let jd = julian_day + 0.5;
    let z = jd as i64;

    let a = if z < 2299161 {
        z
    } else {
        let alpha = ((z as f64 - 1867216.25) / 36524.25) as i64;
        z + 1 + alpha - alpha / 4
    };

    let b = a + 1524;
    let c = ((b as f64 - 122.1) / 365.25) as i64;
    let d = (365.25 * c as f64) as i64;
    let e = ((b - d) as f64 / 30.6001) as i64;

    let day = (b - d - (30.6001 * e as f64) as i64) as u8;
    let month = if e < 14 {
        (e - 1) as u8
    } else {
        (e - 13) as u8
    };
    let year = if month > 2 { c - 4716 } else { c - 4715 };

    (year as i32, month, day)
}

/// Lahiri ayanamsa in degrees at the given UT1 Julian Day.
///
/// Convenience wrapper around [`ayanamsa_deg`] with the default
/// (`AyanamsaSystem::Lahiri`) system. Kept for backward compatibility.
pub fn lahiri_ayanamsa(jd_ut1: f64) -> f64 {
    ayanamsa_deg(jd_ut1, AyanamsaSystem::Lahiri)
}

/// Geocentric ecliptic-of-date longitude of a VSOP87D planet, in degrees.
fn planet_geocentric_longitude(planet: fn(f64) -> vsop87::SphericalCoordinates, jd: f64) -> f64 {
    let rect = |s: vsop87::SphericalCoordinates| {
        let (l, b, r) = (s.longitude(), s.latitude(), s.distance());
        (r * b.cos() * l.cos(), r * b.cos() * l.sin(), r * b.sin())
    };
    let (px, py, _pz) = rect(planet(jd));
    let (ex, ey, _ez) = rect(vsop87::vsop87d::earth(jd));
    norm360((py - ey).atan2(px - ex).to_degrees())
}

/// Geocentric tropical longitude of the Sun.
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
///
/// `system` selects the ayanamsa method used for the sidereal reduction
/// (Part 1.4 — must be named, not assumed).
pub fn graha_position(graha: Graha, jd: f64, system: AyanamsaSystem) -> GrahaPosition {
    let tropical = tropical_longitude(graha, jd);
    let sidereal = norm360(tropical - ayanamsa_deg(jd, system));
    let xrashi = xalen_vedic::rashi::Rashi::from_longitude_deg(sidereal);
    let xnak = xalen_vedic::nakshatra::Nakshatra::from_longitude_deg(sidereal);
    let pada = xalen_vedic::nakshatra::Nakshatra::pada(sidereal);
    GrahaPosition {
        graha,
        tropical,
        sidereal,
        rashi: Rashi::from_index(xrashi.index()),
        nakshatra: Nakshatra::from_index(xnak.index()),
        pada,
    }
}

/// Positions of all 9 grahas at the given Julian Day, in wheel order, using
/// the default (Lahiri) ayanamsa system.
pub fn all_graha_positions(jd: f64) -> Vec<GrahaPosition> {
    all_graha_positions_with(jd, AyanamsaSystem::default())
}

/// Positions of all 9 grahas at the given Julian Day with an explicit
/// ayanamsa system.
pub fn all_graha_positions_with(jd: f64, system: AyanamsaSystem) -> Vec<GrahaPosition> {
    (0..9)
        .map(|i| {
            graha_position(
                Graha::from_index(i).expect("valid graha index 0..9"),
                jd,
                system,
            )
        })
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
    fn test_julian_day_to_date_basic() {
        let (year, month, day) = julian_day_to_date(2451545.0);
        assert_eq!(year, 2000);
        assert_eq!(month, 1);
        assert_eq!(day, 1);
    }

    #[test]
    fn test_norm360() {
        assert_eq!(norm360(370.0), 10.0);
        assert_eq!(norm360(-10.0), 350.0);
        assert_eq!(norm360(0.0), 0.0);
    }

    #[test]
    fn test_sun_longitude_meeus_25b() {
        // Meeus ex. 25.b (VSOP87): 1992 Oct 13.0 TT → geometric λ☉ ≈ 199°54′26.18″
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
        // Meeus ex. 47.a: 1992 Apr 12.0 TT → λ☾ ≈ 133.162655°
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
        // Published Lahiri values: ~23.85° at 2000, ~24.22° at 2026
        let ay2000 = lahiri_ayanamsa(2451545.0);
        assert!((ay2000 - 23.853).abs() < 0.01, "ay2000 = {ay2000}");
        let ay2026 = lahiri_ayanamsa(julian_day(2026, 7, 7, 0.0));
        assert!((ay2026 - 24.22).abs() < 0.05, "ay2026 = {ay2026}");
    }

    #[test]
    fn test_rashi_nakshatra_pada_derivation() {
        use xalen_vedic::nakshatra::Nakshatra as XNak;
        use xalen_vedic::rashi::Rashi as XRashi;
        let pos = |sidereal: f64| {
            let r = XRashi::from_longitude_deg(sidereal);
            let n = XNak::from_longitude_deg(sidereal);
            let p = XNak::pada(sidereal);
            (
                Rashi::from_index(r.index()),
                Nakshatra::from_index(n.index()),
                p,
            )
        };
        assert_eq!(pos(0.0), (Rashi::Mesha, Nakshatra::Ashwini, 1));
        assert_eq!(pos(10.0), (Rashi::Mesha, Nakshatra::Ashwini, 4));
        assert_eq!(pos(360.0 / 27.0 + 0.1).1, Nakshatra::Bharani);
        assert_eq!(pos(29.9).0, Rashi::Mesha);
        assert_eq!(pos(30.1).0, Rashi::Vrishabha);
        assert_eq!(pos(359.9), (Rashi::Meena, Nakshatra::Revati, 4));
    }

    #[test]
    fn test_all_grahas_deterministic() {
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
