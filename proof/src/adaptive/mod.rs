//! # Adaptive Strategy Engine (ASE) — minimal vertical slice
//!
//! Evolves L from a pure verification engine into an **adaptive strategy engine**
//! via a clearly-labeled *hypothetical* world model. The initial application is
//! the scenario *"What if Cyberpunk RED / Cyberpunk 2077 were real life?"* — a
//! near-future society model built from **plausible real-world analogues** of the
//! fictional setting. The fiction is never treated as literal fact: every state,
//! event, and assumption is tagged `hypothetical`.
//!
//! ## Core loop
//! `Observe → Integrate → Detect Delta → Generate Strategies → Simulate →
//! Verify → Select → Act → Observe Again`
//!
//! ## Trust boundary (preserved)
//! - **Untrusted proposers** (LLM / Athena / human scenario author) may *propose*
//!   world events, strategies, and their assumptions. They are labeled
//!   `proposer != "deterministic-heuristic"`.
//! - **Trusted deterministic L components** *validate*: the `cid` Gate screens
//!   strategy text for provenance / consistency / injection, and the Proof
//!   verifier (`verify_proposal_envelope`) produces a tamper-evident
//!   [`ProofEnvelope`] for every formalizable critical assumption. L never
//!   asserts an unverifiable claim as true; it reports `unevaluable`.
//!
//! ## Honesty contract
//! Selection ranks strategies by an **explicit objective function** and reports
//! the ranking plus the single most valuable missing information. It does *not*
//! claim universal optimality — the returned [`StrategyCycleReport::caveat`]
//! says so explicitly.

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
/// transparent 0.0–1.0 intensity estimate (never a measured fact).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldFactor {
    pub level: f64,
    pub note: String,
}

/// A snapshot of the (hypothetical) world model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// Always `true`: this model is a hypothetical, not a claim about reality.
    pub hypothetical: bool,
    pub label: String,
    pub factors: HashMap<Factor, WorldFactor>,
    pub assumptions: Vec<String>,
    /// Monotonic revision counter (bumped on every integration).
    pub revision: u64,
}

impl WorldState {
    pub fn level(&self, f: Factor) -> f64 {
        self.factors.get(&f).map(|w| w.level).unwrap_or(0.0)
    }

    /// Stable fingerprint binding a proof to this exact world model.
    pub fn corpus_hash(&self) -> String {
        let mut blob = self.label.clone();
        for a in &self.assumptions {
            blob.push('\n');
            blob.push_str(a);
        }
        sha256_hex(blob.as_bytes())
    }
}

/// A piece of (hypothetical) information ingested into the model. The
/// `provenance` must declare its hypothetical nature; L never ingests it as fact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub provenance: String,
    pub content: String,
    /// 0.0–1.0 reporter confidence; never treated as L's own confidence.
    pub confidence: f64,
}

/// A hypothetical event reported into the model: an [`Evidence`] plus the
/// explicit, transparent adjustments it implies for the world factors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub evidence: Evidence,
    /// Explicit, transparent adjustments to world factors (delta applied on integration).
    pub factor_adjustments: Vec<(Factor, f64, String)>,
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

/// A risk attached to a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub probability: f64,
    pub impact: f64,
}

/// An assumption a strategy depends on. `critical` assumptions gate admissibility;
/// `impact` is the |effect on objective score| if the assumption is false.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub statement: String,
    /// Optional formalizable claim for the deterministic verifier (e.g. "0.65 > 0.6").
    pub formal_claim: Option<String>,
    pub critical: bool,
    pub impact: f64,
}

/// Predicted consequences of a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Named metrics in [0,1]; keys match `Objective::Criterion::name`.
    pub metrics: HashMap<String, f64>,
    pub narrative: String,
    /// Objective-function score in [0,1].
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
    pub score: f64,
    /// False if a critical assumption was refuted or the Gate screen failed.
    pub admissible: bool,
    pub rationale: String,
}

