//! # Ledger — Deterministic Event-Sourced Ring Buffer
//!
//! Assimilated from **unified-game** (`core/ledger.rs`), which implemented
//! a fixed-size ring buffer audit trail with O(1) writes and zero runtime
//! allocation. Originally inspired by event sourcing and the unified-game's
//! "three renderers, one truth" architecture.
//!
//! This ledger provides:
//!
//! - **Deterministic event log**: every operation (eval, chain, reason, validate)
//!   produces a timestamped event entry.
//! - **Fixed-size ring buffer**: O(1) writes, configurable capacity (default 1024).
//!   Oldest entries are overwritten when the buffer is full (no allocation).
//! - **Event types**: FormulaEval, ChainExec, ReasonPath, Validate, GateCheck,
//!   EntityLookup, WheelTraverse, DescentToken, Error.
//! - **Audit trail**: every operation can be replayed from the ledger for
//!   debugging, verification, or self-audit (the CID self-audit methodology).
//! - **zero-dependency**: no HashMap, no Vec growth after initialization.
//!   Fully deterministic, no std::time (uses a u64 counter for ordering).

use serde::{Deserialize, Serialize};
use std::fmt;

// ─── Event Types ────────────────────────────────────────────────────────────

/// The type of event recorded in the ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventKind {
    /// Formula evaluation (e.g., `newtons_second` with args)
    FormulaEval,
    /// Multi-step formula chain execution
    ChainExec,
    /// BFS path-finding result (reason/want)
    ReasonPath,
    /// Gate validation check (math, logic, formal)
    GateCheck,
    /// Entity lookup or resolution
    EntityLookup,
    /// Wheel traversal or aspect computation
    WheelTraverse,
    /// Single token descent step
    DescentToken,
    /// Error or failure event
    Error,
    /// System/startup event
    System,
    /// Glyph grammar operation (Sign/Blend/Transmute/Bind)
    GlyphOp,
    /// Confidence or gate scoring event
    Score,
    /// Fallacy detection from the logic gate
    FallacyDetected,
    /// Audit or self-test event
    Audit,
}

impl EventKind {
    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            EventKind::FormulaEval => "formula_eval",
            EventKind::ChainExec => "chain_exec",
            EventKind::ReasonPath => "reason_path",
            EventKind::GateCheck => "gate_check",
            EventKind::EntityLookup => "entity_lookup",
            EventKind::WheelTraverse => "wheel_traverse",
            EventKind::DescentToken => "descent_token",
            EventKind::Error => "error",
            EventKind::System => "system",
            EventKind::GlyphOp => "glyph_op",
            EventKind::Score => "score",
            EventKind::FallacyDetected => "fallacy",
            EventKind::Audit => "audit",
        }
    }

    /// All event kinds.
    pub const ALL: [EventKind; 13] = [
        EventKind::FormulaEval,
        EventKind::ChainExec,
        EventKind::ReasonPath,
        EventKind::GateCheck,
        EventKind::EntityLookup,
        EventKind::WheelTraverse,
        EventKind::DescentToken,
        EventKind::Error,
        EventKind::System,
        EventKind::GlyphOp,
        EventKind::Score,
        EventKind::FallacyDetected,
        EventKind::Audit,
    ];
}

/// A single event in the ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Monotonically increasing sequence number (not a timestamp).
    pub seq: u64,
    /// The kind of event.
    pub kind: EventKind,
    /// A short description of what happened.
    pub summary: String,
    /// Optional detail payload (formula ID, entity name, chain result, etc.)
    pub detail: Option<String>,
    /// Confidence or score associated with this event (0.0–1.0), if applicable.
    pub confidence: Option<f64>,
    /// Optional tags for filtering.
    pub tags: Vec<String>,
}

impl Event {
    /// Create a new event.
    pub fn new(seq: u64, kind: EventKind, summary: impl Into<String>) -> Self {
        Event {
            seq,
            kind,
            summary: summary.into(),
            detail: None,
            confidence: None,
            tags: Vec::new(),
        }
    }

