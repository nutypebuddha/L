use crate::domain_graph::Domain;

/// Map a graha to its natural zodiac sign index (0–11).
/// Uses traditional Vedic sign rulership.
fn graha_to_sign_index(domain: Domain) -> usize {
    match domain {
        Domain::Surya => 4,      // Leo
        Domain::Chandra => 3,    // Cancer
        Domain::Mangala => 0,    // Aries
        Domain::Budha => 2,      // Gemini
        Domain::Brihaspati => 8, // Sagittarius
        Domain::Shukra => 1,     // Taurus
        Domain::Shani => 9,      // Capricorn
        Domain::Rahu => 10,      // Aquarius
        Domain::Ketu => 11,      // Pisces
    }
}

/// Precession parameters for the gyro wheel.
#[derive(Debug, Clone)]
pub struct PrecessionParams {
    /// Precession rate in arcseconds per century (IAU 2000: ~50.29").
    pub rate_arcsec_per_century: f64,
    /// Obliquity of ecliptic in degrees (J2000: ~23.439°).
    pub obliquity_deg: f64,
}

impl Default for PrecessionParams {
    fn default() -> Self {
        Self {
            rate_arcsec_per_century: 50.29,
            obliquity_deg: 23.439_291_1,
        }
    }
}

/// Dynamics engine: precession, torque, alignment scoring.
#[derive(Debug, Clone)]
pub struct GyroDynamics {
    pub params: PrecessionParams,
    /// Orientation in degrees (accumulated).
    pub orientation_deg: f64,
    /// Velocity in degrees per Julian day.
    pub velocity_deg_per_day: f64,
    /// History of orientations for smoothing.
    pub orientation_history: Vec<f64>,
}

impl Default for GyroDynamics {
    fn default() -> Self {
        Self::new()
    }
}

impl GyroDynamics {
    pub fn new() -> Self {
        Self {
            params: PrecessionParams::default(),
            orientation_deg: 0.0,
            velocity_deg_per_day: 0.0,
            orientation_history: Vec::new(),
        }
    }

    /// Advance dynamics by `dt` Julian days.
    pub fn update(&mut self, dt: f64) {
        self.orientation_deg += self.velocity_deg_per_day * dt;
        self.orientation_deg = self.orientation_deg.rem_euclid(360.0);
        self.orientation_history.push(self.orientation_deg);
        if self.orientation_history.len() > 100 {
            self.orientation_history.remove(0);
        }
    }

    /// Apply a torque impulse (degrees).
    pub fn apply_impulse(&mut self, torque_deg: f64) {
        self.velocity_deg_per_day += torque_deg;
    }

    /// Smoothed orientation (moving average over history).
    pub fn smoothed_orientation(&self) -> f64 {
        if self.orientation_history.is_empty() {
            return self.orientation_deg;
        }
        let sum: f64 = self.orientation_history.iter().sum();
        sum / self.orientation_history.len() as f64
    }

    /// Predict orientation after `future_dt` Julian days (no state mutation).
    pub fn predict(&self, future_dt: f64) -> f64 {
        (self.orientation_deg + self.velocity_deg_per_day * future_dt).rem_euclid(360.0)
    }

    /// Compute alignment score for a domain (0.0–1.0) based on orientation.
    pub fn alignment(&self, domain: Domain) -> f64 {
        let sign_deg = graha_to_sign_index(domain) as f64 * 30.0;
        let diff = (self.orientation_deg - sign_deg).abs();
        let min_diff = if diff > 180.0 { 360.0 - diff } else { diff };
        (1.0 - min_diff / 180.0).max(0.0)
    }

    /// Return the domain with highest alignment score.
    pub fn dominant(&self) -> Domain {
        let mut best = Domain::Surya;
        let mut best_score = -1.0;
        for graha in Domain::all() {
            let score = self.alignment(graha);
            if score > best_score {
                best_score = score;
                best = graha;
            }
        }
        best
    }
}
