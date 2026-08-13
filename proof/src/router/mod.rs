//! # Gyro — the spinning zodiac wheel
//!
//! The gyroscopic wheel replaces a static edge table with a physical
//! model: a spinning disk with 12 signs around its rim. Token mass applies
//! torque, causing precession. The wheel's current orientation determines
//! which primitive alignment routes tokens to formulas.
//!
//! ```text
//! Token stream → ChangeSorter → 7-axis classification → dominant sign
//!                                    ↓
//!                         Gyro.apply_torque(sign, mass)
//!                                    ↓
//!                         Gyro.update(dt) → precession
//!                                    ↓
//!                         Gyro.aligned_primitives() → which NAND formulas fire
//! ```

pub mod dynamics;
pub mod mapping;
pub mod route;
pub mod state;
pub mod vec3;

pub use dynamics::{GyroDynamics, PrecessionParams};
pub use mapping::{PrimitiveEntry, PrimitiveMapping};
pub use route::{GyroRouter, RouteResult};
pub use state::GyroState;
pub use vec3::{norm360, Vec3};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::Graha;

    #[test]
    fn create_router_state() {
        let gyro = GyroState::new();
        assert!((gyro.current_jd - 2451545.0).abs() < 1e-6);
        assert_eq!(gyro.angular_velocity, 0.0);
        assert_eq!(gyro.token_count, 0);
    }

    #[test]
    fn spin_router_to() {
        let mut gyro = GyroState::new();
        let initial_jd = gyro.current_jd;
        gyro.spin_to(initial_jd + 1.0);
        assert!(gyro.current_jd > initial_jd);
    }

    #[test]
    fn dominant_graha_of_router() {
        let mut gyro = GyroState::new();
        gyro.graha_weights[0] = 1.0;
        let (g, w) = gyro.dominant_graha_info();
        assert_eq!(g, Graha::Surya);
        assert!((w - 1.0).abs() < 1e-6);
    }

    #[test]
    fn primitive_mapping() {
        let mapping = PrimitiveMapping::default();
        let aries_primitives = mapping.for_sign(crate::astrology::Sign::Aries);
        assert!(!aries_primitives.is_empty());
    }
}
