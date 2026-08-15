#[derive(Debug, Clone)]
pub struct TokenCandidate {
    pub id: u32,
    pub token: String,
    pub logit: f64,
    pub probability: f64,
}

impl TokenCandidate {
    pub fn new(id: u32, token: &str, logit: f64) -> Self {
        let probability = 1.0 / (1.0 + (-logit).exp());
        TokenCandidate {
            id,
            token: token.to_string(),
            logit,
            probability,
        }
    }
}

/// The outcome of running a single gate over a candidate.
///
/// This is the core contract of the Gate validation layer:
///
/// - `Pass` — the gate evaluated the candidate and accepts it.
/// - `Fail` — the gate evaluated the candidate and **rejects** it (a definite,
///   checkable violation: contradiction, false equation, malformed structure,
///   prompt injection, score below the configured `Pin.threshold`).
/// - `Unevaluable` — the gate could **not** reach a verdict either way. There was
///   nothing for it to judge against (no applicable input, missing corpus
///   evidence, absent formal structure). This is semantically distinct from
///   `Fail`: an *unevaluable* gate neither confirms nor refutes the candidate.
///
/// A candidate is `Ball::validated` (accepted) **only when every required gate
/// is `Pass`**. A single `Fail` rejects it; a single `Unevaluable` means it was
/// not proven, so it is also not accepted — but the two are reported apart so
/// the caller can distinguish "refuted" from "unproven".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateOutcome {
    Pass,
    Fail,
    Unevaluable,
}

impl GateOutcome {
    pub fn is_pass(&self) -> bool {
        matches!(self, GateOutcome::Pass)
    }
    pub fn is_fail(&self) -> bool {
        matches!(self, GateOutcome::Fail)
    }
    pub fn is_unevaluable(&self) -> bool {
        matches!(self, GateOutcome::Unevaluable)
    }
}

#[derive(Debug, Clone)]
pub struct GateResult {
    pub gate: super::pin::Gate,
    /// Convenience mirror of `outcome == GateOutcome::Pass`. Kept as a field so
    /// the large existing call surface that reads `.passed` keeps working.
    pub passed: bool,
    /// Why the gate reached its verdict (present for `Fail` and `Unevaluable`).
    pub outcome: GateOutcome,
    pub score: f64,
    pub reason: Option<String>,
    /// PCN comparison policy under which a claim was verified, if this gate
    /// performed a policy-bounded comparison (tri-state gate, Stage 1).
    pub policy: Option<super::policy::Policy>,
    /// Corpus claim id the gate matched against, if any (tri-state provenance).
    pub claim_id: Option<String>,
    /// The value a `Flagged` claim should have had (tri-state correction).
    pub corrected_value: Option<String>,
}

impl GateResult {
    pub fn passed(gate: super::pin::Gate, score: f64) -> Self {
        GateResult {
            gate,
            passed: true,
            outcome: GateOutcome::Pass,
            score,
            reason: None,
            policy: None,
            claim_id: None,
            corrected_value: None,
        }
    }

    pub fn failed(gate: super::pin::Gate, score: f64, reason: &str) -> Self {
        GateResult {
            gate,
            passed: false,
            outcome: GateOutcome::Fail,
            score,
            reason: Some(reason.to_string()),
            policy: None,
            claim_id: None,
            corrected_value: None,
        }
    }

    /// The candidate could not be judged by this gate (no applicable input or
    /// missing evidence). Distinct from `failed`, which means it *was* judged
    /// and rejected.
    pub fn unevaluable(gate: super::pin::Gate, score: f64, reason: &str) -> Self {
        GateResult {
            gate,
            passed: false,
            outcome: GateOutcome::Unevaluable,
            score,
            reason: Some(reason.to_string()),
            policy: None,
            claim_id: None,
            corrected_value: None,
        }
    }

    /// Attach the PCN policy under which this gate verified the claim.
    pub fn with_policy(mut self, policy: super::policy::Policy) -> Self {
        self.policy = Some(policy);
        self
    }

    /// Attach the corpus claim id this gate matched against.
    pub fn with_claim_id(mut self, claim_id: String) -> Self {
        self.claim_id = Some(claim_id);
        self
    }

    /// Attach the value a `Flagged` claim should have had.
    pub fn with_corrected(mut self, corrected: String) -> Self {
        self.corrected_value = Some(corrected);
        self
    }
}

#[derive(Debug, Clone)]
pub struct Ball {
    pub candidate: TokenCandidate,
    pub gate_results: Vec<GateResult>,
    pub total_score: f64,
    pub validated: bool,
}

impl Ball {
    pub fn new(candidate: TokenCandidate) -> Self {
        Ball {
            candidate,
            gate_results: Vec::new(),
            total_score: 0.0,
            validated: false,
        }
    }

    pub fn add_result(&mut self, result: GateResult) {
        self.gate_results.push(result);
        self.recalculate_score();
    }

    pub fn recalculate_score(&mut self) {
        if self.gate_results.is_empty() {
            self.total_score = 0.0;
            self.validated = false;
            return;
        }

        let total: f64 = self.gate_results.iter().map(|r| r.score).sum();
        let count = self.gate_results.len() as f64;
        self.total_score = total / count;
        self.validated = self.gate_results.iter().all(|r| r.passed);
    }

    pub fn passed_gate(&self, gate: super::pin::Gate) -> bool {
        self.gate_results.iter().any(|r| r.gate == gate && r.passed)
    }

    pub fn failed_gate(&self, gate: super::pin::Gate) -> bool {
        self.gate_results
            .iter()
            .any(|r| r.gate == gate && !r.passed)
    }

    pub fn all_passed(&self) -> bool {
        self.validated
    }

    /// Aggregate outcome across all recorded gate results.
    ///
    /// Strict acceptance contract: a candidate is accepted (`validated == true`)
    /// only when **every** required gate returned `Pass`. This method reports
    /// the reason acceptance failed:
    ///
    /// - `GateOutcome::Pass` — every gate passed (candidate validated).
    /// - `GateOutcome::Fail` — at least one gate definitively rejected it.
    /// - `GateOutcome::Unevaluable` — no gate rejected it, but at least one gate
    ///   could not reach a verdict, so it was not *proven*. Distinct from `Fail`.
    pub fn overall_outcome(&self) -> GateOutcome {
        if self.gate_results.is_empty() {
            return GateOutcome::Unevaluable;
        }
        let mut any_unevaluable = false;
        for r in &self.gate_results {
            if r.outcome.is_fail() {
                return GateOutcome::Fail;
            }
            if r.outcome.is_unevaluable() {
                any_unevaluable = true;
            }
        }
        if any_unevaluable {
            GateOutcome::Unevaluable
        } else {
            GateOutcome::Pass
        }
    }
}
