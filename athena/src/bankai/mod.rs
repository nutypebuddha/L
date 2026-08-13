//! # Bankai — the solved atomic form
//!
//! In the Bleach universe, Bankai is the ultimate released form of a Zanpakuto — the full
//! power, unleashed. It represents mastery, completeness, and finality.
//!
//! In Athena, Bankai is the **solve layer**. It takes a ShikaiQuery (a structured intent)
//! and executes it atomically to produce a BankaiSolve — the final, complete answer.
//!
//! ## What Bankai contains
//!
//! - **EvalEngine**: evaluates individual formula expressions
//! - **Composition**: chains multiple formulas across the wheel
//! - **Traversal**: follows the wheel graph to explore relationships
//! - **Chain**: the complete solved chain with step-by-step results
//! - **BankaiSolve**: the atomic output — final, verified, complete
//!
//! ## Pipeline Flow
//!
//! ```text
//! Asauchi → Zanpakuto → Shikai → Bankai
//! (public)  (access)   (query)  (solved)
//! ```

mod chain;
mod compose;
mod confidence;
#[cfg(feature = "llm")]
pub mod distill;
mod eval;
pub mod llm_eval;
#[cfg(feature = "llm")]
pub mod llm_router;
mod traverse;

pub use chain::{ChainResult, ChainStatus, ChainStep};
pub use compose::Composition;
pub use confidence::{BoundedConfidence, ConfidenceSemiring};
pub use eval::EvalEngine;
#[cfg(feature = "llm")]
pub use llm_router::{LlmRouteResult, LlmRouter, QueryCategory, RouterIntent, RouterToken};
pub use traverse::Traversal;

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::astrology::ChangeSorter;
use crate::entity::EntityRegistry;
use crate::formula::{Formula, FormulaRegistry, FormulaType};
use crate::gates::confidence::ConfidenceGate;
use crate::gates::fact::FactGate;
use crate::gates::formal::FormalGate;
use crate::gates::logic::LogicGate;
use crate::gates::math::MathGate;
use crate::gates::{Gate, GateOutput, GateResult};
use crate::gyro::GyroRouter;
use crate::shikai::{Ambiguity, ShikaiQuery};
use crate::wheel::{Aspect, Domain, WheelGraph};
use crate::zanpakuto::Identity;

/// Errors from Bankai solve operations.
#[derive(Error, Debug)]
pub enum BankaiError {
    #[error("evaluation error: {0}")]
    EvalError(String),

    #[error("composition error: {0}")]
    CompositionError(String),

    #[error("traversal error: {0}")]
    TraversalError(String),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("formula not found: {0}")]
    FormulaNotFound(String),

    #[error("domain mismatch: {0}")]
    DomainMismatch(String),

    #[error("chain error: {0}")]
    ChainError(String),

    #[error("not authorized: {0}")]
    NotAuthorized(String),

    #[error("no path found from {:?} to '{}'", have, want)]
    PathNotFound { have: Vec<String>, want: String },
}

/// The solved atomic output of a Bankai execution.
///
/// This is the final form — complete, verified, and ready to present.
#[derive(Debug, Clone, Serialize)]
pub struct BankaiSolve {
    /// The original query that triggered this solve.
    pub query: String,
    /// The identity that requested the solve.
    pub identity: String,
    /// The chain result (if applicable).
    pub chain: Option<ChainResult>,
    /// The composition (if applicable).
    pub composition: Option<Composition>,
    /// The traversal (if applicable).
    pub traversal: Option<Traversal>,
    /// Gate validation results.
    pub validation: Option<GateResult>,
    /// The final solved value (if applicable).
    pub solved_value: Option<f64>,
    /// Confidence in the result.
    pub confidence: f64,
    /// Whether the solve was successful.
    pub success: bool,
    /// Summary message.
    pub summary: String,
}

/// The Bankai solve engine — executes queries atomically.
#[derive(Debug, Clone)]
pub struct Bankai {
    pub wheel: WheelGraph,
    pub formulas: FormulaRegistry,
    pub eval: EvalEngine,
    pub entities: EntityRegistry,
    pub gyro: GyroRouter,
    sorter: ChangeSorter,
    /// The capstone model, attached via [`Bankai::with_capstone`]. `kind = "llm"`
    /// formulas evaluate through it (gated); without it they fail cleanly.
    #[cfg(feature = "llm")]
    capstone: Option<std::sync::Arc<std::sync::Mutex<LlmRouter>>>,
}

impl Bankai {
    /// Create a new Bankai solve engine.
    pub fn new(formulas: FormulaRegistry) -> Self {
        Bankai {
            wheel: WheelGraph::new(),
            formulas,
            eval: EvalEngine::new(),
            entities: EntityRegistry::new(),
            gyro: GyroRouter::new(),
            sorter: ChangeSorter::new(),
            #[cfg(feature = "llm")]
            capstone: None,
        }
    }

