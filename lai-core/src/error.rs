//! # LaiError — Unified error hierarchy for the L.ai workspace
//!
//! Every crate in the workspace maps its local error types into `LaiError`
//! at API boundaries, giving callers a single `Result<T, LaiError>` to handle.

pub use crate::formula::FormulaError;
use crate::primitive::NandExprError;

/// Root error type for the entire L.ai system.
#[derive(Debug, thiserror::Error)]
pub enum LaiError {
    #[error(transparent)]
    Reasoning(#[from] ReasoningError),

    #[error(transparent)]
    Validation(#[from] ValidationError),

    #[error(transparent)]
    Formula(#[from] FormulaError),

    #[error(transparent)]
    Inference(#[from] InferenceError),

    #[error(transparent)]
    Parser(#[from] ParserError),

    #[error(transparent)]
    NandExpr(#[from] NandExprError),

    #[error(transparent)]
    Transport(#[from] TransportError),

    #[error("{0}")]
    Internal(String),
}

// ─── Domain errors ──────────────────────────────────────────────────────────

/// Errors originating from reasoning and proof logic.
#[derive(Debug, thiserror::Error)]
pub enum ReasoningError {
    #[error("no proof found for query: {query}")]
    NoProof { query: String },

    #[error("proof rejected: {reason}")]
    Rejected { reason: String },

    #[error("budget exhausted: requested {requested}, available {available}")]
    BudgetExhausted { requested: usize, available: usize },

    #[error("circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("unsupported query type: {0}")]
    UnsupportedQuery(String),
}

/// Errors from input validation and schema checks.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("invalid domain: {0}")]
    InvalidDomain(String),

    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("out of range: {field} = {value} (expected {min}..{max}")]
    OutOfRange {
        field: String,
        value: f64,
        min: f64,
        max: f64,
    },

    #[error("malformed input: {0}")]
    Malformed(String),
}

/// Errors from LLM or neural inference paths.
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("model not loaded: {0}")]
    ModelNotLoaded(String),

    #[error("inference failed: {0}")]
    Failed(String),

    #[error("token budget exceeded during inference")]
    TokenBudgetExceeded,
}

/// Errors from NLP and query parsing.
#[derive(Debug, thiserror::Error)]
pub enum ParserError {
    #[error("syntax error: {0}")]
    Syntax(String),

    #[error("unknown token: {0}")]
    UnknownToken(String),

    #[error("unexpected end of input")]
    UnexpectedEof,
}

/// Errors from network or IPC transport.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("connection refused: {0}")]
    ConnectionRefused(String),

    #[error("timeout after {0}ms")]
    Timeout(u64),

    #[error("protocol error: {0}")]
    Protocol(String),
}

impl From<String> for LaiError {
    fn from(s: String) -> Self {
        LaiError::Internal(s)
    }
}

impl From<&str> for LaiError {
    fn from(s: &str) -> Self {
        LaiError::Internal(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lai_error_display() {
        let err = LaiError::Reasoning(ReasoningError::NoProof {
            query: "test".into(),
        });
        assert!(err.to_string().contains("no proof found"));
    }

    #[test]
    fn test_lai_error_from_string() {
        let err: LaiError = "something broke".into();
        assert!(err.to_string().contains("something broke"));
    }

    #[test]
    fn test_lai_error_from_formula_error() {
        let err = LaiError::Formula(FormulaError::NotFound("add".into()));
        assert!(err.to_string().contains("formula not found"));
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::OutOfRange {
            field: "mass".into(),
            value: -1.0,
            min: 0.0,
            max: 1.0,
        };
        assert!(err.to_string().contains("out of range"));
    }
}
