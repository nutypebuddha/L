// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0

//! L 0.5 strategy-engine foundation types (L Elevation Directive).
//!
//! These are the explicit, machine-readable models the deterministic core was
//! missing: a persistent [`WorldState`] with per-fact [`EpistemicStatus`],
//! [`EntityResolution`] (semantic disambiguation), an [`Evidence`] ledger,
//! [`StateDelta`] (what changed), and a structured [`Strategy`]. They are
//! intentionally plain data + serde so every stage of the strategy pipeline can
//! be independently tested and the proof trail stays recomputable.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Epistemic status of a fact/claim. Never collapse these into a single
/// `known` boolean — L must distinguish "I proved this" from "I believe this"
/// from "this was inferred" from "I do not know".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicStatus {
    Verified,
    Derived,
    Observed,
    CorpusBacked,
    ExternallySourced,
    Inferred,
    Assumed,
    Unknown,
    Contradicted,
    Stale,
}

impl EpistemicStatus {
    /// Stable, human-readable label (used in CLI text output).
    pub fn label(&self) -> &'static str {
        match self {
            EpistemicStatus::Verified => "verified",
            EpistemicStatus::Derived => "derived",
            EpistemicStatus::Observed => "observed",
            EpistemicStatus::CorpusBacked => "corpus_backed",
            EpistemicStatus::ExternallySourced => "externally_sourced",
            EpistemicStatus::Inferred => "inferred",
            EpistemicStatus::Assumed => "assumed",
            EpistemicStatus::Unknown => "unknown",
            EpistemicStatus::Contradicted => "contradicted",
            EpistemicStatus::Stale => "stale",
        }
    }
}

impl std::fmt::Display for EpistemicStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A single fact in the world model, carrying its own epistemic status and the
/// evidence that supports it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    pub statement: String,
    pub status: EpistemicStatus,
    pub evidence_refs: Vec<String>,
}

/// Resolution of a raw mention into a canonical concept, with explicit
/// confidence and alternatives. A route is not a fact: low confidence must be
/// reported, never silently upgraded to an authoritative fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityResolution {
    /// Original surface mention as it appeared in the text.
    pub mention: String,
    /// Canonical name in the knowledge base (or the mention if unresolved).
    pub canonical_name: String,
    /// Type of the resolved entity (e.g. "seed_entity", "concept", "unresolved").
    pub entity_type: String,
    /// Domain the entity resolved into, if known.
    pub domain: Option<String>,
    /// Semantic resolution confidence in [0, 1].
    pub confidence: f64,
    /// Short evidence string for why this resolution was chosen.
    pub evidence: String,
    /// Other candidate canonical names considered.
    pub alternatives: Vec<String>,
}

/// A provenance record for a claim entering or leaving the strategy engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub claim: String,
    pub source: String,
    /// Where the claim came from: corpus / formula / user_input / observation /
    /// external_source / derived / inference / assumption.
    pub source_type: String,
    pub timestamp: Option<String>,
    pub confidence: f64,
    pub epistemic_status: EpistemicStatus,
    pub dependencies: Vec<String>,
}

/// "Unknown" as an actionable object, not a dead end.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unknown {
    pub question: String,
    pub impact: String,
    pub urgency: String,
    pub strategies_affected: usize,
    pub information_gain: String,
}

/// The persistent model of what L currently believes is true. Order-sensitive
/// collections use `BTreeMap`/`Vec` so serialization is deterministic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    /// Monotonic version, bumped on every state delta.
    pub version: u64,
    /// Resolved entity id -> canonical name.
    pub entities: BTreeMap<String, String>,
    /// Free-form concepts mentioned but not yet entity-resolved.
    pub concepts: Vec<String>,
    pub facts: Vec<Fact>,
    pub assumptions: Vec<String>,
    pub observations: Vec<String>,
    pub constraints: Vec<String>,
    /// Named resources and their available amounts.
    pub resources: BTreeMap<String, f64>,
    pub goals: Vec<String>,
    pub relationships: Vec<String>,
    pub uncertainties: Vec<Unknown>,
    pub timestamp: Option<String>,
    pub provenance: Vec<String>,
}

impl WorldState {
    /// Empty world state at version 1.
    pub fn new() -> Self {
        WorldState {
            version: 1,
            entities: BTreeMap::new(),
            concepts: Vec::new(),
            facts: Vec::new(),
            assumptions: Vec::new(),
            observations: Vec::new(),
            constraints: Vec::new(),
            resources: BTreeMap::new(),
            goals: Vec::new(),
            relationships: Vec::new(),
            uncertainties: Vec::new(),
            timestamp: None,
            provenance: Vec::new(),
        }
    }

