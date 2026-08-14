//! # Adaptive Strategy Engine (ASE) — structured, inspectable reasoning pipeline
//!
//! Evolves L from a verification engine into an **adaptive strategy engine** over
//! a clearly-labeled *hypothetical* world model. The initial application is the
//! scenario *"What if Cyberpunk RED / Cyberpunk 2077 were real life?"* — a
//! near-future society model built from **plausible real-world analogues** of the
//! fictional setting. The fiction is never treated as literal fact: every state,
//! event, fact, and assumption is tagged `hypothetical`.
//!
//! ## What the engine actually does
//! Given a situation and new information, it builds a structured model of:
//! - **known facts** (with a confidence each),
//! - **constraints** the strategies must respect,
//! - **objectives** (explicit, contestable),
//! - **relationships** between world factors (a small dependency graph),
//! - **uncertainties** (unknowns with impact),
//! - **competing possibilities** (multiple candidate strategies).
//!
//! It then runs the loop
//! `Observe → Integrate → Detect Delta → Generate → Simulate → Verify →
//! Select → Act → Observe Again`. New information is treated as an *update*: it
//! revises fact confidence, **detects contradictions**, and **recalculates the
//! strategic ranking**. The engine identifies the most valuable missing
//! information (value-of-information), explains *why* a strategy is preferred,
//! and **preserves alternatives when uncertainty is high**.
//!
//! ## Trust boundary (preserved)
//! - **Untrusted proposers** (LLM / Athena / human scenario author) may *propose*
//!   events, strategies, and assumptions; they are labeled `proposer`.
//! - **Trusted deterministic L** *validates*: the `cid` Gate screens strategy
//!   text for provenance / consistency / injection, and the Proof verifier
//!   (`verify_proposal_envelope`) seals a tamper-evident [`ProofEnvelope`] for
//!   every formalizable critical assumption. L never asserts an unverifiable
//!   claim as true; it reports `unevaluable`.
//!
//! ## Honesty contract
//! Selection ranks strategies by an **explicit objective function** (discounted
//! by assumption confidence) and reports the ranking plus the most valuable
//! missing information. It does *not* claim universal optimality — the returned
//! [`StrategyCycleReport::caveat`] says so explicitly.

use std::collections::HashMap;

use cid::core::ball::{Ball, GateOutcome, TokenCandidate};
use cid::core::pin::PinField;
use cid::gates::validate_candidate;
use cid::kb::facts::KnowledgeBase;

use crate::digest::sha256_hex;
use crate::verify::envelope::{ProofEnvelope, ProofVerdict};
use crate::verify::verifier::{verify_proposal_envelope, ProposalKind};

use serde::{Deserialize, Serialize};

/// The ten required dimensions of the near-future hypothetical model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Factor {
    CyberneticAugmentation,
    UbiquitousSurveillance,
    Megacorporations,
    AutonomousWeapons,
    AiAgents,
    NeuralInterfaces,
    EconomicInequality,
    DigitalIdentity,
    Cybercrime,
    AdvancedProsthetics,
}

impl Factor {
    pub fn all() -> &'static [Factor] {
        &[
            Factor::CyberneticAugmentation,
            Factor::UbiquitousSurveillance,
            Factor::Megacorporations,
            Factor::AutonomousWeapons,
            Factor::AiAgents,
            Factor::NeuralInterfaces,
            Factor::EconomicInequality,
            Factor::DigitalIdentity,
            Factor::Cybercrime,
            Factor::AdvancedProsthetics,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Factor::CyberneticAugmentation => "cybernetic augmentation",
            Factor::UbiquitousSurveillance => "ubiquitous surveillance",
            Factor::Megacorporations => "megacorporations",
            Factor::AutonomousWeapons => "autonomous weapons",
            Factor::AiAgents => "AI agents",
            Factor::NeuralInterfaces => "neural interfaces",
            Factor::EconomicInequality => "economic inequality",
            Factor::DigitalIdentity => "digital identity",
            Factor::Cybercrime => "cybercrime",
            Factor::AdvancedProsthetics => "advanced prosthetics",
        }
    }
}

/// One modeled dimension of the world at a point in time. `level` is a
/// transparent 0.0–1.0 intensity estimate (never a measured fact);
/// `confidence` is L's confidence in that estimate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFactor {
    pub level: f64,
    pub note: String,
    pub confidence: f64,
}

/// A structured, confidence-weighted known fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fact {
    pub id: String,
    /// Normalized subject key used for corroboration / contradiction matching.
    pub subject: String,
    /// `true` = the fact asserts the subject holds; `false` = asserts it does not.
    pub asserts: bool,
    pub statement: String,
    /// 0.0–1.0 confidence in this assertion.
    pub confidence: f64,
    pub source: String,
}

impl Fact {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        subject: &str,
        asserts: bool,
        statement: &str,
        confidence: f64,
        source: &str,
    ) -> Self {
        Fact {
            id: id.to_string(),
            subject: subject.to_string(),
            asserts,
            statement: statement.to_string(),
            confidence: confidence.clamp(0.0, 1.0),
            source: source.to_string(),
        }
    }
}

/// A directed influence between two world factors. `strength` is signed: a lever
/// applied to `from` propagates `strength * Δfrom` to `to` in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: Factor,
    pub to: Factor,
    pub strength: f64,
}

/// An explicit unknown whose resolution would change the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    pub id: String,
    pub description: String,
    pub possible_values: Vec<String>,
    /// 0.0–1.0 current confidence in the present working assumption.
    pub confidence: f64,
    /// |effect on objective score| if the uncertainty resolves adversely.
    pub impact: f64,
}

/// A detected contradiction between a new fact and an existing belief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub subject: String,
    pub existing_fact_id: String,
    pub new_fact_id: String,
    pub existing_confidence: f64,
    pub new_confidence: f64,
    pub resolution: String,
}

/// Result of ingesting one hypothetical event into the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    pub world: WorldState,
    pub delta: Delta,
    pub conflicts: Vec<Conflict>,
    /// Fact ids that were corroborated (same subject, same polarity).
    pub corroborated: Vec<String>,
}

/// A snapshot of the (hypothetical) world model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Always `true`: this model is a hypothetical, not a claim about reality.
    pub hypothetical: bool,
    pub label: String,
    pub factors: HashMap<Factor, WorldFactor>,
    /// Structured, confidence-weighted known facts.
    pub facts: Vec<Fact>,
    /// Dependency edges between factors (used by the relational simulator).
    pub relationships: Vec<Relationship>,
    /// Explicit unknowns.
    pub uncertainties: Vec<Uncertainty>,
    pub assumptions: Vec<String>,
    /// Monotonic revision counter (bumped on every integration).
    pub revision: u64,
}

impl WorldState {
    pub fn level(&self, f: Factor) -> f64 {
        self.factors.get(&f).map(|w| w.level).unwrap_or(0.0)
    }

