use crate::astrology::Graha;
use crate::chart::ChartSnapshot;
use crate::domain_graph::Domain;

/// Gyroscopic state of the zodiac wheel.
///
/// Models the 9-graha Vedic wheel as a spinning disk. Token mass applies
/// torque, causing precession. The wheel's orientation determines which
/// primitive alignment routes tokens to formulas.
#[derive(Debug, Clone)]
pub struct GyroState {
    /// Julian date of current orientation.
    pub current_jd: f64,
    /// Mass accumulator per graha domain.
    pub graha_weights: [f64; 9],
    /// Angular velocity in radians per day.
    pub angular_velocity: f64,
    /// Accumulated torque (radians).
    pub torque_accumulator: f64,
    /// Damping factor (0.0–1.0).
    pub damping: f64,
    /// Number of tokens processed since last reset.
    pub token_count: usize,
}

impl Default for GyroState {
    fn default() -> Self {
        Self::new()
    }
}

impl GyroState {
    /// Create a fresh gyro at J2000.0 epoch.
    pub fn new() -> Self {
        Self {
            current_jd: 2451545.0,
            graha_weights: [0.0; 9],
            angular_velocity: 0.0,
            torque_accumulator: 0.0,
            damping: 0.98,
            token_count: 0,
        }
    }

    /// Spin the wheel to a specific Julian date.
    pub fn spin_to(&mut self, target_jd: f64) {
        let dt = target_jd - self.current_jd;
        if dt > 0.0 {
            self.update(dt);
        }
        self.current_jd = target_jd;
    }

    /// Apply a chart snapshot as initial orientation.
    pub fn set_orientation_from_chart(&mut self, chart: &ChartSnapshot) {
        for pos in &chart.graha_positions {
            let idx = pos.graha.index();
            self.graha_weights[idx] += 1.0;
        }
    }

    /// Accumulate torque from a classified token.
    pub fn apply_torque(&mut self, sign_mass: f64) {
        self.torque_accumulator += sign_mass;
        self.token_count += 1;
    }

    /// Advance the wheel by `dt` Julian days.
    pub fn update(&mut self, dt: f64) {
        self.angular_velocity += self.torque_accumulator * dt;
        self.angular_velocity *= self.damping;
        self.current_jd += dt;
        self.torque_accumulator = 0.0;
    }

    /// Return the dominant graha and its weight.
    pub fn dominant_graha_info(&self) -> (Graha, f64) {
        let mut best_idx = 0;
        let mut best_weight = 0.0;
        for (i, w) in self.graha_weights.iter().enumerate() {
            if *w > best_weight {
                best_weight = *w;
                best_idx = i;
            }
        }
        (
            Domain::from_index(best_idx).unwrap_or(Graha::Surya),
            best_weight,
        )
    }

    /// Return normalized weights (0.0–1.0) across all grahas.
    pub fn normalized_graha_weights(&self) -> [f64; 9] {
        let total: f64 = self.graha_weights.iter().sum();
        if total <= 0.0 {
            return [0.0; 9];
        }
        let mut out = [0.0; 9];
        for (i, w) in self.graha_weights.iter().enumerate() {
            out[i] = w / total;
        }
        out
    }

    /// Return current orientation in degrees (mod 360).
    pub fn orientation_deg(&self) -> f64 {
        let deg = self.angular_velocity.to_degrees();
        deg.rem_euclid(360.0)
    }

    /// Return which primitives are aligned at current orientation.
    pub fn aligned_primitives(&self) -> Vec<(Graha, f64)> {
        let mut result = Vec::new();
        let weights = self.normalized_graha_weights();
        for (i, w) in weights.iter().enumerate() {
            if *w > 0.0 {
                if let Some(graha) = Domain::from_index(i) {
                    result.push((graha, *w));
                }
            }
        }
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        result
    }
}
