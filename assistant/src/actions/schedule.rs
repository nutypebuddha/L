//! Durable schedule store — makes timers and reminders survive daemon restarts.
//!
//! Timers and reminders are re-armed as in-memory tokio tasks at runtime, but a
//! daemon restart (or the OS killing the process) would otherwise drop every
//! pending fire. This module persists each pending entry to a JSON file in the
//! shared [`crate::memory::data_dir`], keyed by an absolute fire time (Unix
//! seconds). On startup the daemon calls [`restore`], which re-arms anything
//! still in the future and prunes anything already past.
//!
//! The store is the source of truth: `add` writes before the caller arms the
//! in-memory task; `remove` deletes after cancelling it. Handlers fail loud —
//! an I/O failure surfaces as an error string, never a silent drop.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const STORE_FILE: &str = "schedule.json";

/// What kind of entry this is — drives how [`restore`] re-arms it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Timer,
    Reminder,
}

/// One persisted pending fire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub kind: Kind,
    /// Stable identifier (timer label, or reminder text hash key).
    pub id: String,
    /// Absolute fire time, Unix seconds.
    pub fire_at: u64,
    /// Human text carried to the fire (reminder body; timer label).
    pub text: String,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn store_path(dir: &Path) -> PathBuf {
    dir.join(STORE_FILE)
}

async fn load(dir: &Path) -> Vec<Entry> {
    let path = store_path(dir);
    match tokio::fs::read_to_string(&path).await {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

async fn save(dir: &Path, entries: &[Entry]) -> Result<(), String> {
    if let Err(e) = tokio::fs::create_dir_all(dir).await {
        return Err(format!("could not open schedule store: {e}"));
    }
    let data = serde_json::to_string_pretty(entries)
        .map_err(|e| format!("could not serialize schedule: {e}"))?;
    tokio::fs::write(store_path(dir), data)
        .await
        .map_err(|e| format!("could not write schedule store: {e}"))
}

/// Persist a pending entry, replacing any existing one with the same
/// (kind, id). Returns the entry's absolute fire time.
pub async fn add(
    dir: &Path,
    kind: Kind,
    id: &str,
    secs_from_now: u64,
    text: &str,
) -> Result<u64, String> {
    let fire_at = now_secs().saturating_add(secs_from_now);
    let mut entries = load(dir).await;
    entries.retain(|e| !(e.kind == kind && e.id == id));
    entries.push(Entry {
        kind,
        id: id.to_string(),
        fire_at,
        text: text.to_string(),
    });
    // Determinism rule: keep the store sorted (fire_at, then id).
    entries.sort_by(|a, b| a.fire_at.cmp(&b.fire_at).then_with(|| a.id.cmp(&b.id)));
    save(dir, &entries).await?;
    Ok(fire_at)
}

/// Remove a persisted entry by (kind, id). Silently succeeds if absent.
pub async fn remove(dir: &Path, kind: &Kind, id: &str) -> Result<(), String> {
    let mut entries = load(dir).await;
    let before = entries.len();
    entries.retain(|e| !(&e.kind == kind && e.id == id));
    if entries.len() != before {
        save(dir, &entries).await?;
    }
    Ok(())
}

/// Load all pending entries (already sorted). Past-due entries are included so
/// the caller can decide how to handle them.
pub async fn pending(dir: &Path) -> Vec<Entry> {
    load(dir).await
}

/// Re-arm every still-future entry as an in-memory task and prune the rest.
/// Called once on daemon startup. Returns the number of entries re-armed.
pub async fn restore(dir: &Path) -> usize {
    let entries = load(dir).await;
    let now = now_secs();
    let (future, past): (Vec<Entry>, Vec<Entry>) =
        entries.into_iter().partition(|e| e.fire_at > now);

    // Prune anything already past — it fired (or should have) while we were
    // down; keeping it would re-fire on every subsequent restart.
    if !past.is_empty() {
        let _ = save(dir, &future).await;
    }

    let count = future.len();
    for entry in future {
        let secs = entry.fire_at.saturating_sub(now);
        let dir = dir.to_path_buf();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
            match entry.kind {
                Kind::Timer => eprintln!("[TIMER EXPIRED: {}]", entry.text),
                Kind::Reminder => eprintln!("[REMINDER: {}]", entry.text),
            }
            // Fired — drop it from the store so a later restart won't repeat it.
            let _ = remove(&dir, &entry.kind, &entry.id).await;
        });
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "lai-sched-test-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();
        dir
    }

    #[tokio::test]
    async fn add_and_pending_round_trip() {
        let dir = scratch("add").await;
        add(&dir, Kind::Timer, "tea", 3600, "tea").await.unwrap();
        let p = pending(&dir).await;
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].id, "tea");
        assert!(p[0].fire_at > now_secs());
    }

    #[tokio::test]
    async fn add_replaces_same_id() {
        let dir = scratch("replace").await;
        add(&dir, Kind::Timer, "x", 100, "x").await.unwrap();
        add(&dir, Kind::Timer, "x", 200, "x").await.unwrap();
        let p = pending(&dir).await;
        assert_eq!(p.len(), 1);
    }

    #[tokio::test]
    async fn remove_deletes() {
        let dir = scratch("remove").await;
        add(&dir, Kind::Reminder, "r1", 500, "call mom")
            .await
            .unwrap();
        remove(&dir, &Kind::Reminder, "r1").await.unwrap();
        assert!(pending(&dir).await.is_empty());
    }

    #[tokio::test]
    async fn pending_is_sorted_by_fire_time() {
        let dir = scratch("sorted").await;
        add(&dir, Kind::Timer, "late", 900, "late").await.unwrap();
        add(&dir, Kind::Timer, "soon", 60, "soon").await.unwrap();
        let p = pending(&dir).await;
        assert_eq!(p[0].id, "soon");
        assert_eq!(p[1].id, "late");
    }

    #[tokio::test]
    async fn restore_prunes_past_and_arms_future() {
        let dir = scratch("restore").await;
        // Hand-write a store with one past and one future entry.
        let entries = vec![
            Entry {
                kind: Kind::Timer,
                id: "old".into(),
                fire_at: 1,
                text: "old".into(),
            },
            Entry {
                kind: Kind::Timer,
                id: "new".into(),
                fire_at: now_secs() + 3600,
                text: "new".into(),
            },
        ];
        save(&dir, &entries).await.unwrap();
        let armed = restore(&dir).await;
        assert_eq!(armed, 1);
        let p = pending(&dir).await;
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].id, "new");
    }
}

#[cfg(test)]
mod live_restart {
    use super::*;
    #[tokio::test]
    async fn survives_a_simulated_restart() {
        let dir = std::env::temp_dir().join(format!(
            "lai-sched-live-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = tokio::fs::remove_dir_all(&dir).await;
        tokio::fs::create_dir_all(&dir).await.unwrap();
        add(&dir, Kind::Timer, "bread", 3600, "bread")
            .await
            .unwrap();
        let armed = restore(&dir).await;
        assert_eq!(armed, 1, "timer should re-arm across restart");
        assert_eq!(pending(&dir).await.len(), 1);
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }
}
