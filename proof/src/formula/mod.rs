/// Pure function: Validate a formula ID string.
pub fn validate_formula_id(formula_id: &str) -> bool {
    !formula_id.is_empty() && formula_id.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Pure function: Extract domain from a formula ID.
pub fn extract_formula_domain(formula_id: &str) -> &str {
    formula_id.split('_').next().unwrap_or("")
}

/// Pure function: Check if formula ID is atomic type.
pub fn is_atomic_formula(formula_id: &str) -> bool {
    extract_formula_domain(formula_id) == "atomic"
}

/// Pure function: Check if formula ID is bridging type.
pub fn is_bridging_formula(formula_id: &str) -> bool {
    extract_formula_domain(formula_id) == "bridging"
}

mod glyph;
pub mod nonmath;
mod registry;

pub use glyph::{
    apply_operator, binding_power, decompose_bound, is_bound, Glyph, GlyphOperator, GlyphResult,
    NamedGlyph, GLYPH_COUNT, MAX_GLYPH, OPERATOR_TABLE,
};
pub use registry::FormulaRegistry;

pub use lai_core::formula::{Formula, FormulaError, FormulaType};
