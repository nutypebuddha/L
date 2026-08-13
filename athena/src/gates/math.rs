//! # Math Gate — validates numeric calculations

use super::{gate_output, Gate, GateOutput};
use crate::tanto::{evaluate as tanto_eval, TantoEnv};

/// Evaluate an expression string using Tanto, returning `Result<f64, String>`.
fn eval_expression(expr: &str) -> Result<f64, String> {
    let env = TantoEnv::new();
    tanto_eval(expr, &env).ok_or_else(|| format!("cannot evaluate '{}'", expr))
}

/// Split a claim like "2 + 2 = 4" or "2 + 2 == 4" into (lhs, rhs).
/// Returns None if there's no top-level "=" (i.e. it's a bare expression,
/// or uses "=" inside something like ">=" which we don't treat as equality here).
fn split_equality(target: &str) -> Option<(&str, &str)> {
    // Prefer "==" if present, otherwise fall back to a single "=".
    if let Some(idx) = target.find("==") {
        let (lhs, rest) = target.split_at(idx);
        return Some((lhs, &rest[2..]));
    }
    // Avoid splitting on "=" that's part of "<=", ">=", "!=".
    let bytes = target.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'=' {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            if prev == b'<' || prev == b'>' || prev == b'!' {
                continue;
            }
            let (lhs, rest) = target.split_at(i);
            return Some((lhs, &rest[1..]));
        }
    }
    None
}

/// Validates mathematical expressions and calculations.
pub struct MathGate;

impl MathGate {
    pub fn new() -> Self {
        MathGate
    }

    /// Check if a calculation matches an expected result.
    pub fn check_calculation(&self, expression: &str, expected: f64, tolerance: f64) -> GateOutput {
        match eval_expression(expression) {
            Ok(actual) => {
                let diff = (actual - expected).abs();
                if diff <= tolerance {
                    gate_output(
                        "math",
                        true,
                        1.0 - (diff / (expected.abs().max(1.0))).min(0.5),
                        format!("Calculation verified: {} = {:.6}", expression, actual),
                        vec![],
                        vec![],
                    )
                } else {
                    gate_output(
                        "math",
                        false,
                        0.0,
                        format!(
                            "Mismatch: {} = {:.6}, expected {:.6} (diff: {:.6})",
                            expression, actual, expected, diff
                        ),
                        vec![format!("Expected {:.6}, got {:.6}", expected, actual)],
                        vec![format!("Suggested: {} = {:.6}", expression, actual)],
                    )
                }
            }
            Err(e) => gate_output(
                "math",
                false,
                0.0,
                format!("Parse error in '{}': {}", expression, e),
                vec![format!("Parse error: {}", e)],
                vec!["Check expression syntax".to_string()],
            ),
        }
    }
}

impl Default for MathGate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate for MathGate {
    fn name(&self) -> &str {
        "math"
    }

    fn check(&self, target: &str) -> GateOutput {
        // If this looks like an equality claim ("lhs = rhs" or "lhs == rhs"),
        // split it and verify both sides actually agree instead of trying to
        // parse the whole string as one expression (which always fails,
        // masking true and false claims identically).
        if let Some((lhs, rhs)) = split_equality(target) {
            return match eval_expression(rhs.trim()) {
                Ok(expected) => self.check_calculation(lhs.trim(), expected, 1e-9),
                Err(e) => gate_output(
                    "math",
                    false,
                    0.0,
                    format!("Cannot evaluate right-hand side '{}': {}", rhs.trim(), e),
                    vec![format!("Parse error: {}", e)],
                    vec!["Check expression syntax on the right-hand side".to_string()],
                ),
            };
        }

        // Try to evaluate the expression
        match eval_expression(target) {
            Ok(result) => gate_output(
                "math",
                true,
                1.0,
                format!("Expression '{}' evaluates to {:.6}", target, result),
                vec![],
                vec![],
            ),
            Err(e) => gate_output(
                "math",
                false,
                0.0,
                format!("Cannot evaluate '{}': {}", target, e),
                vec![format!("Parse error: {}", e)],
                vec!["Check for syntax errors or undefined variables".to_string()],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_math() {
        let gate = MathGate::new();
        let result = gate.check("2 + 2");
        assert!(result.passed);
        assert!((result.confidence - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_mismatch() {
        let gate = MathGate::new();
        let result = gate.check_calculation("2 + 2", 5.0, 0.001);
        assert!(!result.passed);
    }

    #[test]
    fn test_tolerance() {
        let gate = MathGate::new();
        let result = gate.check_calculation("2 + 2", 4.001, 0.01);
        assert!(result.passed);
    }

    #[test]
    fn test_invalid_expression() {
        let gate = MathGate::new();
        // Two operators in a row with no operand is invalid
        let result = gate.check("2 + * 2");
        assert!(!result.passed);
    }

    #[test]
    fn test_check_true_equality_claim() {
        let gate = MathGate::new();
        let result = gate.check("2 + 2 = 4");
        assert!(
            result.passed,
            "true equality claim should pass: {:?}",
            result.message
        );
    }

    #[test]
    fn test_check_false_equality_claim() {
        let gate = MathGate::new();
        let result = gate.check("2 + 2 = 5");
        assert!(!result.passed, "false equality claim should fail");
    }

    #[test]
    fn test_check_double_equals_claim() {
        let gate = MathGate::new();
        assert!(gate.check("2 + 2 == 4").passed);
        assert!(!gate.check("2 + 2 == 5").passed);
    }

    #[test]
    fn test_check_does_not_split_comparison_operators() {
        let gate = MathGate::new();
        // ">=" should not be misread as an equality split
        let result = gate.check("5 >= 3");
        // meval doesn't support >=, so this should be a genuine parse error,
        // not a false "true" from splitting on the wrong "="
        assert!(!result.passed);
    }
}
