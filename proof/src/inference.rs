//! Inference — local LLM front for the Laverna personality agent.
//!
//! Behind `--features llm`, the `Copilot` drives a bundled local llama.cpp
//! binary (vendored at `bin/llama/llama`) with a GGUF model from `bin/models/`.
//! The binary is packaged with `laverna` so the assistant works offline out of
//! the box. Override with `LAVERNA_LLAMA_BIN` / `LAVERNA_LLAMA_MODEL`. Every
//! answer is run through `companion::sanitize_answer` so internal
//! mechanics/brand never surface.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::companion::{sanitize_answer, PERSONA_SYSTEM_PROMPT};

/// Hint from the copilot for token descent (kept for the descent pipeline).
#[derive(Debug, Clone, Default)]
pub struct DescentHint {
    pub domains: Vec<String>,
    pub entity: Option<String>,
    pub formula: Option<String>,
    pub confidence: f64,
}

/// Error type for local-model invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopilotError {
    BinaryMissing,
    ModelMissing,
    SpawnFailed(String),
    NoOutput,
}

/// Local llama.cpp-backed persona. Pure-ish: given a prompt it returns the
/// model's natural-language answer (sanitized). No global state.
#[derive(Debug, Clone, Default)]
pub struct Copilot;

impl Copilot {
    /// Directory of the `laverna` executable (best-effort).
    fn exe_dir() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    }

    /// Resolve the local model binary. Env `LAVERNA_LLAMA_BIN` wins; else the
    /// binary vendored next to the distribution at `bin/llama/llama`.
    pub fn binary_path() -> Option<PathBuf> {
        if let Ok(env) = std::env::var("LAVERNA_LLAMA_BIN") {
            if !env.is_empty() {
                return Some(PathBuf::from(env));
            }
        }
        Self::exe_dir()
            .and_then(|d| d.parent().map(|p| p.to_path_buf()))
            .map(|repo| repo.join("bin").join("llama").join("llama"))
    }

    /// Resolve the GGUF model. Env `LAVERNA_LLAMA_MODEL` wins; else the first
    /// `*.gguf` found in `bin/models/`.
    pub fn model_path() -> Option<PathBuf> {
        if let Ok(env) = std::env::var("LAVERNA_LLAMA_MODEL") {
            if !env.is_empty() {
                return Some(PathBuf::from(env));
            }
        }
        let models_dir = Self::exe_dir()
            .and_then(|d| d.parent().map(|p| p.to_path_buf()))
            .map(|repo| repo.join("bin").join("models"))?;
        std::fs::read_dir(&models_dir)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().map(|x| x == "gguf").unwrap_or(false))
    }

    /// Run the persona over `user_query` with `grounded_context` (the routed
    /// tool result) and return the sanitized natural-language answer.
    pub fn answer(&self, user_query: &str, grounded_context: &str) -> Result<String, CopilotError> {
        let bin = Self::binary_path().ok_or(CopilotError::BinaryMissing)?;
        if !bin.exists() {
            return Err(CopilotError::BinaryMissing);
        }
        let model = Self::model_path().ok_or(CopilotError::ModelMissing)?;
        if !model.exists() {
            return Err(CopilotError::ModelMissing);
        }
        let prompt = format!(
            "{}\n\nContext (verified):\n{}\n\nUser: {}\n\nLaverna:",
            PERSONA_SYSTEM_PROMPT, grounded_context, user_query
        );
        let output = Command::new(&bin)
            .arg("-m")
            .arg(&model)
            .arg("-p")
            .arg(&prompt)
            .arg("--temp")
            .arg("0.2")
            .arg("-n")
            .arg("256")
            .arg("--no-display-prompt")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
            .map_err(|e| CopilotError::SpawnFailed(e.to_string()))?;
        if !output.status.success() {
            return Err(CopilotError::SpawnFailed(format!("exit {}", output.status)));
        }
        let raw = String::from_utf8_lossy(&output.stdout);
        let answer = raw
            .lines()
            .find(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .ok_or(CopilotError::NoOutput)?;
        Ok(sanitize_answer(&answer))
    }

    /// Descent hint (unchanged stub path; used by the descent planner).
    pub fn descend_token(
        &self,
        _token: &str,
        _context: Option<&str>,
    ) -> Result<DescentHint, String> {
        Ok(DescentHint::default())
    }
}

pub mod copilot {
    pub use super::{Copilot, CopilotError, DescentHint};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_overrides_binary_path() {
        std::env::set_var("LAVERNA_LLAMA_BIN", "/custom/llama");
        assert_eq!(Copilot::binary_path(), Some(PathBuf::from("/custom/llama")));
        std::env::remove_var("LAVERNA_LLAMA_BIN");
    }

    #[test]
    fn env_overrides_model_path() {
        std::env::set_var("LAVERNA_LLAMA_MODEL", "/custom/model.gguf");
        assert_eq!(
            Copilot::model_path(),
            Some(PathBuf::from("/custom/model.gguf"))
        );
        std::env::remove_var("LAVERNA_LLAMA_MODEL");
    }

    #[test]
    fn bundled_path_uses_bin_layout() {
        // Without env overrides, resolution points at <repo>/bin/llama/llama
        // relative to the running executable. We only assert it falls back to
        // the bundled layout shape (ends with bin/llama/llama) when no env set.
        std::env::remove_var("LAVERNA_LLAMA_BIN");
        if let Some(p) = Copilot::binary_path() {
            assert!(p.ends_with("bin/llama/llama"));
        }
    }
}
