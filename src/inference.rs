//! Inference — LLM copilot integration (stub).
//!
//! Provides the `SandwichCopilot` type used by descent when built with `--features llm`.
//! Full implementation pending llama-gguf integration.

/// Hint from the LLM copilot for token descent.
#[derive(Debug, Clone, Default)]
pub struct DescentHint {
    pub domains: Vec<String>,
    pub entity: Option<String>,
    pub formula: Option<String>,
    pub confidence: f64,
}

/// Qwen copilot for semantic token resolution hints.
#[derive(Debug, Clone, Default)]
pub struct SandwichCopilot;

impl SandwichCopilot {
    pub fn descend_token(
        &self,
        _token: &str,
        _context: Option<&str>,
    ) -> Result<DescentHint, String> {
        Ok(DescentHint::default())
    }
}

pub mod sandwich {
    pub use super::{DescentHint, SandwichCopilot};
}
