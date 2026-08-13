//! # LLM Router — Capstone Layer
//!
//! The capstone on top of Athena's symbolic engine: a tiny LLM copilot
//! (Qwen2.5-0.5B recommended) that assists with natural language routing,
//! formula suggestion, and explanation generation.
//!
//! ## Architecture
//!
//! ```text
//! User Query
//!    │
//!    ▼
//! ┌─────────────────────┐
//! │    LlmRouter        │
//! │  ┌───────────────┐  │
//! │  │ InferenceBackend│ │  ← LocalBackend or RemoteBackend
//! │  └───────────────┘  │
//! │  ┌───────────────┐  │
//! │  │  RouterIntent  │  │  ← structured output for Bankai
//! │  └───────────────┘  │
//! └─────────────────────┘
//!    │
//!    ▼
//! Bankai (symbolic solve)
//! ```
//!
//! ## Determinism Guarantee
//!
//! The LLM copilot is purely advisory. Its outputs influence routing hints
//! and explanations — never core computation. All math, logic, and validation
//! runs through Athena's deterministic symbolic engine.

use crate::inference::{
    config::InferenceConfig, BackendKind, InferenceBackend, InferenceError, InferenceRequest,
    InferenceResponse,
};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

// ─── Routing Types ─────────────────────────────────────────────────────────

/// Structured intent extracted by the LLM router.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterIntent {
    /// High-level query category
    pub category: QueryCategory,
    /// Tokens extracted from query
    pub tokens: Vec<RouterToken>,
    /// Suggested formula chain (formula IDs in order)
    pub suggested_chain: Vec<String>,
    /// Suggested entities to ground in
    pub suggested_entities: Vec<String>,
    /// Confidence in routing (0.0-1.0)
    pub confidence: f64,
    /// Whether to use deterministic path only (no LLM fallback)
    pub deterministic_only: bool,
    /// Raw explanation from copilot
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryCategory {
    // Small models emit lowercase category names no matter what the schema
    // in the prompt shows (observed with Qwen2.5-0.5B, 2026-07-08) — accept
    // both rather than discard a correct route over letter case.
    #[serde(alias = "evaluate")]
    Evaluate,
    #[serde(alias = "chain")]
    Chain,
    #[serde(alias = "compose")]
    Compose,
    #[serde(alias = "reason")]
    Reason,
    #[serde(alias = "entity")]
    Entity,
    #[serde(alias = "search")]
    Search,
    #[serde(alias = "traverse")]
    Traverse,
    #[serde(alias = "validate")]
    Validate,
    #[serde(alias = "info")]
    Info,
    #[serde(alias = "unknown")]
    Unknown,
}

impl std::fmt::Display for QueryCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Evaluate => write!(f, "evaluate"),
            Self::Chain => write!(f, "chain"),
            Self::Compose => write!(f, "compose"),
            Self::Reason => write!(f, "reason"),
            Self::Entity => write!(f, "entity"),
            Self::Search => write!(f, "search"),
            Self::Traverse => write!(f, "traverse"),
            Self::Validate => write!(f, "validate"),
            Self::Info => write!(f, "info"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterToken {
    pub text: String,
    pub domain: Option<String>,
    pub graha: Option<String>,
    pub sign: Option<String>,
    pub aspect_to_next: Option<String>,
    pub mass: f64,
}

/// Result of LLM routing + deterministic execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRouteResult {
    pub intent: RouterIntent,
    pub raw_response: Option<String>,
}

// ─── The Router ────────────────────────────────────────────────────────────

/// Prompt templates for the copilot.
const ROUTING_SYSTEM_PROMPT: &str = r#"You are Athena's routing copilot. Your job is to analyze a user's natural language query and route it to the correct Athena tool.

Athena is a deterministic symbolic reasoning engine. Available tools:
- **evaluate**: Compute a formula with given numbers (e.g., "what's F=ma with mass=5, accel=9.8?")
- **reason**: Find a path from known variables to a desired output (e.g., "how do I get force from mass and velocity?")
- **compose**: Chain formulas across domains with aspect validation
- **search**: Find formulas by keyword (e.g., "find formulas about momentum")
- **entity**: Look up an entity (e.g., "tell me about lithium")
- **traverse**: Explore the zodiac wheel from a domain
- **validate**: Check a claim through math/logic gates
- **info**: System information

Respond with a JSON object:
{
  "category": "evaluate" | "reason" | "compose" | "search" | "entity" | "traverse" | "validate" | "info" | "unknown",
  "tokens": [{"text": "string", "domain": "string or null", "mass": 0.0}],
  "suggested_chain": ["formula_id1", "formula_id2"],
  "suggested_entities": ["entity_id"],
  "confidence": 0.0-1.0,
  "deterministic_only": false,
  "explanation": "brief reasoning about why this route was chosen"
}

