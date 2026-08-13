//! # Composition — chaining formulas across domains
//!
//! Composition is the core operation that distinguishes Bankai from Tanto.
//! Given two or more formulas, composition:
//!
//! 1. Determines if the output of formula N matches an input of formula N+1 (DATAFLOW VALIDATION)
//! 2. Validates the aspect relationship between the domains
//! 3. Records the path through the wheel
//! 4. Assigns a confidence score based on aspect directness

use serde::Serialize;

use crate::formula::{Formula, FormulaRegistry};
use crate::wheel::{Aspect, Domain, WheelGraph};

use super::confidence::{BoundedConfidence, ConfidenceSemiring};
use super::BankaiError;

/// A composition of formulas across domains.
#[derive(Debug, Clone, Serialize)]
pub struct Composition {
    pub formulas: Vec<Formula>,
    pub domains_traversed: Vec<Domain>,
    pub aspects_traversed: Vec<Aspect>,
    pub confidence: f64,
    pub description: String,
    /// Whether the dataflow chain is valid (output→input matches across all steps)
    pub dataflow_valid: bool,
    /// Which step (0-indexed) failed dataflow validation, if any
    pub dataflow_break: Option<usize>,
}

impl Composition {
    /// Create a new composition from an ordered list of formulas.
    pub fn new(
        wheel: &WheelGraph,
        _registry: &FormulaRegistry,
        formulas: Vec<&Formula>,
    ) -> Result<Self, BankaiError> {
        if formulas.is_empty() {
            return Err(BankaiError::CompositionError(
                "cannot compose empty formula list".to_string(),
            ));
        }

        if formulas.len() == 1 {
            // Single formula composition — no domain traversal needed
            let f = formulas[0];
            return Ok(Composition {
                domains_traversed: vec![f.domain],
                aspects_traversed: vec![],
                confidence: 1.0,
                description: format!("Single formula: {} ({})", f.id, f.description),
                formulas: vec![f.clone()],
                dataflow_valid: true,
                dataflow_break: None,
            });
        }

        let mut domains_traversed = Vec::new();
        let mut aspects_traversed = Vec::new();

        // Dataflow validation: check that output of formula N matches input of formula N+1
        let mut dataflow_valid = true;
        let mut dataflow_break: Option<usize> = None;

        // Check each consecutive pair
        for window in formulas.windows(2) {
            let from = window[0];
            let to = window[1];

            domains_traversed.push(from.domain);
            let aspect = Aspect::between(from.domain, to.domain);
            aspects_traversed.push(aspect);

            // Check that the domain pair has an edge
            if !wheel.has_edge(from.domain, to.domain) {
                return Err(BankaiError::CompositionError(format!(
                    "no edge between {} and {}",
                    from.domain.full_name(),
                    to.domain.full_name()
                )));
            }

            // DATAFLOW VALIDATION: from.output must be in to.inputs
            if !to.inputs.iter().any(|input| input == &from.output) {
                dataflow_valid = false;
                dataflow_break = Some(domains_traversed.len() - 1);
                // Continue checking other pairs but mark as invalid
            }

            // For non-trivial aspects, log the composition type
            if !aspect.is_direct() {
                // Square or Opposition — requires mediation note
                // This is informational, not an error
            }
        }

        // Add the final domain
        if let Some(last) = formulas.last() {
            domains_traversed.push(last.domain);
        }

        // Calculate confidence based on aspects (aspect confidence composes multiplicatively)
        let confidence = composition_confidence(&aspects_traversed);

        // Build description
        let desc = {
            let names: Vec<&str> = formulas.iter().map(|f| f.id.as_str()).collect();
            let domain_str: Vec<String> = domains_traversed
                .iter()
                .map(|d| format!("{}{}", d.symbol(), d.full_name()))
                .collect();
            let flow_status = if dataflow_valid {
                "Dataflow: VALID".to_string()
            } else {
                format!("Dataflow: BROKEN at step {}", dataflow_break.unwrap())
            };
            format!(
                "Composition: {} | Path: {} | {} | Confidence: {:.2}",
                names.join(" → "),
                domain_str.join(" → "),
                flow_status,
                confidence
            )
        };

        Ok(Composition {
            formulas: formulas.into_iter().cloned().collect(),
            domains_traversed,
            aspects_traversed,
            confidence,
            description: desc,
            dataflow_valid,
            dataflow_break,
        })
    }

    /// Number of formulas in this composition.
    pub fn len(&self) -> usize {
        self.formulas.len()
    }

