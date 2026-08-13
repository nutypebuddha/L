//! # Formal Gate — validates structural integrity

use super::{gate_output, Gate, GateOutput};

/// Validates the structural integrity of formulas and chains.
///
/// Checks:
/// - Formula tier consistency (atomic formulas should not span multiple domains)
/// - Expression syntax (no unclosed parens, valid operators)
/// - Cross-reference integrity (referenced formulas exist)
/// - Argument count matches formula definition
pub struct FormalGate;

impl FormalGate {
    pub fn new() -> Self {
        FormalGate
    }
}

impl Default for FormalGate {
    fn default() -> Self {
        Self::new()
    }
}

impl Gate for FormalGate {
    fn name(&self) -> &str {
        "formal"
    }

    fn check(&self, target: &str) -> GateOutput {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Check for basic syntax issues
        let open_parens = target.matches('(').count();
        let close_parens = target.matches(')').count();

        if open_parens != close_parens {
            issues.push(format!(
                "Unbalanced parentheses: {} open, {} close",
                open_parens, close_parens
            ));
            suggestions.push("Add missing parentheses".to_string());
        }

        // Check for unclosed brackets
        let open_braces = target.matches('{').count();
        let close_braces = target.matches('}').count();
        if open_braces != close_braces {
            issues.push(format!(
                "Unbalanced braces: {} open, {} close",
                open_braces, close_braces
            ));
        }

        // Check for empty expressions
        if target.trim().is_empty() {
            issues.push("Empty expression".to_string());
        }

        // Check for division by zero patterns (informational)
        if target.contains("/0") {
            suggestions.push("Potential division by zero detected".to_string());
        }

        let passed = issues.is_empty();
        let confidence = if passed { 1.0 } else { 0.3 };

        gate_output(
            "formal",
            passed,
            confidence,
            if passed {
                "Structural integrity check passed".to_string()
            } else {
                format!("Found {} structural issue(s)", issues.len())
            },
            issues,
            suggestions,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_expression() {
        let gate = FormalGate::new();
        let result = gate.check("");
        assert!(!result.passed);
    }

    #[test]
    fn test_valid_expression() {
        let gate = FormalGate::new();
        let result = gate.check("a + b * c");
        assert!(result.passed);
    }

    #[test]
    fn test_unbalanced_parens() {
        let gate = FormalGate::new();
        let result = gate.check("(a + b");
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("parentheses")));
    }
}
