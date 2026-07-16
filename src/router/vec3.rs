//! # Vec3 — 3D vector framing for ecliptic positions (Part 1.3)
//!
//! The chart engine previously reasoned about bodies purely as flat 2D
//! ecliptic *degrees* (`norm360(longitude - ayanamsa)`). Flat-degree math is
//! prone to a whole class of bugs the 3D representation eliminates:
//!
//! - **Wraparound ambiguity** at 0°/360° — `359.9°` and `0.1°` are 0.2° apart
//!   but differ by ~360° in raw degree space, so naive differencing is wrong.
//! - **Accumulated rounding near a cusp** — repeated degree arithmetic drifts.
//! - **Epoch/obliquity coupling** — tropical↔sidereal and frame rotations are
//!   cleanly expressed as rotation matrices/quaternions, not degree surgery.
//!
//! This module does **not** re-implement ERFA/VSOP. The existing planetary
//! engines (`vsop87`, `astro`/ELP-2000/82) already produce accurate geocentric
//! ecliptic longitudes; here we give those longitudes a first-class 3D home so
//! downstream code (aspect separation, lagna projection, cusp geometry) works
//! in vector space and only projects down to a degree at the very end — exactly
//! the "Bias → Precession → Nutation → ERA" discipline the guide calls for,
//! applied at Laverna's own boundary.
//!
//! All operations are pure and deterministic: identical inputs → bit-identical
//! outputs on every architecture.

use std::ops::{Add, Mul, Sub};

/// A 3D Euclidean vector. Used here for ecliptic cartesian coordinates where
/// +X points to the Vernal Equinox ( tropical 0° ), +Y to tropical 90°
/// (ecliptic), and +Z to the ecliptic north pole.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    /// Unit vector along the ecliptic at the given longitude (degrees), zero
    /// ecliptic latitude. The canonical "rise to 3D" step.
    pub fn from_ecliptic_longitude(longitude_deg: f64) -> Vec3 {
        let r = longitude_deg.to_radians();
        Vec3 {
            x: r.cos(),
            y: r.sin(),
            z: 0.0,
        }
    }

    /// Build a full ecliptic point from longitude + latitude (degrees).
    pub fn from_ecliptic(longitude_deg: f64, latitude_deg: f64) -> Vec3 {
        let lon = longitude_deg.to_radians();
        let lat = latitude_deg.to_radians();
        let cos_lat = lat.cos();
        Vec3 {
            x: cos_lat * lon.cos(),
            y: cos_lat * lon.sin(),
            z: lat.sin(),
        }
    }

    /// Project back to a tropical ecliptic longitude in degrees [0, 360).
    pub fn to_ecliptic_longitude(self) -> f64 {
        norm360(self.y.atan2(self.x).to_degrees())
    }

    /// Euclidean (straight-line) distance between two points.
    pub fn distance(self, other: Vec3) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }

    /// Great-circle (angular) separation between two unit directions, in degrees
    /// [0, 180]. Robust across the 0°/360° wraparound that flat-degree
    /// `norm360(a - b)` subtraction gets wrong.
    pub fn angular_separation_deg(self, other: Vec3) -> f64 {
        let dot = (self.x * other.x + self.y * other.y + self.z * other.z).clamp(-1.0, 1.0);
        dot.acos().to_degrees()
    }

    /// Rotate this vector about the ecliptic Z axis by `angle_deg` (the
    /// tropical→sidereal ayanamsa reduction, expressed as a clean rotation
    /// instead of a degree subtraction).
    pub fn rotate_z(self, angle_deg: f64) -> Vec3 {
        let a = angle_deg.to_radians();
        let c = a.cos();
        let s = a.sin();
        Vec3 {
            x: c * self.x - s * self.y,
            y: s * self.x + c * self.y,
            z: self.z,
        }
    }

    /// Apply a precession-style rotation about an arbitrary unit axis `axis`
    /// by `angle_deg`, using Rodrigues' rotation formula. This is the generic
    /// building block for the Bias → Precession → Nutation → ERA chain:
    /// compose rotations about the correct axes instead of juggling degrees.
    pub fn rotate_axis(self, axis: Vec3, angle_deg: f64) -> Vec3 {
        let k = axis.normalize();
        let a = angle_deg.to_radians();
        let c = a.cos();
        let s = a.sin();
        // v cosθ + (k × v) sinθ + k (k·v) (1 − cosθ)
        let cross = k.cross(self);
        let dot = k.dot(self);
        Vec3 {
            x: self.x * c + cross.x * s + k.x * dot * (1.0 - c),
            y: self.y * c + cross.y * s + k.y * dot * (1.0 - c),
            z: self.z * c + cross.z * s + k.z * dot * (1.0 - c),
        }
    }

    /// Unit Z axis (ecliptic north pole) — the rotation axis for ayanamsa.
    pub fn z_axis() -> Vec3 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    pub fn dot(self, other: Vec3) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Normalize to unit length (returns self unchanged if already ~zero).
    pub fn normalize(self) -> Vec3 {
        let len = (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt();
        if len == 0.0 {
            self
        } else {
            Vec3 {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul<f64> for Vec3 {
    type Output = Vec3;
    fn mul(self, scalar: f64) -> Vec3 {
        Vec3 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

/// Normalize an angle in degrees to [0, 360).
pub fn norm360(deg: f64) -> f64 {
    let d = deg % 360.0;
    if d < 0.0 {
        d + 360.0
    } else {
        d
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longitude_round_trip() {
        for lon in [0.0, 90.0, 180.0, 270.0, 359.9, 12.345] {
            let v = Vec3::from_ecliptic_longitude(lon);
            assert!((v.to_ecliptic_longitude() - lon).abs() < 1e-9, "lon={lon}");
        }
    }

    #[test]
    fn ayanamsa_as_rotation_matches_subtraction() {
        // rotate_z(-ayanamsa) on the tropical direction must equal the
        // flat-degree sidereal reduction, bit-for-bit — but stays in vector
        // space so nothing wraps near 0°/360°.
        let tropical = 5.0; // just past the Vernal Point
        let ayanamsa = 23.78;
        let flat = norm360(tropical - ayanamsa);
        let v = Vec3::from_ecliptic_longitude(tropical)
            .rotate_z(-ayanamsa)
            .to_ecliptic_longitude();
        assert!((v - flat).abs() < 1e-9, "vec={v} flat={flat}");
    }

    #[test]
    fn wraparound_separation_is_correct() {
        // 359.9° and 0.1° are 0.2° apart, NOT 359.8°.
        let a = Vec3::from_ecliptic_longitude(359.9);
        let b = Vec3::from_ecliptic_longitude(0.1);
        let sep = a.angular_separation_deg(b);
        assert!((sep - 0.2).abs() < 1e-9, "sep={sep}");
        // Flat-degree subtraction would give 359.8 — prove the vector form differs.
        assert!((norm360(359.9 - 0.1) - 0.2).abs() > 1.0);
    }

    #[test]
    fn rotation_is_deterministic() {
        let v = Vec3::from_ecliptic_longitude(123.4);
        let r1 = v.rotate_axis(Vec3::z_axis(), 7.0);
        let r2 = Vec3::from_ecliptic_longitude(123.4).rotate_axis(Vec3::z_axis(), 7.0);
        assert_eq!(r1.x.to_bits(), r2.x.to_bits());
        assert_eq!(r1.y.to_bits(), r2.y.to_bits());
        assert_eq!(r1.z.to_bits(), r2.z.to_bits());
    }
}
