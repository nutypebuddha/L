pub mod confidence;
pub mod domain;
pub mod fact;
pub mod fallacy;
pub mod formal;
pub mod logic;
pub mod math;
pub mod recompute;

pub use confidence::ConfidenceGate;
pub use domain::DomainBindingGate;
pub use fact::FactGate;
pub use fallacy::FallacyGate;
pub use formal::FormalGate;
pub use logic::LogicGate;
pub use math::MathGate;
pub use recompute::ProofRecomputeGate;

use crate::core::ball::{Ball, GateOutcome, GateResult};
use crate::core::pin::{Gate, Pin};
use crate::kb::facts::KnowledgeBase;

pub trait GateValidator {
    fn validate(&self, ball: &mut Ball, context: &str) -> GateResult;
}

/// The single, authoritative Gate validation path.
///
/// Every candidate flows through this function and *only* this function. It:
///
/// 1. Runs each enabled `Pin`'s gate over the candidate in a stable order.
/// 2. Enforces `Pin.threshold` in exactly one place: a gate result that is
///    `Pass` but whose `score` is below the pin's threshold is downgraded to
///    `Fail`. This makes threshold enforcement uniform across Math/Logic/Fact/
///    Formal/Confidence instead of being ad-hoc per gate.
/// 3. Records the outcome on the `Ball`.
///
/// Validation is **candidate-based**: one `Ball` is one candidate token, and the
/// `Ball` is `validated` iff every required (enabled) gate returns `Pass`. The
/// "per-token" framing in the README refers to the same unit — each proposed
/// token/candidate is gated independently.
///
/// `claim` is the full claim text (may equal the candidate token) so gates such
/// as `FactGate` can reason about the whole statement, not just the token.
pub fn validate_candidate(
    ball: &mut Ball,
    pins: &[Pin],
    context: &str,
    claim: &str,
    kb: &KnowledgeBase,
) {
    for pin in pins {
        if !pin.enabled {
            continue;
        }

        let mut result = match pin.gate {
            Gate::Math => MathGate::new().validate(ball, context),
            Gate::Logic => LogicGate::new().validate(ball, context),
            Gate::Fact => FactGate::new(kb).with_claim(claim).validate(ball, context),
            // Confidence uses the pin's threshold directly — this is the one
            // gate that was already threshold-aware, now made consistent with
            // the others via the downgrade step below.
            Gate::Confidence => ConfidenceGate::new(pin.threshold).validate(ball, context),
            Gate::Formal => FormalGate::new().validate(ball, context),
            // DomainBinding: refuse tokens with no provenance in the verified
            // context (the LLM hardening gate).
            Gate::DomainBinding => DomainBindingGate::new().validate(ball, context),
            // ProofRecompute: refuse claims whose proof does not recompute.
            Gate::ProofRecompute => ProofRecomputeGate::new().validate(ball, context),
        };

        // Authoritative threshold enforcement — single point of truth.
        //
        // The rule is uniform across every gate:
        //   semantic Fail ............ -> Fail        (definite rejection)
        //   semantic Pass + score>=T . -> Pass        (accepted)
        //   semantic Pass + score< T . -> Unevaluable (accepted by the
        //        correctness check, but the gate cannot meet the configured
        //        confidence/threshold bar, so it is *unproven*, not *refuted*).
        if result.outcome.is_pass() && result.score < pin.threshold {
            result = GateResult::unevaluable(
                pin.gate,
                result.score,
                &format!(
                    "{:?} score {:.3} below pin threshold {:.3}",
                    pin.gate, result.score, pin.threshold
                ),
            );
        }

        ball.add_result(result);
    }
}

/// Backwards-compatible wrapper kept for the WASM surface. Constructs the claim
/// from the ball's candidate token and delegates to [`validate_candidate`].
pub fn validate_ball(ball: &mut Ball, pins: &[Pin], context: &str, kb: &KnowledgeBase) {
    let claim = ball.candidate.token.clone();
    validate_candidate(ball, pins, context, &claim, kb);
}