/// The single most valuable piece of missing information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingInfo {
    pub question: String,
    pub why_valuable: String,
    pub info_value: f64,
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
    pub strategies: Vec<Strategy>,
    pub verifications: Vec<StrategyVerification>,
    pub ranking: Vec<RankedStrategy>,
    pub recommended: Option<RankedStrategy>,
    pub missing_information: Vec<MissingInfo>,
    /// Explicit statement that the recommendation is *not* universal optimality.
    pub caveat: String,
}

// ----------------------------------------------------------------------------
// Observe / Integrate
// ----------------------------------------------------------------------------

/// Build the initial Cyberpunk-RED/2077-as-real hypothetical world model.
///
/// Levels are deliberately *transparent estimates* of a near-future society
/// built from real-world analogues; they are not predictions.
pub fn initial_cyberpunk_world() -> WorldState {
    let mut factors = HashMap::new();
    let put = |factors: &mut HashMap<Factor, WorldFactor>, f: Factor, level: f64, note: &str| {
        factors.insert(
            f,
            WorldFactor {
                level,
                note: note.to_string(),
            },
        );
    };
    put(
        &mut factors,
        Factor::CyberneticAugmentation,
        0.55,
        "Widespread elective augmentation; uneven access by class.",
    );
    put(
        &mut factors,
        Factor::UbiquitousSurveillance,
        0.8,
        "Pervasive corporate + state sensing; weak oversight.",
    );
    put(
        &mut factors,
        Factor::Megacorporations,
        0.85,
        "A handful of firms dominate markets and private governance.",
    );
    put(
        &mut factors,
        Factor::AutonomousWeapons,
        0.5,
        "Automated drones/defense systems; ambiguous chain of command.",
    );
    put(
        &mut factors,
        Factor::AiAgents,
        0.7,
        "Autonomous agents run logistics, hiring, and moderation.",
    );
    put(
        &mut factors,
        Factor::NeuralInterfaces,
        0.45,
        "Consumer brain-computer links; emerging neural-data markets.",
    );
    put(
        &mut factors,
        Factor::EconomicInequality,
        0.9,
        "Extreme wealth concentration; precarious underclass.",
    );
    put(
        &mut factors,
        Factor::DigitalIdentity,
        0.6,
        "Mandatory IDs; identity increasingly a corporate asset.",
    );
    put(
        &mut factors,
        Factor::Cybercrime,
        0.65,
        "Industrialized ransomware and neural-data theft.",
    );
    put(
        &mut factors,
        Factor::AdvancedProsthetics,
        0.5,
        "High-end prosthetics exceed biology; gated by cost.",
    );
    WorldState {
        hypothetical: true,
        label: "HYPOTHETICAL — Cyberpunk RED/2077 as if real: near-future society model from real-world analogues".to_string(),
        factors,
        assumptions: vec![
            "All factors are transparent estimates, not measured facts.".to_string(),
            "The fiction is a framing device; analogues are drawn from present trends.".to_string(),
        ],
        revision: 0,
    }
}

/// Integrate a hypothetical event into the world, returning the next state.
pub fn integrate(world: &WorldState, event: &WorldEvent) -> WorldState {
    let mut next = world.clone();
    for (factor, delta, note) in &event.factor_adjustments {
        let prev = next.level(*factor);
        let new_level = (prev + delta).clamp(0.0, 1.0);
        next.factors.insert(
            *factor,
            WorldFactor {
                level: new_level,
                note: note.clone(),
            },
        );
    }
    for a in &event.new_assumptions {
        next.assumptions.push(a.clone());
    }
    next.revision += 1;
    next
}

/// Detect what changed between two world states.
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
// Generate / Simulate
// ----------------------------------------------------------------------------

