//! # Capstone model manifest — the LLM copilot as KB data
//!
//! Model identity (file, sha256 pin, source repo) and generation defaults
//! live in `capstone/*.toml`, loaded at runtime exactly like `formulas/` and
//! `entities/`. Swapping the capstone model — e.g. for a fine-tuned
//! Athena-Qwen — is a data change, not a code change.
//!
//! Search order mirrors model discovery: `./capstone/`, `capstone/` next to
//! the executable (tarball layout), then `$XDG_CONFIG_HOME/athena/capstone/`.
//! Within a directory, `.toml` files are taken in sorted order and the first
//! parseable manifest wins (multi-model role selection comes later).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A parsed capstone manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelManifest {
    pub model: ManifestModel,
    #[serde(default)]
    pub prompt: Option<ManifestPrompt>,
}

/// The `[model]` table: identity, integrity, generation defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestModel {
    /// Stable identifier for this model entry
    pub id: String,
    /// GGUF filename searched for in the model dirs
    pub file: String,
    /// sha256 integrity pin (hex) — verified on auto-download
    #[serde(default)]
    pub sha256: Option<String>,
    /// Exact file size in bytes (progress reporting)
    #[serde(default)]
    pub size_bytes: Option<u64>,
    /// Source locator, e.g. `hf:Qwen/Qwen2.5-0.5B-Instruct-GGUF`
    #[serde(default)]
    pub source: Option<String>,
    /// What the wheel uses this model for (e.g. "router")
    #[serde(default)]
    pub role: Option<String>,
    /// Context size in tokens
    #[serde(default)]
    pub context: Option<usize>,
    /// Max tokens to generate
    #[serde(default)]
    pub max_tokens: Option<usize>,
}

/// The `[prompt]` table.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestPrompt {
    #[serde(default)]
    pub system: Option<String>,
}

impl ModelManifest {
    /// Parse a manifest from TOML text.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// The HuggingFace repo id, if `source` uses the `hf:` scheme.
    pub fn hf_repo_id(&self) -> Option<&str> {
        self.model.source.as_deref()?.strip_prefix("hf:")
    }

    /// Load the first parseable manifest from the standard search dirs.
    pub fn discover() -> Option<Self> {
        for dir in manifest_search_dirs() {
            if let Some(m) = Self::from_dir(&dir) {
                return Some(m);
            }
        }
        None
    }

    /// Load the first parseable `.toml` (sorted order) in a directory.
    fn from_dir(dir: &Path) -> Option<Self> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .ok()?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        entries.sort();
        for path in entries {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            match Self::from_toml(&text) {
                Ok(m) => return Some(m),
                Err(e) => {
                    eprintln!("Warning: capstone manifest {}: {}", path.display(), e);
                }
            }
        }
        None
    }
}

/// Manifest search directories, in priority order (mirrors model discovery).
pub fn manifest_search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("./capstone/")];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            dirs.push(exe_dir.join("capstone"));
        }
    }
    if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        if !val.is_empty() {
            dirs.push(PathBuf::from(val).join("athena/capstone"));
        }
    } else if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".config/athena/capstone"));
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"
[model]
id = "qwen2.5-0.5b-instruct-q4km"
file = "qwen2.5-0.5b-instruct-q4_k_m.gguf"
sha256 = "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db"
size_bytes = 491400032
source = "hf:Qwen/Qwen2.5-0.5B-Instruct-GGUF"
role = "router"
context = 1024
max_tokens = 64

[prompt]
system = "You are Athena's copilot."
"#;

    #[test]
    fn test_parse_full_manifest() {
        let m = ModelManifest::from_toml(EXAMPLE).unwrap();
        assert_eq!(m.model.id, "qwen2.5-0.5b-instruct-q4km");
        assert_eq!(m.model.file, "qwen2.5-0.5b-instruct-q4_k_m.gguf");
        assert_eq!(m.hf_repo_id(), Some("Qwen/Qwen2.5-0.5B-Instruct-GGUF"));
        assert_eq!(m.model.context, Some(1024));
        assert_eq!(m.model.max_tokens, Some(64));
        assert_eq!(
            m.prompt.unwrap().system.as_deref(),
            Some("You are Athena's copilot.")
        );
    }

    #[test]
    fn test_minimal_manifest() {
        let m = ModelManifest::from_toml("[model]\nid = \"x\"\nfile = \"x.gguf\"\n").unwrap();
        assert_eq!(m.model.sha256, None);
        assert_eq!(m.hf_repo_id(), None);
        assert!(m.prompt.is_none());
    }

    #[test]
    fn test_non_hf_source_yields_no_repo() {
        let m = ModelManifest::from_toml(
            "[model]\nid = \"x\"\nfile = \"x.gguf\"\nsource = \"file:/models\"\n",
        )
        .unwrap();
        assert_eq!(m.hf_repo_id(), None);
    }

    #[test]
    fn test_repo_manifest_parses_and_matches_download_pin() {
        // The checked-in manifest must stay parseable and its pin must match
        // the built-in fallback in download.rs.
        let text = include_str!("../../capstone/athena-qwen.toml");
        let m = ModelManifest::from_toml(text).unwrap();
        assert_eq!(
            m.model.file,
            crate::inference::config::DEFAULT_MODEL_FILENAME
        );
        assert!(m.model.sha256.is_some());
    }
}
