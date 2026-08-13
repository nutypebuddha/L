//! # Formula — pure primitive operations
//!
//! Formulas are pure primitive operations — math gates and logic gates only.
//! No domain-specific formulas, no bridging, no vortex spirals.
//! All domain knowledge lives in entities and the wheel graph.
//!
//! ## Math Gates (15)
//!
//! add, sub, mul, div, mod, pow, sqrt, neg, abs, log10, ln, floor, ceil, min, max
//!
//! All provable via Peano arithmetic or real field axioms.
//!
//! ## Logic Gates (7)
//!
//! nand, and, or, not, xor, nor, xnor
//!
//! All derivable from NAND (Sheffer stroke — functionally complete).
//! Provable by truth table.

use serde::{Deserialize, Serialize};

use crate::wheel::Domain;

mod glyph;
mod registry;

pub use glyph::{
    apply_operator, binding_power, decompose_bound, is_bound, Glyph, GlyphOperator, GlyphResult,
    NamedGlyph, GLYPH_COUNT, MAX_GLYPH, OPERATOR_TABLE,
};
pub use registry::FormulaRegistry;

pub use lai_core::formula::{FormulaError, FormulaType};

/// A single primitive formula definition.
///
/// Every formula is a pure primitive operation with typed inputs,
/// a single output, and an evaluable expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Formula {
    /// Unique identifier (e.g. "add", "nand", "sqrt")
    pub id: String,

    /// The type of formula (math gate or logic gate)
    pub formula_type: FormulaType,

    /// The domain this formula belongs to.
    /// Math gates → Aries (Math & Logic).
    /// Logic gates → Scorpio (CS & AI).
    pub domain: Domain,

    /// Input parameter names.
    pub inputs: Vec<String>,

    /// Output parameter name.
    pub output: String,

    /// The evaluable expression using meval-compatible syntax.
    /// For math: arithmetic expression like "a + b", "sqrt(x)"
    /// For logic: boolean expression like "1 - a*b" (NAND)
    pub expression: String,

    /// Human-readable description.
    pub description: String,

    /// Zodiac symbol associations for the wheel classifier.
    /// These replace the old string tags and are used by Zanpakuto
    /// to match tokens to domains on the wheel.
    #[serde(default)]
    pub zodiac: Vec<String>,

    /// Optional evidence or source for this formula's provability.
    pub evidence: Option<String>,

    /// Additional domains this formula is also registered under.
    ///
    /// Populated when a later-loaded formula file declares the same `id`
    /// with a different `domain` (e.g. `aries_math.toml`'s "add" vs.
    /// `mangala_math.toml`'s "add" — same content, dual Western/Vedic
    /// cosmology). Without this, `FormulaRegistry::register` used to
    /// silently overwrite the earlier domain tag on collision.
    #[serde(default)]
    pub also_domains: Vec<Domain>,
}

impl Formula {
    /// Create a new primitive formula.
    pub fn new(
        id: &str,
        formula_type: FormulaType,
        domain: Domain,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula {
            id: id.to_string(),
            formula_type,
            domain,
            inputs: inputs.into_iter().map(String::from).collect(),
            output: output.to_string(),
            expression: expression.to_string(),
            description: description.to_string(),
            zodiac: Vec::new(),
            evidence: None,
            also_domains: Vec::new(),
        }
    }

    /// Create a math gate formula (convenience constructor).
    pub fn math(
        id: &str,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula::new(
            id,
            FormulaType::Math,
            Domain::Mangala,
            inputs,
            output,
            expression,
            description,
        )
    }

    /// Create a logic gate formula (convenience constructor).
    pub fn logic(
        id: &str,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula::new(
            id,
            FormulaType::Logic,
            Domain::Budha,
            inputs,
            output,
            expression,
            description,
        )
    }

    /// Create an atomic formula (convenience, equivalent to `math` with explicit domain).
    /// This preserves the API used throughout the codebase and tests.
    pub fn atomic(
        id: &str,
        domain: Domain,
        inputs: Vec<&str>,
        output: &str,
        expression: &str,
        description: &str,
    ) -> Self {
        Formula::new(
            id,
            FormulaType::Math,
            domain,
            inputs,
            output,
            expression,
            description,
        )
    }

    /// Create a math gate with zodiac associations.
    pub fn with_zodiac(mut self, zodiac: Vec<&str>) -> Self {
        self.zodiac = zodiac.into_iter().map(String::from).collect();
        self
    }

    /// Attach evidence to this formula.
    pub fn with_evidence(mut self, evidence: &str) -> Self {
        self.evidence = Some(evidence.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_math_formula() {
        let f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition");
        assert_eq!(f.formula_type, FormulaType::Math);
        assert_eq!(f.domain, Domain::Mangala);
        assert_eq!(f.id, "add");
        assert_eq!(f.inputs, vec!["a", "b"]);
        assert_eq!(f.output, "sum");
    }

    #[test]
    fn test_create_logic_formula() {
        let f = Formula::logic("nand", vec!["a", "b"], "out", "1 - a*b", "NAND gate");
        assert_eq!(f.formula_type, FormulaType::Logic);
        assert_eq!(f.domain, Domain::Budha);
        assert_eq!(f.output, "out");
    }

    #[test]
    fn test_formula_with_zodiac() {
        let f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition")
            .with_zodiac(vec!["♈", "aries", "math"]);
        assert_eq!(f.zodiac.len(), 3);
        assert!(f.zodiac.contains(&"♈".to_string()));
    }

    #[test]
    fn test_formula_with_evidence() {
        let f = Formula::math("add", vec!["a", "b"], "sum", "a + b", "Addition")
            .with_evidence("Peano arithmetic: S(a) + b = S(a + b)");
        assert!(f.evidence.is_some());
    }

    #[test]
    fn test_formula_type_description() {
        assert!(FormulaType::Math.description().contains("Math"));
        assert!(FormulaType::Logic.description().contains("Logic"));
    }
}
