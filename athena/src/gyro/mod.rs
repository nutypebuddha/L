//! # Gyro — the spinning zodiac wheel
//!
//! The gyroscopic wheel replaces the static 12×12 edge table with a physical
//! model: a spinning disk with 12 signs around its rim. Token mass applies
//! torque, causing precession. The wheel's current orientation determines which primitive alignment
//! routes tokens to formulas.
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

mod dynamics;
mod mapping;
mod router;
mod state;

pub use dynamics::{GyroDynamics, PrecessionParams};
pub use mapping::{PrimitiveEntry, PrimitiveMapping};
pub use router::{GyroRouter, RouteResult};
pub use state::{Axis, GyroState, Orientation};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gyro_state_new() {
        let gyro = GyroState::new();
        assert_eq!(gyro.orientation.0, 0.0); // Aries at 0°
        assert_eq!(gyro.angular_velocity, 0.0);
    }

    #[test]
    fn test_gyro_apply_torque() {
        let mut gyro = GyroState::new();
        let initial_torque = gyro.torque_accumulator;
        // Apply at Taurus (30°) while gyro is at Aries (0°) - lever arm = sin(30°) = 0.5
        gyro.apply_torque(crate::astrology::Sign::Taurus, 1.0);
        assert!(gyro.torque_accumulator > initial_torque);
    }

    #[test]
    fn test_gyro_update() {
        let mut gyro = GyroState::new();
        gyro.angular_velocity = 1.0; // 1 sign per second
        gyro.update(1.0); // 1 second
                          // Should have precessed
        assert!(gyro.orientation.0 > 0.0);
    }

    #[test]
    fn test_primitive_mapping() {
        let mapping = PrimitiveMapping::default();
        // Aries (math/logic) should have NAND, NOT, AND primitives
        let aries_primitives = mapping.for_sign(crate::astrology::Sign::Aries);
        assert!(!aries_primitives.is_empty());
    }
}