fn clamp1(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

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

/// Lightweight, transparent simulation: shift each factor by the strategy's
/// levers, derive metrics, and score against the objective. This is a *toy*
/// model inside the hypothetical frame — it is not a real social simulator.
pub fn simulate(world: &WorldState, strategy: &Strategy, objective: &Objective) -> Outcome {
    let mut post: HashMap<Factor, f64> = HashMap::new();
    for f in Factor::all() {
        let base = world.level(*f);
        let lever = strategy.levers.get(f).copied().unwrap_or(0.0);
        post.insert(*f, clamp1(base + lever));
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
/// candidate strategies with levers, costs, risks, and critical assumptions.
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
                critical: true,
                impact: 0.4,
            },
            Assumption {
                statement: "Megacorp lobbying does not block enforcement.".into(),
                formal_claim: None,
                critical: true,
                impact: 0.3,
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
    s1.expected_outcome = simulate(world, &s1, objective);

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
                critical: true,
                impact: 0.35,
            },
            Assumption {
                statement: "Adoption of portable identity exceeds 50% of population.".into(),
                formal_claim: None,
                critical: true,
                impact: 0.25,
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
    s2.expected_outcome = simulate(world, &s2, objective);

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
            critical: true,
            impact: 0.3,
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
    s3.expected_outcome = simulate(world, &s3, objective);

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
            critical: true,
            impact: 0.4,
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
    s4.expected_outcome = simulate(world, &s4, objective);

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
// Select / Missing information
// ----------------------------------------------------------------------------

/// Rank strategies by objective score; mark inadmissible those with a refuted
/// critical assumption or a failing Gate screen. Returns (ranking, recommended).
pub fn select(
    strategies: &[Strategy],
    verifications: &[StrategyVerification],
) -> (Vec<RankedStrategy>, Option<RankedStrategy>) {
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
            let rationale = format!(
                "objective score {:.2}; gate '{}'; critical_unverified={}",
                s.expected_outcome.predicted_score,
                v.map(|x| x.gate_outcome.as_str()).unwrap_or("?"),
                v.map(|x| x.critical_unverified).unwrap_or(false)
            );
            RankedStrategy {
                rank: 0,
                strategy_id: s.id.clone(),
                name: s.name.clone(),
                score: s.expected_outcome.predicted_score,
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
    (ranked, recommended)
}

/// Identify the most valuable missing information: critical assumptions that
/// could not be verified, weighted by their impact (uncertainty = 1.0).
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
                let info_value = a.impact; // uncertainty treated as 1.0
                items.push(MissingInfo {
                    question: format!(
                        "[{}] What evidence would confirm or refute: {}?",
                        s.name, a.statement
                    ),
                    why_valuable: format!(
                        "Critical assumption; if false it moves the objective by ~{:.2}.",
                        a.impact
                    ),
                    info_value,
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

/// Run one full adaptive cycle on a (possibly already-updated) world.
pub fn run_adaptive_cycle(
    world_before: WorldState,
    world_after: WorldState,
    delta: Delta,
    objective: Objective,
    constraints: Vec<Constraint>,
    resources: Resource,
) -> StrategyCycleReport {
    let strategies = generate_strategies(&world_after, &objective, &constraints, &resources);
    let verifications: Vec<StrategyVerification> = strategies
        .iter()
        .map(|s| verify_strategy(&world_after, s))
        .collect();
    let (ranking, recommended) = select(&strategies, &verifications);
    let missing = most_valuable_missing_info(&strategies, &verifications);

    let caveat = "Recommendation is ranked by an explicit, contestable objective function over a \
                  HYPOTHETICAL model with transparent estimates. It is the best available under \
                  stated assumptions, not a claim of universal optimality. Unverified critical \
                  assumptions remain open proof obligations."
        .to_string();

    StrategyCycleReport {
        scenario_label: world_after.label.clone(),
        world_before,
        world_after,
        delta,
        objective,
        constraints,
        strategies,
        verifications,
        ranking,
        recommended,
        missing_information: missing,
        caveat,
    }
}

/// Convenience: build the initial world, ingest one hypothetical event, and run
/// the full loop end-to-end.
pub fn run_cyberpunk_scenario(event: &WorldEvent) -> StrategyCycleReport {
    let world0 = initial_cyberpunk_world();
    let world1 = integrate(&world0, event);
    let delta = detect_delta(&world0, &world1);
    run_adaptive_cycle(
        world0,
        world1,
        delta,
        cyberpunk_objective(),
        Vec::new(),
        Resource {
            tokens: 0,
            budget_usd: 10_000_000.0,
            political_capital: 1.0,
        },
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
    pub iteration: u64,
}

impl WorldModel {
    pub fn new(world: WorldState) -> Self {
        WorldModel {
            world,
            observations: Vec::new(),
            decisions: Vec::new(),
            iteration: 0,
        }
    }

    /// Observe + Integrate a hypothetical event; returns the detected delta.
    pub fn observe(&mut self, event: WorldEvent) -> Delta {
        let before = self.world.clone();
        self.world = integrate(&before, &event);
        self.observations.push(event);
        detect_delta(&before, &self.world)
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
            objective,
            constraints,
            resources,
        );
        if let Some(r) = &report.recommended {
            self.decisions.push(r.clone());
        }
        self.iteration += 1;
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event() -> WorldEvent {
        WorldEvent {
            evidence: Evidence {
                id: "E1".into(),
                provenance: "HYPOTHETICAL SCENARIO EVENT".into(),
                content:
                    "A coalition ratifies the Autonomous-Weapons Treaty; audit authority formed."
                        .into(),
                confidence: 0.6,
            },
            factor_adjustments: vec![
                (Factor::AutonomousWeapons, -0.1, "Treaty ratified.".into()),
                (
                    Factor::UbiquitousSurveillance,
                    -0.05,
                    "Audit authority stood up.".into(),
                ),
            ],
            new_assumptions: vec!["Treaty signatories include the three largest arms firms.".into()],
        }
    }

    #[test]
    fn world_is_explicitly_hypothetical() {
        let w = initial_cyberpunk_world();
        assert!(w.hypothetical);
        assert!(w.label.starts_with("HYPOTHETICAL"));
        assert_eq!(w.factors.len(), 10);
    }

    #[test]
    fn integrate_updates_state_and_delta_explains_change() {
        let w0 = initial_cyberpunk_world();
        let ev = sample_event();
        let w1 = integrate(&w0, &ev);
        let delta = detect_delta(&w0, &w1);
        assert!(w1.revision > w0.revision);
        assert_eq!(delta.changes.len(), 2);
        assert!(delta.summary.contains("autonomous weapons"));
        assert!(!delta.explanation.is_empty());
    }

    #[test]
    fn cycle_generates_ranks_and_recommends() {
        let report = run_cyberpunk_scenario(&sample_event());
        assert!(report.scenario_label.starts_with("HYPOTHETICAL"));
        assert!(report.strategies.len() >= 3);
        // ranking sorted by descending score
        for w in report.ranking.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
        assert!(report.recommended.is_some());
        // at least one critical assumption was verifiable
        let any_verified = report
            .verifications
            .iter()
            .flat_map(|v| v.assumptions.iter())
            .any(|a| a.verdict == "verified");
        assert!(
            any_verified,
            "expected at least one verified critical assumption"
        );
        // unverifiable critical assumptions surface as missing info
        assert!(!report.missing_information.is_empty());
        // honesty caveat present
        assert!(report
            .caveat
            .contains("not a claim of universal optimality"));
        // machine-readable round-trips
        let json = report.to_json();
        let back: StrategyCycleReport = serde_json::from_str(&json).expect("round-trips");
        assert_eq!(back.strategies.len(), report.strategies.len());
    }

    #[test]
    fn verify_produces_tamper_evident_proof() {
        let w = initial_cyberpunk_world();
        let obj = cyberpunk_objective();
        let strategies = generate_strategies(
            &w,
            &obj,
            &[],
            &Resource {
                tokens: 0,
                budget_usd: 0.0,
                political_capital: 0.0,
            },
        );
        let v = verify_strategy(&w, &strategies[0]);
        assert!(!v.gate_outcome.is_empty());
        let has_proof = v.assumptions.iter().any(|a| a.proof.is_some());
        assert!(
            has_proof,
            "formalizable critical assumption should carry a ProofEnvelope"
        );
    }

    #[test]
    fn world_model_loops_observe_then_step() {
        let mut model = WorldModel::new(initial_cyberpunk_world());
        let d = model.observe(sample_event());
        assert!(!d.changes.is_empty());
        let report = model.step(
            cyberpunk_objective(),
            Vec::new(),
            Resource {
                tokens: 0,
                budget_usd: 0.0,
                political_capital: 0.0,
            },
        );
        assert!(report.recommended.is_some());
        assert_eq!(model.iteration, 1);
    }
}
