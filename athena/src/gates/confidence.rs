//! # Confidence Gate — assigns confidence scores

use super::{gate_output, Gate, GateOutput};

/// Confidence thresholds and scoring.
///
/// Confidence is computed from:
/// - Number of hops in the chain (fewer = higher confidence)
/// - Aspect directness (direct aspects = higher confidence)
/// - Formula tier (atomic > bridging > vortex)
/// - Evidence availability
pub struct ConfidenceGate;

impl ConfidenceGate {
    pub fn new() -> Self {
        ConfidenceGate
    }

    /// Calculate confidence for a chain of reasoning.
    ///
    /// * `chain_length`: number of formula evaluations in the chain
    /// * `has_evidence`: whether the claim has supporting formula evidence
    /// * `tension_count`: number of Square/Opposition aspects crossed
    pub fn score_chain(
        &self,
        chain_length: usize,
        has_evidence: bool,
        tension_count: usize,
    ) -> f64 {
        let mut score = 1.0;

        // Longer chains reduce confidence
        if chain_length > 1 {
            score *= 0.9_f64.powi(chain_length as i32 - 1);
        }

        // Tension aspects reduce confidence
        if tension_count > 0 {
            score *= 0.85_f64.powi(tension_count as i32);
        }

        // Evidence bonus
        if has_evidence {
            score = (score * 1.1).min(1.0);
        }

        // Floor at 0.05
        score.max(0.05)
    }
}

impl Default for ConfidenceGate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate for ConfidenceGate {
    fn name(&self) -> &str {
        "confidence"
    }

    fn check(&self, target: &str) -> GateOutput {
        // Parse target as "chain_length,tension_count,has_evidence" or just use defaults
        let parts: Vec<&str> = target.split(',').collect();
        let (chain_length, tension_count, has_evidence) = if parts.len() >= 3 {
            (
                parts[0].trim().parse::<usize>().unwrap_or(1),
                parts[1].trim().parse::<usize>().unwrap_or(0),
                parts[2].trim().parse::<bool>().unwrap_or(false),
            )
        } else {
            (1, 0, false)
        };

        let score = self.score_chain(chain_length, has_evidence, tension_count);
        let passed = score >= 0.5;

        gate_output(
            "confidence",
            passed,
            score,
            format!(
                "Chain confidence: {:.2} ({} hop(s), {} tension aspect(s), evidence: {})",
                score, chain_length, tension_count, has_evidence
            ),
            if !passed {
                vec![format!("Confidence {:.2} below threshold 0.50", score)]
            } else {
                vec![]
            },
            if !passed {
                vec![
                    "Shorten the chain".to_string(),
                    "Reduce tension aspects".to_string(),
                    "Add supporting evidence".to_string(),
                ]
            } else {
                vec![]
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_hop_high_confidence() {
        let gate = ConfidenceGate::new();
        let score = gate.score_chain(1, true, 0);
        assert!(score > 0.9);
    }

    #[test]
    fn test_multi_hop_reduces_confidence() {
        let gate = ConfidenceGate::new();
        let score_1 = gate.score_chain(1, false, 0);
        let score_5 = gate.score_chain(5, false, 0);
        assert!(score_5 < score_1);
    }

    #[test]
    fn test_tension_reduces_confidence() {
        let gate = ConfidenceGate::new();
        let score_0 = gate.score_chain(2, false, 0);
        let score_2 = gate.score_chain(2, false, 2);
        assert!(score_2 < score_0);
    }

    #[test]
    fn test_evidence_bonus() {
        let gate = ConfidenceGate::new();
        let score_no = gate.score_chain(2, false, 0);
        let score_yes = gate.score_chain(2, true, 0);
        assert!(score_yes >= score_no);
    }

    #[test]
    fn test_check_parsing() {
        let gate = ConfidenceGate::new();
        let result = gate.check("3,2,true");
        assert!(result.confidence < 1.0);
        assert!(result.confidence > 0.0);
    }
}
