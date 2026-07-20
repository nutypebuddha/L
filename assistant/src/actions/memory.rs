//! Agentic memory tools — let the model persist and recall durable user facts.
//!
//! These operate on the same on-disk `preferences.toml` that [`crate::memory::Memory`]
//! owns, via [`Preferences::load`]/[`Preferences::save`] against the shared
//! [`crate::memory::data_dir`]. Keeping a single store means a fact the agent
//! writes with `remember` is the same fact injected into the next prompt's
//! memory block — no divergence between the runtime cache and disk.
//!
//! All three handlers fail loud: on any I/O error they return a human-readable
//! string rather than propagating, matching the rest of the action surface.

use crate::memory::{data_dir, preferences::Preferences};
use std::path::Path;

async fn load(dir: &Path) -> Result<Preferences, String> {
    if let Err(e) = tokio::fs::create_dir_all(dir).await {
        return Err(format!("could not open memory store: {e}"));
    }
    Preferences::load(dir)
        .await
        .map_err(|e| format!("could not read memory store: {e}"))
}

/// Persist a durable fact under `key`. Overwrites any existing value.
pub async fn remember(key: &str, value: &str) -> String {
    remember_in(&data_dir(), key, value).await
}

/// Recall a stored fact by `key`, or list all facts when `key` is empty.
pub async fn recall(key: &str) -> String {
    recall_in(&data_dir(), key).await
}

/// Delete a stored fact by `key`.
pub async fn forget(key: &str) -> String {
    forget_in(&data_dir(), key).await
}

async fn remember_in(dir: &Path, key: &str, value: &str) -> String {
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() || value.is_empty() {
        return "remember needs a non-empty key and value".to_string();
    }
    let mut prefs = match load(dir).await {
        Ok(p) => p,
        Err(e) => return e,
    };
    prefs.facts.insert(key.to_string(), value.to_string());
    match prefs.save(dir).await {
        Ok(()) => format!("Remembered: {key} = {value}"),
        Err(e) => format!("could not save memory: {e}"),
    }
}

async fn recall_in(dir: &Path, key: &str) -> String {
    let key = key.trim();
    let prefs = match load(dir).await {
        Ok(p) => p,
        Err(e) => return e,
    };
    if key.is_empty() {
        if prefs.facts.is_empty() {
            return "No facts remembered yet.".to_string();
        }
        // Determinism rule: sort keys before listing.
        let mut facts: Vec<(&String, &String)> = prefs.facts.iter().collect();
        facts.sort_by(|a, b| a.0.cmp(b.0));
        let body: Vec<String> = facts.iter().map(|(k, v)| format!("- {k}: {v}")).collect();
        return body.join("\n");
    }
    match prefs.facts.get(key) {
        Some(v) => format!("{key}: {v}"),
        None => format!("Nothing remembered for '{key}'."),
    }
}

async fn forget_in(dir: &Path, key: &str) -> String {
    let key = key.trim();
    if key.is_empty() {
        return "forget needs a key".to_string();
    }
    let mut prefs = match load(dir).await {
        Ok(p) => p,
        Err(e) => return e,
    };
    if prefs.facts.remove(key).is_none() {
        return format!("Nothing remembered for '{key}'.");
    }
    match prefs.save(dir).await {
        Ok(()) => format!("Forgot: {key}"),
        Err(e) => format!("could not save memory: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A unique scratch directory per test — no shared global state, so tests
    /// run safely in parallel against the `*_in` core functions.
    async fn scratch(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "lai-mem-test-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();
        dir
    }

    #[tokio::test]
    async fn remember_rejects_empty() {
        let dir = scratch("empty").await;
        assert!(remember_in(&dir, "", "x").await.contains("non-empty"));
        assert!(remember_in(&dir, "k", "").await.contains("non-empty"));
    }

    #[tokio::test]
    async fn round_trip() {
        let dir = scratch("roundtrip").await;
        assert!(remember_in(&dir, "city", "Berlin").await.contains("Berlin"));
        assert!(recall_in(&dir, "city").await.contains("Berlin"));
        assert!(forget_in(&dir, "city").await.contains("Forgot"));
        assert!(recall_in(&dir, "city").await.contains("Nothing remembered"));
    }

    #[tokio::test]
    async fn recall_all_is_sorted() {
        let dir = scratch("sorted").await;
        remember_in(&dir, "zebra", "1").await;
        remember_in(&dir, "apple", "2").await;
        let all = recall_in(&dir, "").await;
        let a = all.find("apple").unwrap();
        let z = all.find("zebra").unwrap();
        assert!(a < z, "expected apple before zebra: {all}");
    }

    #[tokio::test]
    async fn forget_missing_is_reported() {
        let dir = scratch("missing").await;
        assert!(forget_in(&dir, "nope").await.contains("Nothing remembered"));
    }
}
