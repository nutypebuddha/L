use crate::core::ball::GateResult;
use crate::core::pin::Gate;
use crate::state::machine::State;

/// Stable, machine-classifiable outcome of a validation (Item 3). Replaces the
/// `passed: bool` that conflated three distinct states — the caller could read
/// `passed: true` and confidently report success even when L had silently
/// *corrected* the claim (`fix_count > 0`) or could not evaluate it at all.
///
/// `passed` is retained only as an internal field; `verdict` is the contract.
/// `ok` and `corrected` mean the claim is now in a good state, `failed` means it
/// is wrong and L could not make it pass, `unevaluable` means L had nothing to
/// judge it against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationVerdict {
    Ok,
    Corrected,
    Failed,
    Unevaluable,
}

impl ValidationVerdict {
    /// Stable string form (Item 3 contract). Lowercase, never changes — callers
    /// branch on this, not on any surrounding prose.
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationVerdict::Ok => "ok",
            ValidationVerdict::Corrected => "corrected",
            ValidationVerdict::Failed => "failed",
            ValidationVerdict::Unevaluable => "unevaluable",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub validated_text: String,
    pub original_text: String,
    pub confidence: f64,
    pub passed: bool,
    pub fixes: Vec<TokenFix>,
    pub warnings: Vec<String>,
    pub gate_scores: Vec<GateScore>,
    pub state: State,
    pub cost_usd: f64,
}

impl ValidationResult {
    /// Derive the stable verdict from the (internal) `passed` flag, the applied
    /// fixes, and whether any gate actually ran. This is what callers must read;
    /// it can never be the silent `passed: true`-with-fixes trap, because a
    /// corrected claim is always reported as `corrected`.
    pub fn verdict(&self) -> ValidationVerdict {
        if self.gate_scores.is_empty() {
            return ValidationVerdict::Unevaluable;
        }
        if !self.passed {
            return ValidationVerdict::Failed;
        }
        if self.fix_count() > 0 {
            return ValidationVerdict::Corrected;
        }
        ValidationVerdict::Ok
    }
}

impl ValidationResult {
    pub fn new(original: &str) -> Self {
        ValidationResult {
            validated_text: original.to_string(),
            original_text: original.to_string(),
            confidence: 0.0,
            passed: false,
            fixes: Vec::new(),
            warnings: Vec::new(),
            gate_scores: Vec::new(),
            state: State::Normal,
            cost_usd: 0.0,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    pub fn with_passed(mut self, passed: bool) -> Self {
        self.passed = passed;
        self
    }

    pub fn with_fixes(mut self, fixes: Vec<TokenFix>) -> Self {
        self.fixes = fixes;
        self
    }

    pub fn with_warnings(mut self, warnings: Vec<String>) -> Self {
        self.warnings = warnings;
        self
    }

    pub fn with_gate_scores(mut self, scores: Vec<GateScore>) -> Self {
        self.gate_scores = scores;
        self
    }

    pub fn with_state(mut self, state: State) -> Self {
        self.state = state;
        self
    }

    pub fn with_cost(mut self, cost_usd: f64) -> Self {
        self.cost_usd = cost_usd;
        self
    }

    pub fn has_fixes(&self) -> bool {
        !self.fixes.is_empty()
    }

    pub fn fix_count(&self) -> usize {
        self.fixes.len()
    }

    pub fn gate_passed(&self, gate: Gate) -> bool {
        self.gate_scores.iter().any(|s| s.gate == gate && s.passed)
    }

    pub fn gate_score(&self, gate: Gate) -> Option<f64> {
        self.gate_scores
            .iter()
            .find(|s| s.gate == gate)
            .map(|s| s.score)
    }
}

#[derive(Debug, Clone)]
pub struct TokenFix {
    pub original: String,
    pub fixed: String,
    pub reason: String,
    pub confidence: f64,
}

impl TokenFix {
    pub fn new(original: &str, fixed: &str, reason: &str, confidence: f64) -> Self {
        TokenFix {
            original: original.to_string(),
            fixed: fixed.to_string(),
            reason: reason.to_string(),
            confidence,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GateScore {
    pub gate: Gate,
    pub passed: bool,
    pub score: f64,
    pub details: String,
}

impl GateScore {
    pub fn from_gate_result(result: &GateResult) -> Self {
        GateScore {
            gate: result.gate,
            passed: result.passed,
            score: result.score,
            details: result.reason.clone().unwrap_or_default(),
        }
    }

    pub fn passed(gate: Gate, score: f64, details: &str) -> Self {
        GateScore {
            gate,
            passed: true,
            score,
            details: details.to_string(),
        }
    }

    pub fn failed(gate: Gate, score: f64, details: &str) -> Self {
        GateScore {
            gate,
            passed: false,
            score,
            details: details.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CidError {
    BudgetExhausted {
        remaining_tokens: u64,
        remaining_cost: f64,
    },
    RateLimited {
        retry_after_ms: u64,
    },
    Timeout {
        timeout_ms: u64,
    },
    InvalidInput(String),
    LlmError(String),
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for CidError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CidError::BudgetExhausted {
                remaining_tokens,
                remaining_cost,
            } => {
                write!(
                    f,
                    "Budget exhausted: {} tokens, ${:.6} remaining",
                    remaining_tokens, remaining_cost
                )
            }
            CidError::RateLimited { retry_after_ms } => {
                write!(f, "Rate limited: retry after {}ms", retry_after_ms)
            }
            CidError::Timeout { timeout_ms } => {
                write!(f, "Timeout after {}ms", timeout_ms)
            }
            CidError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            CidError::LlmError(msg) => write!(f, "LLM error: {}", msg),
            CidError::IoError(msg) => write!(f, "IO error: {}", msg),
            CidError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for CidError {}

pub type CidResult<T> = Result<T, CidError>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::pin::Gate;

    fn with_gate() -> ValidationResult {
        ValidationResult::new("x").with_gate_scores(vec![GateScore::passed(Gate::Math, 0.9, "ok")])
    }

    #[test]
    fn verdict_unevaluable_without_gates() {
        // L had nothing to judge the claim against.
        let r = ValidationResult::new("x");
        assert_eq!(r.verdict(), ValidationVerdict::Unevaluable);
    }

    #[test]
    fn verdict_ok_when_passed_no_fixes() {
        let r = with_gate().with_passed(true);
        assert_eq!(r.verdict(), ValidationVerdict::Ok);
    }

    #[test]
    fn verdict_corrected_when_passed_with_fixes() {
        // The historical trap: `passed: true` while L silently corrected the claim.
        // The verdict can never be `ok` here — it must report `corrected`.
        let r = with_gate().with_passed(true).with_fixes(vec![TokenFix::new(
            "2+2=5",
            "2+2=4",
            "arithmetic",
            0.9,
        )]);
        assert_eq!(r.verdict(), ValidationVerdict::Corrected);
        assert_eq!(r.fix_count(), 1);
    }

    #[test]
    fn verdict_failed_when_not_passed() {
        let r = with_gate().with_passed(false);
        assert_eq!(r.verdict(), ValidationVerdict::Failed);
    }

    #[test]
    fn verdict_as_str_is_stable_api() {
        assert_eq!(ValidationVerdict::Ok.as_str(), "ok");
        assert_eq!(ValidationVerdict::Corrected.as_str(), "corrected");
        assert_eq!(ValidationVerdict::Failed.as_str(), "failed");
        assert_eq!(ValidationVerdict::Unevaluable.as_str(), "unevaluable");
    }
}
