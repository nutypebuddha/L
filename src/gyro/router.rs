use crate::astrology::Sign;
use crate::gyro::dynamics::GyroDynamics;
use crate::gyro::mapping::PrimitiveMapping;
use crate::gyro::state::GyroState;
use crate::wheel::Domain;

/// A single fired primitive during routing.
#[derive(Debug, Clone)]
pub struct FiredPrimitive {
    /// Which primitive fired (e.g. "nand", "not").
    pub primitive: &'static str,
    /// The sign it was routed through.
    pub sign: Sign,
    /// Semantic hint.
    pub hint: &'static str,
}

/// Result of routing a token through the gyro pipeline.
#[derive(Debug, Clone)]
pub struct RouteResult {
    /// The dominant sign classified from the token.
    pub dominant_sign: Sign,
    /// Primitives that fired.
    pub fired: Vec<FiredPrimitive>,
    /// Alignment score (0.0–1.0).
    pub alignment: f64,
    /// Gyro orientation after routing.
    pub orientation_deg: f64,
}

/// Routes tokens through the gyro pipeline:
///   token → sign classification → torque → alignment → primitive firing
#[derive(Debug, Clone)]
pub struct GyroRouter {
    /// The mapping from signs to primitives.
    mapping: PrimitiveMapping,
}

impl Default for GyroRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl GyroRouter {
    pub fn new() -> Self {
        Self {
            mapping: PrimitiveMapping::new(),
        }
    }

    /// Classify a token to a sign index (simple hash-based heuristic).
    pub fn classify_token(&self, token: &str) -> Sign {
        let hash: usize = token.bytes().fold(0usize, |acc, b| {
            acc.wrapping_mul(31).wrapping_add(b as usize)
        });
        let idx = hash % 12;
        // SAFETY: 0..11 maps directly to Sign variants
        match idx {
            0 => Sign::Aries,
            1 => Sign::Taurus,
            2 => Sign::Gemini,
            3 => Sign::Cancer,
            4 => Sign::Leo,
            5 => Sign::Virgo,
            6 => Sign::Libra,
            7 => Sign::Scorpio,
            8 => Sign::Sagittarius,
            9 => Sign::Capricorn,
            10 => Sign::Aquarius,
            _ => Sign::Pisces,
        }
    }

    /// Process a single token through the gyro pipeline.
    pub fn process_token(
        &self,
        token: &str,
        gyro: &mut GyroState,
        dynamics: &mut GyroDynamics,
    ) -> RouteResult {
        let sign = self.classify_token(token);
        let mass = 1.0 / (1.0 + token.len() as f64);

        // Apply torque
        let torque_deg = (sign as i32 as f64 - 6.0) * 5.0 * mass;
        gyro.apply_torque(mass);
        dynamics.apply_impulse(torque_deg);

        // Update dynamics
        dynamics.update(0.001);

        // Get alignment
        let alignment =
            dynamics.alignment(Domain::from_index(sign as usize).unwrap_or(Domain::Surya));

        // Fire primitives
        let entries = self.mapping.for_sign(sign);
        let fired: Vec<FiredPrimitive> = entries
            .iter()
            .map(|e| FiredPrimitive {
                primitive: e.primitive,
                sign,
                hint: e.hint,
            })
            .collect();

        RouteResult {
            dominant_sign: sign,
            fired,
            alignment,
            orientation_deg: dynamics.smoothed_orientation(),
        }
    }

    /// Process a multi-token query.
    pub fn process_query(
        &self,
        query: &str,
        gyro: &mut GyroState,
        dynamics: &mut GyroDynamics,
    ) -> Vec<RouteResult> {
        let tokens: Vec<&str> = query.split_whitespace().collect();
        let mut results = Vec::with_capacity(tokens.len());
        for token in tokens {
            let result = self.process_token(token, gyro, dynamics);
            // Update gyro state between tokens
            gyro.spin_to(gyro.current_jd + 0.001);
            results.push(result);
        }
        results
    }
}
