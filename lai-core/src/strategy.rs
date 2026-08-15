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
    Computed,
    CorpusBacked,
    Observed,
    ExternallySourced,
    Derived,
    Inferred,
    Hypothesis,
    Assumed,
    Unknown,
    Contradicted,
    Stale,
    Rejected,
}

impl EpistemicStatus {
    /// Stable, human-readable label (used in CLI text output).
    pub fn label(&self) -> &'static str {
        match self {
            EpistemicStatus::Verified => "verified",
            EpistemicStatus::Computed => "computed",
            EpistemicStatus::CorpusBacked => "corpus_backed",
            EpistemicStatus::Observed => "observed",
            EpistemicStatus::ExternallySourced => "externally_sourced",
            EpistemicStatus::Derived => "derived",
            EpistemicStatus::Inferred => "inferred",
            EpistemicStatus::Hypothesis => "hypothesis",
            EpistemicStatus::Assumed => "assumed",
            EpistemicStatus::Unknown => "unknown",
            EpistemicStatus::Contradicted => "contradicted",
            EpistemicStatus::Stale => "stale",
            EpistemicStatus::Rejected => "rejected",
        }
    }
}

impl std::fmt::Display for EpistemicStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A single claim (belief node) in the world model. Claims are distinct from raw
/// [`Observation`]s: an observation is what was perceived; a claim is what L
/// believes given that observation. A claim carries its own epistemic status, the
/// evidence it rests on, and the dependency edges used for truth maintenance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub id: String,
    pub statement: String,
    pub status: EpistemicStatus,
    /// Ids of [`Evidence`] records supporting the claim.
    pub evidence_refs: Vec<String>,
    /// Ids of other claims this claim depends on. If a dependency is invalidated,
    /// truth maintenance cascades to this claim as well.
    pub dependencies: Vec<String>,
    /// Origin of the claim (e.g. "observation", "corpus", "inference").
    pub source: Option<String>,
    pub timestamp: Option<String>,
    pub confidence: f64,
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

/// A raw perception, distinct from any belief derived from it. Per the directive,
/// "what happened" must not be collapsed into "what L believes happened". The
/// candidate [`Claim`]s are produced from observations, never the other way round.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    /// Raw content exactly as perceived.
    pub content: String,
    /// Origin: UserInput / Corpus / ExternalSource / ToolOutput / Observation.
    pub source: String,
    pub timestamp: Option<String>,
    /// Trust in the observation in [0, 1] (see directive §45 trust tiers).
    pub reliability: f64,
    /// Scenario / temporal context the observation was made in.
    pub context: Option<String>,
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

/// Resolution lifecycle for a [`Contradiction`] (directive §13).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionStatus {
    Unresolved,
    ResolvedByEvidence,
    ResolvedByContext,
    ResolvedByTime,
    ResolvedByUser,
    Superseded,
    PersistentConflict,
}

/// A contradiction between two or more claims. Never silently overwrite conflicting
/// knowledge; record it and mark dependent strategies accordingly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Contradiction {
    pub id: String,
    pub claim_ids: Vec<String>,
    pub evidence: Vec<String>,
    pub context: Option<String>,
    /// "critical" | "major" | "minor" — critical contradictions block strategies.
    pub severity: String,
    pub resolution_status: ContradictionStatus,
}

