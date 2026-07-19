use crate::compute;
use crate::scoring::ball::{Ball, GateResult};
use crate::scoring::pin::GateKind;

fn eval_tanto(expr: &str) -> Option<f64> {
    let env = compute::create_env();
    compute::evaluate_nl(expr, &env)
}

fn check_equation_correctness(token: &str) -> (bool, f64) {
    // T59: a token containing a relational operator is a comparison claim
    // ("5 >= 3", "9.11 < 9.9"), not an equation. Evaluate it as ONE Tanto
    // expression — the parser now has first-class <, >, <=, >=, != and
    // (T59) requires full token consumption, so this only succeeds for a
    // genuine, complete comparison. It can't be fooled by a bare equation
    // like "2 + 3 = 5": `=` is still not part of the Tanto grammar, so that
    // whole-string parse fails here and falls through to the equation-split
    // path below, unchanged from before.
    //
    // Gated on substring match rather than "did eval_tanto succeed" so this
    // cannot change behavior for plain arithmetic like "2 - 2": those never
    // contained a relational operator, so they skip this branch entirely
    // and keep the pre-existing "parses = passes, regardless of value"
    // semantics a few lines down (T44).
    if token.contains('<') || token.contains('>') || token.contains("!=") {
        return match eval_tanto(token) {
            Some(v) if v != 0.0 => (true, 0.95),
            Some(_) => (false, 0.1),
            None => (false, 0.1),
        };
    }
    if let Some(eq_pos) = token.find('=') {
        let left_operand = token[..eq_pos].trim();
        let right_operand = token[eq_pos + 1..].trim();
        match (eval_tanto(left_operand), eval_tanto(right_operand)) {
            (Some(l_val), Some(r_val)) => {
                if (l_val - r_val).abs() < 1e-10 {
                    return (true, 0.98);
                }
                if (l_val - r_val).abs() < 0.001 {
                    return (true, 0.90);
                }
                return (false, 0.1);
            }
            _ => return (false, 0.1),
        }
    }
    // T44: Standalone expression with operators — verify Tanto can parse it.
    if eval_tanto(token).is_none()
        && token
            .bytes()
            .any(|c| matches!(c, b'+' | b'-' | b'*' | b'/' | b'^'))
    {
        return (false, 0.1);
    }
    (true, 0.7)
}

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

fn check_operator_validity(token: &str) -> (bool, f64) {
    if token.is_empty() {
        return (false, 0.0);
    }
    if eval_tanto(token).is_some() {
        return (true, 0.95);
    }
    // Equations containing `=` are handled by check_equation_correctness's
    // split-and-compare path, not by Tanto's eval — the `=` sign is not a
    // Tanto token. Don't reject them here as malformed operator expressions.
    if token.contains('=') {
        return (true, 0.90);
    }
    // T44: Tanto could not parse it — if it contains operators, it's malformed.
    let has_operators = token
        .bytes()
        .any(|c| matches!(c, b'+' | b'-' | b'*' | b'/' | b'^'));
    if has_operators {
        return (false, 0.15);
    }
    let first = token.as_bytes()[0];
    let valid_start = matches!(
        first,
        b'+' | b'-' | b'*' | b'/' | b'^' | b'=' | b'(' | b')' | b'[' | b']' | b'.' | b'0'..=b'9'
    ) || token
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'.');
    (valid_start, if valid_start { 0.85 } else { 0.2 })
}

fn check_domain_consistency(token: &str) -> (bool, f64) {
    if let Some(val) = eval_tanto(token) {
        if val.is_infinite() || val.is_nan() {
            return (false, 0.1);
        }
        if val.abs() > 1e308 {
            return (false, 0.2);
        }
        return (true, 0.95);
    }
    (true, 0.7)
}

pub fn validate(ball: &mut Ball, context: &str) -> GateResult {
    let token = &ball.candidate.token;

    let balance_ok = check_balanced_equation(context, token);
    let (correctness_ok, correctness_score) = check_equation_correctness(token);
    let (operator_ok, operator_score) = check_operator_validity(token);
    let (domain_ok, domain_score) = check_domain_consistency(token);

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
        GateResult::passed(GateKind::Math, avg_score)
    } else {
        GateResult::failed(GateKind::Math, avg_score, &reason.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::ball::TokenCandidate;

    #[test]
    fn test_math_gate_simple() {
        let candidate = TokenCandidate::new(0, "2+3", 0.5);
        let mut ball = Ball::new(candidate);
        let result = validate(&mut ball, "math expression");
        assert!(result.passed);
    }

    #[test]
    fn test_math_gate_equation() {
        let candidate = TokenCandidate::new(0, "2+3 = 5", 0.5);
        let mut ball = Ball::new(candidate);
        let result = validate(&mut ball, "check equation");
        assert!(result.passed);
    }

    #[test]
    fn test_math_gate_unbalanced() {
        let candidate = TokenCandidate::new(0, "(2+3", 0.5);
        let mut ball = Ball::new(candidate);
        let result = validate(&mut ball, "math expression");
        assert!(!result.passed);
    }

    // ── T59: validate-level regressions ──
    //
    // These are the literal commands run against the shipped binary that
    // found the bug: `lai validate "9.11 < 9.9"` and `"9.11 > 9.9"` BOTH
    // reported `passed: true`, and `lai validate "5 >= 3"` — a true claim —
    // reported `passed: false` with "Equation does not balance: 5 != 3",
    // because `find('=')` matched the `=` inside `>=` and split the string
    // as if the operator were plain equality.

    fn validate_token(token: &str) -> bool {
        let candidate = TokenCandidate::new(0, token, 0.5);
        let mut ball = Ball::new(candidate);
        validate(&mut ball, "math expression").passed
    }

    #[test]
    fn t59_decimal_comparison_no_longer_passes_both_directions() {
        // The exact repro: before the fix, both of these reported passed:true.
        assert!(validate_token("9.11 < 9.9"));
        assert!(!validate_token("9.11 > 9.9"));
    }

    #[test]
    fn t59_two_char_operators_no_longer_coerced_to_equality() {
        // Before the fix: "5 >= 3" (true) was REJECTED because 5 != 3.
        assert!(validate_token("5 >= 3"));
        assert!(!validate_token("3 >= 5"));
        assert!(validate_token("3 <= 5"));
        assert!(!validate_token("5 <= 3"));
        assert!(validate_token("5 != 3"));
        assert!(!validate_token("5 != 5"));
    }

    #[test]
    fn t59_does_not_regress_plain_equations_or_arithmetic() {
        // Equations ('=') are untouched — still handled by the pre-existing
        // split-and-compare path, not the new relational branch.
        assert!(validate_token("2 + 3 = 5"));
        assert!(!validate_token("2 + 3 = 6"));
        // Bare arithmetic with no relational operator keeps the T44
        // "parses = passes, independent of value" semantics. This is the
        // specific case that would have broken if the T59 fix were gated on
        // "did eval_tanto succeed" instead of "does the token contain a
        // relational operator": "2 - 2" evaluates to exactly 0.0, which
        // would read as false under naive truthiness despite being a
        // perfectly valid expression.
        assert!(validate_token("2+3"));
        assert!(validate_token("2-2"));
    }
}