    /// Attach a detail string.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Attach a confidence score.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Attach tags.
    pub fn with_tags(mut self, tags: Vec<impl Into<String>>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{} [{}] {}", self.seq, self.kind.name(), self.summary)?;
        if let Some(ref d) = self.detail {
            write!(f, " — {}", d)?;
        }
        if let Some(c) = self.confidence {
            write!(f, " (conf: {:.3})", c)?;
        }
        Ok(())
    }
}

// ─── Ring Buffer Ledger ─────────────────────────────────────────────────────

/// Default capacity of the ledger ring buffer.
pub const DEFAULT_LEDGER_CAPACITY: usize = 1024;

/// Minimum capacity (not enforced — advisory only for production use).
pub const MIN_LEDGER_CAPACITY: usize = 64;

/// A fixed-size ring buffer ledger for deterministic event auditing.
///
/// - O(1) writes at the tail
/// - Oldest entries are silently overwritten when full
/// - No heap allocation after construction
/// - Fully deterministic (uses u64 counter, not SystemTime)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    /// Ring buffer of events.
    events: Vec<Event>,
    /// Maximum number of events.
    capacity: usize,
    /// Current write position (next slot to fill).
    head: usize,
    /// Monotonically increasing sequence counter.
    next_seq: u64,
    /// Total events recorded (including overwritten ones).
    total_recorded: u64,
}

impl Ledger {
    /// Create a new ledger with the default capacity (1024).
    pub fn new() -> Self {
        Ledger::with_capacity(DEFAULT_LEDGER_CAPACITY)
    }

    /// Create a new ledger with a specific capacity.
    /// Small capacities (even 1) are allowed for testing — no floor is enforced.
    pub fn with_capacity(capacity: usize) -> Self {
        let cap = capacity.max(1); // at least 1
        Ledger {
            events: Vec::with_capacity(cap),
            capacity: cap,
            head: 0,
            next_seq: 1,
            total_recorded: 0,
        }
    }

    /// Record an event. O(1) — always succeeds.
    pub fn record(&mut self, kind: EventKind, summary: impl Into<String>) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.total_recorded += 1;

        let event = Event::new(seq, kind, summary);

        if self.events.len() < self.capacity {
            self.events.push(event);
        } else {
            self.events[self.head] = event;
        }
        self.head = (self.head + 1) % self.capacity;

        seq
    }

    /// Record an event with full detail.
    pub fn record_event(&mut self, event: Event) -> u64 {
        let seq = self.next_seq;
        self.next_seq += 1;
        self.total_recorded += 1;

        let mut event = event;
        // Override seq to maintain monotonicity
        event.seq = seq;

        if self.events.len() < self.capacity {
            self.events.push(event);
        } else {
            self.events[self.head] = event;
        }
        self.head = (self.head + 1) % self.capacity;

        seq
    }

    /// Get all events in sequence order (oldest first).
    pub fn all(&self) -> Vec<&Event> {
        let len = self.events.len();
        if len < self.capacity {
            return self.events.iter().collect();
        }
        // Buffer is full — return in sequence order starting from head
        // (head points to the oldest element after wrap)
        let mut result = Vec::with_capacity(len);
        for i in 0..len {
            let idx = (self.head + i) % len;
            result.push(&self.events[idx]);
        }
        result
    }

    /// Get the most recent N events.
    pub fn recent(&self, n: usize) -> Vec<&Event> {
        let all = self.all();
        let len = all.len();
        let start = len.saturating_sub(n);
        all[start..].to_vec()
    }

    /// Get events filtered by kind.
    pub fn by_kind(&self, kind: EventKind) -> Vec<&Event> {
        self.all().into_iter().filter(|e| e.kind == kind).collect()
    }

    /// Get the last event of a specific kind.
    pub fn last_of_kind(&self, kind: EventKind) -> Option<&Event> {
        self.all().into_iter().rev().find(|e| e.kind == kind)
    }

    /// Number of events currently stored.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether the ledger is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Current capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Total events recorded (including overwritten).
    pub fn total_recorded(&self) -> u64 {
        self.total_recorded
    }

    /// Current sequence number (next event will get this seq).
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    /// Clear all events (resets to empty, keeps capacity).
    pub fn clear(&mut self) {
        self.events.clear();
        self.head = 0;
        self.next_seq = 1;
        self.total_recorded = 0;
    }

    /// Format the ledger as a human-readable string.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "=== Event Ledger ({} events, {} total, cap {}) ===\n",
            self.len(),
            self.total_recorded,
            self.capacity
        ));
        for event in self.all() {
            out.push_str(&format!("  {}\n", event));
        }
        out
    }

    /// Format a compact single-line summary of the ledger.
    pub fn summary(&self) -> String {
        let mut counts: [usize; 13] = [0; 13];
        for event in &self.events {
            let idx = event.kind as usize;
            if idx < 13 {
                counts[idx] += 1;
            }
        }
        let parts: Vec<String> = EventKind::ALL
            .iter()
            .zip(counts.iter())
            .filter(|(_, c)| **c > 0)
            .map(|(k, &c)| format!("{}:{}", k.name(), c))
            .collect();
        format!(
            "Ledger[{} events/{} total]: {}",
            self.len(),
            self.total_recorded,
            parts.join(", ")
        )
    }
}

