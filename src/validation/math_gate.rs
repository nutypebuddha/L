use crate::compute;
use crate::scoring::ball::{Ball, GateResult};
use crate::scoring::pin::GateKind;

fn eval_tanto(expr: &str) -> Option<f64> {
    let env = compute::create_env();
    compute::evaluate_nl(expr, &env)
}

fn check_equation_correctness(token: &str) -> (bool, f64) {
    if let Some(eq_pos) = token.find('=') {
        let lhs = token[..eq_pos].trim();
        let rhs = token[eq_pos + 1..].trim();
        match (eval_tanto(lhs), eval_tanto(rhs)) {
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
}