    /// Create a new Bankai solve engine with an entity registry for grounded context.
    pub fn with_entities(formulas: FormulaRegistry, entities: EntityRegistry) -> Self {
        Bankai {
            wheel: WheelGraph::new(),
            formulas,
            eval: EvalEngine::new(),
            entities,
            gyro: GyroRouter::new(),
            sorter: ChangeSorter::new(),
            #[cfg(feature = "llm")]
            capstone: None,
        }
    }

    /// Attach the capstone model so `kind = "llm"` formulas can evaluate.
    #[cfg(feature = "llm")]
    pub fn with_capstone(mut self, router: LlmRouter) -> Self {
        self.capstone = Some(std::sync::Arc::new(std::sync::Mutex::new(router)));
        self
    }

    /// Match a formula input name to an entity constant key.
    /// Strips common unit suffixes (_kg, _m, _ms2, _s, _au, _km, _w, _hz, _pa, _c, _k)
    /// and tries common synonyms.
    /// Match a formula input name to an entity constant key.
    /// Strips common unit suffixes (_kg, _m, _ms2, _s, _au, _km, _w, _hz, _pa, _c, _k)
    /// and tries common synonyms.
    pub fn match_constant(constants: &HashMap<String, f64>, input: &str) -> Option<f64> {
        // Direct match
        if let Some(&v) = constants.get(input) {
            return Some(v);
        }
        // Strip unit suffixes
        let suffixes = [
            "_kg", "_g", "_m", "_cm", "_mm", "_km", "_au", "_s", "_ms", "_us", "_ns", "_hz",
            "_khz", "_mhz", "_ghz", "_pa", "_kpa", "_mpa", "_gpa", "_c", "_k", "_f", "_w", "_kw",
            "_mw", "_gw", "_v", "_mv", "_kv", "_a", "_ma", "_ua", "_ohm", "_kohm", "_mohm", "_f",
            "_uf", "_pf", "_h", "_mh", "_uh", "_t", "_mt", "_gt", "_j", "_kj", "_mj", "_gj",
            "_cal", "_kcal", "_ev", "_kev", "_mev", "_gev", "_mol", "_mmol", "_cd", "_lm", "_lx",
            "_bq", "_ci", "_sv", "_msv", "_usv", "_kat",
        ];
        for suffix in &suffixes {
            if let Some(stripped) = input.strip_suffix(suffix) {
                if let Some(&v) = constants.get(stripped) {
                    return Some(v);
                }
                // Also try the reverse: constant has suffix, input doesn't
                let with_suffix = format!("{}{}", input, suffix);
                if let Some(&v) = constants.get(&with_suffix) {
                    return Some(v);
                }
            }
        }
        // Common synonyms
        let synonyms: &[(&str, &[&str])] = &[
            ("mass", &["mass_kg", "mass_g", "m"]),
            (
                "acceleration",
                &[
                    "acceleration_ms2",
                    "accel",
                    "a",
                    "gravity",
                    "surface_gravity_ms2",
                ],
            ),
            (
                "velocity",
                &["velocity_ms", "vel", "v", "speed", "orbital_velocity_ms"],
            ),
            ("force", &["force_n", "f", "thrust_n"]),
            ("energy", &["energy_j", "e", "work_j", "heat_j"]),
            ("power", &["power_w", "p", "luminosity_w"]),
            ("time", &["time_s", "t", "period_s", "orbital_period_days"]),
            (
                "distance",
                &[
                    "distance_m",
                    "m",
                    "dist",
                    "d",
                    "radius_m",
                    "distance_from_sun_au",
                    "distance_from_earth_km",
                ],
            ),
            ("radius", &["radius_m", "radius_km", "r"]),
            ("gravity", &["surface_gravity_ms2", "g", "gravity_ms2"]),
            (
                "temperature",
                &["temperature_k", "temperature_c", "temp", "t"],
            ),
            ("pressure", &["pressure_pa", "p", "atmospheric_pressure_pa"]),
            ("volume", &["volume_m3", "vol", "v"]),
            ("density", &["density_kgm3", "rho", "density"]),
            ("charge", &["charge_c", "q"]),
            ("current", &["current_a", "i"]),
            ("voltage", &["voltage_v", "v", "potential_v"]),
            ("resistance", &["resistance_ohm", "r"]),
            ("capacitance", &["capacitance_f", "c"]),
            ("inductance", &["inductance_h", "l"]),
            ("frequency", &["frequency_hz", "f", "freq"]),
            ("wavelength", &["wavelength_m", "lambda"]),
            ("angle", &["angle_rad", "theta", "phi"]),
            ("area", &["area_m2", "a"]),
        ];
        for (canon, alts) in synonyms {
            if input == *canon {
                for alt in alts.iter().copied() {
                    if let Some(&v) = constants.get(alt) {
                        return Some(v);
                    }
                }
            }
        }
        None
    }

    /// Solve a ShikaiQuery — execute it atomically and return the BankaiSolve.
    ///
    /// This is the "Bankai release" — the full power of the system, unleashed
    /// to produce a complete, verified answer.
    pub fn solve(&mut self, query: &ShikaiQuery, _identity: &Identity) -> BankaiSolve {
        match &query.intent {
            crate::shikai::Intent::Evaluate => self.solve_evaluate(query),
            crate::shikai::Intent::Validate => self.solve_validate(query),
            crate::shikai::Intent::Traverse => self.solve_traverse(query),
            crate::shikai::Intent::Compose => self.solve_compose(query),
            crate::shikai::Intent::Search => self.solve_search(query),
            crate::shikai::Intent::Info => self.solve_info(),
        }
    }