    pub fn confidence_of(&self, f: Factor) -> f64 {
        self.factors.get(&f).map(|w| w.confidence).unwrap_or(0.0)
    }

    /// Stable fingerprint binding a proof to this exact world model.
    pub fn corpus_hash(&self) -> String {
        let mut blob = self.label.clone();
        for a in &self.assumptions {
            blob.push('\n');
            blob.push_str(a);
        }
        for f in &self.facts {
            blob.push('\n');
            blob.push_str(&format!("{}|{}|{}", f.subject, f.asserts, f.confidence));
        }
        sha256_hex(blob.as_bytes())
    }
}

/// A hypothetical event reported into the model. The `source` must declare its
/// hypothetical nature; L never ingests it as fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    /// Must declare hypothetical provenance; L never ingests as fact.
    pub provenance: String,
    pub content: String,
    /// 0.0–1.0 reporter confidence; never treated as L's own confidence.
    pub confidence: f64,
}

/// A hypothetical event: an [`Evidence`] plus the explicit, transparent
/// adjustments it implies for the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub evidence: Evidence,
    /// Adjustments to world factor levels (delta applied on integration).
    pub factor_adjustments: Vec<(Factor, f64, String)>,
    /// New structured facts asserted by this event.
    pub new_facts: Vec<Fact>,
    /// Free-form assumption notes.
    pub new_assumptions: Vec<String>,
}

/// A single observed change between two world states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChange {
    pub factor: Factor,
    pub from: f64,
    pub to: f64,
    pub note: String,
}

/// What changed after integrating an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub changes: Vec<FieldChange>,
    pub summary: String,
    pub explanation: String,
}

impl Delta {
    pub fn empty() -> Self {
        Delta {
            changes: Vec::new(),
            summary: "(no change)".to_string(),
            explanation: String::new(),
        }
    }
}

/// A criterion of the explicit objective function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub name: String,
    pub weight: f64,
    /// `true` → higher metric is better; `false` → lower is better.
    pub maximize: bool,
}

/// The explicit objective the engine optimizes *against* (not "the good").
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    pub name: String,
    pub criteria: Vec<Criterion>,
}

/// A constraint strategies must respect. `verifiable` ones carry a formal claim
/// checked by the deterministic verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub description: String,
    pub verifiable: bool,
    pub claim: Option<String>,
}

/// Resources available to execute a strategy.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Resource {
    pub tokens: u64,
    pub budget_usd: f64,
    pub political_capital: f64,
}

impl Resource {
    pub fn default_pool() -> Self {
        Resource {
            tokens: 0,
            budget_usd: 10_000_000.0,
            political_capital: 1.0,
        }
    }
}

/// A risk attached to a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
}

/// An assumption a strategy depends on. `subject` links it to a [`Fact`] so its
/// confidence tracks the world model; `critical` assumptions gate admissibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub statement: String,
    /// Optional formalizable claim for the deterministic verifier (e.g. "0.65 > 0.6").
    pub formal_claim: Option<String>,
    pub subject: Option<String>,
    pub critical: bool,
    /// |effect on objective score| if the assumption is false.
    pub impact: f64,
    /// 0.0–1.0 confidence, seeded from the matching fact(s) in the world model.
    pub confidence: f64,
}

/// Predicted consequences of a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Named metrics in [0,1]; keys match `Objective::Criterion::name`.
    pub metrics: HashMap<String, f64>,
    pub narrative: String,
    /// Objective-function score in [0,1] (before confidence discount).
    pub predicted_score: f64,
}

/// A candidate strategy. `proposer` records the trust status of its origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: String,
    pub name: String,
    pub description: String,
    /// "deterministic-heuristic" (trusted generator) or an untrusted label.
    pub proposer: String,
    pub assumptions: Vec<Assumption>,
    pub cost: Resource,
    pub risks: Vec<Risk>,
    /// Intervention magnitudes applied to world factors during simulation.
    pub levers: HashMap<Factor, f64>,
    pub expected_outcome: Outcome,
}

/// Verification result for one assumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssumptionVerification {
    pub statement: String,
    /// "verified" | "refuted" | "unevaluable"
    pub verdict: String,
    pub proof: Option<ProofEnvelope>,
}

/// Trust + proof record for one strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVerification {
    pub strategy_id: String,
    /// Outcome of the `cid` Gate screen of the strategy text.
    pub gate_outcome: String,
    pub assumptions: Vec<AssumptionVerification>,
    /// True if any *critical* assumption could not be verified.
    pub critical_unverified: bool,
}

/// A strategy ranked by the objective function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedStrategy {
    pub rank: usize,
    pub strategy_id: String,
    pub name: String,
    /// Objective score discounted by assumption confidence.
    pub score: f64,
    /// Mean confidence of critical assumptions (transparency).
    pub confidence: f64,
    /// False if a critical assumption was refuted or the Gate screen failed.
    pub admissible: bool,
    pub rationale: String,
}

/// Value-of-information entry: an unknown and how much the decision swings on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoI {
    pub unknown: String,
    /// 0.0–1.0 information value (decision sensitivity).
    pub info_value: f64,
    pub rationale: String,
}

/// Machine-readable record of one full adaptive cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyCycleReport {
    pub scenario_label: String,
    pub world_before: WorldState,
    pub world_after: WorldState,
    pub delta: Delta,
    pub objective: Objective,
    pub constraints: Vec<Constraint>,
    pub facts: Vec<Fact>,
    pub relationships: Vec<Relationship>,
    pub uncertainties: Vec<Uncertainty>,
    pub conflicts: Vec<Conflict>,
    pub strategies: Vec<Strategy>,
    pub verifications: Vec<StrategyVerification>,
    pub ranking: Vec<RankedStrategy>,
    pub recommended: Option<RankedStrategy>,
    pub alternatives: Vec<RankedStrategy>,
    pub keep_alternatives: bool,
    pub value_of_information: Vec<VoI>,
    pub missing_information: Vec<MissingInfo>,
    pub preference_explanation: String,
    /// Explicit statement that the recommendation is *not* universal optimality.
    pub caveat: String,
}

/// The single most valuable piece of missing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingInfo {
    pub question: String,
    pub why_valuable: String,
    pub info_value: f64,
}

