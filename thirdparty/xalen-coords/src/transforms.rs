use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EclipticPosition {
    pub longitude: f64, // radians
    pub latitude: f64,  // radians
    pub distance: f64,  // AU
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EquatorialPosition {
    pub right_ascension: f64, // radians
    pub declination: f64,     // radians
    pub distance: f64,        // AU
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CartesianPosition {
    pub x: f64, // AU
    pub y: f64, // AU
    pub z: f64, // AU
}

impl EclipticPosition {
    pub fn longitude_deg(&self) -> f64 {
        self.longitude.to_degrees()
    }
    pub fn latitude_deg(&self) -> f64 {
        self.latitude.to_degrees()
    }

    pub fn normalize(mut self) -> Self {
        self.longitude = self.longitude.rem_euclid(std::f64::consts::TAU);
        self
    }
}

/// Rate of change of an ecliptic position — the body's apparent daily motion.
///
/// Longitude/latitude rates are in **radians per day**, distance in **AU per
/// day**. The longitude rate is the quantity astrologers call "speed": negative
/// means retrograde. Use the `*_deg_per_day` helpers for degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EclipticSpeed {
    pub longitude: f64, // radians/day (negative = retrograde)
    pub latitude: f64,  // radians/day
    pub distance: f64,  // AU/day
}

impl EclipticSpeed {
    pub fn longitude_deg_per_day(&self) -> f64 {
        self.longitude.to_degrees()
    }
    pub fn latitude_deg_per_day(&self) -> f64 {
        self.latitude.to_degrees()
    }
    /// True when the body is moving retrograde (decreasing ecliptic longitude).
    pub fn is_retrograde(&self) -> bool {
        self.longitude < 0.0
    }
}

impl EquatorialPosition {
    pub fn ra_hours(&self) -> f64 {
        self.right_ascension.to_degrees() / 15.0
    }
    pub fn dec_deg(&self) -> f64 {
        self.declination.to_degrees()
    }
}

pub fn ecliptic_to_equatorial(ecl: &EclipticPosition, epsilon: f64) -> EquatorialPosition {
    let cos_eps = epsilon.cos();
    let sin_eps = epsilon.sin();
    let cos_lat = ecl.latitude.cos();
    let sin_lat = ecl.latitude.sin();
    let cos_lon = ecl.longitude.cos();
    let sin_lon = ecl.longitude.sin();

    let ra = (sin_lon * cos_eps - sin_lat / cos_lat * sin_eps).atan2(cos_lon);
    let dec = (sin_lat * cos_eps + cos_lat * sin_eps * sin_lon).asin();

    EquatorialPosition {
        right_ascension: ra.rem_euclid(std::f64::consts::TAU),
        declination: dec,
        distance: ecl.distance,
    }
}

pub fn equatorial_to_ecliptic(eq: &EquatorialPosition, epsilon: f64) -> EclipticPosition {
    let cos_eps = epsilon.cos();
    let sin_eps = epsilon.sin();
    let cos_dec = eq.declination.cos();
    let sin_dec = eq.declination.sin();
    let cos_ra = eq.right_ascension.cos();
    let sin_ra = eq.right_ascension.sin();

    let lon = (sin_ra * cos_eps + sin_dec / cos_dec * sin_eps).atan2(cos_ra);
    let lat = (sin_dec * cos_eps - cos_dec * sin_eps * sin_ra).asin();

    EclipticPosition {
        longitude: lon.rem_euclid(std::f64::consts::TAU),
        latitude: lat,
        distance: eq.distance,
    }
}

pub fn ecliptic_to_cartesian(ecl: &EclipticPosition) -> CartesianPosition {
    let cos_lat = ecl.latitude.cos();
    CartesianPosition {
        x: ecl.distance * cos_lat * ecl.longitude.cos(),
        y: ecl.distance * cos_lat * ecl.longitude.sin(),
        z: ecl.distance * ecl.latitude.sin(),
    }
}

pub fn cartesian_to_ecliptic(cart: &CartesianPosition) -> EclipticPosition {
    let r = (cart.x * cart.x + cart.y * cart.y + cart.z * cart.z).sqrt();
    let lon = cart.y.atan2(cart.x);
    // Distinguish a genuine null vector (r == 0 → SOFA returns zero latitude)
    // from a NaN-contaminated input (r is NaN → propagate NaN rather than
    // silently reporting the equator). `r > 0.0` is false for BOTH cases, so a
    // bare `else { 0.0 }` would mask invalid input as latitude 0.
    let lat = if r > 0.0 {
        (cart.z / r).asin()
    } else if r == 0.0 {
        0.0
    } else {
        f64::NAN
    };

    EclipticPosition {
        longitude: lon.rem_euclid(std::f64::consts::TAU),
        latitude: lat,
        distance: r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEG_TO_RAD, RAD_TO_DEG};

    #[test]
    fn ecliptic_equatorial_roundtrip() {
        let epsilon = 23.4393 * DEG_TO_RAD;
        let ecl = EclipticPosition {
            longitude: 45.0 * DEG_TO_RAD,
            latitude: 5.0 * DEG_TO_RAD,
            distance: 1.0,
        };
        let eq = ecliptic_to_equatorial(&ecl, epsilon);
        let ecl2 = equatorial_to_ecliptic(&eq, epsilon);
        assert!(
            (ecl.longitude - ecl2.longitude).abs() < 1e-10,
            "Longitude roundtrip failed"
        );
        assert!(
            (ecl.latitude - ecl2.latitude).abs() < 1e-10,
            "Latitude roundtrip failed"
        );
    }

    #[test]
    fn cartesian_ecliptic_roundtrip() {
        let ecl = EclipticPosition {
            longitude: 120.0 * DEG_TO_RAD,
            latitude: -3.0 * DEG_TO_RAD,
            distance: 5.2,
        };
        let cart = ecliptic_to_cartesian(&ecl);
        let ecl2 = cartesian_to_ecliptic(&cart);
        assert!((ecl.longitude - ecl2.longitude).abs() < 1e-10);
        assert!((ecl.latitude - ecl2.latitude).abs() < 1e-10);
        assert!((ecl.distance - ecl2.distance).abs() < 1e-10);
    }

    #[test]
    fn vernal_equinox_ra_is_zero() {
        let epsilon = 23.4393 * DEG_TO_RAD;
        let ecl = EclipticPosition {
            longitude: 0.0,
            latitude: 0.0,
            distance: 1.0,
        };
        let eq = ecliptic_to_equatorial(&ecl, epsilon);
        assert!(
            eq.right_ascension.abs() < 1e-10,
            "RA at vernal equinox should be 0"
        );
        assert!(
            eq.declination.abs() < 1e-10,
            "Dec at vernal equinox should be 0"
        );
    }

    #[test]
    fn summer_solstice_dec_equals_obliquity() {
        let epsilon = 23.4393 * DEG_TO_RAD;
        let ecl = EclipticPosition {
            longitude: 90.0 * DEG_TO_RAD,
            latitude: 0.0,
            distance: 1.0,
        };
        let eq = ecliptic_to_equatorial(&ecl, epsilon);
        assert!(
            (eq.declination - epsilon).abs() < 1e-10,
            "Dec at summer solstice should equal obliquity: {} vs {}",
            eq.dec_deg(),
            epsilon * RAD_TO_DEG
        );
    }

    #[test]
    fn cartesian_to_ecliptic_zero_vector_is_finite() {
        // A zero vector (r == 0) must not produce NaN latitude (0/0).
        let ecl = cartesian_to_ecliptic(&CartesianPosition {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });
        assert!(ecl.longitude.is_finite(), "longitude must be finite");
        assert!(ecl.latitude.is_finite(), "latitude must be finite");
        assert!(ecl.distance.is_finite(), "distance must be finite");
        assert_eq!(ecl.latitude, 0.0, "zero vector latitude defaults to 0");
        assert_eq!(ecl.distance, 0.0, "zero vector distance is 0");
    }

    #[test]
    fn cartesian_to_ecliptic_nan_input_propagates_nan() {
        // A NaN-contaminated vector must NOT be silently reported as the equator
        // (latitude 0): `r` becomes NaN, so latitude must propagate NaN rather
        // than taking the zero-vector fallback path.
        let ecl = cartesian_to_ecliptic(&CartesianPosition {
            x: f64::NAN,
            y: 0.0,
            z: 1.0,
        });
        assert!(
            ecl.latitude.is_nan(),
            "NaN input must propagate a NaN latitude, not silently report 0"
        );
    }

    #[test]
    fn normalize_longitude() {
        let ecl = EclipticPosition {
            longitude: -30.0 * DEG_TO_RAD,
            latitude: 0.0,
            distance: 1.0,
        };
        let n = ecl.normalize();
        assert!(n.longitude >= 0.0 && n.longitude < std::f64::consts::TAU);
    }
}
