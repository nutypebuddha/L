use crate::scoring::ball::{Ball, GateResult};
use crate::scoring::pin::GateKind;

fn check_proof_structure(token: &str, context: &str) -> (bool, f64) {
    let lower_token = token.to_lowercase();
    let lower_context = context.to_lowercase();

    let proof_keywords = [
        "prove",
        "proof",
        "theorem",
        "lemma",
        "corollary",
        "hypothesis",
        "assumption",
        "given",
        "therefore",
        "thus",
        "hence",
        "implies",
        "conclude",
        "q.e.d.",
        "qed",
    ];

    let has_proof_keyword = proof_keywords
        .iter()
        .any(|kw| lower_token.contains(kw) || lower_context.contains(kw));

    if !has_proof_keyword {
        return (true, 0.7);
    }

    let logical_structures = [
        "for all",
        "exists",
        "if and only if",
        "iff",
        "implies",
        "=>",
        "->",
        "∀",
        "∃",
        "↔",
        "→",
    ];

    let has_logical_structure = logical_structures
        .iter()
        .any(|ls| lower_token.contains(ls) || lower_context.contains(ls));

    let quantifiers = [
        "for every",
        "for each",
        "there exists",
        "there is",
        "for all x",
        "for any",
    ];

    let has_quantifiers = quantifiers
        .iter()
        .any(|q| lower_token.contains(q) || lower_context.contains(q));

    let score = if has_logical_structure && has_quantifiers {
        0.9
    } else if has_logical_structure || has_quantifiers {
        0.8
    } else {
        0.7
    };

    (true, score)
}

fn check_logical_errors(token: &str, context: &str) -> (bool, f64) {
    let lower_token = token.to_lowercase();
    let lower_context = context.to_lowercase();

    let scope_errors = ["for all x there exists", "there exists for all"];
    let has_scope_error = scope_errors
        .iter()
        .any(|e| lower_token.contains(e) || lower_context.contains(e));

    let division_by_zero = ["divide by zero", "division by zero", "/ 0", "/0"];
    let has_division_by_zero = division_by_zero
        .iter()
        .any(|d| lower_token.contains(d) || lower_context.contains(d));

    let circular = ["because", "since", "as shown above", "as proved"];
    let has_circular = circular
        .iter()
        .any(|c| lower_token.contains(c) && lower_context.contains(c));

    if has_scope_error || has_division_by_zero || has_circular {
        return (false, 0.2);
    }

    (true, 0.8)
}

fn check_well_formed(token: &str, _context: &str) -> (bool, f64) {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;

    for ch in token.chars() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            '[' => bracket_depth += 1,
            ']' => bracket_depth -= 1,
            _ => {}
        }

        if paren_depth < 0 || bracket_depth < 0 {
            return (false, 0.3);
        }
    }

    if paren_depth != 0 || bracket_depth != 0 {
        return (false, 0.4);
    }

    if token.trim().len() < 2 {
        return (true, 0.5);
    }

    (true, 0.85)
}

pub fn validate(ball: &mut Ball, context: &str) -> GateResult {
    let token = &ball.candidate.token;

    let (structure_ok, structure_score) = check_proof_structure(token, context);
    let (errors_ok, errors_score) = check_logical_errors(token, context);
    let (well_formed_ok, well_formed_score) = check_well_formed(token, context);

    let scores = [structure_score, errors_score, well_formed_score];
    let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;

    let passed = structure_ok && errors_ok && well_formed_ok;
    let reason = if !structure_ok {
        Some("Invalid proof structure".to_string())
    } else if !errors_ok {
        Some("Logical error detected".to_string())
    } else if !well_formed_ok {
        Some("Malformed expression".to_string())
    } else {
        None
    };

    if passed {
        GateResult::passed(GateKind::Formal, avg_score)
    } else {
        GateResult::failed(GateKind::Formal, avg_score, &reason.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::ball::TokenCandidate;

    #[test]
    fn test_formal_gate_valid() {
        let candidate = TokenCandidate::new(0, "For all x, P(x) implies Q(x)", 0.5);
        let mut ball = Ball::new(candidate);
        let result = validate(&mut ball, "theorem proof");
        assert!(result.passed);
    }

    #[test]
    fn test_formal_gate_division_by_zero() {
        let candidate = TokenCandidate::new(0, "divide by zero", 0.5);
        let mut ball = Ball::new(candidate);
        let result = validate(&mut ball, "math proof");
        assert!(!result.passed);
    }

    #[test]
    fn test_formal_gate_unbalanced() {
        let candidate = TokenCandidate::new(0, "(x + 1", 0.5);
        let mut ball = Ball::new(candidate);
        let result = validate(&mut ball, "expression");
        assert!(!result.passed);
    }
}