    /// Compute evaluation confidence from query signals.
    /// Traces to: candidate count, entity resolution, argument fill quality, ambiguity, domain alignment.
    /// Uses BoundedConfidence semiring: compose (⊗) for each multiplicative signal.
    fn eval_confidence(
        &self,
        query: &ShikaiQuery,
        candidate_count: usize,
        args_filled: usize,
        args_total: usize,
        domain_aligned: bool,
    ) -> f64 {
        let mut score = BoundedConfidence::one();

        // Fewer candidates = more confidence
        if candidate_count > 1 {
            let penalty = 0.85_f64.powi(candidate_count as i32 - 1);
            score = score.compose(&BoundedConfidence(penalty));
        }

        // Entity grounding bonus — only if domain aligned
        if query.entity_context.is_some() {
            let bonus = if domain_aligned { 1.1 } else { 0.85 }; // penalty for domain mismatch
            let grounded = (score.0 * bonus).min(1.0);
            score = BoundedConfidence(grounded);
        }

        // Argument fill quality
        if args_total > 0 {
            let fill_ratio = args_filled as f64 / args_total as f64;
            score = score.compose(&BoundedConfidence(0.5 + 0.5 * fill_ratio));
        }

        // Ambiguity penalty
        for amb in &query.ambiguity {
            match amb {
                Ambiguity::NoDomain => score = score.compose(&BoundedConfidence(0.85)),
                Ambiguity::MultipleCandidates(n) => {
                    let p = 0.90_f64.powi(*n as i32 - 1).max(0.7);
                    score = score.compose(&BoundedConfidence(p));
                }
                Ambiguity::GrammarOnly => score = score.compose(&BoundedConfidence(0.75)),
            }
        }

        score.0.clamp(0.1, 1.0)
    }

