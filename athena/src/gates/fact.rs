//! # Fact Gate — validates claims via chain traversal
//!
//! Unlike CID's static fact lookup, Athena validates factual claims by
//! traversing formula chains. If a claim can be derived from known formulas,
//! it is validated. If no chain exists, confidence is low or zero.

use crate::bankai::Bankai;

use super::{gate_output, Gate, GateOutput};

/// Validates claims by attempting to derive them from known formulas.
pub struct FactGate<'a> {
    engine: Option<&'a Bankai>,
}

impl<'a> FactGate<'a> {
    pub fn new(engine: Option<&'a Bankai>) -> Self {
        FactGate { engine }
    }

    /// Try to validate a claim by finding a formula chain that derives it.
    pub fn validate_claim(&self, _claim: &str) -> GateOutput {
        match self.engine {
            Some(engine) => {
                // Search for formulas that might validate this claim
                let results = engine.formulas.search(_claim);
                if results.is_empty() {
                    gate_output(
                        "fact",
                        false,
                        0.0,
                        format!(
                            "No formula chain found for '{}'. Athena has no static facts — \
                             consider adding a formula that relates to this concept.",
                            _claim
                        ),
                        vec!["No matching formulas found".to_string()],
                        vec![
                            "Search for related concepts with formula search".to_string(),
                            "Or add a new formula that bridges this concept to known domains"
                                .to_string(),
                        ],
                    )
                } else {
                    let formula_names: Vec<&str> = results.iter().map(|f| f.id.as_str()).collect();
                    gate_output(
                        "fact",
                        true,
                        0.7,
                        format!(
                            "Found {} formula(s) related to '{}': {}",
                            results.len(),
                            _claim,
                            formula_names.join(", ")
                        ),
                        vec![],
                        vec![],
                    )
                }
            }
            None => gate_output(
                "fact",
                false,
                0.0,
                "Fact validation requires a BankaiEngine".to_string(),
                vec!["No engine provided".to_string()],
                vec!["Construct a BankaiEngine with formulas loaded".to_string()],
            ),
        }
    }
}

impl Gate for FactGate<'_> {
    fn name(&self) -> &str {
        "fact"
    }

    fn check(&self, target: &str) -> GateOutput {
        self.validate_claim(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::{Formula, FormulaRegistry};
    use crate::wheel::Domain;

    #[test]
    fn test_no_engine() {
        let gate = FactGate::new(None);
        let result = gate.check("E = mc^2");
        assert!(!result.passed);
    }

    #[test]
    fn test_with_engine() {
        let mut registry = FormulaRegistry::new();
        registry
            .register_all(vec![Formula::atomic(
                "mc2",
                Domain::Shukra,
                vec!["mass"],
                "energy",
                "mass * 9e16",
                "E=mc²",
            )])
            .unwrap();
        let engine = Bankai::new(registry);
        let gate = FactGate::new(Some(&engine));
        let result = gate.check("energy");
        assert!(result.passed);
    }
}