    /// Apply a delta, producing the next version. Facts/constraints/assumptions
    /// are appended (contradictions are surfaced by the caller, not overwritten
    /// silently). Returns the new state with `version` bumped by 1.
    pub fn apply(&self, delta: &StateDelta) -> WorldState {
        let mut next = self.clone();
        next.version = self.version + 1;
        for f in &delta.added_facts {
            next.facts.push(Fact {
                id: format!("fact_{}", next.facts.len() + 1),
                statement: f.clone(),
                status: EpistemicStatus::Inferred,
                evidence_refs: Vec::new(),
            });
        }
        for c in &delta.modified_constraints {
            next.constraints.push(c.clone());
        }
        for e in &delta.new_entities {
            next.entities.insert(e.clone(), e.clone());
        }
        for a in &delta.invalidated_assumptions {
            next.assumptions.retain(|x| x != a);
        }
        next
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// An explicit representation of what changed between two world states.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StateDelta {
    pub added_facts: Vec<String>,
    pub removed_facts: Vec<String>,
    pub modified_constraints: Vec<String>,
    pub new_entities: Vec<String>,
    pub invalidated_assumptions: Vec<String>,
    pub affected_goals: Vec<String>,
}

/// A first-class, machine-readable strategy. Not prose: every field is
/// structured so it can be evaluated, compared, and audited.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub objective: String,
    /// Explicit assumptions the strategy depends on.
    pub assumptions: Vec<String>,
    /// Hard constraints that must hold (cannot be violated).
    pub constraints: Vec<String>,
    /// Concrete actions / allocations recommended.
    pub actions: Vec<String>,
    /// Resources the strategy consumes.
    pub resources: BTreeMap<String, f64>,
    pub expected_outcomes: Vec<String>,
    pub risks: Vec<String>,
    pub dependencies: Vec<String>,
    pub evidence: Vec<String>,
    /// Confidence in [0, 1].
    pub confidence: f64,
    /// Robustness in [0, 1] (survives uncertainty vs. only-works-perfectly).
    pub robustness: f64,
}

impl Strategy {
    /// Human-readable explanation synthesized from the structured fields.
    pub fn explain(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!(
            "Strategy {} — objective: {}\n",
            self.id, self.objective
        ));
        s.push_str(&format!(
            "  confidence {:.2} | robustness {:.2}\n",
            self.confidence, self.robustness
        ));
        if !self.assumptions.is_empty() {
            s.push_str("  assumptions:\n");
            for a in &self.assumptions {
                s.push_str(&format!("    - {a}\n"));
            }
        }
        if !self.constraints.is_empty() {
            s.push_str("  hard constraints:\n");
            for c in &self.constraints {
                s.push_str(&format!("    - {c}\n"));
            }
        }
        if !self.actions.is_empty() {
            s.push_str("  actions:\n");
            for a in &self.actions {
                s.push_str(&format!("    - {a}\n"));
            }
        }
        if !self.expected_outcomes.is_empty() {
            s.push_str("  expected outcomes:\n");
            for o in &self.expected_outcomes {
                s.push_str(&format!("    - {o}\n"));
            }
        }
        if !self.risks.is_empty() {
            s.push_str("  risks:\n");
            for r in &self.risks {
                s.push_str(&format!("    - {r}\n"));
            }
        }
        if !self.evidence.is_empty() {
            s.push_str("  evidence:\n");
            for e in &self.evidence {
                s.push_str(&format!("    - {e}\n"));
            }
        }
        s
    }
}

/// Final status of a strategy after Gate (matches the directive's vocabulary).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrategyVerdict {
    Verified,
    ConditionallyVerified,
    Unverified,
    Rejected,
    BlockedByUnknowns,
    BlockedByContradiction,
}

impl StrategyVerdict {
    pub fn label(&self) -> &'static str {
        match self {
            StrategyVerdict::Verified => "verified",
            StrategyVerdict::ConditionallyVerified => "conditionally_verified",
            StrategyVerdict::Unverified => "unverified",
            StrategyVerdict::Rejected => "rejected",
            StrategyVerdict::BlockedByUnknowns => "blocked_by_unknowns",
            StrategyVerdict::BlockedByContradiction => "blocked_by_contradiction",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn world_state_apply_bumps_version_and_adds_fact() {
        let ws = WorldState::new();
        let delta = StateDelta {
            added_facts: vec!["new fact".into()],
            ..Default::default()
        };
        let next = ws.apply(&delta);
        assert_eq!(next.version, 2);
        assert_eq!(next.facts.len(), 1);
        assert_eq!(next.facts[0].statement, "new fact");
        assert_eq!(next.facts[0].status, EpistemicStatus::Inferred);
    }

    #[test]
    fn epistemic_status_serde_round_trip() {
        let j = serde_json::to_string(&EpistemicStatus::CorpusBacked).unwrap();
        assert_eq!(j, "\"corpus_backed\"");
        let back: EpistemicStatus = serde_json::from_str(&j).unwrap();
        assert_eq!(back, EpistemicStatus::CorpusBacked);
    }

    #[test]
    fn strategy_explain_contains_id() {
        let s = Strategy {
            id: "S01".into(),
            objective: "demo".into(),
            assumptions: vec![],
            constraints: vec![],
            actions: vec!["x 1".into()],
            resources: BTreeMap::new(),
            expected_outcomes: vec![],
            risks: vec![],
            dependencies: vec![],
            evidence: vec![],
            confidence: 0.5,
            robustness: 0.5,
        };
        assert!(s.explain().contains("S01"));
    }
}