/// The persistent model of what L currently believes is true. Order-sensitive
/// collections use `BTreeMap`/`Vec` so serialization is deterministic. World state
/// is immutable by default: updates produce a new, higher-versioned state via
/// [`WorldState::apply`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    /// Stable identifier for this world (e.g. "cyberpunk", "session-7").
    pub id: String,
    /// Monotonic version, bumped on every state delta.
    pub version: u64,
    /// Resolved entity id -> canonical name.
    pub entities: BTreeMap<String, String>,
    /// Free-form concepts mentioned but not yet entity-resolved.
    pub concepts: Vec<String>,
    /// Raw observations (what was perceived), kept distinct from beliefs.
    pub observations: Vec<Observation>,
    /// Belief nodes derived from observations.
    pub claims: Vec<Claim>,
    pub assumptions: Vec<String>,
    pub constraints: Vec<String>,
    /// Named resources and their available amounts.
    pub resources: BTreeMap<String, f64>,
    pub goals: Vec<String>,
    pub relationships: Vec<String>,
    pub uncertainties: Vec<Unknown>,
    /// First-class contradictions between claims.
    pub contradictions: Vec<Contradiction>,
    pub timestamp: Option<String>,
    pub provenance: Vec<String>,
}

impl WorldState {
    /// Empty world state at version 1.
    pub fn new() -> Self {
        WorldState {
            id: "world".into(),
            version: 1,
            entities: BTreeMap::new(),
            concepts: Vec::new(),
            observations: Vec::new(),
            claims: Vec::new(),
            assumptions: Vec::new(),
            constraints: Vec::new(),
            resources: BTreeMap::new(),
            goals: Vec::new(),
            relationships: Vec::new(),
            uncertainties: Vec::new(),
            contradictions: Vec::new(),
            timestamp: None,
            provenance: Vec::new(),
        }
    }

    /// Apply a delta, producing the next version. Updates never silently mutate
    /// the past. If `delta.invalidated_claims` names claims, they are marked
    /// `Contradicted` and truth maintenance cascades to every claim that depends
    /// (transitively) on them. Returns the new state with `version` bumped by 1.
    pub fn apply(&self, delta: &StateDelta) -> WorldState {
        use std::collections::HashSet;
        let mut next = self.clone();
        next.version = self.version + 1;
        for e in &delta.added_entities {
            next.entities.insert(e.clone(), e.clone());
        }
        for e in &delta.removed_entities {
            next.entities.remove(e);
        }
        for c in &delta.added_claims {
            next.claims.push(c.clone());
        }
        // Truth maintenance: invalidated claims cascade to their dependents.
        if !delta.invalidated_claims.is_empty() {
            let mut invalid: HashSet<String> = delta.invalidated_claims.iter().cloned().collect();
            loop {
                let before = invalid.len();
                for claim in &next.claims {
                    if invalid.contains(&claim.id) {
                        continue;
                    }
                    if claim.dependencies.iter().any(|d| invalid.contains(d)) {
                        invalid.insert(claim.id.clone());
                    }
                }
                if invalid.len() == before {
                    break;
                }
            }
            for claim in &mut next.claims {
                if invalid.contains(&claim.id) {
                    claim.status = EpistemicStatus::Contradicted;
                }
            }
        }
        for r in &delta.modified_relationships {
            next.relationships.push(r.clone());
        }
        for c in &delta.changed_constraints {
            next.constraints.push(c.clone());
        }
        for g in &delta.changed_goals {
            next.goals.push(g.clone());
        }
        for u in &delta.new_unknowns {
            next.uncertainties.push(u.clone());
        }
        for u in &delta.resolved_unknowns {
            next.uncertainties.retain(|x| x.question != *u);
        }
        for c in &delta.new_contradictions {
            next.contradictions.push(c.clone());
        }
        next
    }
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

/// An explicit representation of what changed between two world states. The whole
/// point of the strategy engine is to answer "WHAT CHANGED?" before "WHAT SHOULD WE
/// DO?". Field set follows directive §11.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StateDelta {
    pub added_entities: Vec<String>,
    pub removed_entities: Vec<String>,
    pub added_claims: Vec<Claim>,
    pub invalidated_claims: Vec<String>,
    pub modified_relationships: Vec<String>,
    pub changed_constraints: Vec<String>,
    pub changed_goals: Vec<String>,
    pub new_unknowns: Vec<Unknown>,
    pub resolved_unknowns: Vec<String>,
    pub new_contradictions: Vec<Contradiction>,
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

/// A first-class mathematical action (directive §16). An action transforms state;
/// it is not prose. It carries preconditions, effects, cost, resources, risks, and
/// the evidence it rests on so a strategy becomes a graph of state transitions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub name: String,
    /// Conditions that must hold before the action applies.
    pub preconditions: Vec<String>,
    /// Effects the action produces on the world state.
    pub effects: Vec<String>,
    pub cost: f64,
    /// Resource consumption keyed by resource name.
    pub resources: BTreeMap<String, f64>,
    pub duration: Option<String>,
    pub risks: Vec<String>,
    pub dependencies: Vec<String>,
    /// "reversible" | "irreversible" | None (unknown).
    pub reversibility: Option<String>,
    pub evidence: Vec<String>,
}