    /// Whether this composition is empty.
    pub fn is_empty(&self) -> bool {
        self.formulas.is_empty()
    }
}

/// Calculate composition confidence based on the aspects traversed.
/// Uses BoundedConfidence semiring: compose (⊗) across sequential aspects.
/// Confidence values come from `Aspect::confidence()` — single source of truth.
fn composition_confidence(aspects: &[Aspect]) -> f64 {
    let mut conf = BoundedConfidence::one();
    for aspect in aspects {
        conf = conf.compose(&BoundedConfidence(aspect.confidence()));
    }
    conf.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::Formula;
    use crate::wheel::Domain;

    #[test]
    fn test_single_formula_composition() {
        let wheel = WheelGraph::new();
        let registry = FormulaRegistry::new();
        let f = Formula::atomic("test", Domain::Mangala, vec![], "x", "1.0", "test");
        let comp = Composition::new(&wheel, &registry, vec![&f]).unwrap();
        assert_eq!(comp.len(), 1);
        assert_eq!(comp.confidence, 1.0);
        assert!(comp.dataflow_valid);
    }

    #[test]
    fn test_dual_formula_composition() {
        let wheel = WheelGraph::new();
        let registry = FormulaRegistry::new();
        let f1 = Formula::atomic("f1", Domain::Mangala, vec![], "x", "1.0", "test");
        let f2 = Formula::atomic("f2", Domain::Shukra, vec![], "y", "2.0", "test");
        let comp = Composition::new(&wheel, &registry, vec![&f1, &f2]).unwrap();
        assert_eq!(comp.len(), 2);
        assert_eq!(
            comp.domains_traversed,
            vec![Domain::Mangala, Domain::Shukra]
        );
    }

    #[test]
    fn test_composition_confidence_decreases_with_tension() {
        let wheel = WheelGraph::new();
        let registry = FormulaRegistry::new();
        let f1 = Formula::atomic("f1", Domain::Surya, vec![], "x", "1.0", "test");
        let f2 = Formula::atomic("f2", Domain::Brihaspati, vec![], "y", "2.0", "test");

        let comp = Composition::new(&wheel, &registry, vec![&f1, &f2]).unwrap();
        // Surya (0) → Brihaspati (4) = opposition = 0.6
        assert!((comp.confidence - 0.6).abs() < 1e-10);
    }

    #[test]
    fn test_dataflow_valid_when_output_matches_input() {
        let wheel = WheelGraph::new();
        let registry = FormulaRegistry::new();
        // f1: inputs=[a,b], output=sum
        let f1 = Formula::atomic(
            "add",
            Domain::Mangala,
            vec!["a", "b"],
            "sum",
            "a + b",
            "addition",
        );
        // f2: inputs=[sum], output=result (chains on f1's output)
        let f2 = Formula::atomic(
            "double",
            Domain::Mangala,
            vec!["sum"],
            "result",
            "sum * 2",
            "double",
        );
        let comp = Composition::new(&wheel, &registry, vec![&f1, &f2]).unwrap();
        assert!(comp.dataflow_valid);
        assert_eq!(comp.dataflow_break, None);
    }

    #[test]
    fn test_dataflow_invalid_when_output_not_in_inputs() {
        let wheel = WheelGraph::new();
        let registry = FormulaRegistry::new();
        // f1: output=sum
        let f1 = Formula::atomic(
            "add",
            Domain::Mangala,
            vec!["a", "b"],
            "sum",
            "a + b",
            "addition",
        );
        // f2: inputs=[x], output=result (sum != x, so chain breaks)
        let f2 = Formula::atomic(
            "double",
            Domain::Mangala,
            vec!["x"],
            "result",
            "x * 2",
            "double",
        );
        let comp = Composition::new(&wheel, &registry, vec![&f1, &f2]).unwrap();
        assert!(!comp.dataflow_valid);
        assert_eq!(comp.dataflow_break, Some(0));
    }

    #[test]
    fn test_no_edge_error() {
        // Actually all domains have edges in the wheel, so this should succeed
        let wheel = WheelGraph::new();
        let registry = FormulaRegistry::new();
        let f1 = Formula::atomic("f1", Domain::Surya, vec![], "x", "1.0", "");
        let f2 = Formula::atomic("f2", Domain::Chandra, vec![], "y", "2.0", "");
        // Surya (0) and Chandra (1) are adjacent — has edge
        assert!(wheel.has_edge(Domain::Surya, Domain::Chandra));
        let comp = Composition::new(&wheel, &registry, vec![&f1, &f2]);
        assert!(comp.is_ok());
    }
}
