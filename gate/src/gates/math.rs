use super::GateValidator;
use crate::core::ball::{Ball, GateResult};
use crate::core::pin::Gate;
use crate::core::policy::Policy;
use crate::tanto::TantoEnv;

/// Outcome of classifying an equation claim against Tanto under PCN comparison
/// policies (tri-state gate, Stage 1).
struct EquationOutcome {
    /// Whether the token contained an `=` to compare at all.
    has_equation: bool,
    /// Whether the two sides agree under the reported `policy`.
    correct: bool,
    /// PCN policy the comparison used (`exact` / `rounded`), if any.
    policy: Option<Policy>,
    /// True value of the left-hand side when the claim was wrong.
    corrected: Option<String>,
}

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

    /// Classify an equation claim like "2+3 = 5" against Tanto under PCN
    /// comparison policies. `exact` = both sides equal within 1e-10;
    /// `rounded` = within 0.001 (float rounding); otherwise the claim is wrong
    /// and `corrected` carries the true value of the left-hand side. No `=`
    /// present means there is nothing to compare (not a math claim to verify).
    fn classify_equation(token: &str) -> EquationOutcome {
        if let Some(eq_pos) = token.find('=') {
            let lhs = token[..eq_pos].trim();
            let rhs = token[eq_pos + 1..].trim();
            let (l_val, r_val) = (Self::eval_tanto(lhs), Self::eval_tanto(rhs));
            match (l_val, r_val) {
                (Some(l), Some(r)) => {
                    let diff = (l - r).abs();
                    if diff < 1e-10 {
                        EquationOutcome {
                            has_equation: true,
                            correct: true,
                            policy: Some(Policy::Exact),
                            corrected: None,
                        }
                    } else if diff < 0.001 {
                        EquationOutcome {
                            has_equation: true,
                            correct: true,
                            policy: Some(Policy::Rounded),
                            corrected: None,
                        }
                    } else {
                        EquationOutcome {
                            has_equation: true,
                            correct: false,
                            policy: None,
                            corrected: Some(format!("{}", l)),
                        }
                    }
                }
                (Some(l), None) => EquationOutcome {
                    has_equation: true,
                    correct: false,
                    policy: None,
                    corrected: Some(format!("{}", l)),
                },
                _ => EquationOutcome {
                    has_equation: true,
                    correct: false,
                    policy: None,
                    corrected: None,
                },
            }
        } else {
            EquationOutcome {
                has_equation: false,
                correct: true,
                policy: None,
                corrected: None,
            }
        }
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

        let eq = Self::classify_equation(token);
        let balance_ok = Self::check_balanced_equation(context, token);
        let (correctness_ok, correctness_score) = if eq.has_equation {
            (
                eq.correct,
                if eq.correct {
                    if eq.policy == Some(Policy::Exact) {
                        0.98
                    } else {
                        0.90
                    }
                } else {
                    0.1
                },
            )
        } else {
            (true, 0.7)
        };
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

        let mut result = if passed {
            GateResult::passed(Gate::Math, avg_score)
        } else {
            GateResult::failed(Gate::Math, avg_score, &reason.unwrap_or_default())
        };
        if let Some(p) = eq.policy {
            result = result.with_policy(p);
        }
        if let Some(c) = eq.corrected {
            result = result.with_corrected(c);
        }
        result
    }
}