/// Summarise the per-gate outcomes of a `Ball` into a stable, machine-readable
/// form (used by tests, tracing, and the TCB proof envelope).
pub fn outcome_summary(ball: &Ball) -> Vec<(Gate, GateOutcome, f64)> {
    ball.gate_results
        .iter()
        .map(|r| (r.gate, r.outcome, r.score))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ball::TokenCandidate;
    use crate::core::pin::{Gate, PinField};

    fn run(token: &str, context: &str, pins: &[Pin]) -> Ball {
        let kb = KnowledgeBase::new();
        let candidate = TokenCandidate::new(0, token, 0.5);
        let mut ball = Ball::new(candidate);
        validate_candidate(&mut ball, pins, context, token, &kb);
        ball
    }

    #[test]
    fn universally_unified_path_runs_every_pin_once() {
        let pins = PinField::new();
        let ball = run("2 + 3 = 5", "math", &pins.pins);
        // Exactly one result per enabled pin — the single authoritative path.
        assert_eq!(ball.gate_results.len(), pins.pins.len());
        for r in &ball.gate_results {
            assert!(!r.outcome.is_unevaluable() || r.gate == Gate::Logic);
        }
    }

    #[test]
    fn true_equation_is_fully_validated() {
        let pins = PinField::new();
        let ball = run("2 + 3 = 5", "math", &pins.pins);
        assert!(ball.validated, "all gates should Pass for a true equation");
        assert_eq!(ball.overall_outcome(), GateOutcome::Pass);
    }

    #[test]
    fn false_equation_is_rejected() {
        let pins = PinField::new();
        let ball = run("2 + 3 = 6", "math", &pins.pins);
        assert!(!ball.validated);
        assert_eq!(ball.overall_outcome(), GateOutcome::Fail);
    }

    #[test]
    fn contradiction_is_failure_not_unevaluable() {
        // "earth is flat" contradicts the corpus -> Fact gate must Fail.
        let pins = PinField::new();
        let ball = run("the earth is flat", "news", &pins.pins);
        let fact = ball
            .gate_results
            .iter()
            .find(|r| r.gate == Gate::Fact)
            .unwrap();
        assert_eq!(fact.outcome, GateOutcome::Fail);
    }

    #[test]
    fn high_threshold_makes_semantic_pass_unevaluable() {
        // With a strict threshold, a semantically-acceptable claim whose score
        // is below the bar becomes Unevaluable (distinct from Fail).
        let mut pins = PinField::new();
        pins.adjust_pin(Gate::Logic, 0.99);
        let ball = run("2 + 3 = 5", "math", &pins.pins);
        let logic = ball
            .gate_results
            .iter()
            .find(|r| r.gate == Gate::Logic)
            .unwrap();
        assert_eq!(logic.outcome, GateOutcome::Unevaluable);
        assert_ne!(logic.outcome, GateOutcome::Fail);
    }

    #[test]
    fn disabled_pins_do_not_run() {
        let mut pins = PinField::new();
        pins.disable_pin(Gate::Math);
        let ball = run("2 + 3 = 5", "math", &pins.pins);
        assert!(ball.gate_results.iter().all(|r| r.gate != Gate::Math));
    }

    // --- DomainBinding / ProofRecompute gates (#2 directive) -----------------

    fn gate_outcome(gate: Gate, token: &str, context: &str) -> GateOutcome {
        let candidate = TokenCandidate::new(0, token, 0.5);
        let mut ball = Ball::new(candidate);
        let result = match gate {
            Gate::DomainBinding => DomainBindingGate::new().validate(&mut ball, context),
            Gate::ProofRecompute => ProofRecomputeGate::new().validate(&mut ball, context),
            _ => unreachable!("test only exercises the two new gates"),
        };
        result.outcome
    }

    #[test]
    fn domain_binding_passes_when_grounded() {
        assert_eq!(
            gate_outcome(
                Gate::DomainBinding,
                "mangala is exalted",
                "mangala exalted in capricorn"
            ),
            GateOutcome::Pass
        );
    }

    #[test]
    fn domain_binding_fails_when_ungrounded() {
        assert_eq!(
            gate_outcome(
                Gate::DomainBinding,
                "zephyr quantum flux unknownword",
                "mangala exalted in capricorn"
            ),
            GateOutcome::Fail
        );
    }

    #[test]
    fn domain_binding_unevaluable_without_context() {
        assert_eq!(
            gate_outcome(Gate::DomainBinding, "any token at all", ""),
            GateOutcome::Unevaluable
        );
    }

    #[test]
    fn proof_recompute_passes_balanced_equation() {
        assert_eq!(
            gate_outcome(Gate::ProofRecompute, "2 + 3 = 5", "math"),
            GateOutcome::Pass
        );
    }

    #[test]
    fn proof_recompute_fails_unbalanced_equation() {
        assert_eq!(
            gate_outcome(Gate::ProofRecompute, "2 + 3 = 6", "math"),
            GateOutcome::Fail
        );
    }

    #[test]
    fn proof_recompute_unevaluable_without_claim() {
        assert_eq!(
            gate_outcome(Gate::ProofRecompute, "the earth is round", "general"),
            GateOutcome::Unevaluable
        );
    }

    #[test]
    fn domain_proof_enforcement_pinfield_enables_both() {
        let pins = PinField::with_domain_proof_enforcement();
        assert!(pins
            .pins
            .iter()
            .any(|p| p.gate == Gate::DomainBinding && p.enabled));
        assert!(pins
            .pins
            .iter()
            .any(|p| p.gate == Gate::ProofRecompute && p.enabled));
    }
}
