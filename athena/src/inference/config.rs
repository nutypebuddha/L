//! # Inference configuration
//!
//! Priority order (highest to lowest):
//! 1. CLI flags (not handled here — passed directly by main.rs)
//! 2. Environment variables (`ATHENA_INFERENCE_*`)
//! 3. Config file (`~/.config/athena/inference.toml` or `./athena.inference.toml`)
//! 4. Built-in defaults

use crate::inference::BackendKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Default model filename for Qwen2.5-0.5B
pub const DEFAULT_MODEL_FILENAME: &str = "qwen2.5-0.5b-instruct-q4_k_m.gguf";

/// Default system prompt for the copilot
pub const DEFAULT_SYSTEM_PROMPT: &str =
    "You are Athena's copilot — a helpful reasoning assistant. \
You assist with formula suggestions, natural language explanations, and routing hints. \
You do NOT perform computation — Athena's symbolic engine handles that. \
Keep responses concise and accurate.";

/// Default endpoint for remote backends
pub const DEFAULT_ENDPOINT_URL: &str = "http://127.0.0.1:8080/v1";

/// The model cache directory: `$XDG_CACHE_HOME/athena/models/`,
/// falling back to `~/.cache/athena/models/`.
pub fn model_cache_dir() -> Option<PathBuf> {
    if let Ok(val) = std::env::var("XDG_CACHE_HOME") {
        if !val.is_empty() {
            return Some(PathBuf::from(val).join("athena").join("models"));
        }
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".cache/athena/models"))
}

/// Model search directories, in priority order:
/// `./models/`, `models/` next to the executable (tarball layout),
/// the XDG cache dir, `/sdcard/Download/athena-export/models/`
/// (Android proot export).
pub fn model_search_dirs() -> Vec<PathBuf> {
    let mut dirs = vec![PathBuf::from("./models/")];
    // Tarball bundle: binary and models/ sit side by side, so the bundled
    // model is found no matter which directory athena is invoked from.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            dirs.push(exe_dir.join("models"));
        }
    }
    if let Some(cache) = model_cache_dir() {
        dirs.push(cache);
    }
    dirs.push(PathBuf::from("/sdcard/Download/athena-export/models/"));
    dirs
}

/// Inference configuration — all fields have sensible defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InferenceConfig {
    /// Which backend to use: "local" or "remote"
    pub backend: BackendKind,

    // ── Local backend ──────────────────────────────────
    /// Path to the GGUF model file
    pub model_path: Option<String>,
    /// Context size in tokens
    pub context_size: usize,
    /// Number of CPU threads (0 = auto)
    pub threads: usize,

    // ── Remote backend ─────────────────────────────────
    /// URL of the OpenAI-compatible API endpoint
    pub endpoint_url: Option<String>,
    /// API key (if required)
    pub api_key: Option<String>,

    // ── Generation parameters ──────────────────────────
    /// Sampling temperature
    pub temperature: f64,
    /// Maximum tokens to generate
    pub max_tokens: usize,
    /// Top-p sampling
    pub top_p: f64,
    /// Stop sequences
    pub stop: Vec<String>,
    /// System prompt for the model
    pub system_prompt: String,

    // ── Model download ─────────────────────────────────
    /// Auto-download model from HuggingFace if not found
    pub auto_download: bool,
    /// HuggingFace repo ID
    pub hf_repo_id: String,
    /// HuggingFace filename
    pub hf_filename: String,
    /// sha256 integrity pin for the model (usually from the capstone
    /// manifest); falls back to the built-in pin for the default model
    pub model_sha256: Option<String>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            backend: BackendKind::Local,
            model_path: None,
            context_size: 2048,
            threads: 0,
            endpoint_url: None,
            api_key: None,
            temperature: 0.7,
            max_tokens: 512,
            top_p: 0.9,
            stop: Vec::new(),
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            // Fallback for slim installs: fetch the pinned Qwen GGUF on first
            // use if no model is found (sha256-verified; see download.rs).
            auto_download: true,
            hf_repo_id: "Qwen/Qwen2.5-0.5B-Instruct-GGUF".to_string(),
            hf_filename: DEFAULT_MODEL_FILENAME.to_string(),
            model_sha256: None,
        }
    }
}

impl InferenceConfig {
    /// Load configuration from all sources, merging with priority:
    /// capstone manifest < config file < env vars < overrides
    pub fn load(overrides: ConfigOverrides) -> Self {
        let mut config = Self::default();

        // 0. Capstone manifest (KB data — model identity + generation defaults)
        if let Some(manifest) = crate::inference::manifest::ModelManifest::discover() {
            config.apply_manifest(&manifest);
        }

        // 1. Try config file
        if let Some(file_config) = Self::from_file() {
            config.merge(file_config);
        }

        // 2. Apply env vars (overrides config file)
        config.apply_env();

        // 3. Apply explicit overrides (highest priority)
        config.apply_overrides(overrides);

        config
    }