Keep responses concise. Only output the JSON, no extra text."#;

/// The LLM Router — caps Athena with a tiny copilot.
///
/// (`Debug` is manual: the backend is a `dyn` object with no useful state to
/// print, and `Bankai` derives `Debug` around an optional router.)
pub struct LlmRouter {
    /// The inference backend (local or remote)
    backend: Mutex<Box<dyn InferenceBackend + Send>>,
    /// Configuration
    config: InferenceConfig,
}

impl std::fmt::Debug for LlmRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LlmRouter").finish_non_exhaustive()
    }
}

impl LlmRouter {
    /// Default model filename for Qwen2.5-0.5B
    pub const DEFAULT_MODEL_FILENAME: &str = "qwen2.5-0.5b-instruct-q4_k_m.gguf";

    /// Create a new LlmRouter from an existing backend.
    pub fn new(backend: Box<dyn InferenceBackend + Send>, config: InferenceConfig) -> Self {
        Self {
            backend: Mutex::new(backend),
            config,
        }
    }

    /// Create a new LlmRouter from config (loads backend automatically).
    pub fn from_config(config: InferenceConfig) -> Result<Self, InferenceError> {
        let backend = Self::create_backend(&config)?;
        Ok(Self {
            backend: Mutex::new(backend),
            config,
        })
    }

    /// Create a new LlmRouter with default config.
    pub fn new_default() -> Result<Self, Box<dyn std::error::Error>> {
        let config = InferenceConfig::load(Default::default());
        let backend = Self::create_backend(&config)?;
        Ok(Self {
            backend: Mutex::new(backend),
            config,
        })
    }

    /// Create the appropriate backend based on config.
    fn create_backend(
        config: &InferenceConfig,
    ) -> Result<Box<dyn InferenceBackend + Send>, InferenceError> {
        match config.backend {
            BackendKind::Local => {
                #[cfg(feature = "llm")]
                {
                    let backend = crate::inference::local::LocalBackend::new(config.clone());
                    Ok(Box::new(backend))
                }
                #[cfg(not(feature = "llm"))]
                {
                    let _ = config;
                    Err(InferenceError::BackendUnavailable(
                        "local backend requires 'llm' feature".to_string(),
                    ))
                }
            }
            BackendKind::Remote => {
                #[cfg(feature = "llm")]
                {
                    let backend = crate::inference::remote::RemoteBackend::new(config.clone());
                    Ok(Box::new(backend))
                }
                #[cfg(not(feature = "llm"))]
                {
                    let _ = config;
                    Err(InferenceError::BackendUnavailable(
                        "remote backend requires 'llm' feature".to_string(),
                    ))
                }
            }
        }
    }

    /// Route a natural language query to a structured intent.
    pub fn route(&mut self, query: &str) -> Result<RouterIntent, Box<dyn std::error::Error>> {
        let mut backend = self
            .backend
            .lock()
            .map_err(|e| format!("lock error: {}", e))?;

        let request = InferenceRequest::new(query.to_string())
            .with_system(ROUTING_SYSTEM_PROMPT)
            .with_temp(0.1); // low temperature for deterministic routing

        let response = backend.generate(request)?;

        // Try to parse as JSON
        let text = response.text.trim().to_string();

        // Extract JSON if wrapped in markdown code blocks
        let json_str = if let Some(start) = text.find("```json") {
            let start = start + 7;
            let end = text[start..]
                .find("```")
                .map(|i| start + i)
                .unwrap_or(text.len());
            text[start..end].trim()
        } else if let Some(start) = text.find('{') {
            let end = text[start..]
                .rfind('}')
                .map(|i| start + i + 1)
                .unwrap_or(text.len());
            &text[start..end]
        } else {
            &text
        };

        match serde_json::from_str::<RouterIntent>(json_str) {
            Ok(mut intent) => {
                // Ensure deterministic_only for low confidence
                if intent.confidence < 0.3 {
                    intent.deterministic_only = true;
                }
                Ok(intent)
            }
            Err(e) => {
                // If JSON parsing fails, return a fallback with Unknown category
                Ok(RouterIntent {
                    category: QueryCategory::Unknown,
                    tokens: vec![],
                    suggested_chain: vec![],
                    suggested_entities: vec![],
                    confidence: 0.1,
                    deterministic_only: true,
                    explanation: format!(
                        "Copilot returned non-JSON response (parse error: {}). Raw: {}",
                        e,
                        text.chars().take(200).collect::<String>()
                    ),
                })
            }
        }
    }