/// The internal compiler-IR of a strategy (directive §14). Natural language must
/// not flow directly into the optimizer; it flows through semantic grounding into
/// this structure, which the planner/optimizer then consumes. Objective, hard
/// constraints, soft constraints, assumptions, and unknowns are kept strictly
/// separate (§15).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyIR {
    pub objective: String,
    /// Version of the [`WorldState`] this IR was derived from.
    pub initial_state_version: u64,
    pub goal_conditions: Vec<String>,
    /// Resolved entity mention -> canonical name.
    pub entities: BTreeMap<String, String>,
    pub assumptions: Vec<String>,
    pub hard_constraints: Vec<String>,
    pub soft_constraints: Vec<String>,
    pub actions: Vec<Action>,
    pub resources: BTreeMap<String, f64>,
    pub risks: Vec<String>,
    pub dependencies: Vec<String>,
    /// Open questions that, if answered, could change the strategy.
    pub unknowns: Vec<String>,
    pub evidence: Vec<String>,
}

impl StrategyIR {
    /// Empty IR for a given objective, derived from world-state version `version`.
    pub fn new(objective: &str, version: u64) -> Self {
        StrategyIR {
            objective: objective.to_string(),
            initial_state_version: version,
            goal_conditions: Vec::new(),
            entities: BTreeMap::new(),
            assumptions: Vec::new(),
            hard_constraints: Vec::new(),
            soft_constraints: Vec::new(),
            actions: Vec::new(),
            resources: BTreeMap::new(),
            risks: Vec::new(),
            dependencies: Vec::new(),
            unknowns: Vec::new(),
            evidence: Vec::new(),
        }
    }
}

/// A structured counterexample (directive §20). More valuable than "try harder":
/// it names the violated condition, the triggering state, and concrete repairs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Counterexample {
    pub violated_constraint: String,
    pub triggering_state: String,
    pub action: String,
    pub expected_effect: String,
    pub actual_result: String,
    pub repair_candidates: Vec<String>,
}

/// Machine-readable adversarial analysis of a strategy (directive §19/§25). Athena
/// produces this, not prose: assumption challenges, counterexamples, contradictory
/// evidence, failure modes, alternative strategies, sensitivity points, missing
/// information, and a recommended next action. Deterministic and auditable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChallengeReport {
    pub strategy_id: String,
    pub critical_assumptions: Vec<String>,
    pub assumption_challenges: Vec<String>,
    pub counterexamples: Vec<Counterexample>,
    pub contradictory_evidence: Vec<String>,
    pub failure_modes: Vec<String>,
    pub alternative_strategies: Vec<String>,
    pub sensitivity_points: Vec<String>,
    pub missing_information: Vec<String>,
    pub recommended_next_action: String,
}