    /// Load from a TOML config file.
    /// Searches: `./athena.inference.toml`, `~/.config/athena/inference.toml`
    fn from_file() -> Option<Self> {
        // Current directory
        let local = PathBuf::from("athena.inference.toml");
        if local.exists() {
            let content = std::fs::read_to_string(&local).ok()?;
            let cfg: Self = toml::from_str(&content).ok()?;
            return Some(cfg);
        }

        // ~/.config/athena/inference.toml
        if let Some(config_dir) = dirs_config_dir() {
            let path = config_dir.join("inference.toml");
            if path.exists() {
                let content = std::fs::read_to_string(&path).ok()?;
                let cfg: Self = toml::from_str(&content).ok()?;
                return Some(cfg);
            }
        }

        None
    }

    /// Apply a capstone manifest (model identity + generation defaults).
    fn apply_manifest(&mut self, m: &crate::inference::manifest::ModelManifest) {
        self.hf_filename = m.model.file.clone();
        if let Some(repo) = m.hf_repo_id() {
            self.hf_repo_id = repo.to_string();
        }
        if let Some(sha) = &m.model.sha256 {
            self.model_sha256 = Some(sha.clone());
        }
        if let Some(ctx) = m.model.context {
            self.context_size = ctx;
        }
        if let Some(mt) = m.model.max_tokens {
            self.max_tokens = mt;
        }
        if let Some(prompt) = &m.prompt {
            if let Some(system) = &prompt.system {
                self.system_prompt = system.clone();
            }
        }
    }

    /// Apply environment variable overrides.
    fn apply_env(&mut self) {
        if let Ok(val) = std::env::var("ATHENA_INFERENCE_BACKEND") {
            if let Ok(kind) = val.parse::<BackendKind>() {
                self.backend = kind;
            }
        }
        if let Ok(val) = std::env::var("ATHENA_MODEL_PATH") {
            self.model_path = Some(val);
        }
        if let Ok(val) = std::env::var("ATHENA_ENDPOINT_URL") {
            self.endpoint_url = Some(val);
        }
        if let Ok(val) = std::env::var("ATHENA_API_KEY") {
            self.api_key = Some(val);
        }
        if let Ok(val) = std::env::var("ATHENA_TEMPERATURE") {
            if let Ok(v) = val.parse::<f64>() {
                self.temperature = v;
            }
        }
        if let Ok(val) = std::env::var("ATHENA_MAX_TOKENS") {
            if let Ok(v) = val.parse::<usize>() {
                self.max_tokens = v;
            }
        }
        if let Ok(val) = std::env::var("ATHENA_CONTEXT_SIZE") {
            if let Ok(v) = val.parse::<usize>() {
                self.context_size = v;
            }
        }
        if let Ok(val) = std::env::var("ATHENA_SYSTEM_PROMPT") {
            self.system_prompt = val;
        }
    }

    /// Apply CLI overrides.
    fn apply_overrides(&mut self, overrides: ConfigOverrides) {
        if let Some(backend) = overrides.backend {
            self.backend = backend;
        }
        if let Some(path) = overrides.model_path {
            self.model_path = Some(path);
        }
        if let Some(url) = overrides.endpoint_url {
            self.endpoint_url = Some(url);
        }
        if let Some(temp) = overrides.temperature {
            self.temperature = temp;
        }
        if let Some(tokens) = overrides.max_tokens {
            self.max_tokens = tokens;
        }
    }

    /// Merge another config into self (other takes precedence).
    fn merge(&mut self, other: Self) {
        if other.backend != BackendKind::Local {
            self.backend = other.backend;
        }
        if other.model_path.is_some() {
            self.model_path = other.model_path;
        }
        if other.endpoint_url.is_some() {
            self.endpoint_url = other.endpoint_url;
        }
        if other.api_key.is_some() {
            self.api_key = other.api_key;
        }
        self.temperature = other.temperature;
        if other.max_tokens != 512 {
            self.max_tokens = other.max_tokens;
        }
        if other.context_size != 2048 {
            self.context_size = other.context_size;
        }
        if other.system_prompt != DEFAULT_SYSTEM_PROMPT {
            self.system_prompt = other.system_prompt;
        }
        self.auto_download = other.auto_download;
        if other.hf_repo_id != "Qwen/Qwen2.5-0.5B-Instruct-GGUF" {
            self.hf_repo_id = other.hf_repo_id;
        }
        if other.hf_filename != DEFAULT_MODEL_FILENAME {
            self.hf_filename = other.hf_filename;
        }
        if other.model_sha256.is_some() {
            self.model_sha256 = other.model_sha256;
        }
    }