    /// Generate a free-form response from the copilot.
    pub fn generate(
        &mut self,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<InferenceResponse, InferenceError> {
        let mut backend = self
            .backend
            .lock()
            .map_err(|e| InferenceError::InferenceFailed(format!("lock error: {}", e)))?;

        let mut request = InferenceRequest::new(prompt.to_string());
        if let Some(sys) = system {
            request = request.with_system(sys);
        }

        backend.generate(request)
    }

    /// Generate at temperature 0 — for gated `kind = "llm"` formula
    /// evaluation, where the same prompt should reproduce the same value as
    /// closely as the backend allows.
    pub fn generate_deterministic(
        &mut self,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<InferenceResponse, InferenceError> {
        let mut backend = self
            .backend
            .lock()
            .map_err(|e| InferenceError::InferenceFailed(format!("lock error: {}", e)))?;

        let mut request = InferenceRequest::new(prompt.to_string()).with_temp(0.0);
        if let Some(sys) = system {
            request = request.with_system(sys);
        }

        backend.generate(request)
    }

    /// Check backend health.
    pub fn health(&self) -> crate::inference::HealthStatus {
        self.backend
            .lock()
            .map(|b| b.health())
            .unwrap_or(crate::inference::HealthStatus {
                healthy: false,
                model_loaded: None,
                backend_kind: self.config.backend.clone(),
                context_size: Some(self.config.context_size),
                message: "Failed to acquire backend lock".to_string(),
            })
    }

    /// Get backend capabilities.
    pub fn capabilities(&self) -> Vec<crate::inference::Capability> {
        self.backend
            .lock()
            .map(|b| b.capabilities())
            .unwrap_or_default()
    }

    /// Get the config reference.
    pub fn config(&self) -> &InferenceConfig {
        &self.config
    }

    /// Reload with a new config.
    pub fn reload(&mut self, config: InferenceConfig) -> Result<(), InferenceError> {
        let backend = Self::create_backend(&config)?;
        self.config = config;
        *self
            .backend
            .lock()
            .map_err(|e| InferenceError::InferenceFailed(format!("lock error: {}", e)))? = backend;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_category_accepts_lowercase() {
        // Qwen2.5-0.5B emits lowercase category names regardless of the
        // schema shown in the prompt (live playtest, 2026-07-08); the
        // aliases must keep accepting them.
        for (raw, expected) in [
            ("\"validate\"", "Validate"),
            ("\"Validate\"", "Validate"),
            ("\"evaluate\"", "Evaluate"),
            ("\"search\"", "Search"),
        ] {
            let cat: QueryCategory = serde_json::from_str(raw).unwrap();
            assert_eq!(format!("{:?}", cat), expected);
        }
    }

    #[test]
    fn test_router_intent_serialization() {
        let intent = RouterIntent {
            category: QueryCategory::Evaluate,
            tokens: vec![RouterToken {
                text: "force".to_string(),
                domain: Some("shukra".to_string()),
                graha: None,
                sign: None,
                aspect_to_next: None,
                mass: 1.0,
            }],
            suggested_chain: vec!["newtons_second".to_string()],
            suggested_entities: vec![],
            confidence: 0.95,
            deterministic_only: false,
            explanation: "Query asks for force calculation".to_string(),
        };

        let json = serde_json::to_string(&intent).unwrap();
        let parsed: RouterIntent = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed.category, QueryCategory::Evaluate));
        assert_eq!(parsed.suggested_chain[0], "newtons_second");
        assert!((parsed.confidence - 0.95).abs() < 1e-6);
    }

    #[test]
    fn test_query_category_display() {
        assert_eq!(QueryCategory::Evaluate.to_string(), "evaluate");
        assert_eq!(QueryCategory::Reason.to_string(), "reason");
        assert_eq!(QueryCategory::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_router_intent_deterministic_fallback() {
        let mut intent = RouterIntent {
            category: QueryCategory::Search,
            tokens: vec![],
            suggested_chain: vec![],
            suggested_entities: vec![],
            confidence: 0.1, // low confidence
            deterministic_only: false,
            explanation: "test".to_string(),
        };

        if intent.confidence < 0.3 {
            intent.deterministic_only = true;
        }

        assert!(intent.deterministic_only);
    }

    #[test]
    fn test_router_fallback_parse() {
        // Simulate a non-JSON response
        let text = "I think this is about force calculation.".to_string();
        let json_str = if let Some(start) = text.find('{') {
            let end = text[start..]
                .rfind('}')
                .map(|i| start + i + 1)
                .unwrap_or(text.len());
            &text[start..end]
        } else {
            &text
        };

        let result = serde_json::from_str::<RouterIntent>(json_str);
        assert!(result.is_err()); // no JSON found
    }
}