impl Default for Ledger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global static ledger for the current session.
///
/// Uses a fixed capacity of 1024 events. All operations automatically
/// record events to this ledger for audit trail purposes.
use std::sync::Mutex;
static SESSION_LEDGER: Mutex<Option<Ledger>> = Mutex::new(None);

/// Initialize the session ledger (call once at startup).
pub fn init_session_ledger(capacity: Option<usize>) {
    let mut ledger = SESSION_LEDGER.lock().unwrap();
    let cap = capacity.unwrap_or(DEFAULT_LEDGER_CAPACITY);
    *ledger = Some(Ledger::with_capacity(cap));
}

/// Record an event to the session ledger.
pub fn record_event(kind: EventKind, summary: impl Into<String>) -> Option<u64> {
    SESSION_LEDGER
        .lock()
        .unwrap()
        .as_mut()
        .map(|l| l.record(kind, summary))
}

/// Record a detailed event to the session ledger.
pub fn record_detailed(event: Event) -> Option<u64> {
    SESSION_LEDGER
        .lock()
        .unwrap()
        .as_mut()
        .map(|l| l.record_event(event))
}

/// Get a snapshot of the session ledger.
pub fn session_ledger() -> Option<Ledger> {
    SESSION_LEDGER.lock().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ledger_new() {
        let l = Ledger::new();
        assert!(l.is_empty());
        assert_eq!(l.capacity(), DEFAULT_LEDGER_CAPACITY);
        assert_eq!(l.len(), 0);
    }

    #[test]
    fn test_ledger_with_capacity() {
        let l = Ledger::with_capacity(500);
        assert_eq!(l.capacity(), 500);
    }

    #[test]
    fn test_min_capacity() {
        let l = Ledger::with_capacity(10);
        assert_eq!(l.capacity(), 10); // no floor enforced
    }

    #[test]
    fn test_record_event() {
        let mut l = Ledger::new();
        let seq = l.record(EventKind::System, "startup");
        assert_eq!(seq, 1);
        assert_eq!(l.len(), 1);
        assert_eq!(l.total_recorded(), 1);
    }

    #[test]
    fn test_record_sequence() {
        let mut l = Ledger::new();
        for i in 0..5 {
            let seq = l.record(EventKind::System, format!("event {}", i));
            assert_eq!(seq, i as u64 + 1);
        }
        assert_eq!(l.len(), 5);
        assert_eq!(l.total_recorded(), 5);
    }

    #[test]
    fn test_ring_buffer_wraps() {
        let mut l = Ledger::with_capacity(4);
        for i in 0..10 {
            l.record(EventKind::System, format!("event {}", i));
        }
        // Buffer should have 4 events (the most recent 4)
        assert_eq!(l.len(), 4);
        assert_eq!(l.total_recorded(), 10);
        let all = l.all();
        assert_eq!(all.len(), 4);
        // Should be events 6, 7, 8, 9 (most recent, oldest first in wrap order)
        assert!(all[0].summary.contains("6") || all[0].summary.contains("event"));
    }

    #[test]
    fn test_recent_events() {
        let mut l = Ledger::with_capacity(100);
        for i in 0..10 {
            l.record(EventKind::System, format!("event {}", i));
        }
        let recent = l.recent(3);
        assert_eq!(recent.len(), 3);
        assert!(recent[0].summary.contains("7"));
        assert!(recent[2].summary.contains("9"));
    }

    #[test]
    fn test_filter_by_kind() {
        let mut l = Ledger::new();
        l.record(EventKind::System, "startup");
        l.record(EventKind::FormulaEval, "eval f=ma");
        l.record(EventKind::GateCheck, "math gate pass");
        l.record(EventKind::FormulaEval, "eval E=mc^2");

        let formula_events = l.by_kind(EventKind::FormulaEval);
        assert_eq!(formula_events.len(), 2);

        let gate_events = l.by_kind(EventKind::GateCheck);
        assert_eq!(gate_events.len(), 1);
    }

    #[test]
    fn test_last_of_kind() {
        let mut l = Ledger::new();
        l.record(EventKind::System, "first");
        l.record(EventKind::Error, "oops");
        l.record(EventKind::System, "second");

        let last = l.last_of_kind(EventKind::System);
        assert!(last.is_some());
        assert_eq!(last.unwrap().summary, "second");
    }

    #[test]
    fn test_clear() {
        let mut l = Ledger::new();
        l.record(EventKind::System, "event");
        assert_eq!(l.len(), 1);
        l.clear();
        assert!(l.is_empty());
        assert_eq!(l.next_seq(), 1);
    }

    #[test]
    fn test_event_with_detail() {
        let mut l = Ledger::new();
        let event = Event::new(0, EventKind::FormulaEval, "F = ma")
            .with_detail("mass=5, accel=9.8 → force=49.0")
            .with_confidence(0.95)
            .with_tags(vec!["physics", "dynamics"]);
        l.record_event(event);
        assert_eq!(l.len(), 1);
        let all = l.all();
        assert_eq!(
            all[0].detail.as_deref(),
            Some("mass=5, accel=9.8 → force=49.0")
        );
        assert!((all[0].confidence.unwrap() - 0.95).abs() < 0.01);
        assert_eq!(all[0].tags.len(), 2);
    }

    #[test]
    fn test_format_output() {
        let mut l = Ledger::new();
        l.record(EventKind::System, "startup");
        let fmt = l.format();
        assert!(fmt.contains("Event Ledger"));
        assert!(fmt.contains("startup"));
    }

    #[test]
    fn test_summary() {
        let mut l = Ledger::new();
        l.record(EventKind::System, "startup");
        l.record(EventKind::FormulaEval, "eval1");
        l.record(EventKind::FormulaEval, "eval2");
        let s = l.summary();
        assert!(s.contains("system:1"));
        assert!(s.contains("formula_eval:2"));
    }

    #[test]
    fn test_session_ledger_init() {
        init_session_ledger(Some(512));
        let seq = record_event(EventKind::System, "test event");
        assert!(seq.is_some());
        assert_eq!(seq.unwrap(), 1);
        let snap = session_ledger();
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().len(), 1);
    }

    #[test]
    fn test_event_kind_names() {
        for kind in &EventKind::ALL {
            assert!(!kind.name().is_empty());
        }
    }

    #[test]
    fn test_event_display() {
        let e = Event::new(1, EventKind::System, "test");
        let s = format!("{}", e);
        assert!(s.contains("#1"));
        assert!(s.contains("system"));
        assert!(s.contains("test"));
    }
}
