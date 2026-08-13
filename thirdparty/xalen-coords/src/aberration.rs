//! Annual aberration of starlight.
//!
//! Annual aberration is the apparent angular displacement of a star caused by
//! the finite speed of light combined with the Earth's orbital velocity. Its
//! magnitude is bounded by the **constant of aberration**
//! κ = 20.49552″ (IAU 1976 system; the value used by Swiss Ephemeris and the
//! Astronomical Almanac for the mean Earth orbital speed).
//!
//! For a star at ecliptic longitude λ and latitude β, the first-order
//! displacement in ecliptic longitude (the dominant, circular-orbit term) is
//!
//! ```text
//! Δλ_aberr = −κ · cos(⊙ − λ) / cos β            (″, circular term)
//! ```
//!
//! where ⊙ is the Sun's geometric (true) ecliptic longitude of date. The full
//! expression carries a second, eccentricity term
//! `+ e · κ · cos(ϖ − λ) / cos β` (ϖ = longitude of perihelion, e ≈ 0.0167),
//! which contributes at most ~0.34″ and is included here when the Sun's
//! longitude is supplied.
//!
//! IMPORTANT — circular dependency note: the *full* longitude term requires the
//! Sun's geometric longitude of date, which lives in the planetary engine
//! (`xalen-vsop87` / higher crates), not in `xalen-coords`. To avoid a
//! dependency cycle, this module does NOT compute the Sun's position itself.
//! Callers that already have ⊙ pass it to [`annual_aberration_longitude`].
//! Callers that do not have ⊙ can still bound the effect with
//! [`constant_of_aberration`] and must DOCUMENT that the directional (full)
//! term is omitted — we never silently fake a Sun longitude.

use crate::ARCSEC_TO_RAD;

/// IAU 1976 constant of aberration, κ, in arcseconds.
///
/// κ = 20.49552″. This is the maximum magnitude of the annual-aberration
/// displacement for a star on the ecliptic; the actual displacement scales it
/// by `cos(⊙ − λ) / cos β`. Source: IAU (1976) System of Astronomical
/// Constants; identical to the value Swiss Ephemeris uses internally.
pub const CONSTANT_OF_ABERRATION_ARCSEC: f64 = 20.49552;

/// Mean eccentricity of the Earth's orbit at J2000 (used for the small
/// e·κ eccentricity term of annual aberration). Source: VSOP87 / IAU 2006.
const EARTH_ECCENTRICITY_J2000: f64 = 0.016_708_634;

/// Mean longitude of perihelion of the Earth's orbit at J2000, in degrees
/// (ϖ ≈ 102.937°). Used only for the small eccentricity term.
const PERIHELION_LONGITUDE_J2000_DEG: f64 = 102.937_348;

/// Return the constant of aberration κ in **radians**.
///
/// This is the bound on the annual-aberration displacement, NOT the
/// displacement itself. Use it when the Sun's geometric longitude is not
/// available and you only need to bound or annotate the effect.
pub fn constant_of_aberration() -> f64 {
    CONSTANT_OF_ABERRATION_ARCSEC * ARCSEC_TO_RAD
}

/// Annual aberration in **ecliptic longitude**, in radians.
///
/// Computes the first-order (circular) term plus the small eccentricity term:
///
/// ```text
/// Δλ = [ −κ·cos(⊙ − λ) + e·κ·cos(ϖ − λ) ] / cos β
/// ```
///
/// # Arguments
/// * `star_lon_rad` — the star's ecliptic longitude of date, λ (radians).
/// * `star_lat_rad` — the star's ecliptic latitude, β (radians).
/// * `sun_lon_rad`  — the Sun's geometric (true) ecliptic longitude of date,
///   ⊙ (radians). The caller MUST supply a real value — this module never
///   fabricates one.
///
/// Returns the displacement to ADD to the star's geometric longitude to obtain
/// the aberration-corrected (apparent) longitude.
pub fn annual_aberration_longitude(star_lon_rad: f64, star_lat_rad: f64, sun_lon_rad: f64) -> f64 {
    let kappa = constant_of_aberration();
    let cos_beta = star_lat_rad.cos();
    // Guard against the (astronomically impossible for catalogued stars) pole
    // singularity at β = ±90°.
    if cos_beta.abs() < 1e-12 {
        return 0.0;
    }
    let circular = -kappa * (sun_lon_rad - star_lon_rad).cos();
    let perihelion = PERIHELION_LONGITUDE_J2000_DEG.to_radians();
    let eccentric = EARTH_ECCENTRICITY_J2000 * kappa * (perihelion - star_lon_rad).cos();
    (circular + eccentric) / cos_beta
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RAD_TO_DEG;

    #[test]
    fn kappa_is_iau1976_value() {
        assert!((CONSTANT_OF_ABERRATION_ARCSEC - 20.49552).abs() < 1e-9);
        let k_arcsec = constant_of_aberration() / ARCSEC_TO_RAD;
        assert!((k_arcsec - 20.49552).abs() < 1e-9);
    }

    #[test]
    fn aberration_bounded_by_kappa() {
        // For a star ON the ecliptic (β = 0), the displacement must never
        // exceed κ·(1 + e) in magnitude across all Sun positions.
        let kappa_deg = CONSTANT_OF_ABERRATION_ARCSEC / 3600.0;
        let bound = kappa_deg * (1.0 + EARTH_ECCENTRICITY_J2000);
        for i in 0..360 {
            let sun = (i as f64).to_radians();
            let lam = 100.0_f64.to_radians();
            let d = annual_aberration_longitude(lam, 0.0, sun) * RAD_TO_DEG;
            assert!(
                d.abs() <= bound + 1e-9,
                "aberration {d}° exceeds bound {bound}° at sun {i}°"
            );
        }
    }

    #[test]
    fn aberration_vanishes_at_quadrature() {
        // When ⊙ − λ = ±90°, the dominant circular term is zero; only the tiny
        // eccentricity term remains (< 0.35″).
        let lam = 100.0_f64.to_radians();
        let sun = (100.0 + 90.0_f64).to_radians();
        let d_arcsec = annual_aberration_longitude(lam, 0.0, sun) / ARCSEC_TO_RAD;
        assert!(
            d_arcsec.abs() < 0.35,
            "at quadrature only the eccentricity term (<0.35\") should remain, got {d_arcsec}\""
        );
    }

    #[test]
    fn latitude_amplifies_displacement() {
        // cos β in the denominator increases |Δλ| for non-zero latitude.
        let lam = 50.0_f64.to_radians();
        let sun = 50.0_f64.to_radians(); // ⊙ = λ → maximum circular term
        let on_ecliptic = annual_aberration_longitude(lam, 0.0, sun).abs();
        let off_ecliptic = annual_aberration_longitude(lam, 30.0_f64.to_radians(), sun).abs();
        assert!(off_ecliptic > on_ecliptic);
    }
}
