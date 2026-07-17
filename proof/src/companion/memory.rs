//! Companion structured memory (Stage 2 v0.1).
//!
//! The *source of truth* for personalisation. For v0.1 this is a plain
//! serializable value (persisted to JSON by the CLI; v1.0 upgrades to
//! SQLite with the same schema). Memory is NEVER used to fabricate a
//! factual answer — it only stores user-stated facts / preferences /
//! commitments, and personalises tone.
//!
//! Pure by construction: no `static`, no global state. A `CompanionMemory`
//! is created and passed in; load/save are explicit IO in the CLI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CompanionMemory {
    /// User-stated facts (name, role, ...). Each carries its own source
    /// so a recalled fact is auditable, not an anonymous assertion.
    pub facts: Vec<MemoryItem>,
    /// Preferences that personalise tone (units, language, ...).
    pub preferences: Vec<MemoryItem>,
    /// Commitments the user asked the companion to remember.
    pub commitments: Vec<MemoryItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    /// Where the item came from: "stated" (user said it), "verified"
    /// (produced by a Laverna tool call), or "inferred" (never trusted
    /// for factual claims).
    pub source: MemorySource,
    /// ISO-8601 timestamp of when it was stored.
    pub at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemorySource {
    Stated,
    Verified,
    Inferred,
}

impl CompanionMemory {
    /// Pure: empty memory.
    pub fn new() -> Self {
        Self::default()
    }

    /// Pure: upsert a fact by key, preserving the most recent value.
    /// Uses a `BTreeMap`-style scan (deterministic, no HashSet ordering
    /// nondeterminism) so the persisted JSON is stable across runs.
    pub fn set_fact(&mut self, key: &str, value: &str, source: MemorySource, at: &str) {
        if let Some(slot) = self.facts.iter_mut().find(|f| f.key == key) {
            slot.value = value.to_string();
            slot.source = source;
            slot.at = at.to_string();
            return;
        }
        self.facts.push(MemoryItem {
            key: key.to_string(),
            value: value.to_string(),
            source,
            at: at.to_string(),
        });
    }

    /// Pure: look up a fact by key (deterministic first-match).
    pub fn get_fact(&self, key: &str) -> Option<&MemoryItem> {
        self.facts.iter().find(|f| f.key == key)
    }

    /// Pure: only facts whose source is Verified may back a factual claim.
    /// Stated/Inferred items are tone-only and must never be presented
    /// as verified computation.
    pub fn verified_facts(&self) -> Vec<&MemoryItem> {
        self.facts
            .iter()
            .filter(|f| f.source == MemorySource::Verified)
            .collect()
    }

    /// Pure: stable, deterministic serialization order (facts sorted by key).
    /// Sorting avoids the HashMap/Vec-ordering nondeterminism the repo
    /// bans in output paths.
    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        let mut facts = self.facts.clone();
        facts.sort_by(|a, b| a.key.cmp(&b.key));
        let ordered = CompanionMemory {
            facts,
            preferences: self.preferences.clone(),
            commitments: self.commitments.clone(),
        };
        serde_json::to_string_pretty(&ordered)
    }

    /// Upsert a preference (personalises tone); mirrors `set_fact`.
    pub fn set_preference(&mut self, key: &str, value: &str, source: MemorySource, at: &str) {
        if let Some(slot) = self.preferences.iter_mut().find(|p| p.key == key) {
            slot.value = value.to_string();
            slot.source = source;
            slot.at = at.to_string();
            return;
        }
        self.preferences.push(MemoryItem {
            key: key.to_string(),
            value: value.to_string(),
            source,
            at: at.to_string(),
        });
    }

    /// Upsert a commitment the user asked the companion to remember.
    pub fn set_commitment(&mut self, key: &str, value: &str, source: MemorySource, at: &str) {
        if let Some(slot) = self.commitments.iter_mut().find(|c| c.key == key) {
            slot.value = value.to_string();
            slot.source = source;
            slot.at = at.to_string();
            return;
        }
        self.commitments.push(MemoryItem {
            key: key.to_string(),
            value: value.to_string(),
            source,
            at: at.to_string(),
        });
    }

    /// Look up a preference by key.
    pub fn get_preference(&self, key: &str) -> Option<&MemoryItem> {
        self.preferences.iter().find(|p| p.key == key)
    }

    /// Look up a commitment by key.
    pub fn get_commitment(&self, key: &str) -> Option<&MemoryItem> {
        self.commitments.iter().find(|c| c.key == key)
    }

    /// Persist to `path` as pretty JSON. Explicit IO (no global state); the
    /// caller owns the path. Creates parent directories if needed.
    /// Errors are returned, never silently swallowed.
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("memory dir create error: {e}"))?;
        }
        let json = self
            .to_canonical_json()
            .map_err(|e| format!("memory serialize error: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("memory write error: {e}"))
    }

    /// Load from `path`. A missing file is NOT an error — it means fresh
    /// memory (the companion starts with none). Corrupt JSON is an error.
    pub fn load_from_path(path: &std::path::Path) -> Result<CompanionMemory, String> {
        if !path.exists() {
            return Ok(CompanionMemory::new());
        }
        let raw = std::fs::read_to_string(path).map_err(|e| format!("memory read error: {e}"))?;
        serde_json::from_str(&raw).map_err(|e| format!("memory parse error: {e}"))
    }
}