    /// Resolve the model path by searching in standard locations.
    pub fn resolve_model_path(&self) -> Option<PathBuf> {
        // Explicit path takes priority
        if let Some(path) = &self.model_path {
            let p = shellexpand(path);
            if p.exists() {
                return Some(p);
            }
        }

        // Search standard paths
        let filename = self.hf_filename.as_str();
        for dir in model_search_dirs() {
            let candidate = dir.join(filename);
            if candidate.exists() {
                return Some(candidate);
            }
        }

        None
    }
}

/// CLI overrides that take highest priority.
#[derive(Debug, Default)]
pub struct ConfigOverrides {
    pub backend: Option<BackendKind>,
    pub model_path: Option<String>,
    pub endpoint_url: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<usize>,
}

/// Get the config directory (`~/.config/athena/`).
fn dirs_config_dir() -> Option<PathBuf> {
    // Try XDG_CONFIG_HOME first, fallback to ~/.config
    if let Ok(val) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(val).join("athena");
        if p.exists() || std::fs::create_dir_all(&p).is_ok() {
            return Some(p);
        }
    }
    let home = std::env::var("HOME").ok()?;
    let p = PathBuf::from(&home).join(".config").join("athena");
    if p.exists() || std::fs::create_dir_all(&p).is_ok() {
        return Some(p);
    }
    None
}

/// Simple shell-style tilde expansion.
fn shellexpand(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = InferenceConfig::default();
        assert_eq!(cfg.backend, BackendKind::Local);
        assert_eq!(cfg.temperature, 0.7);
        assert_eq!(cfg.max_tokens, 512);
    }

    #[test]
    fn test_env_overrides() {
        unsafe {
            std::env::set_var("ATHENA_TEMPERATURE", "0.3");
            std::env::set_var("ATHENA_MAX_TOKENS", "1024");
            std::env::set_var("ATHENA_INFERENCE_BACKEND", "remote");
        }
        let mut cfg = InferenceConfig::default();
        cfg.apply_env();
        assert!((cfg.temperature - 0.3).abs() < 1e-6);
        assert_eq!(cfg.max_tokens, 1024);
        assert_eq!(cfg.backend, BackendKind::Remote);

        // Clean up for other tests
        unsafe {
            std::env::remove_var("ATHENA_TEMPERATURE");
            std::env::remove_var("ATHENA_MAX_TOKENS");
            std::env::remove_var("ATHENA_INFERENCE_BACKEND");
        }
    }

    #[test]
    fn test_config_overrides() {
        let mut cfg = InferenceConfig::default();
        let overrides = ConfigOverrides {
            backend: Some(BackendKind::Remote),
            model_path: Some("/tmp/test.gguf".to_string()),
            endpoint_url: Some("http://localhost:9999".to_string()),
            temperature: Some(0.1),
            max_tokens: Some(2048),
        };
        cfg.apply_overrides(overrides);
        assert_eq!(cfg.backend, BackendKind::Remote);
        assert_eq!(cfg.model_path, Some("/tmp/test.gguf".to_string()));
        assert!((cfg.temperature - 0.1).abs() < 1e-6);
        assert_eq!(cfg.max_tokens, 2048);
    }

    #[test]
    fn test_backend_kind_from_str() {
        assert_eq!("local".parse::<BackendKind>().unwrap(), BackendKind::Local);
        assert_eq!(
            "remote".parse::<BackendKind>().unwrap(),
            BackendKind::Remote
        );
        assert!("unknown".parse::<BackendKind>().is_err());
    }

    #[test]
    fn test_model_cache_dir_honors_xdg() {
        unsafe {
            std::env::set_var("XDG_CACHE_HOME", "/custom/cache");
        }
        assert_eq!(
            model_cache_dir(),
            Some(PathBuf::from("/custom/cache/athena/models"))
        );

        // Clean up for other tests
        unsafe {
            std::env::remove_var("XDG_CACHE_HOME");
        }
        if let Ok(home) = std::env::var("HOME") {
            assert_eq!(
                model_cache_dir(),
                Some(PathBuf::from(home).join(".cache/athena/models"))
            );
        }
    }

    #[test]
    fn test_shellexpand() {
        let expanded = shellexpand("~/test/file.toml");
        if let Ok(home) = std::env::var("HOME") {
            assert_eq!(expanded, PathBuf::from(home).join("test/file.toml"));
        }
        let plain = shellexpand("/absolute/path.toml");
        assert_eq!(plain, PathBuf::from("/absolute/path.toml"));
    }
}
