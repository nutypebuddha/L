// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0

use super::GateValidator;
use crate::core::ball::{Ball, GateOutcome, GateResult};
use crate::core::pin::Gate;
use crate::gates::math::MathGate;

/// ProofRecomputeGate: a token that asserts a verifiable numeric claim must have
/// a proof that *recomputes* to match. This is the "proof recomputation match"
/// gate, built directly on the same deterministic Tanto evaluator the Math gate
/// uses, so a claim and its recomputation share one evaluation path.
///
/// - Deterministically checkable equation that balances -> Pass.
/// - Equation that does not balance -> Fail (the proof is wrong).
/// - No recomputable claim present -> Unevaluable (provenance not applicable
///   here, not a refutation).
pub struct ProofRecomputeGate;

impl ProofRecomputeGate {
    pub fn new() -> Self {
        ProofRecomputeGate
    }
}

impl Default for ProofRecomputeGate {
    fn default() -> Self {
        Self::new()
    }
}

impl GateValidator for ProofRecomputeGate {
    fn validate(&self, ball: &mut Ball, context: &str) -> GateResult {
        // Only claims that actually assert an equation have a proof to recompute.
        // Free-text / non-equation tokens have no recomputable claim, so they are
        // Unevaluable (not a refutation) and left to the other gates.
        if !ball.candidate.token.contains('=') {
            return GateResult::unevaluable(
                Gate::ProofRecompute,
                0.0,
                "no recomputable claim to verify",
            );
        }
        let math = MathGate::new().validate(ball, context);
        match math.outcome {
            GateOutcome::Pass => GateResult::passed(Gate::ProofRecompute, math.score),
            GateOutcome::Fail => GateResult::failed(
                Gate::ProofRecompute,
                math.score,
                "claim does not recompute (proof mismatch)",
            ),
            GateOutcome::Unevaluable => GateResult::unevaluable(
                Gate::ProofRecompute,
                0.0,
                "no recomputable claim to verify",
            ),
        }
    }
}