/// Generate a deterministic challenge report for a strategy against an optional
/// world state. No LLM: the analysis is derived structurally from the strategy's
/// assumptions, constraints, risks, resources, and the world's open unknowns /
/// contradictions (directive §24: DO-NOT-ACT-YET is a valid machine decision).
pub fn generate_challenge(strategy: &Strategy, ws: Option<&WorldState>) -> ChallengeReport {
    let assumption_challenges = strategy
        .assumptions
        .iter()
        .map(|a| format!("what if \"{a}\" is false or only partially true?"))
        .collect::<Vec<_>>();
    let mut contradictory_evidence = Vec::new();
    let mut missing_information = Vec::new();
    if let Some(w) = ws {
        for u in &w.uncertainties {
            missing_information.push(u.question.clone());
        }
        for c in &w.contradictions {
            contradictory_evidence.push(format!(
                "contradiction {} (severity {}) unresolved",
                c.id, c.severity
            ));
        }
    }
    let mut failure_modes = strategy.risks.clone();
    for (k, v) in &strategy.resources {
        failure_modes.push(format!("resource {k} drops below required {v}"));
    }
    for a in &strategy.assumptions {
        failure_modes.push(format!("assumption fails: {a}"));
    }
    let sensitivity_points = strategy
        .constraints
        .iter()
        .map(|c| format!("sensitive to violation of: {c}"))
        .collect::<Vec<_>>();
    let alternative_strategies = vec![
        "defer and gather information (Ask/Research) before committing".to_string(),
        "minimal-footprint variant: same objective at lower resource cost".to_string(),
        "hedged allocation: diversify across pillars to reduce single-point failure".to_string(),
    ];
    let recommended_next_action = if !missing_information.is_empty() {
        "research / ask — resolve open unknowns before acting".to_string()
    } else {
        "act — no blocking unknowns; proceed with monitoring".to_string()
    };
    let counterexamples = strategy
        .constraints
        .iter()
        .map(|c| Counterexample {
            violated_constraint: c.clone(),
            triggering_state: "constraint not enforced at execution".into(),
            action: strategy.actions.first().cloned().unwrap_or_default(),
            expected_effect: "constraint satisfied".into(),
            actual_result: format!("constraint violated: {c}"),
            repair_candidates: vec![
                "tighten Gate pre-condition check".into(),
                "add pre-execution validation".into(),
            ],
        })
        .collect();
    ChallengeReport {
        strategy_id: strategy.id.clone(),
        critical_assumptions: strategy.assumptions.clone(),
        assumption_challenges,
        counterexamples,
        contradictory_evidence,
        failure_modes,
        alternative_strategies,
        sensitivity_points,
        missing_information,
        recommended_next_action,
    }
}

/// Result of applying one [`Action`] to a [`WorldState`] (directive §17/§30). A
/// transition records the before/after state, the effects produced, and any
/// constraint/resource violations — so simulation is inspectable and reproducible
/// rather than a black box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transition {
    pub before: WorldState,
    pub action: Action,
    pub after: WorldState,
    pub effects: Vec<String>,
    pub violations: Vec<String>,
    pub evidence: Vec<String>,
}

/// A state-transition engine (directive §30). Plugs into the planner so a strategy
/// becomes a sequence/graph of transitions instead of prose.
pub trait Simulator {
    /// Apply `action` to `state`, returning the resulting transition.
    fn step(&self, state: &WorldState, action: &Action) -> Transition;
}

/// Deterministic simulator: applies an action's effects as claims and refuses the
/// action when its preconditions or resource budget are not met by the current
/// state. No probabilistic behavior — build this first (§17), stochastic later.
pub struct DeterministicSimulator;