    /// Evaluate a formula or expression.
    fn solve_evaluate(&mut self, query: &ShikaiQuery) -> BankaiSolve {
        let mut args_map = HashMap::new();
        for (k, v) in &query.args {
            args_map.insert(k.clone(), *v);
        }

        // Fill missing args from seed entity context (if any entity was resolved in Shikai)
        // Track domain alignment per formula for confidence adjustment
        let mut formula_domain_aligned: HashMap<String, bool> = HashMap::new();

        // ─── Gyroscopic preprocessing ───────────────────────────────────────
        // Process query tokens through the gyro to get aligned NAND primitives
        let gyro_results: Vec<crate::gyro::RouteResult> =
            self.gyro.process_query(&query.original, &self.sorter);

        // Use gyro confidence to adjust overall confidence
        let gyro_confidence = if gyro_results.is_empty() {
            1.0
        } else {
            gyro_results.iter().map(|r| r.confidence).sum::<f64>() / gyro_results.len() as f64
        };

        if let Some(entity_id) = &query.entity_context {
            // Try seed entity first (for properties/constants), then runtime entity
            if let Some(seed) = self.entities.get_seed(entity_id) {
                let seed_domain = seed
                    .classification
                    .as_ref()
                    .and_then(|c| c.dominant_sign())
                    .map(crate::wheel::Domain::from_sign)
                    .unwrap_or(crate::wheel::Domain::Mangala);
                for formula_id in &query.formula_ids {
                    if let Some(formula) = self.formulas.get(formula_id) {
                        let domain_aligned = formula.domain == seed_domain;
                        formula_domain_aligned.insert(formula_id.clone(), domain_aligned);
                        for input in &formula.inputs {
                            if !args_map.contains_key(input) {
                                // Try properties first, then constants
                                let val = seed
                                    .properties
                                    .get(input)
                                    .cloned()
                                    .or_else(|| Self::match_constant(&seed.constants, input));
                                if let Some(v) = val {
                                    args_map.insert(input.clone(), v);
                                }
                            }
                        }
                    }
                }
            } else if let Some(entity) = self.entities.get(entity_id) {
                // Fallback to runtime entity values
                for formula_id in &query.formula_ids {
                    if let Some(formula) = self.formulas.get(formula_id) {
                        let entity_sign = entity
                            .dominant_sign()
                            .map(crate::wheel::Domain::from_sign)
                            .unwrap_or(crate::wheel::Domain::Mangala);
                        let domain_aligned = formula.domain == entity_sign;
                        formula_domain_aligned.insert(formula_id.clone(), domain_aligned);
                        for input in &formula.inputs {
                            if !args_map.contains_key(input) {
                                if let Some(v) = entity.values.get(input) {
                                    args_map.insert(input.clone(), *v);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Try each formula
        for formula_id in &query.formula_ids {
            if let Some(formula) = self.formulas.get(formula_id) {
                match self.eval_formula(formula, &args_map) {
                    Ok(value) => {
                        let args_filled = query.args.len();
                        let args_needed = formula.inputs.len();
                        let domain_aligned =
                            *formula_domain_aligned.get(formula_id).unwrap_or(&true);
                        let base_confidence = self.eval_confidence(
                            query,
                            query.formula_ids.len(),
                            args_filled,
                            args_needed,
                            domain_aligned,
                        );
                        // Combine base confidence with gyro alignment confidence
                        let confidence = (base_confidence * gyro_confidence).clamp(0.1, 1.0);
                        let chain_conf = ConfidenceGate::new().score_chain(
                            query.formula_ids.len().max(1),
                            true,
                            0,
                        );
                        let validation = GateResult::new(vec![GateOutput {
                            gate: "confidence".into(),
                            passed: chain_conf >= 0.5,
                            confidence: chain_conf,
                            message: format!(
                                "Chain confidence: {:.2} ({} formula(s), evidence: true)",
                                chain_conf,
                                query.formula_ids.len()
                            ),
                            issues: vec![],
                            suggestions: vec![],
                        }]);
                        let success = confidence >= 0.3;
                        return BankaiSolve {
                            query: query.original.clone(),
                            identity: String::new(),
                            chain: None,
                            composition: None,
                            traversal: None,
                            validation: Some(validation),
                            solved_value: Some(value),
                            confidence,
                            success,
                            summary: format!(
                                "Bankai: {} → {} = {:.6} (domain: {}){}",
                                formula_id,
                                formula.output,
                                value,
                                formula.domain.full_name(),
                                if domain_aligned {
                                    ""
                                } else {
                                    " [domain mismatch]"
                                }
                            ),
                        };
                    }
                    Err(e) => {
                        return BankaiSolve {
                            query: query.original.clone(),
                            identity: String::new(),
                            chain: None,
                            composition: None,
                            traversal: None,
                            validation: None,
                            solved_value: None,
                            confidence: 0.0,
                            success: false,
                            summary: format!("Bankai failed: {}", e),
                        };
                    }
                }
            }
        }

        // No formula found — try direct evaluation
        let direct_confidence = self.eval_confidence(query, 0, 0, 0, true) * 0.7;
        let env = crate::tanto::TantoEnv::new();
        match crate::tanto::evaluate(&query.original, &env) {
            Some(value) => BankaiSolve {
                query: query.original.clone(),
                identity: String::new(),
                chain: None,
                composition: None,
                traversal: None,
                validation: None,
                solved_value: Some(value),
                confidence: direct_confidence,
                success: direct_confidence >= 0.3,
                summary: format!(
                    "Bankai: {} = {:.6} (confidence: {:.2})",
                    query.original, value, direct_confidence
                ),
            },
            None => BankaiSolve {
                query: query.original.clone(),
                identity: String::new(),
                chain: None,
                composition: None,
                traversal: None,
                validation: None,
                solved_value: None,
                confidence: 0.0,
                success: false,
                summary: "Bankai: cannot evaluate".to_string(),
            },
        }
    }

    /// Validate a claim through all available gates.
    fn solve_validate(&self, query: &ShikaiQuery) -> BankaiSolve {
        let text = &query.original;

        let math_result = MathGate::new().check(text);
        let formal_result = FormalGate::new().check(text);
        let logic_result = LogicGate::new().check(text);
        let fact_result = FactGate::new(Some(self)).check(text);

        let chain_len = query.formula_ids.len().max(1);
        let has_evidence = !query.formula_ids.is_empty() || query.entity_context.is_some();
        let confidence_result =
            ConfidenceGate::new().check(&format!("{},0,{}", chain_len, has_evidence));

        let gate_result = GateResult::new(vec![
            math_result,
            formal_result,
            logic_result,
            fact_result,
            confidence_result,
        ]);

        BankaiSolve {
            query: query.original.clone(),
            identity: String::new(),
            chain: None,
            composition: None,
            traversal: None,
            validation: Some(gate_result.clone()),
            solved_value: None,
            confidence: gate_result.overall_confidence,
            success: gate_result.overall_passed,
            summary: format!(
                "Bankai validate: {} (confidence: {:.2})",
                gate_result.summary, gate_result.overall_confidence
            ),
        }
    }

    /// Traverse the wheel graph.
    fn solve_traverse(&self, query: &ShikaiQuery) -> BankaiSolve {
        let start = query.domains.first().copied().unwrap_or(Domain::Mangala);
        let traversal = Traversal::new(&self.wheel, &self.formulas, start, 5);
        let visited = traversal.domains_visited().len();
        let max_possible = 12;
        let coverage_confidence = visited as f64 / max_possible as f64;
        // Base coverage confidence, then apply ambiguity discounts via semiring
        let mut traverse_confidence = BoundedConfidence(0.5 + 0.5 * coverage_confidence);
        for amb in &query.ambiguity {
            traverse_confidence = match amb {
                Ambiguity::NoDomain => traverse_confidence.compose(&BoundedConfidence(0.85)),
                Ambiguity::MultipleCandidates(_) => {
                    traverse_confidence.compose(&BoundedConfidence(0.9))
                }
                Ambiguity::GrammarOnly => traverse_confidence.compose(&BoundedConfidence(0.8)),
            };
        }
        let traverse_confidence = traverse_confidence.0;

        BankaiSolve {
            query: query.original.clone(),
            identity: String::new(),
            chain: None,
            composition: None,
            traversal: Some(traversal.clone()),
            validation: None,
            solved_value: None,
            confidence: traverse_confidence,
            success: true,
            summary: format!(
                "Bankai traverse: {} — visited {} domains, found {} formulas (confidence: {:.2})",
                traversal.format_path(),
                visited,
                traversal.formula_count(),
                traverse_confidence,
            ),
        }
    }

    /// Compose formulas.
    fn solve_compose(&self, query: &ShikaiQuery) -> BankaiSolve {
        let formula_refs: Vec<&Formula> = query
            .formula_ids
            .iter()
            .filter_map(|id| self.formulas.get(id))
            .collect();

        if formula_refs.is_empty() {
            return BankaiSolve {
                query: query.original.clone(),
                identity: String::new(),
                chain: None,
                composition: None,
                traversal: None,
                validation: None,
                solved_value: None,
                confidence: 0.0,
                success: false,
                summary: "Bankai: no formulas found to compose".to_string(),
            };
        }

        match Composition::new(&self.wheel, &self.formulas, formula_refs) {
            Ok(composition) => BankaiSolve {
                query: query.original.clone(),
                identity: String::new(),
                chain: None,
                composition: Some(composition.clone()),
                traversal: None,
                validation: None,
                solved_value: None,
                confidence: composition.confidence,
                success: true,
                summary: format!(
                    "Bankai compose: {} (confidence: {:.4})",
                    composition.description, composition.confidence
                ),
            },
            Err(e) => BankaiSolve {
                query: query.original.clone(),
                identity: String::new(),
                chain: None,
                composition: None,
                traversal: None,
                validation: None,
                solved_value: None,
                confidence: 0.0,
                success: false,
                summary: format!("Bankai compose failed: {}", e),
            },
        }
    }

    /// Search the formula database.
    fn solve_search(&self, query: &ShikaiQuery) -> BankaiSolve {
        let results = self.formulas.search(&query.original);
        let count = results.len();
        // Confidence based on how many results relative to total and ambiguity
        let total_formulas = self.formulas.len();
        let coverage = if total_formulas > 0 {
            count as f64 / total_formulas as f64
        } else {
            0.0
        };
        let search_confidence = if count > 0 {
            let base = 0.5 + 0.4 * (1.0 - coverage).min(1.0); // few results = more precise = higher confidence
            let mut c = BoundedConfidence(base);
            for amb in &query.ambiguity {
                c = match amb {
                    Ambiguity::NoDomain => c.compose(&BoundedConfidence(0.9)),
                    Ambiguity::MultipleCandidates(_) => c.compose(&BoundedConfidence(0.95)),
                    Ambiguity::GrammarOnly => c.compose(&BoundedConfidence(0.8)),
                };
            }
            c.0
        } else {
            0.0
        };

        BankaiSolve {
            query: query.original.clone(),
            identity: String::new(),
            chain: None,
            composition: None,
            traversal: None,
            validation: None,
            solved_value: None,
            confidence: search_confidence,
            success: count > 0,
            summary: if count > 0 {
                format!(
                    "Bankai search: found {} formula(s): {}",
                    count,
                    results
                        .iter()
                        .map(|f| f.id.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                format!(
                    "Bankai search: no formulas found for '{}'. Try a different keyword.",
                    query.original
                )
            },
        }
    }

    /// Get system info.
    fn solve_info(&self) -> BankaiSolve {
        BankaiSolve {
            query: "info".to_string(),
            identity: String::new(),
            chain: None,
            composition: None,
            traversal: None,
            validation: None,
            solved_value: None,
            confidence: 1.0,
            success: true,
            summary: format!(
                "Athena v{} — Bankai released. {} formulas across {} domains.",
                crate::VERSION,
                self.formulas.len(),
                self.wheel.all_nodes().len()
            ),
        }
    }

    /// Evaluate a single formula with args.
    pub fn evaluate(
        &self,
        formula_id: &str,
        args: &HashMap<String, f64>,
    ) -> Result<f64, BankaiError> {
        let formula = self
            .formulas
            .get(formula_id)
            .ok_or_else(|| BankaiError::FormulaNotFound(formula_id.to_string()))?;
        self.eval_formula(formula, args)
    }

    /// Route a formula to the right evaluator by kind: math/logic through the
    /// deterministic engine, llm through the gated capstone.
    fn eval_formula(
        &self,
        formula: &Formula,
        args: &HashMap<String, f64>,
    ) -> Result<f64, BankaiError> {
        if formula.formula_type == FormulaType::Llm {
            return self.eval_llm_formula(formula, args);
        }
        self.eval.evaluate(formula, args)
    }

    /// Evaluate a `kind = "llm"` formula through the capstone: render the
    /// prompt template, generate at temperature 0, and admit the value only
    /// if it passes the deterministic gates in [`llm_eval`].
    #[cfg(feature = "llm")]
    fn eval_llm_formula(
        &self,
        formula: &Formula,
        args: &HashMap<String, f64>,
    ) -> Result<f64, BankaiError> {
        let prompt = llm_eval::render_prompt(formula, args)?;
        let capstone = self.capstone.as_ref().ok_or_else(|| {
            BankaiError::EvalError(format!(
                "formula '{}' is kind=llm but no capstone is attached (Bankai::with_capstone)",
                formula.id
            ))
        })?;
        let mut router = capstone
            .lock()
            .map_err(|e| BankaiError::EvalError(format!("capstone lock poisoned: {}", e)))?;
        let response = router
            .generate_deterministic(&prompt, Some(llm_eval::LLM_EVAL_SYSTEM))
            .map_err(|e| {
                BankaiError::EvalError(format!(
                    "capstone inference failed for '{}': {}",
                    formula.id, e
                ))
            })?;
        llm_eval::gate_llm_output(&response.text, &formula.id)
    }

    #[cfg(not(feature = "llm"))]
    fn eval_llm_formula(
        &self,
        formula: &Formula,
        _args: &HashMap<String, f64>,
    ) -> Result<f64, BankaiError> {
        Err(BankaiError::EvalError(format!(
            "formula '{}' is kind=llm but this build has no capstone (rebuild with --features llm)",
            formula.id
        )))
    }

    /// Execute a chain of formulas.
    pub fn chain(
        &self,
        formula_ids: &[&str],
        args: &HashMap<String, f64>,
    ) -> Result<ChainResult, BankaiError> {
        let formulas: Vec<&Formula> = formula_ids
            .iter()
            .map(|id| {
                self.formulas
                    .get(id)
                    .ok_or_else(|| BankaiError::FormulaNotFound(id.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let composition = Composition::new(&self.wheel, &self.formulas, formulas)
            .map_err(|e| BankaiError::CompositionError(e.to_string()))?;

        let mut steps = Vec::new();
        let mut current_args = args.clone();

        for formula in &composition.formulas {
            match self.eval_formula(formula, &current_args) {
                Ok(result) => {
                    steps.push(ChainStep {
                        formula_id: formula.id.clone(),
                        domain: formula.domain,
                        inputs: current_args.clone(),
                        output_name: formula.output.clone(),
                        output_value: result,
                        status: ChainStatus::Success,
                        error: None,
                    });
                    current_args.insert(formula.output.clone(), result);
                }
                Err(e) => {
                    steps.push(ChainStep {
                        formula_id: formula.id.clone(),
                        domain: formula.domain,
                        inputs: current_args.clone(),
                        output_name: formula.output.clone(),
                        output_value: 0.0,
                        status: ChainStatus::Failed,
                        error: Some(e.to_string()),
                    });
                    break;
                }
            }
        }

        let success = steps
            .last()
            .is_some_and(|s| matches!(s.status, ChainStatus::Success));

        // Compute chain confidence via semiring
        let mut chain_conf = BoundedConfidence::one();
        for step in &steps {
            let step_conf = match step.status {
                // A gated LLM step is admitted but never proven — it carries
                // a fraction of the confidence a deterministic step earns.
                ChainStatus::Success => match self.formulas.get(&step.formula_id) {
                    Some(f) if f.formula_type == FormulaType::Llm => {
                        BoundedConfidence(llm_eval::LLM_STEP_CONFIDENCE)
                    }
                    _ => BoundedConfidence(0.95),
                },
                ChainStatus::Failed => BoundedConfidence(0.0),
                ChainStatus::Skipped => BoundedConfidence(0.0),
            };
            chain_conf = chain_conf.compose(&step_conf);
        }

        Ok(ChainResult {
            steps,
            total_domains: composition.domains_traversed.clone(),
            total_aspects: composition.aspects_traversed.clone(),
            success,
            confidence: chain_conf.0,
        })
    }

    /// Find the shortest formula chain from available variables to a desired one.
    ///
    /// Uses BFS over the formula graph: each formula is an edge that consumes its
    /// inputs and produces its output. Returns the shortest sequence of formula IDs,
    /// or `PathNotFound` if no chain exists within `max_depth` steps.
    pub fn find_path(
        &self,
        have: &[String],
        want: &str,
        max_depth: usize,
    ) -> Result<Vec<String>, BankaiError> {
        use std::collections::VecDeque;

        let initial_have: HashSet<String> = have.iter().cloned().collect();
        if initial_have.contains(want) {
            return Ok(Vec::new());
        }

        let all_formulas: Vec<&Formula> = self.formulas.all();

        let mut queue: VecDeque<(Vec<String>, HashSet<String>)> = VecDeque::new();
        queue.push_back((Vec::new(), initial_have));

        for _ in 0..max_depth {
            let level_size = queue.len();
            for _ in 0..level_size {
                let (path, have_set) = queue.pop_front().unwrap();

                for formula in &all_formulas {
                    // Skip if already in path
                    if path.contains(&formula.id) {
                        continue;
                    }
                    // Check all inputs are satisfied
                    if formula.inputs.iter().all(|i| have_set.contains(i.as_str())) {
                        if formula.output == want {
                            let mut result = path.clone();
                            result.push(formula.id.clone());
                            return Ok(result);
                        }

                        let mut new_have = have_set.clone();
                        new_have.insert(formula.output.clone());
                        let mut new_path = path.clone();
                        new_path.push(formula.id.clone());
                        queue.push_back((new_path, new_have));
                    }
                }
            }
        }

        Err(BankaiError::PathNotFound {
            have: have.to_vec(),
            want: want.to_string(),
        })
    }

    /// Traverse from a starting domain.
    pub fn traverse(&self, start: Domain, max_depth: usize) -> Traversal {
        Traversal::new(&self.wheel, &self.formulas, start, max_depth)
    }

    /// Compose formulas.
    pub fn compose(&self, formula_ids: &[&str]) -> Result<Composition, BankaiError> {
        let formulas: Vec<&Formula> = formula_ids
            .iter()
            .map(|id| {
                self.formulas
                    .get(id)
                    .ok_or_else(|| BankaiError::FormulaNotFound(id.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Composition::new(&self.wheel, &self.formulas, formulas)
            .map_err(|e| BankaiError::CompositionError(e.to_string()))
    }

    /// Render the full pipeline for a query: show what each layer would produce.
    /// Get a reference to the change sorter for token classification.
    pub fn change_sorter(&self) -> &ChangeSorter {
        &self.sorter
    }

    /// Get a reference to the gyro router for gyroscopic state inspection.
    pub fn gyro_state(&self) -> &crate::gyro::GyroState {
        self.gyro.gyro_state()
    }

    /// Get a mutable reference to the gyro router for direct state manipulation.
    pub fn gyro_state_mut(&mut self) -> &mut crate::gyro::GyroState {
        self.gyro.gyro_state_mut()
    }

    pub fn describe_pipeline(&self, query_str: &str, identity: &Identity) -> String {
        // This is a meta-function — it shows the pipeline layers without executing
        format!(
            "\
╔══════════════════════════════════════════╗
║           Athena Pipeline                ║
╠══════════════════════════════════════════╣
║  Asauchi  │ Public interface             ║
║  ─────────┼───────────────────────────── ║
║  Zanpakuto│ Identity: {}              ║
║           │ Tier: {:?}                  ║
║  ─────────┼───────────────────────────── ║
║  Shikai   │ Query: \"{}\"            ║
║  ─────────┼───────────────────────────── ║
║  Bankai   │ Ready to solve               ║
╚══════════════════════════════════════════╝",
            identity.name, identity.tier, query_str
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::{Formula, FormulaType};
    use crate::shikai::Intent;
    use crate::zanpakuto::AccessTier;

    /// A `kind = "llm"` formula with no capstone attached (or in a lean
    /// build) must fail with a message naming the kind — never fall through
    /// to meval, which would choke on the prose template.
    #[test]
    fn test_llm_formula_without_capstone_fails_cleanly() {
        let mut registry = FormulaRegistry::new();
        registry
            .register(Formula::new(
                "estimate_reading_minutes",
                FormulaType::Llm,
                Domain::Budha,
                vec!["word_count"],
                "reading_minutes",
                "Estimate how many minutes an average adult needs to read {word_count} words.",
                "LLM-estimated reading time",
            ))
            .unwrap();
        let bankai = Bankai::new(registry);
        let mut args = HashMap::new();
        args.insert("word_count".to_string(), 1200.0);
        let err = bankai
            .evaluate("estimate_reading_minutes", &args)
            .unwrap_err();
        assert!(
            err.to_string().contains("kind=llm"),
            "unexpected error: {}",
            err
        );
    }

    fn setup_bankai() -> (Bankai, Identity) {
        let mut registry = FormulaRegistry::new();
        registry
            .register_all(vec![
                Formula::atomic(
                    "newtons_second",
                    Domain::Shukra,
                    vec!["mass", "acceleration"],
                    "force",
                    "mass * acceleration",
                    "F = ma",
                ),
                Formula::atomic(
                    "pythagorean",
                    Domain::Mangala,
                    vec!["a", "b"],
                    "c",
                    "(a^2 + b^2).sqrt()",
                    "Pythagorean theorem",
                ),
                Formula::atomic(
                    "kinetic_energy",
                    Domain::Shukra,
                    vec!["mass", "velocity"],
                    "ke",
                    "0.5 * mass * velocity^2",
                    "KE = ½mv²",
                ),
                Formula::new(
                    "work_energy",
                    FormulaType::Math,
                    Domain::Shukra,
                    vec!["force", "distance"],
                    "work",
                    "force * distance",
                    "Work = Force × Distance",
                ),
                Formula::new(
                    "force_to_work",
                    FormulaType::Math,
                    Domain::Shukra,
                    vec!["ke", "distance"],
                    "work",
                    "ke / distance",
                    "Work from kinetic energy and distance",
                ),
            ])
            .unwrap();
        let bankai = Bankai::new(registry);
        let identity = Identity {
            name: "test".to_string(),
            tier: AccessTier::Bankai,
            capabilities: vec![],
            scope: vec![],
            session: "test".to_string(),
        };
        (bankai, identity)
    }

    #[test]
    fn test_solve_evaluate() {
        let (mut bankai, identity) = setup_bankai();
        let query = ShikaiQuery {
            original: "newtons_second mass=5 acceleration=9.8".to_string(),
            intent: Intent::Evaluate,
            domains: vec![Domain::Shukra],
            formula_ids: vec!["newtons_second".to_string()],
            args: vec![("mass".to_string(), 5.0), ("acceleration".to_string(), 9.8)],
            entity_context: None,
            level: None,
            cycle: None,
            ambiguity: vec![],
        };
        let solve = bankai.solve(&query, &identity);
        assert!(solve.success);
        assert!((solve.solved_value.unwrap() - 49.0).abs() < 0.1);
    }

    #[test]
    fn test_solve_validate() {
        let (mut bankai, identity) = setup_bankai();
        let query = ShikaiQuery {
            original: "2 + 2".to_string(),
            intent: Intent::Validate,
            domains: vec![],
            formula_ids: vec![],
            args: vec![],
            entity_context: None,
            level: None,
            cycle: None,
            ambiguity: vec![],
        };
        let solve = bankai.solve(&query, &identity);
        assert!(solve.success, "Validate of '2 + 2' must succeed");
        // Validate answers yes/no; it does not compute a solved numeric value.
    }

    #[test]
    fn test_solve_search() {
        let (mut bankai, identity) = setup_bankai();
        let query = ShikaiQuery {
            original: "pythagorean".to_string(),
            intent: Intent::Search,
            domains: vec![],
            formula_ids: vec![],
            args: vec![],
            entity_context: None,
            level: None,
            cycle: None,
            ambiguity: vec![],
        };
        let solve = bankai.solve(&query, &identity);
        assert!(solve.success);
    }

    #[test]
    fn test_solve_info() {
        let (mut bankai, identity) = setup_bankai();
        let query = ShikaiQuery {
            original: "info".to_string(),
            intent: Intent::Info,
            domains: vec![],
            formula_ids: vec![],
            args: vec![],
            entity_context: None,
            level: None,
            cycle: None,
            ambiguity: vec![],
        };
        let solve = bankai.solve(&query, &identity);
        assert!(solve.success);
        assert!(solve.summary.contains("Bankai released"));
    }

    #[test]
    fn test_evaluate_single() {
        let (bankai, _) = setup_bankai();
        let mut args = HashMap::new();
        args.insert("mass".to_string(), 5.0);
        args.insert("acceleration".to_string(), 10.0);
        let result = bankai.evaluate("newtons_second", &args).unwrap();
        assert!((result - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_chain_success() {
        let (bankai, _) = setup_bankai();
        let mut args = HashMap::new();
        args.insert("mass".to_string(), 2.0);
        args.insert("acceleration".to_string(), 9.8);
        let result = bankai.chain(&["newtons_second"], &args).unwrap();
        assert!(result.success);
    }

    #[test]
    fn test_find_path_single_step() {
        let (bankai, _) = setup_bankai();
        let have = vec!["mass".to_string(), "acceleration".to_string()];
        let path = bankai.find_path(&have, "force", 5).unwrap();
        assert_eq!(path, vec!["newtons_second"]);
    }

    #[test]
    fn test_find_path_two_step() {
        let (bankai, _) = setup_bankai();
        let have = vec![
            "mass".to_string(),
            "velocity".to_string(),
            "distance".to_string(),
        ];
        let path = bankai.find_path(&have, "work", 5).unwrap();
        // kinetic_energy produces 'ke', force_to_work consumes ke+distance → work
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], "kinetic_energy");
        assert_eq!(path[1], "force_to_work");
    }

    #[test]
    fn test_find_path_already_have() {
        let (bankai, _) = setup_bankai();
        let have = vec!["force".to_string()];
        let path = bankai.find_path(&have, "force", 5).unwrap();
        assert!(path.is_empty());
    }

    #[test]
    fn test_find_path_not_found() {
        let (bankai, _) = setup_bankai();
        let have = vec!["magic".to_string()];
        let err = bankai.find_path(&have, "unicorn", 5).unwrap_err();
        assert!(matches!(err, BankaiError::PathNotFound { .. }));
    }

    #[test]
    fn test_find_path_chain_execution_two_step() {
        let (bankai, _) = setup_bankai();
        let have = vec![
            "mass".to_string(),
            "velocity".to_string(),
            "distance".to_string(),
        ];
        let path = bankai.find_path(&have, "work", 5).unwrap();
        assert_eq!(path.len(), 2);

        let refs: Vec<&str> = path.iter().map(|s| s.as_str()).collect();
        let mut args = std::collections::HashMap::new();
        args.insert("mass".to_string(), 2.0);
        args.insert("velocity".to_string(), 3.0);
        args.insert("distance".to_string(), 4.0);
        let result = bankai.chain(&refs, &args).unwrap();
        assert!(result.success);
        // kinetic_energy: 0.5 * 2 * 3^2 = 9.0
        // force_to_work: 9.0 / 4.0 = 2.25
        assert!((result.final_output().unwrap() - 2.25).abs() < 1e-10);
    }
}
