use super::GateValidator;
use crate::core::ball::{Ball, GateResult};
use crate::core::pin::Gate;
use crate::tanto::TantoEnv;

/// MathGate: validates math expressions using Tanto's deterministic evaluator.
/// Tanto is the single evaluation path — no fallback parsers needed.
pub struct MathGate;

impl MathGate {
    pub fn new() -> Self {
        MathGate
    }
}

impl Default for MathGate {
    fn default() -> Self {
        Self::new()
    }
}

impl MathGate {
    /// Evaluate any math expression using Tanto's full parser.
    /// Supports: + - * / ^ % sqrt sin cos tan exp ln log10 pow hypot,
    /// constants (pi, e, c, g, h, etc.), natural language ("15% of 240"),
    /// and named ops (add, sub, mul, div, avg, etc.).
    fn eval_tanto(expr: &str) -> Option<f64> {
        let env = TantoEnv::new();
        crate::tanto::evaluate_nl(expr, &env)
    }

    /// Check an equation like "2+3 = 5" for correctness using Tanto.
    fn check_equation_correctness(token: &str) -> (bool, f64) {
        if let Some(eq_pos) = token.find('=') {
            let lhs = token[..eq_pos].trim();
            let rhs = token[eq_pos + 1..].trim();
            match (Self::eval_tanto(lhs), Self::eval_tanto(rhs)) {
                (Some(l_val), Some(r_val)) => {
                    if (l_val - r_val).abs() < 1e-10 {
                        return (true, 0.98);
                    }
                    if (l_val - r_val).abs() < 0.001 {
                        return (true, 0.90); // close enough (float rounding)
                    }
                    return (false, 0.1);
                }
                _ => return (false, 0.1),
            }
        }
        (true, 0.7) // no equation to check
    }

    /// Check balanced parentheses and brackets in an expression
    fn check_balanced_equation(context: &str, token: &str) -> bool {
        let mut paren_depth = 0i32;
        let mut bracket_depth = 0i32;
        for ch in context.bytes().chain(token.bytes()) {
            match ch {
                b'(' => paren_depth += 1,
                b')' => paren_depth -= 1,
                b'[' => bracket_depth += 1,
                b']' => bracket_depth -= 1,
                _ => {}
            }
            if paren_depth < 0 || bracket_depth < 0 {
                return false;
            }
        }
        paren_depth == 0 && bracket_depth == 0
    }

    /// Check if a token has valid math operators
    fn check_operator_validity(token: &str) -> (bool, f64) {
        if token.is_empty() {
            return (false, 0.0);
        }
        // Natural-language / multi-word input is not a math expression to
        // validate. Pass it through so the Fact gate can assess the claim
        // (previously this failed every free-text claim, making gate validate
        // refuse all factual statements). Equations are still checked by
        // check_equation_correctness.
        if token.chars().any(|c| c.is_whitespace()) {
            return (true, 0.8);
        }
        // Tanto handles all valid expressions, so we just check it's evaluable
        if Self::eval_tanto(token).is_some() {
            return (true, 0.95);
        }
        // Allow bare names (might be variables or non-math tokens)
        let first = token.as_bytes()[0];
        let valid_start = matches!(
            first,
            b'+' | b'-' | b'*' | b'/' | b'^' | b'=' | b'(' | b')' | b'[' | b']' | b'.' | b'0'
                ..=b'9'
        ) || token
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.');
        (valid_start, if valid_start { 0.85 } else { 0.2 })
    }

    /// Check if a value is physically plausible using Tanto + sanity ranges
    fn check_domain_consistency(token: &str) -> (bool, f64) {
        if let Some(val) = Self::eval_tanto(token) {
            if val.is_infinite() || val.is_nan() {
                return (false, 0.1);
            }
            if val.abs() > 1e308 {
                return (false, 0.2);
            }
            return (true, 0.95);
        }
        // Bare non-math token — not a domain issue
        (true, 0.7)
    }
}

impl GateValidator for MathGate {
    fn validate(&self, ball: &mut Ball, context: &str) -> GateResult {
        let token = &ball.candidate.token;

        let balance_ok = Self::check_balanced_equation(context, token);
        let (correctness_ok, correctness_score) = Self::check_equation_correctness(token);
        let (operator_ok, operator_score) = Self::check_operator_validity(token);
        let (domain_ok, domain_score) = Self::check_domain_consistency(token);

        let scores = [
            if balance_ok && correctness_ok {
                0.95
            } else {
                correctness_score
            },
            operator_score,
            domain_score,
        ];
        let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;

        let passed = balance_ok && correctness_ok && operator_ok && domain_ok;
        let reason = if !balance_ok {
            Some("Unbalanced parentheses or brackets".to_string())
        } else if !correctness_ok {
            Some("Equation does not balance (Tanto evaluated both sides)".to_string())
        } else if !operator_ok {
            Some("Invalid token format".to_string())
        } else if !domain_ok {
            Some("Value out of valid domain (inf/nan)".to_string())
        } else {
            None
        };

        if passed {
            GateResult::passed(Gate::Math, avg_score)
        } else {
            GateResult::failed(Gate::Math, avg_score, &reason.unwrap_or_default())
        }
    }
}