/// Default on-disk location: `<data_dir>/laverna/companion-memory.json`,
/// where `data_dir` is the OS user-data dir (XDG on Linux, the app sandbox
/// on Android, `~/Library/Application Support` on macOS). Deterministic and
/// user-scoped; never a hardcoded absolute path.
pub fn default_path() -> std::path::PathBuf {
    let mut dir = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from(".laverna"));
    dir.push("laverna");
    dir.push("companion-memory.json");
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_is_idempotent_and_deterministic() {
        let mut m = CompanionMemory::new();
        m.set_fact("name", "ada", MemorySource::Stated, "2026-07-16T00:00:00Z");
        m.set_fact(
            "name",
            "ada l.",
            MemorySource::Stated,
            "2026-07-16T01:00:00Z",
        );
        assert_eq!(m.facts.len(), 1);
        assert_eq!(m.get_fact("name").unwrap().value, "ada l.");
    }

    #[test]
    fn verified_facts_filter() {
        let mut m = CompanionMemory::new();
        m.set_fact("a", "1", MemorySource::Verified, "t");
        m.set_fact("b", "2", MemorySource::Stated, "t");
        assert_eq!(m.verified_facts().len(), 1);
    }

    #[test]
    fn canonical_json_is_stable() {
        let mut m = CompanionMemory::new();
        m.set_fact("zeta", "z", MemorySource::Stated, "t");
        m.set_fact("alpha", "a", MemorySource::Verified, "t");
        let j = m.to_canonical_json().unwrap();
        // alpha (sorted first) must precede zeta in the serialized form.
        assert!(j.find("alpha").unwrap() < j.find("zeta").unwrap());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("companion-memory.json");

        let mut m = CompanionMemory::new();
        m.set_fact("name", "ada", MemorySource::Stated, "2026-07-16T00:00:00Z");
        m.set_preference(
            "units",
            "metric",
            MemorySource::Stated,
            "2026-07-16T00:00:00Z",
        );
        m.set_commitment(
            "weekly_review",
            "sunday",
            MemorySource::Stated,
            "2026-07-16T00:00:00Z",
        );
        m.save_to_path(&path).expect("save");

        let loaded = CompanionMemory::load_from_path(&path).expect("load");
        assert_eq!(loaded, m, "round-tripped memory must equal original");
        assert_eq!(loaded.get_fact("name").unwrap().value, "ada");
        assert_eq!(loaded.get_preference("units").unwrap().value, "metric");
        assert_eq!(
            loaded.get_commitment("weekly_review").unwrap().value,
            "sunday"
        );
    }

    #[test]
    fn load_missing_file_is_fresh_memory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("does-not-exist.json");
        let m = CompanionMemory::load_from_path(&path).expect("missing file -> fresh");
        assert!(m.facts.is_empty());
        assert!(m.preferences.is_empty());
        assert!(m.commitments.is_empty());
    }

    #[test]
    fn load_corrupt_file_errors() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("corrupt.json");
        std::fs::write(&path, "{not valid json").unwrap();
        assert!(CompanionMemory::load_from_path(&path).is_err());
    }

    #[test]
    fn default_path_is_user_scoped() {
        let p = default_path();
        let parts: Vec<_> = p.iter().map(|c| c.to_string_lossy()).collect();
        assert!(
            parts.ends_with(&["laverna".into(), "companion-memory.json".into()]),
            "default path should end in laverna/companion-memory.json, got {}",
            p.display()
        );
    }
}