impl Simulator for DeterministicSimulator {
    fn step(&self, state: &WorldState, action: &Action) -> Transition {
        let mut violations = Vec::new();
        // Preconditions must be supported by some claim/assumption/constraint text.
        for pre in &action.preconditions {
            let p = pre.to_lowercase();
            let satisfied = state
                .claims
                .iter()
                .any(|c| c.statement.to_lowercase().contains(&p))
                || state
                    .assumptions
                    .iter()
                    .any(|a| a.to_lowercase().contains(&p))
                || state
                    .constraints
                    .iter()
                    .any(|c| c.to_lowercase().contains(&p));
            if !satisfied {
                violations.push(format!("unmet precondition: {pre}"));
            }
        }
        // Resource budget: state must cover the action's resource demand.
        for (k, v) in &action.resources {
            let have = state.resources.get(k).copied().unwrap_or(0.0);
            if have + 1e-9 < *v {
                violations.push(format!("insufficient {k}: have {have}, need {v}"));
            }
        }
        let after = if violations.is_empty() {
            let mut claims = Vec::new();
            for (i, eff) in action.effects.iter().enumerate() {
                claims.push(Claim {
                    id: format!("{}_eff_{}", action.id, i),
                    statement: eff.clone(),
                    status: EpistemicStatus::Inferred,
                    evidence_refs: action.evidence.clone(),
                    dependencies: vec![action.id.clone()],
                    source: Some("simulation".into()),
                    timestamp: None,
                    confidence: 0.9,
                });
            }
            state.apply(&StateDelta {
                added_claims: claims,
                ..Default::default()
            })
        } else {
            state.clone()
        };
        Transition {
            before: state.clone(),
            action: action.clone(),
            after,
            effects: if violations.is_empty() {
                action.effects.clone()
            } else {
                Vec::new()
            },
            violations,
            evidence: action.evidence.clone(),
        }
    }
}