fn clamp1(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

// ----------------------------------------------------------------------------
// Observe / Integrate (with confidence revision + contradiction detection)
// ----------------------------------------------------------------------------

/// Revise an existing fact's confidence when new, same-subject evidence arrives.
/// Same polarity → confidence moves toward the new evidence (corroboration);
/// opposite polarity → both confidences are discounted (conflict).
fn revise(existing: &mut Fact, incoming: &Fact) -> Option<Conflict> {
    if existing.subject != incoming.subject {
        return None;
    }
    if existing.asserts == incoming.asserts {
        // Corroboration: nudge existing confidence toward the incoming value.
        existing.confidence = ((existing.confidence + incoming.confidence) / 2.0).clamp(0.0, 1.0);
        return None;
    }
    // Contradiction: discount both.
    let old = existing.confidence;
    existing.confidence = (existing.confidence * 0.4).clamp(0.0, 1.0);
    let conflict = Conflict {
        subject: existing.subject.clone(),
        existing_fact_id: existing.id.clone(),
        new_fact_id: incoming.id.clone(),
        existing_confidence: old,
        new_confidence: incoming.confidence,
        resolution: "contradiction detected; both fact confidences discounted".to_string(),
    };
    Some(conflict)
}

/// Integrate a hypothetical event into the world, returning the next state plus
/// detected conflicts and corroborations.
pub fn integrate_event(world: &WorldState, event: &WorldEvent) -> IntegrationResult {
    let mut next = world.clone();
    let mut conflicts = Vec::new();
    let mut corroborated = Vec::new();

    for f in &event.new_facts {
        if let Some(existing) = next.facts.iter_mut().find(|e| e.subject == f.subject) {
            if existing.asserts == f.asserts {
                revise(existing, f);
                corroborated.push(existing.id.clone());
            } else {
                let c = revise(existing, f);
                if let Some(c) = c {
                    conflicts.push(c);
                }
            }
        } else {
            next.facts.push(f.clone());
        }
    }

    for (factor, delta, note) in &event.factor_adjustments {
        let prev = next.level(*factor);
        let new_level = clamp1(prev + delta);
        let prev_conf = next.confidence_of(*factor);
        next.factors.insert(
            *factor,
            WorldFactor {
                level: new_level,
                note: note.clone(),
                confidence: prev_conf,
            },
        );
    }
    for a in &event.new_assumptions {
        next.assumptions.push(a.clone());
    }
    next.revision += 1;

    let delta = detect_delta(world, &next);
    IntegrationResult {
        world: next,
        delta,
        conflicts,
        corroborated,
    }
}

/// Detect what changed between two world states (factors + fact count).
pub fn detect_delta(before: &WorldState, after: &WorldState) -> Delta {
    let mut changes = Vec::new();
    for f in Factor::all() {
        let from = before.level(*f);
        let to = after.level(*f);
        if (to - from).abs() > 1e-9 {
            let note = after
                .factors
                .get(f)
                .map(|w| w.note.clone())
                .unwrap_or_default();
            changes.push(FieldChange {
                factor: *f,
                from,
                to,
                note,
            });
        }
    }
    let summary = if changes.is_empty() {
        "(no factor changed)".to_string()
    } else {
        changes
            .iter()
            .map(|c| format!("{} {:+.2}", c.factor.label(), c.to - c.from))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let explanation = format!(
        "Integrated revision {}→{}: {} factor(s) moved. {}",
        before.revision,
        after.revision,
        changes.len(),
        summary
    );
    Delta {
        changes,
        summary,
        explanation,
    }
}

// ----------------------------------------------------------------------------
// Generate / Simulate (relational)
// ----------------------------------------------------------------------------

/// Objective score in [0,1] for a set of named metrics under an objective.
pub fn score_objective(objective: &Objective, metrics: &HashMap<String, f64>) -> f64 {
    let mut total_w = 0.0;
    let mut acc = 0.0;
    for c in &objective.criteria {
        let v = metrics.get(&c.name).copied().unwrap_or(0.0).clamp(0.0, 1.0);
        let contrib = if c.maximize { v } else { 1.0 - v };
        acc += c.weight * contrib;
        total_w += c.weight;
    }
    if total_w == 0.0 {
        0.0
    } else {
        acc / total_w
    }
}

/// Seed an assumption's confidence from the world model: the mean positive
/// belief over matching facts (asserts=true → +confidence; asserts=false →
/// 1−confidence). Falls back to a neutral 0.7 when no fact matches.
fn assumption_confidence(world: &WorldState, subject: &Option<String>) -> f64 {
    match subject {
        None => 0.7,
        Some(s) => {
            let matched: Vec<&Fact> = world.facts.iter().filter(|f| f.subject == *s).collect();
            if matched.is_empty() {
                return 0.7;
            }
            let pos: f64 = matched
                .iter()
                .map(|f| {
                    if f.asserts {
                        f.confidence
                    } else {
                        1.0 - f.confidence
                    }
                })
                .sum();
            (pos / matched.len() as f64).clamp(0.0, 1.0)
        }
    }
}

/// Lightweight, transparent relational simulation: apply each strategy lever,
/// propagate it along the relationship graph one hop, derive metrics, and score
/// against the objective. This is a *toy* model inside the hypothetical frame —
/// it is not a real social simulator.
pub fn simulate(world: &WorldState, strategy: &Strategy, objective: &Objective) -> Outcome {
    // First-hop propagation of lever effects through relationships.
    let mut deltas: HashMap<Factor, f64> = strategy.levers.clone();
    for rel in &world.relationships {
        if let Some(&d) = deltas.get(&rel.from) {
            if d.abs() > 1e-9 {
                *deltas.entry(rel.to).or_insert(0.0) += rel.strength * d;
            }
        }
    }

    let mut post: HashMap<Factor, f64> = HashMap::new();
    for f in Factor::all() {
        let base = world.level(*f);
        let d = deltas.get(f).copied().unwrap_or(0.0);
        post.insert(*f, clamp1(base + d));
    }

    let mut metrics = HashMap::new();
    metrics.insert(
        "human_security".to_string(),
        clamp1(1.0 - 0.6 * post[&Factor::AutonomousWeapons] - 0.4 * post[&Factor::Cybercrime]),
    );
    metrics.insert(
        "privacy".to_string(),
        clamp1(1.0 - 0.7 * post[&Factor::UbiquitousSurveillance]),
    );
    metrics.insert(
        "equity".to_string(),
        clamp1(1.0 - 0.8 * post[&Factor::EconomicInequality]),
    );
    metrics.insert(
        "autonomy".to_string(),
        clamp1(
            0.5 + 0.25 * post[&Factor::NeuralInterfaces]
                + 0.25 * post[&Factor::AdvancedProsthetics]
                - 0.3 * post[&Factor::Megacorporations],
        ),
    );
    metrics.insert(
        "resilience".to_string(),
        clamp1(0.4 + 0.3 * post[&Factor::AiAgents] - 0.3 * post[&Factor::Cybercrime]),
    );
    let predicted_score = score_objective(objective, &metrics);
    let narrative = format!(
        "Under '{}', simulated post-intervention metrics: privacy {:.2}, equity {:.2}, \
         human_security {:.2}, autonomy {:.2}, resilience {:.2}. Objective score {:.2}.",
        strategy.name,
        metrics["privacy"],
        metrics["equity"],
        metrics["human_security"],
        metrics["autonomy"],
        metrics["resilience"],
        predicted_score,
    );
    Outcome {
        metrics,
        narrative,
        predicted_score,
    }
}

/// Deterministic heuristic generators for the cyberpunk scenario. Returns
/// candidate strategies with levers, costs, risks, and critical assumptions
/// whose confidence is seeded from the current world facts.
pub fn generate_strategies(
    world: &WorldState,
    objective: &Objective,
    _constraints: &[Constraint],
    _resources: &Resource,
) -> Vec<Strategy> {
    let mut strategies = Vec::new();

    let mut s1 = Strategy {
        id: "S1".into(),
        name: "Mandatory Autonomous-Weapons Treaty".into(),
        description:
            "International binding treaty placing autonomous weapons under human command authority \
             with independent verification and corporate liability."
                .into(),
        proposer: "deterministic-heuristic".into(),
        assumptions: vec![
            Assumption {
                statement: "Treaty compliance exceeds 60% within five years.".into(),
                formal_claim: Some("0.65 > 0.60".into()),
                subject: Some("autonomous-weapons-treaty".into()),
                critical: true,
                impact: 0.4,
                confidence: 0.0,
            },
            Assumption {
                statement: "Megacorp lobbying does not block enforcement.".into(),
                formal_claim: None,
                subject: Some("megacorp-lobbying".into()),
                critical: true,
                impact: 0.3,
                confidence: 0.0,
            },
        ],
        cost: Resource {
            tokens: 0,
            budget_usd: 2_000_000.0,
            political_capital: 0.7,
        },
        risks: vec![Risk {
            description: "Enforcement captured by the firms it regulates.".into(),
            probability: 0.5,
            impact: 0.6,
        }],
        levers: HashMap::new(),
        expected_outcome: Outcome {
            metrics: HashMap::new(),
            narrative: String::new(),
            predicted_score: 0.0,
        },
    };
    s1.levers.insert(Factor::AutonomousWeapons, -0.4);
    s1.levers.insert(Factor::Megacorporations, -0.1);
    s1.levers.insert(Factor::AiAgents, -0.05);

    let mut s2 = Strategy {
        id: "S2".into(),
        name: "Public Data Trust & Surveillance Audit".into(),
        description: "Independent oversight of sensing infrastructure; citizens hold portable, \
             cryptographic digital identity; surveillance logs are auditable."
            .into(),
        proposer: "deterministic-heuristic".into(),
        assumptions: vec![
            Assumption {
                statement: "Audit authority is granted meaningful subpoena power.".into(),
                formal_claim: Some("0.70 > 0.50".into()),
                subject: Some("surveillance-oversight".into()),
                critical: true,
                impact: 0.35,
                confidence: 0.0,
            },
            Assumption {
                statement: "Adoption of portable identity exceeds 50% of population.".into(),
                formal_claim: None,
                subject: Some("portable-identity-adoption".into()),
                critical: true,
                impact: 0.25,
                confidence: 0.0,
            },
        ],
        cost: Resource {
            tokens: 0,
            budget_usd: 800_000.0,
            political_capital: 0.4,
        },
        risks: vec![Risk {
            description: "Trust captured or rendered performative.".into(),
            probability: 0.4,
            impact: 0.4,
        }],
        levers: HashMap::new(),
        expected_outcome: Outcome {
            metrics: HashMap::new(),
            narrative: String::new(),
            predicted_score: 0.0,
        },
    };
    s2.levers.insert(Factor::UbiquitousSurveillance, -0.4);
    s2.levers.insert(Factor::Cybercrime, -0.2);
    s2.levers.insert(Factor::DigitalIdentity, 0.1);

    let mut s3 = Strategy {
        id: "S3".into(),
        name: "Open Neural-Interface Standards + Public Prosthetics".into(),
        description:
            "Open standards and public provisioning of neural interfaces and prosthetics to \
             prevent a neuro-divide and neural-data enclosure."
                .into(),
        proposer: "deterministic-heuristic".into(),
        assumptions: vec![Assumption {
            statement: "Standards are adopted before neural-data markets lock in.".into(),
            formal_claim: None,
            subject: Some("neural-standards-adopted".into()),
            critical: true,
            impact: 0.3,
            confidence: 0.0,
        }],
        cost: Resource {
            tokens: 0,
            budget_usd: 1_200_000.0,
            political_capital: 0.3,
        },
        risks: vec![Risk {
            description: "Standards ignored by dominant platforms.".into(),
            probability: 0.45,
            impact: 0.35,
        }],
        levers: HashMap::new(),
        expected_outcome: Outcome {
            metrics: HashMap::new(),
            narrative: String::new(),
            predicted_score: 0.0,
        },
    };
    s3.levers.insert(Factor::NeuralInterfaces, 0.1);
    s3.levers.insert(Factor::AdvancedProsthetics, 0.2);
    s3.levers.insert(Factor::EconomicInequality, -0.2);
    s3.levers.insert(Factor::Megacorporations, -0.1);

    let mut s4 = Strategy {
        id: "S4".into(),
        name: "Solidarity Economy & Redistribution".into(),
        description: "Progressive redistribution, public augmentation access, and worker-owned \
             platforms to blunt extreme inequality."
            .into(),
        proposer: "deterministic-heuristic".into(),
        assumptions: vec![Assumption {
            statement: "Redistribution is politically sustainable across cycles.".into(),
            formal_claim: None,
            subject: Some("redistribution-sustainable".into()),
            critical: true,
            impact: 0.4,
            confidence: 0.0,
        }],
        cost: Resource {
            tokens: 0,
            budget_usd: 3_000_000.0,
            political_capital: 0.6,
        },
        risks: vec![Risk {
            description: "Capital flight or informalization of labor.".into(),
            probability: 0.5,
            impact: 0.5,
        }],
        levers: HashMap::new(),
        expected_outcome: Outcome {
            metrics: HashMap::new(),
            narrative: String::new(),
            predicted_score: 0.0,
        },
    };
    s4.levers.insert(Factor::EconomicInequality, -0.5);
    s4.levers.insert(Factor::CyberneticAugmentation, 0.1);

    for s in [&mut s1, &mut s2, &mut s3, &mut s4] {
        for a in &mut s.assumptions {
            a.confidence = assumption_confidence(world, &a.subject);
        }
        s.expected_outcome = simulate(world, s, objective);
    }

    strategies.push(s1);
    strategies.push(s2);
    strategies.push(s3);
    strategies.push(s4);
    strategies
}

// ----------------------------------------------------------------------------
// Verify (trusted deterministic L)
// ----------------------------------------------------------------------------

/// Screen a strategy's text with the `cid` Gate (provenance / consistency /
/// injection). Returns the overall outcome string.
fn gate_screen(text: &str) -> String {
    let pins = PinField::new();
    let kb = KnowledgeBase::new();
    let candidate = TokenCandidate::new(0, text, 0.5);
    let mut ball = Ball::new(candidate);
    validate_candidate(&mut ball, &pins.pins, "strategy-screen", text, &kb);
    match ball.overall_outcome() {
        GateOutcome::Pass => "pass",
        GateOutcome::Fail => "fail",
        GateOutcome::Unevaluable => "unevaluable",
    }
    .to_string()
}

/// Verify every formalizable critical assumption with the Proof verifier and
/// screen the strategy text with the Gate.
pub fn verify_strategy(world: &WorldState, strategy: &Strategy) -> StrategyVerification {
    let corpus = world.corpus_hash();
    let gate_outcome = gate_screen(&strategy.description);

    let mut assumption_verifs = Vec::new();
    let mut critical_unverified = false;
    for a in &strategy.assumptions {
        let (verdict, proof) = match &a.formal_claim {
            Some(claim) => {
                let env = verify_proposal_envelope(
                    claim,
                    ProposalKind::Arithmetic,
                    corpus.as_bytes(),
                    vec![a.statement.clone()],
                );
                let verdict = match env.verdict {
                    ProofVerdict::Accepted => "verified",
                    ProofVerdict::Refused => "refuted",
                    ProofVerdict::Unevaluable => "unevaluable",
                };
                (verdict.to_string(), Some(env))
            }
            None => ("unevaluable".to_string(), None),
        };
        if a.critical && verdict != "verified" {
            critical_unverified = true;
        }
        assumption_verifs.push(AssumptionVerification {
            statement: a.statement.clone(),
            verdict,
            proof,
        });
    }

    StrategyVerification {
        strategy_id: strategy.id.clone(),
        gate_outcome,
        assumptions: assumption_verifs,
        critical_unverified,
    }
}

// ----------------------------------------------------------------------------
// Select / Value-of-information / Explanation / Alternatives
// ----------------------------------------------------------------------------

/// Discount an objective score by assumption confidence: `0.5 + 0.5 * c`, so a
/// fully-confident strategy is unscaled and a zero-confidence one is halved.
fn confidence_multiplier(c: f64) -> f64 {
    0.5 + 0.5 * c.clamp(0.0, 1.0)
}

fn mean_critical_confidence(s: &Strategy) -> f64 {
    let crit: Vec<f64> = s
        .assumptions
        .iter()
        .filter(|a| a.critical)
        .map(|a| a.confidence)
        .collect();
    if crit.is_empty() {
        1.0
    } else {
        crit.iter().sum::<f64>() / crit.len() as f64
    }
}

/// Rank strategies by confidence-discounted objective score. Returns
/// `(ranking, recommended, keep_alternatives, alternatives)`. Alternatives are
/// preserved whenever uncertainty is high (unverified critical assumptions, low
/// mean confidence, or a thin margin to the runner-up).
pub fn select(
    strategies: &[Strategy],
    verifications: &[StrategyVerification],
) -> (
    Vec<RankedStrategy>,
    Option<RankedStrategy>,
    bool,
    Vec<RankedStrategy>,
) {
    let verify_by_id: HashMap<&str, &StrategyVerification> = verifications
        .iter()
        .map(|v| (v.strategy_id.as_str(), v))
        .collect();

    let mut ranked: Vec<RankedStrategy> = strategies
        .iter()
        .map(|s| {
            let v = verify_by_id.get(s.id.as_str());
            let gate_failed = v.map(|x| x.gate_outcome == "fail").unwrap_or(false);
            let refuted = v
                .map(|x| {
                    x.assumptions.iter().any(|a| {
                        a.verdict == "refuted"
                            && s.assumptions
                                .iter()
                                .any(|sa| sa.critical && sa.statement == a.statement)
                    })
                })
                .unwrap_or(false);
            let admissible = !gate_failed && !refuted;
            let conf = mean_critical_confidence(s);
            let score = s.expected_outcome.predicted_score * confidence_multiplier(conf);
            let rationale = format!(
                "objective {:.2} x confidence {:.2} (gate '{}', critical_unverified={})",
                s.expected_outcome.predicted_score,
                confidence_multiplier(conf),
                v.map(|x| x.gate_outcome.as_str()).unwrap_or("?"),
                v.map(|x| x.critical_unverified).unwrap_or(false)
            );
            RankedStrategy {
                rank: 0,
                strategy_id: s.id.clone(),
                name: s.name.clone(),
                score,
                confidence: conf,
                admissible,
                rationale,
            }
        })
        .collect();

    ranked.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, r) in ranked.iter_mut().enumerate() {
        r.rank = i + 1;
    }

    let recommended = ranked
        .iter()
        .find(|r| r.admissible)
        .cloned()
        .or_else(|| ranked.first().cloned());

    let margin = match (ranked.first(), ranked.get(1)) {
        (Some(a), Some(b)) => a.score - b.score,
        _ => 1.0,
    };
    let uncertainty_high = ranked
        .first()
        .map(|r| r.confidence < 0.7 || margin < 0.05)
        .unwrap_or(false)
        || recommended
            .as_ref()
            .map(|r| {
                verify_by_id
                    .get(r.strategy_id.as_str())
                    .map(|x| x.critical_unverified)
                    .unwrap_or(false)
            })
            .unwrap_or(false);

    let alternatives: Vec<RankedStrategy> = if uncertainty_high {
        ranked.iter().skip(1).take(3).cloned().collect()
    } else {
        Vec::new()
    };

    (ranked, recommended, uncertainty_high, alternatives)
}

/// Value-of-information: for each world factor currently held with low
/// confidence, flip it to its opposite extreme and re-rank, measuring how much
/// the *decision* (recommended strategy and its score) swings. High swing =
/// high information value.
pub fn value_of_information(
    world: &WorldState,
    objective: &Objective,
    resources: &Resource,
) -> Vec<VoI> {
    let base_strategies = generate_strategies(world, objective, &[], resources);
    let base_verifs: Vec<StrategyVerification> = base_strategies
        .iter()
        .map(|s| verify_strategy(world, s))
        .collect();
    let (_base_ranking, base_rec, _, _) = select(&base_strategies, &base_verifs);
    let base_rec_id = base_rec.as_ref().map(|r| r.strategy_id.clone());
    let base_rec_score = base_rec.as_ref().map(|r| r.score).unwrap_or(0.0);

    let mut vois = Vec::new();
    for f in Factor::all() {
        let conf = world.confidence_of(*f);
        if conf >= 0.7 {
            continue;
        }
        let mut perturbed = world.clone();
        let lvl = perturbed.level(*f);
        let new_lvl = if lvl >= 0.5 { 0.0 } else { 1.0 };
        perturbed.factors.insert(
            *f,
            WorldFactor {
                level: new_lvl,
                note: "VOI perturbation".to_string(),
                confidence: conf,
            },
        );
        let strats = generate_strategies(&perturbed, objective, &[], resources);
        let vers: Vec<StrategyVerification> = strats
            .iter()
            .map(|s| verify_strategy(&perturbed, s))
            .collect();
        let (ranking, rec, _, _) = select(&strats, &vers);
        let rec_id_now = rec
            .as_ref()
            .map(|r| r.strategy_id.clone())
            .unwrap_or_else(|| ranking[0].strategy_id.clone());
        let rec_changed = rec.as_ref().map(|r| r.strategy_id.clone()) != base_rec_id;
        let new_score = rec.as_ref().map(|r| r.score).unwrap_or(0.0);
        let swing = if rec_changed { 1.0 } else { 0.0 };
        let score_delta = (new_score - base_rec_score).abs();
        let info_value = 0.6 * swing + 0.4 * score_delta.min(1.0);
        vois.push(VoI {
            unknown: format!("factor '{}' (current confidence {:.2})", f.label(), conf),
            info_value,
            rationale: format!(
                "Flipping '{}' to its opposite extreme {} the recommended strategy {} \
                 and moves its score by {:.2}.",
                f.label(),
                if rec_changed {
                    "changes"
                } else {
                    "does not change"
                },
                if rec_changed {
                    format!(
                        "from {} to {}",
                        base_rec_id.clone().unwrap_or_default(),
                        rec_id_now
                    )
                } else {
                    "at all".to_string()
                },
                score_delta
            ),
        });
    }
    vois.sort_by(|a, b| {
        b.info_value
            .partial_cmp(&a.info_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    vois
}

/// Identify the most valuable missing information: critical assumptions that
/// could not be verified, weighted by their impact.
pub fn most_valuable_missing_info(
    strategies: &[Strategy],
    verifications: &[StrategyVerification],
) -> Vec<MissingInfo> {
    let verify_by_id: HashMap<&str, &StrategyVerification> = verifications
        .iter()
        .map(|v| (v.strategy_id.as_str(), v))
        .collect();
    let mut items = Vec::new();
    for s in strategies {
        let v = match verify_by_id.get(s.id.as_str()) {
            Some(v) => v,
            None => continue,
        };
        for (a, av) in s.assumptions.iter().zip(v.assumptions.iter()) {
            if a.critical && av.verdict != "verified" {
                items.push(MissingInfo {
                    question: format!(
                        "[{}] What evidence would confirm or refute: {}?",
                        s.name, a.statement
                    ),
                    why_valuable: format!(
                        "Critical assumption; if false it moves the objective by ~{:.2}.",
                        a.impact
                    ),
                    info_value: a.impact,
                });
            }
        }
    }
    items.sort_by(|a, b| {
        b.info_value
            .partial_cmp(&a.info_value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    items
}

/// Produce a human-readable explanation of why the top strategy is preferred and
/// how it compares to the runner-up.
pub fn explain_preference(
    ranking: &[RankedStrategy],
    strategies: &[Strategy],
    verifications: &[StrategyVerification],
) -> String {
    if ranking.is_empty() {
        return "No strategies were generated.".to_string();
    }
    let top = &ranking[0];
    let top_strat = strategies.iter().find(|s| s.id == top.strategy_id).cloned();
    let mut s = format!(
        "Preferred strategy: {} (adjusted score {:.2}, mean critical-assumption confidence {:.2}).",
        top.name, top.score, top.confidence
    );
    s.push_str(&format!(" Rationale: {}.", top.rationale));
    if let Some(sc) = ranking.get(1) {
        let gap = top.score - sc.score;
        s.push_str(&format!(
            " Runner-up is {} (score {:.2}); {} is preferred by {:.2} under the stated objective \
             and confidence discounting.",
            sc.name, sc.score, top.name, gap
        ));
    }
    if let Some(ts) = top_strat {
        let risks = ts
            .risks
            .iter()
            .map(|r| r.description.clone())
            .collect::<Vec<_>>()
            .join("; ");
        s.push_str(&format!(" Key risks: {}.", risks));
    }
    let v = verifications
        .iter()
        .find(|x| x.strategy_id == top.strategy_id);
    if v.map(|x| x.critical_unverified).unwrap_or(false) {
        s.push_str(
            " Uncertainty is high: critical assumptions are unverified or contradicted, so this \
             recommendation is provisional and alternatives are retained.",
        );
    }
    s
}

// ----------------------------------------------------------------------------
// Orchestration
// ----------------------------------------------------------------------------

/// Default objective for the cyberpunk scenario: maximize human security,
/// privacy, equity, autonomy, and resilience.
pub fn cyberpunk_objective() -> Objective {
    Objective {
        name: "minimize harm / maximize durable flourishing (hypothetical)".into(),
        criteria: vec![
            Criterion {
                name: "human_security".into(),
                weight: 0.25,
                maximize: true,
            },
            Criterion {
                name: "privacy".into(),
                weight: 0.2,
                maximize: true,
            },
            Criterion {
                name: "equity".into(),
                weight: 0.25,
                maximize: true,
            },
            Criterion {
                name: "autonomy".into(),
                weight: 0.15,
                maximize: true,
            },
            Criterion {
                name: "resilience".into(),
                weight: 0.15,
                maximize: true,
            },
        ],
    }
}

/// Default relationship graph for the cyberpunk scenario (factor → factor, signed).
pub fn cyberpunk_relationships() -> Vec<Relationship> {
    vec![
        Relationship {
            from: Factor::Megacorporations,
            to: Factor::EconomicInequality,
            strength: 0.6,
        },
        Relationship {
            from: Factor::Megacorporations,
            to: Factor::AutonomousWeapons,
            strength: 0.3,
        },
        Relationship {
            from: Factor::EconomicInequality,
            to: Factor::Cybercrime,
            strength: 0.5,
        },
        Relationship {
            from: Factor::AiAgents,
            to: Factor::Cybercrime,
            strength: -0.2,
        },
        Relationship {
            from: Factor::NeuralInterfaces,
            to: Factor::DigitalIdentity,
            strength: 0.4,
        },
        Relationship {
            from: Factor::CyberneticAugmentation,
            to: Factor::AdvancedProsthetics,
            strength: 0.5,
        },
        Relationship {
            from: Factor::UbiquitousSurveillance,
            to: Factor::AutonomousWeapons,
            strength: 0.2,
        },
    ]
}

/// Default uncertainties for the cyberpunk scenario.
pub fn cyberpunk_uncertainties() -> Vec<Uncertainty> {
    vec![
        Uncertainty {
            id: "U-inequality".into(),
            description: "True extent of wealth concentration and its trajectory.".into(),
            possible_values: vec!["worsening".into(), "stable".into(), "improving".into()],
            confidence: 0.4,
            impact: 0.4,
        },
        Uncertainty {
            id: "U-corp-capture".into(),
            description: "Degree of corporate capture of governance.".into(),
            possible_values: vec!["high".into(), "moderate".into(), "low".into()],
            confidence: 0.5,
            impact: 0.3,
        },
    ]
}

/// Build the initial Cyberpunk-RED/2077-as-real hypothetical world model.
///
/// Levels are transparent estimates of a near-future society from real-world
/// analogues; facts carry confidence; the relationship graph and uncertainties
/// are attached. None of it is a claim about reality.
pub fn initial_cyberpunk_world() -> WorldState {
    let mut factors = HashMap::new();
    let put = |factors: &mut HashMap<Factor, WorldFactor>,
               f: Factor,
               level: f64,
               conf: f64,
               note: &str| {
        factors.insert(
            f,
            WorldFactor {
                level,
                note: note.to_string(),
                confidence: conf,
            },
        );
    };
    put(
        &mut factors,
        Factor::CyberneticAugmentation,
        0.55,
        0.6,
        "Widespread elective augmentation; uneven access by class.",
    );
    put(
        &mut factors,
        Factor::UbiquitousSurveillance,
        0.8,
        0.7,
        "Pervasive corporate + state sensing; weak oversight.",
    );
    put(
        &mut factors,
        Factor::Megacorporations,
        0.85,
        0.5,
        "A handful of firms dominate markets and private governance.",
    );
    put(
        &mut factors,
        Factor::AutonomousWeapons,
        0.5,
        0.5,
        "Automated drones/defense systems; ambiguous chain of command.",
    );
    put(
        &mut factors,
        Factor::AiAgents,
        0.7,
        0.6,
        "Autonomous agents run logistics, hiring, and moderation.",
    );
    put(
        &mut factors,
        Factor::NeuralInterfaces,
        0.45,
        0.5,
        "Consumer brain-computer links; emerging neural-data markets.",
    );
    put(
        &mut factors,
        Factor::EconomicInequality,
        0.9,
        0.4,
        "Extreme wealth concentration; precarious underclass.",
    );
    put(
        &mut factors,
        Factor::DigitalIdentity,
        0.6,
        0.6,
        "Mandatory IDs; identity increasingly a corporate asset.",
    );
    put(
        &mut factors,
        Factor::Cybercrime,
        0.65,
        0.5,
        "Industrialized ransomware and neural-data theft.",
    );
    put(
        &mut factors,
        Factor::AdvancedProsthetics,
        0.5,
        0.6,
        "High-end prosthetics exceed biology; gated by cost.",
    );

    let facts = vec![
        Fact::new(
            "F1",
            "autonomous-weapons-treaty",
            true,
            "An autonomous-weapons treaty is in force.",
            0.4,
            "HYPOTHETICAL",
        ),
        Fact::new(
            "F2",
            "redistribution-sustainable",
            true,
            "Redistribution is politically sustainable.",
            0.8,
            "HYPOTHETICAL",
        ),
        Fact::new(
            "F3",
            "surveillance-oversight",
            false,
            "Independent surveillance oversight does not yet exist.",
            0.3,
            "HYPOTHETICAL",
        ),
    ];

    WorldState {
        hypothetical: true,
        label: "HYPOTHETICAL — Cyberpunk RED/2077 as if real: near-future society model from real-world analogues".to_string(),
        factors,
        facts,
        relationships: cyberpunk_relationships(),
        uncertainties: cyberpunk_uncertainties(),
        assumptions: vec![
            "All factors are transparent estimates, not measured facts.".to_string(),
            "The fiction is a framing device; analogues are drawn from present trends.".to_string(),
        ],
        revision: 0,
    }
}

/// Run one full adaptive cycle on a (possibly already-updated) world.
pub fn run_adaptive_cycle(
    world_before: WorldState,
    world_after: WorldState,
    delta: Delta,
    conflicts: Vec<Conflict>,
    objective: Objective,
    constraints: Vec<Constraint>,
    resources: Resource,
) -> StrategyCycleReport {
    let strategies = generate_strategies(&world_after, &objective, &constraints, &resources);
    let verifications: Vec<StrategyVerification> = strategies
        .iter()
        .map(|s| verify_strategy(&world_after, s))
        .collect();
    let (ranking, recommended, keep_alternatives, alternatives) =
        select(&strategies, &verifications);
    let missing = most_valuable_missing_info(&strategies, &verifications);
    let voi = value_of_information(&world_after, &objective, &resources);
    let explanation = explain_preference(&ranking, &strategies, &verifications);

    let caveat = "Recommendation is ranked by an explicit, contestable objective function over a \
                  HYPOTHETICAL model with transparent estimates and confidence-discounted scores. \
                  It is the best available under stated assumptions, not a claim of universal \
                  optimality. Unverified or contradicted critical assumptions remain open proof \
                  obligations."
        .to_string();

    StrategyCycleReport {
        scenario_label: world_after.label.clone(),
        world_before,
        world_after: world_after.clone(),
        delta,
        objective,
        constraints,
        facts: world_after.facts.clone(),
        relationships: world_after.relationships.clone(),
        uncertainties: world_after.uncertainties.clone(),
        conflicts,
        strategies,
        verifications,
        ranking,
        recommended,
        alternatives,
        keep_alternatives,
        value_of_information: voi,
        missing_information: missing,
        preference_explanation: explanation,
        caveat,
    }
}

/// Convenience: build the initial world, ingest one hypothetical event, and run
/// the full loop end-to-end.
pub fn run_cyberpunk_scenario(event: &WorldEvent) -> StrategyCycleReport {
    let world0 = initial_cyberpunk_world();
    let integrated = integrate_event(&world0, event);
    let world1 = integrated.world;
    let delta = integrated.delta;
    run_adaptive_cycle(
        world0,
        world1,
        delta,
        integrated.conflicts,
        cyberpunk_objective(),
        Vec::new(),
        Resource::default_pool(),
    )
}

impl StrategyCycleReport {
    /// Serialize the full record as pretty JSON (machine-readable).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("StrategyCycleReport is serializable")
    }
}

/// Stateful wrapper enabling repeated `Observe → … → Observe Again` iterations.
pub struct WorldModel {
    pub world: WorldState,
    pub observations: Vec<WorldEvent>,
    pub decisions: Vec<RankedStrategy>,
    /// (recommended strategy id, adjusted score, iteration) per step — used to
    /// measure whether the recommendation adapts as information arrives.
    pub recommendation_history: Vec<(String, f64, u64)>,
    pub iteration: u64,
}

impl WorldModel {
    pub fn new(world: WorldState) -> Self {
        WorldModel {
            world,
            observations: Vec::new(),
            decisions: Vec::new(),
            recommendation_history: Vec::new(),
            iteration: 0,
        }
    }

    /// Observe + Integrate a hypothetical event; returns the detected delta and
    /// any detected contradictions.
    pub fn observe(&mut self, event: WorldEvent) -> IntegrationResult {
        let before = self.world.clone();
        let integrated = integrate_event(&before, &event);
        self.world = integrated.world.clone();
        self.observations.push(event);
        integrated
    }

    /// Generate → Simulate → Verify → Select one round of strategies.
    pub fn step(
        &mut self,
        objective: Objective,
        constraints: Vec<Constraint>,
        resources: Resource,
    ) -> StrategyCycleReport {
        let before = self.world.clone();
        let report = run_adaptive_cycle(
            before,
            self.world.clone(),
            Delta::empty(),
            Vec::new(),
            objective,
            constraints,
            resources,
        );
        if let Some(r) = &report.recommended {
            self.recommendation_history
                .push((r.strategy_id.clone(), r.score, self.iteration));
            self.decisions.push(r.clone());
        }
        self.iteration += 1;
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_event() -> WorldEvent {
        WorldEvent {
            evidence: Evidence {
                id: "E0".into(),
                provenance: "HYPOTHETICAL SCENARIO EVENT".into(),
                content: "Baseline observation of the near-future society.".into(),
                confidence: 0.6,
            },
            factor_adjustments: vec![],
            new_facts: vec![],
            new_assumptions: vec![],
        }
    }

    #[test]
    fn world_is_explicitly_hypothetical_and_structured() {
        let w = initial_cyberpunk_world();
        assert!(w.hypothetical);
        assert!(w.label.starts_with("HYPOTHETICAL"));
        assert_eq!(w.factors.len(), 10);
        assert_eq!(w.facts.len(), 3);
        assert!(!w.relationships.is_empty());
        assert!(!w.uncertainties.is_empty());
    }

    #[test]
    fn integrate_revises_confidence_and_detects_contradiction() {
        let w0 = initial_cyberpunk_world();
        // F2 currently asserts redistribution-sustainable (true) at confidence 0.8.
        let f2 = w0
            .facts
            .iter()
            .find(|f| f.subject == "redistribution-sustainable")
            .unwrap();
        assert!(f2.asserts && (f2.confidence - 0.8).abs() < 1e-6);

        // Contradicting event: asserts NOT sustainable at confidence 0.7.
        let ev = WorldEvent {
            evidence: Evidence {
                id: "E1".into(),
                provenance: "HYPOTHETICAL".into(),
                content: "Redistribution proves politically unsustainable.".into(),
                confidence: 0.7,
            },
            factor_adjustments: vec![],
            new_facts: vec![Fact::new(
                "F2b",
                "redistribution-sustainable",
                false,
                "Redistribution is NOT politically sustainable.",
                0.7,
                "HYPOTHETICAL",
            )],
            new_assumptions: vec![],
        };
        let res = integrate_event(&w0, &ev);
        assert_eq!(res.conflicts.len(), 1);
        let f2_after = res.world.facts.iter().find(|f| f.id == "F2").unwrap();
        assert!(f2_after.confidence < 0.8, "confidence must be discounted");
    }

    #[test]
    fn simulate_uses_relationships() {
        let w = initial_cyberpunk_world();
        let obj = cyberpunk_objective();
        let mut s = Strategy {
            id: "X".into(),
            name: "X".into(),
            description: "x".into(),
            proposer: "d".into(),
            assumptions: vec![],
            cost: Resource::default_pool(),
            risks: vec![],
            levers: HashMap::new(),
            expected_outcome: Outcome {
                metrics: HashMap::new(),
                narrative: String::new(),
                predicted_score: 0.0,
            },
        };
        s.levers.insert(Factor::Megacorporations, 0.5);
        let out = simulate(&w, &s, &obj);
        // Megacorporations↑ must also raise EconomicInequality via relationship.
        assert!(
            out.metrics["equity"] < 0.2,
            "relationship should worsen equity"
        );
    }

    #[test]
    fn cycle_generates_ranks_and_recommends_with_explanation() {
        let report = run_cyberpunk_scenario(&base_event());
        assert!(report.scenario_label.starts_with("HYPOTHETICAL"));
        assert!(report.strategies.len() >= 3);
        for w in report.ranking.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
        assert!(report.recommended.is_some());
        assert!(!report.preference_explanation.is_empty());
        // at least one critical assumption should be verifiable
        let any_verified = report
            .verifications
            .iter()
            .flat_map(|v| v.assumptions.iter())
            .any(|a| a.verdict == "verified");
        assert!(any_verified);
        // unverifiable critical assumptions surface as missing info
        assert!(!report.missing_information.is_empty());
        // value-of-information is populated
        assert!(!report.value_of_information.is_empty());
        // uncertainty is high at baseline (critical assumptions unevaluable) -> alternatives kept
        assert!(report.keep_alternatives);
        assert!(!report.alternatives.is_empty());
        assert!(report
            .caveat
            .contains("not a claim of universal optimality"));
        let json = report.to_json();
        let back: StrategyCycleReport = serde_json::from_str(&json).expect("round-trips");
        assert_eq!(back.strategies.len(), report.strategies.len());
    }

    #[test]
    fn verify_produces_tamper_evident_proof() {
        let w = initial_cyberpunk_world();
        let obj = cyberpunk_objective();
        let strategies = generate_strategies(&w, &obj, &[], &Resource::default_pool());
        let v = verify_strategy(&w, &strategies[0]);
        assert!(!v.gate_outcome.is_empty());
        assert!(v.assumptions.iter().any(|a| a.proof.is_some()));
    }

    #[test]
    fn incremental_information_changes_recommendation_rationally() {
        let mut model = WorldModel::new(initial_cyberpunk_world());
        let r0 = model.step(cyberpunk_objective(), Vec::new(), Resource::default_pool());
        let baseline = r0.recommended.as_ref().unwrap().strategy_id.clone();
        assert_eq!(baseline, "S4", "baseline should prefer redistribution");

        // New information contradicts S4's core assumption (redistribution not sustainable).
        let ev = WorldEvent {
            evidence: Evidence {
                id: "E2".into(),
                provenance: "HYPOTHETICAL".into(),
                content: "Field data shows redistribution is politically unsustainable.".into(),
                confidence: 0.7,
            },
            factor_adjustments: vec![(Factor::EconomicInequality, -0.05, "Partial relief.".into())],
            new_facts: vec![Fact::new(
                "F4",
                "redistribution-sustainable",
                false,
                "Redistribution is NOT politically sustainable.",
                0.7,
                "HYPOTHETICAL",
            )],
            new_assumptions: vec![],
        };
        let integ = model.observe(ev);
        assert!(
            !integ.conflicts.is_empty(),
            "contradiction must be detected"
        );
        let r1 = model.step(cyberpunk_objective(), Vec::new(), Resource::default_pool());

        // The recommendation must adapt: S4 should no longer be preferred.
        let after = r1.recommended.as_ref().unwrap().strategy_id.clone();
        assert_ne!(after, "S4", "contradictory evidence should displace S4");
        assert_ne!(
            after, baseline,
            "recommendation should change with new information"
        );
        assert_eq!(model.recommendation_history.len(), 2);
        assert!(model.recommendation_history[0].0 != model.recommendation_history[1].0);
        // S4's confidence-discounted score must have dropped.
        let s4_before = r0
            .ranking
            .iter()
            .find(|x| x.strategy_id == "S4")
            .unwrap()
            .score;
        let s4_after = r1
            .ranking
            .iter()
            .find(|x| x.strategy_id == "S4")
            .unwrap()
            .score;
        assert!(
            s4_after < s4_before,
            "S4 score should fall after contradiction"
        );
        // Top value-of-information should concern redistribution sustainability.
        assert!(
            r1.value_of_information[0]
                .unknown
                .contains("redistribution")
                || r1.missing_information[0]
                    .question
                    .contains("Redistribution")
        );
    }
}
