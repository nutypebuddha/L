//! # Model auto-download — sha256-pinned GGUF fetch from HuggingFace
//!
//! Fallback for slim installs: when no model is found in the search paths
//! and `auto_download` is enabled, fetch the pinned Qwen GGUF into the
//! model cache dir (`$XDG_CACHE_HOME/athena/models/`), verify its sha256,
//! and hand the path to the local backend.
//!
//! Tarball installs never hit this path — the bundled `models/` dir next to
//! the binary is found first by `resolve_model_path`.

use crate::inference::config::{model_cache_dir, InferenceConfig, DEFAULT_MODEL_FILENAME};
use crate::inference::InferenceError;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;

/// sha256 of the pinned default model (HuggingFace LFS oid for
/// `qwen2.5-0.5b-instruct-q4_k_m.gguf` in `Qwen/Qwen2.5-0.5B-Instruct-GGUF`).
pub const DEFAULT_MODEL_SHA256: &str =
    "74a4da8c9fdbcd15bd1f6d01d621410d31c6fc00986f5eb687824e7b93d7a9db";

/// Exact size of the pinned default model, for progress reporting.
pub const DEFAULT_MODEL_SIZE: u64 = 491_400_032;

/// The default HuggingFace repo the pin applies to.
pub const DEFAULT_HF_REPO: &str = "Qwen/Qwen2.5-0.5B-Instruct-GGUF";

/// Resolve the model path, downloading it if allowed and necessary.
///
/// Order: existing file in search paths → auto-download (if enabled) → error.
pub fn ensure_model(config: &InferenceConfig) -> Result<PathBuf, InferenceError> {
    if let Some(path) = config.resolve_model_path() {
        return Ok(path);
    }
    if !config.auto_download {
        return Err(InferenceError::ModelNotLoaded(
            "no model file found and auto_download is disabled. \
             Set ATHENA_MODEL_PATH, place the GGUF in ./models/ or \
             the model cache dir, or enable auto_download"
                .to_string(),
        ));
    }
    download_model(config)
}

/// The HuggingFace resolve URL for the configured model.
pub fn hf_url(config: &InferenceConfig) -> String {
    format!(
        "https://huggingface.co/{}/resolve/main/{}",
        config.hf_repo_id, config.hf_filename
    )
}

/// The pinned sha256 for the configured model, if we know one.
///
/// A pin supplied by config (usually the capstone manifest) wins; the
/// built-in constant covers the default repo+filename pair. Models with
/// neither download unverified (with a warning).
pub fn pinned_sha256(config: &InferenceConfig) -> Option<String> {
    if let Some(sha) = &config.model_sha256 {
        return Some(sha.clone());
    }
    (config.hf_repo_id == DEFAULT_HF_REPO && config.hf_filename == DEFAULT_MODEL_FILENAME)
        .then(|| DEFAULT_MODEL_SHA256.to_string())
}

/// Download the configured model into the cache dir, verifying sha256
/// when a pin is known. Writes to a `.part` file and renames on success,
/// so an interrupted download never masquerades as a model.
fn download_model(config: &InferenceConfig) -> Result<PathBuf, InferenceError> {
    let dir = model_cache_dir().ok_or_else(|| {
        InferenceError::ConfigError("cannot determine model cache dir (HOME unset)".to_string())
    })?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| InferenceError::ConfigError(format!("create {}: {}", dir.display(), e)))?;

    let dest = dir.join(&config.hf_filename);
    let part = dest.with_extension("gguf.part");
    let url = hf_url(config);
    let pin = pinned_sha256(config);

    eprintln!(
        "Athena: downloading model {} (~{} MB) from {}",
        config.hf_filename,
        DEFAULT_MODEL_SIZE / 1_000_000,
        url
    );
    if pin.is_none() {
        eprintln!("Athena: WARNING — no sha256 pin for custom model; skipping verification");
    }

    let resp = ureq::get(&url)
        .call()
        .map_err(|e| InferenceError::BackendUnavailable(format!("model download failed: {}", e)))?;
    let total: u64 = resp
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MODEL_SIZE);

    let mut reader = resp.into_body().into_reader();
    let mut file = std::fs::File::create(&part)
        .map_err(|e| InferenceError::ConfigError(format!("create {}: {}", part.display(), e)))?;

    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20]; // 1 MiB
    let mut written: u64 = 0;
    let mut last_report: u64 = 0;
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| InferenceError::BackendUnavailable(format!("download read: {}", e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        file.write_all(&buf[..n])
            .map_err(|e| InferenceError::ConfigError(format!("download write: {}", e)))?;
        written += n as u64;
        if written - last_report >= 50_000_000 {
            eprintln!(
                "Athena: ... {} / {} MB",
                written / 1_000_000,
                total / 1_000_000
            );
            last_report = written;
        }
    }
    file.flush()
        .map_err(|e| InferenceError::ConfigError(format!("download flush: {}", e)))?;
    drop(file);

    if let Some(expected) = &pin {
        let got = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        if &got != expected {
            let _ = std::fs::remove_file(&part);
            return Err(InferenceError::ConfigError(format!(
                "sha256 mismatch for {}: expected {}, got {} — download removed",
                config.hf_filename, expected, got
            )));
        }
        eprintln!("Athena: sha256 verified ({}...)", &expected[..12]);
    }

    std::fs::rename(&part, &dest)
        .map_err(|e| InferenceError::ConfigError(format!("finalize model file: {}", e)))?;
    eprintln!("Athena: model cached at {}", dest.display());
    Ok(dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hf_url_default() {
        let cfg = InferenceConfig::default();
        assert_eq!(
            hf_url(&cfg),
            "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF\
             /resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf"
        );
    }

    #[test]
    fn test_pin_applies_only_to_default_model() {
        let cfg = InferenceConfig::default();
        assert_eq!(pinned_sha256(&cfg), Some(DEFAULT_MODEL_SHA256.to_string()));

        let mut custom = InferenceConfig::default();
        custom.hf_filename = "other-model-q8_0.gguf".to_string();
        assert_eq!(pinned_sha256(&custom), None);
    }

    #[test]
    fn test_manifest_pin_wins_over_builtin() {
        let mut cfg = InferenceConfig::default();
        cfg.model_sha256 = Some("abc123".to_string());
        assert_eq!(pinned_sha256(&cfg), Some("abc123".to_string()));

        // A manifest pin makes custom models verifiable too
        let mut custom = InferenceConfig::default();
        custom.hf_filename = "athena-qwen-finetuned-q4.gguf".to_string();
        custom.model_sha256 = Some("def456".to_string());
        assert_eq!(pinned_sha256(&custom), Some("def456".to_string()));
    }
}