/// Deterministically apply a sequence of actions, threading the world state forward.
pub fn simulate_sequence(state: &WorldState, actions: &[Action]) -> Vec<Transition> {
    let sim = DeterministicSimulator;
    let mut cur = state.clone();
    let mut out = Vec::new();
    for a in actions {
        let t = sim.step(&cur, a);
        cur = t.after.clone();
        out.push(t);
    }
    out
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
    fn world_state_apply_bumps_version_and_adds_claim() {
        let ws = WorldState::new();
        let delta = StateDelta {
            added_claims: vec![Claim {
                id: "c1".into(),
                statement: "new claim".into(),
                status: EpistemicStatus::Inferred,
                evidence_refs: vec![],
                dependencies: vec![],
                source: Some("observation".into()),
                timestamp: None,
                confidence: 0.5,
            }],
            ..Default::default()
        };
        let next = ws.apply(&delta);
        assert_eq!(next.version, 2);
        assert_eq!(next.claims.len(), 1);
        assert_eq!(next.claims[0].statement, "new claim");
        assert_eq!(next.claims[0].status, EpistemicStatus::Inferred);
    }

    #[test]
    fn truth_maintenance_cascades_invalidation() {
        let mut ws = WorldState::new();
        // A depends on B.
        ws.claims.push(Claim {
            id: "A".into(),
            statement: "derived from B".into(),
            status: EpistemicStatus::Derived,
            evidence_refs: vec![],
            dependencies: vec!["B".into()],
            source: None,
            timestamp: None,
            confidence: 1.0,
        });
        ws.claims.push(Claim {
            id: "B".into(),
            statement: "base claim".into(),
            status: EpistemicStatus::CorpusBacked,
            evidence_refs: vec![],
            dependencies: vec![],
            source: None,
            timestamp: None,
            confidence: 1.0,
        });
        let next = ws.apply(&StateDelta {
            invalidated_claims: vec!["B".into()],
            ..Default::default()
        });
        let a = next.claims.iter().find(|c| c.id == "A").unwrap();
        let b = next.claims.iter().find(|c| c.id == "B").unwrap();
        assert_eq!(b.status, EpistemicStatus::Contradicted);
        assert_eq!(
            a.status,
            EpistemicStatus::Contradicted,
            "dependent claim must cascade"
        );
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

#[cfg(test)]
mod sim_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn action(name: &str, pre: Vec<String>, res: BTreeMap<String, f64>) -> Action {
        Action {
            id: name.into(),
            name: name.into(),
            preconditions: pre,
            effects: vec![format!("did {name}")],
            cost: 1.0,
            resources: res,
            duration: None,
            risks: vec![],
            dependencies: vec![],
            reversibility: Some("reversible".into()),
            evidence: vec!["test".into()],
        }
    }

    #[test]
    fn simulator_applies_effects_when_preconditions_met() {
        let mut ws = WorldState::new();
        ws.constraints.push("budget <= 100".into());
        ws.resources.insert("unit".into(), 20.0);
        let a = action("step", vec!["budget <= 100".into()], {
            let mut m = BTreeMap::new();
            m.insert("unit".into(), 5.0);
            m
        });
        let t = {
            let sim = DeterministicSimulator;
            sim.step(&ws, &a)
        };
        assert!(
            t.violations.is_empty(),
            "got violations: {:?}",
            t.violations
        );
        assert_eq!(t.after.version, 2);
        assert!(t.after.claims.iter().any(|c| c.statement == "did step"));
    }

    #[test]
    fn simulator_refuses_unmet_precondition_and_resource_shortfall() {
        let ws = WorldState::new();
        let a = action("step", vec!["must hold".into()], {
            let mut m = BTreeMap::new();
            m.insert("unit".into(), 5.0);
            m
        });
        let t = {
            let sim = DeterministicSimulator;
            sim.step(&ws, &a)
        };
        assert!(!t.violations.is_empty());
        assert!(t
            .violations
            .iter()
            .any(|v| v.contains("unmet precondition")));
        assert!(t.violations.iter().any(|v| v.contains("insufficient unit")));
        // State is unchanged when the action is refused.
        assert_eq!(t.after, ws);
    }

    #[test]
    fn simulate_sequence_threads_state() {
        let mut ws = WorldState::new();
        ws.resources.insert("unit".into(), 20.0);
        let a = action("only", vec![], {
            let mut m = BTreeMap::new();
            m.insert("unit".into(), 1.0);
            m
        });
        let ts = simulate_sequence(&ws, &[a.clone(), a]);
        assert_eq!(ts.len(), 2);
        assert_eq!(ts[1].after.version, 3);
    }
}

#[cfg(test)]
mod challenge_tests {
    use super::*;
    use std::collections::BTreeMap;

    fn sample_strategy() -> Strategy {
        Strategy {
            id: "S01".into(),
            objective: "demo".into(),
            assumptions: vec!["network stays up".into()],
            constraints: vec!["budget <= 100".into()],
            actions: vec!["loom x 10".into()],
            resources: {
                let mut m = BTreeMap::new();
                m.insert("unit".into(), 20.0);
                m
            },
            expected_outcomes: vec![],
            risks: vec!["single point of failure".into()],
            dependencies: vec![],
            evidence: vec!["optimizer".into()],
            confidence: 0.7,
            robustness: 0.5,
        }
    }

    #[test]
    fn challenge_reports_assumptions_and_counterexamples() {
        let s = sample_strategy();
        let r = generate_challenge(&s, None);
        assert_eq!(r.strategy_id, "S01");
        assert_eq!(r.critical_assumptions, s.assumptions);
        assert_eq!(r.assumption_challenges.len(), s.assumptions.len());
        assert_eq!(r.counterexamples.len(), s.constraints.len());
        assert!(r.recommended_next_action.contains("act"));
        assert!(r.alternative_strategies.len() >= 3);
    }

    #[test]
    fn challenge_flags_missing_information_from_world() {
        let s = sample_strategy();
        let mut ws = WorldState::new();
        ws.uncertainties.push(Unknown {
            question: "is monitoring active?".into(),
            impact: "high".into(),
            urgency: "high".into(),
            strategies_affected: 1,
            information_gain: "high".into(),
        });
        let r = generate_challenge(&s, Some(&ws));
        assert!(r
            .missing_information
            .contains(&"is monitoring active?".to_string()));
        assert!(r.recommended_next_action.contains("research"));
    }
}
