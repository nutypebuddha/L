//! # Shikai — the query and intent layer
//!
//! In the Bleach universe, Shikai is the first released form of a Zanpakuto — the wielder
//! speaks a release command and the blade transforms. The power is now accessible, but the
//! *full* release (Bankai) is still beyond reach.
//!
//! In Athena, Shikai is the **prompt and query processing layer**. It:
//! - Takes natural language or structured input from the user
//! - Identifies the *intent* behind the query
//! - Translates that intent into a formula chain specification
//! - Returns a ShikaiResponse — a structured query ready for Bankai execution
//!
//! A user whose Zanpakuto is in Shikai state can express complex queries, but the
//! actual *atomic solve* happens in Bankai.

pub mod grammar;
pub mod intent;

use serde::Serialize;
use thiserror::Error;

use crate::entity::EntityRegistry;
use crate::formula::{Formula, FormulaRegistry};
use crate::gyro::GyroState;
use crate::wheel::{BleachLayer, Domain, UnderstandingAxis, WheelGraph};
use crate::zanpakuto::nlp::NlpContext;
use crate::zanpakuto::Identity;

pub use intent::Intent;

/// Errors from Shikai query processing.
#[derive(Error, Debug)]
pub enum ShikaiError {
    #[error("unrecognized intent: {0}")]
    UnrecognizedIntent(String),

    #[error("no formulas found for query: {0}")]
    NoFormulasFound(String),

    #[error("ambiguous query: {0}")]
    AmbiguousQuery(String),

    #[error("translation error: {0}")]
    TranslationError(String),
}

/// Signals that Shikai is uncertain about part of the query interpretation.
#[derive(Debug, Clone, Serialize)]
pub enum Ambiguity {
    /// No domain could be inferred from the query.
    NoDomain,
    /// Multiple formula candidates matched (count).
    MultipleCandidates(usize),
    /// Only formula-pattern matches, no explicit formula names.
    GrammarOnly,
}

/// A processed query ready for execution.
#[derive(Debug, Clone, Serialize)]
pub struct ShikaiQuery {
    /// The original query text.
    pub original: String,
    /// The identified intent.
    pub intent: Intent,
    /// The domain(s) this query targets.
    pub domains: Vec<Domain>,
    /// The formula IDs to execute.
    pub formula_ids: Vec<String>,
    /// Arguments for the formulas.
    pub args: Vec<(String, f64)>,
    /// Entity ID resolved from the query (if any), for grounded context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_context: Option<String>,

    /// K-12 level on the understanding axis, detected from query keywords
    /// (kindergarten/grade/level/basic/intermediate/advanced).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,

    /// Spiral cycle, detected from query keywords ("cycle 2", "deeper pass").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cycle: Option<u8>,

    /// Signals that Shikai is uncertain about parts of this interpretation.
    /// Empty means no ambiguity detected — Shikai is confident.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ambiguity: Vec<Ambiguity>,
}

/// The Shikai layer — processes prompts into structured queries.
#[derive(Debug, Clone)]
pub struct Shikai {
    pub wheel: WheelGraph,
    pub formulas: FormulaRegistry,
    pub entities: EntityRegistry,
}

impl Shikai {
    /// Create a new Shikai layer.
    pub fn new(formulas: FormulaRegistry) -> Self {
        Shikai {
            wheel: WheelGraph::new(),
            formulas,
            entities: EntityRegistry::new(),
        }
    }

    /// Create a new Shikai layer with an entity registry for grounded context resolution.
    pub fn with_entities(formulas: FormulaRegistry, entities: EntityRegistry) -> Self {
        Shikai {
            wheel: WheelGraph::new(),
            formulas,
            entities,
        }
    }

    /// Identify the intent of a query string.
    ///
    /// `nlp_hint` is an optional intent from Zanpakuto NLP preprocessing.
    /// If provided, it serves as the primary signal for ambiguous queries
    /// where no explicit keyword is found.
    pub fn identify_intent(
        &self,
        query: &str,
        nlp_hint: Option<Intent>,
    ) -> Result<Intent, ShikaiError> {
        intent::identify_intent(query, &self.formulas, &self.entities, nlp_hint)
            .map_err(ShikaiError::UnrecognizedIntent)
    }

    /// Process a query string into a structured ShikaiQuery.
    ///
    /// This is the "release command" — speaking the query activates Shikai.
    ///
    /// `nlp_ctx` is an optional `NlpContext` from Zanpakuto NLP preprocessing.
    /// When available, it provides richer intent, domain, entity, and formula
    /// detection than the ad-hoc keyword matching alone. Pass `None` for
    /// backward compatibility (no NLP preprocessing).
    pub fn process(
        &self,
        query: &str,
        identity: &Identity,
        nlp_ctx: Option<&NlpContext>,
    ) -> Result<ShikaiQuery, ShikaiError> {
        self.process_with_context(query, identity, nlp_ctx, None, None)
    }

    /// Process a query with full descent and gyro context.
    ///
    /// `matrix` and `gyro` come from the descent + gyro pipeline.
    /// When available, Shikai uses the settling matrix for domain weighting
    /// and the gyro orientation for primitive alignment awareness.
    ///
    /// This is the fully wired pipeline entry point.
    pub fn process_with_context(
        &self,
        query: &str,
        identity: &Identity,
        nlp_ctx: Option<&NlpContext>,
        matrix: Option<&crate::descent::SettlingMatrix>,
        gyro: Option<&GyroState>,
    ) -> Result<ShikaiQuery, ShikaiError> {
        // ── NLP hints from Zanpakuto preprocessing ──────────────────────
        // These are computed by the NLP engine (tokenizer, stemmer, keyword scorer).
        // They serve as primary signals with ad-hoc keyword matching as fallback.
        let nlp_hint = nlp_ctx.map(|ctx| ctx.likely_intent);
        let nlp_domain = nlp_ctx.and_then(|ctx| ctx.likely_domain);
        let nlp_entity = nlp_ctx.and_then(|ctx| ctx.likely_entity.clone());
        let _nlp_query_type = nlp_ctx.map(|ctx| ctx.query_type);

        // ── Intent ───────────────────────────────────────────────────────
        // NLP hint provides a pre-scored intent. If NLP detected strong intent
        // (e.g., "search" → Search, "traverse" → Traverse), it's used.
        // Explicit keywords like "validate", "solve" at query start still win.
        let intent = self.identify_intent(query, nlp_hint)?;

        // ── Arguments ────────────────────────────────────────────────────
        let args = self.extract_args(query);

        // ── K-12 level and spiral cycle ──────────────────────────────────
        let (detected_level, detected_cycle) = self.detect_level(query);

        // ── Entity context ───────────────────────────────────────────────
        // Use NLP entity matches (fuzzy, stem-aware) as primary signal.
        // Fall back to simple exact-ID match for backward compat.
        let entity_context =
            nlp_entity.or_else(|| intent::resolve_entity_context(query, &self.entities));

        // ── Domain ───────────────────────────────────────────────────────
        // Priority:
        //   1. NLP likely_domain (keyword-based scoring across 12 domains)
        //   2. Explicit domain name mention (extract_domain — substring match)
        //   3. Grammar-inferred domain (from matching grammar rules) — LAZY: only computed if needed
        //   4. Entity-resolved domain (from entity_context)
        //   5. None (will use default in Evaluate, empty in others)
        let effective_domain = nlp_domain
            .or_else(|| self.extract_domain(query))
            .or_else(|| grammar::infer_domain_from_grammar(&self.formulas, query))
            .or_else(|| {
                entity_context.as_ref().and_then(|id| {
                    // Try seed entity first (has classification with dominant sign),
                    // then runtime entity
                    self.entities
                        .get_seed(id)
                        .and_then(|s| s.classification.as_ref())
                        .and_then(|c| c.dominant_sign())
                        .map(crate::wheel::Domain::from_sign)
                        .or_else(|| {
                            self.entities
                                .get(id)
                                .and_then(|e| e.dominant_sign())
                                .map(crate::wheel::Domain::from_sign)
                        })
                })
            });

        // ── Descent-boosted domain ────────────────────────────────────────
        // If the settling matrix resolved strong domains, they override or augment
        // the NLP/grammar domain. The matrix is more reliable because it analyzed
        // each token individually through 7 layers of descent.
        let descent_domain = matrix.and_then(|m| {
            m.aggregate_western
                .dominant_sign()
                .map(crate::wheel::Domain::from_sign)
        });
        // Use descent domain if NLP didn't find one, or if descent is more confident
        let final_domain = effective_domain.or(descent_domain);

        // ── Gyro alignment ────────────────────────────────────────────────
        // The gyro's current orientation tells us which primitives are aligned.
        // This influences formula selection: prefer formulas whose domain matches
        // the gyro's current sign.
        let gyro_sign = gyro.map(|g| g.current_sign());
        let _gyro_aligned_domain = gyro_sign.map(crate::wheel::Domain::from_sign);

        let mut ambiguity = Vec::new();
        if final_domain.is_none() {
            ambiguity.push(Ambiguity::NoDomain);
        }

        // ── Query type awareness ─────────────────────────────────────────
        // NLP query type (Math/Grammar/Code/Logic/Unknown) hints at routing.
        // For Grammar queries, prefer grammar rule IDs over formulas.

        match &intent {
            Intent::Evaluate => {
                // Use NLP exact formula IDs if available (fast path).
                // Otherwise fall through to ad-hoc search.
                let formula_ids = if nlp_ctx.is_some_and(|ctx| ctx.has_exact_formula_id) {
                    nlp_ctx
                        .map(|ctx| ctx.exact_formula_ids.clone())
                        .unwrap_or_default()
                } else {
                    self.find_formulas_for_query(
                        query,
                        identity,
                        entity_context.as_deref(),
                        detected_level,
                        &args,
                    )?
                };

                if formula_ids.len() > 1 {
                    ambiguity.push(Ambiguity::MultipleCandidates(formula_ids.len()));
                }

                // Fill missing args from seed entity properties (or runtime entity values)
                let mut final_args = args;
                if let Some(ref entity_id) = entity_context {
                    // Try seed entity first (has properties/constants)
                    if let Some(seed) = self.entities.get_seed(entity_id) {
                        for formula_id in &formula_ids {
                            if let Some(formula) = self.formulas.get(formula_id) {
                                for input in &formula.inputs {
                                    if !final_args.iter().any(|(k, _)| k == input) {
                                        if let Some(val) = seed.properties.get(input) {
                                            final_args.push((input.clone(), *val));
                                        }
                                    }
                                }
                            }
                        }
                    } else if let Some(entity) = self.entities.get(entity_id) {
                        // Fallback to runtime entity values
                        for formula_id in &formula_ids {
                            if let Some(formula) = self.formulas.get(formula_id) {
                                for input in &formula.inputs {
                                    if !final_args.iter().any(|(k, _)| k == input) {
                                        if let Some(val) = entity.values.get(input) {
                                            final_args.push((input.clone(), *val));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                Ok(ShikaiQuery {
                    original: query.to_string(),
                    intent,
                    domains: final_domain
                        .map_or_else(|| vec![crate::wheel::Domain::Mangala], |d| vec![d]),
                    formula_ids,
                    args: final_args,
                    entity_context,
                    level: detected_level,
                    cycle: detected_cycle,
                    ambiguity,
                })
            }
            Intent::Search => Ok(ShikaiQuery {
                original: query.to_string(),
                intent,
                domains: final_domain.map_or_else(Vec::new, |d| vec![d]),
                formula_ids: vec![],
                args: vec![],
                entity_context,
                level: detected_level,
                cycle: detected_cycle,
                ambiguity: vec![],
            }),
            Intent::Info => Ok(ShikaiQuery {
                original: query.to_string(),
                intent,
                domains: vec![],
                formula_ids: vec![],
                args: vec![],
                entity_context: None,
                level: None,
                cycle: None,
                ambiguity: vec![],
            }),
            Intent::Validate | Intent::Traverse | Intent::Compose => {
                // Fast-path: only search formulas if the query contains a plausible formula ID
                // (words with underscores or camelCase). Non-Evaluate intents don't
                // need formula IDs to function — they work with just domain/intent context.
                let formula_ids = if query.contains('_') || query.chars().any(|c| c.is_uppercase())
                {
                    self.find_formulas_for_query(
                        query,
                        identity,
                        entity_context.as_deref(),
                        detected_level,
                        &args,
                    )
                    .unwrap_or_default()
                } else {
                    Vec::new()
                };
                if formula_ids.len() > 1 {
                    ambiguity.push(Ambiguity::MultipleCandidates(formula_ids.len()));
                }
                Ok(ShikaiQuery {
                    original: query.to_string(),
                    intent,
                    domains: final_domain.map_or_else(Vec::new, |d| vec![d]),
                    formula_ids,
                    args,
                    entity_context,
                    level: detected_level,
                    cycle: detected_cycle,
                    ambiguity,
                })
            }
        }
    }

    /// Infer domain from matching grammar rules.
    ///
    /// Wraps `grammar::infer_domain_from_grammar` for convenience in tests.
    #[allow(dead_code)]
    fn infer_domain_from_grammar(&self, query: &str) -> Option<Domain> {
        grammar::infer_domain_from_grammar(&self.formulas, query)
    }

    /// Extract a domain mention from a query string.
    ///
    /// Uses `eq_ignore_ascii_case` to avoid allocating `.to_lowercase()`
    /// on the entire query string.
    fn extract_domain(&self, query: &str) -> Option<Domain> {
        // Check each domain name in the query using case-insensitive substring match
        for d in crate::wheel::ALL_DOMAINS.iter() {
            let name = d.full_name_lower();
            let name_bytes = name.as_bytes();
            let name_len = name_bytes.len();
            if query.len() >= name_len
                && query
                    .as_bytes()
                    .windows(name_len)
                    .any(|w| w.eq_ignore_ascii_case(name_bytes))
            {
                return Some(*d);
            }
        }
        None
    }

    /// Extract numeric arguments from a query (key=value pairs).
    fn extract_args(&self, query: &str) -> Vec<(String, f64)> {
        let mut args = Vec::new();
        for part in query.split_whitespace() {
            if let Some(eq_pos) = part.find('=') {
                let key = &part[..eq_pos];
                let val_str = &part[eq_pos + 1..];
                if let Ok(val) = val_str.parse::<f64>() {
                    args.push((key.to_string(), val));
                }
            }
        }
        args
    }

    /// Detect K-12 level and spiral cycle from query keywords.
    ///
    /// Supports:
    /// - `"kindergarten"`, `"k"` → level 0
    /// - `"grade N"`, `"level N"` → level N (0-12)
    /// - `"basic"` → level 0
    /// - `"elementary"` → level 3
    /// - `"intermediate"` → level 6
    /// - `"advanced"` → level 9
    /// - `"mastery"`, `"expert"` → level 12
    /// - `"cycle N"` → cycle N
    /// - `"deeper"`, `"spiral"` → cycle + 1
    fn detect_level(&self, query: &str) -> (Option<u8>, Option<u8>) {
        let lower = query.to_lowercase();
        let mut level: Option<u8> = None;
        let mut cycle: Option<u8> = None;
        let mut prev: Option<&str> = None;

        // Single pass: check adjacent pairs and single-word keywords simultaneously
        for raw_w in lower.split_whitespace() {
            let w = raw_w.trim_matches(|c: char| !c.is_alphanumeric());

            // Adjacent pair check (e.g. "grade 5", "level 3", "cycle 2")
            if let Some(p) = prev {
                if let Ok(n) = w.parse::<u8>() {
                    if p == "grade" || p == "level" {
                        level = Some(n.min(12));
                    }
                    if p == "cycle" || p == "pass" {
                        cycle = Some(n);
                    }
                }
            }

            // Single-word level keywords (only if no adjacent match was found)
            if level.is_none() {
                match w {
                    "kindergarten" | "k" | "foundation" | "basic" => level = Some(0),
                    "elementary" => level = Some(3),
                    "intermediate" => level = Some(6),
                    "advanced" | "specialized" => level = Some(9),
                    "mastery" | "expert" | "master" | "graduate" => level = Some(12),
                    _ => {}
                }
            }

            // Single-word cycle keywords (only if no adjacent match was found)
            if cycle.is_none() {
                match w {
                    "deeper" | "spiral" | "advanced" => cycle = Some(1),
                    "mastery" | "depth" => cycle = Some(2),
                    _ => {}
                }
            }

            prev = Some(w);
        }

        (level, cycle)
    }

    /// Rank formula candidates by how completely the user-supplied args cover
    /// each formula's inputs. A query like "compound interest principal=1000
    /// rate=0.07 periods=10" names its inputs exactly — the formula whose
    /// input set they cover must outrank a keyword-similar formula whose
    /// inputs they don't. Coverage ties break on how many query words appear
    /// in the formula id ("ohms law" must pick ohms_law over joule_heating,
    /// which shares the same input set). Stable sort preserves search
    /// relevance for remaining ties; no-op when the query supplied no args.
    fn rank_by_arg_coverage<'a>(
        formulas: Vec<&'a Formula>,
        query: &str,
        args: &[(String, f64)],
    ) -> Vec<&'a Formula> {
        if args.is_empty() {
            return formulas;
        }
        let query_words: Vec<String> = query
            .split_whitespace()
            .filter(|w| !w.contains('='))
            .map(|w| w.to_lowercase())
            .collect();
        let mut ranked = formulas;
        ranked.sort_by(|a, b| {
            let coverage = |f: &Formula| -> f64 {
                let covered = f
                    .inputs
                    .iter()
                    .filter(|input| args.iter().any(|(k, _)| k == *input))
                    .count();
                covered as f64 / f.inputs.len().max(1) as f64
            };
            let id_matches = |f: &Formula| -> usize {
                f.id.split('_')
                    .filter(|part| query_words.iter().any(|w| w == part || w.starts_with(part)))
                    .count()
            };
            coverage(b)
                .partial_cmp(&coverage(a))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| id_matches(b).cmp(&id_matches(a)))
        });
        ranked
    }

    /// Find formulas relevant to a query.
    ///
    /// Tries, in order:
    /// 1. Direct query keyword match against formula IDs/descriptions
    /// 2. Word-by-word keyword extraction (>3 chars)
    /// 3. Entity property overlap (if entity_context is provided — finds formulas whose
    ///    inputs match the entity's properties)
    /// 4. Grammar rule match (if no math formulas match but grammar rules do,
    ///    return grammar rule IDs so the query can proceed with domain context)
    fn find_formulas_for_query(
        &self,
        query: &str,
        _identity: &Identity,
        entity_context: Option<&str>,
        detected_level: Option<u8>,
        args: &[(String, f64)],
    ) -> Result<Vec<String>, ShikaiError> {
        // Strategy 0: exact formula ID match — if the user typed a formula name
        // verbatim, respect it even when word-index union drowns it in noise.
        let exact_ids: Vec<String> = query
            .split_whitespace()
            .filter_map(|word| {
                let stripped = word.split('=').next().unwrap_or(word);
                if self.formulas.get(stripped).is_some() {
                    Some(stripped.to_string())
                } else {
                    None
                }
            })
            .collect();
        if !exact_ids.is_empty() {
            return if let Some(lvl) = detected_level {
                let exact: Vec<&Formula> = exact_ids
                    .iter()
                    .filter_map(|id| self.formulas.get(id))
                    .collect();
                let filtered = self.filter_by_level(exact, lvl);
                Ok(filtered.into_iter().map(|f| f.id.clone()).take(5).collect())
            } else {
                Ok(exact_ids.into_iter().take(5).collect())
            };
        }

        // Strategy 1: direct keyword search, re-ranked by arg coverage
        let results = self.formulas.search(query);
        if !results.is_empty() {
            let results = Self::rank_by_arg_coverage(results, query, args);
            return if let Some(lvl) = detected_level {
                let filtered = self.filter_by_level(results, lvl);
                Ok(filtered.into_iter().map(|f| f.id.clone()).take(5).collect())
            } else {
                Ok(results.into_iter().map(|f| f.id.clone()).take(5).collect())
            };
        }

        // Strategy 2: word-by-word keyword extraction — batch into single combined search
        let keywords: Vec<&str> = query.split_whitespace().filter(|w| w.len() > 3).collect();
        if !keywords.is_empty() {
            let combined = keywords.join(" ");
            let sub_results = self.formulas.search(&combined);
            let sub_results = Self::rank_by_arg_coverage(sub_results, query, args);
            if !sub_results.is_empty() {
                return if let Some(lvl) = detected_level {
                    let filtered = self.filter_by_level(sub_results, lvl);
                    Ok(filtered.into_iter().map(|f| f.id.clone()).take(3).collect())
                } else {
                    Ok(sub_results
                        .into_iter()
                        .map(|f| f.id.clone())
                        .take(3)
                        .collect())
                };
            }
        }

        // Strategy 3: entity property overlap — find formulas whose inputs match
        // the entity's properties/values, filtered by domain alignment
        if let Some(entity_id) = entity_context {
            // Try seed entity first (has properties), then runtime entity (has values)
            let entity_domain: Option<crate::wheel::Domain>;
            let entity_properties: Vec<(String, f64)>;

            if let Some(seed) = self.entities.get_seed(entity_id) {
                entity_domain = seed
                    .classification
                    .as_ref()
                    .and_then(|c| c.dominant_sign())
                    .map(crate::wheel::Domain::from_sign);
                entity_properties = seed
                    .properties
                    .iter()
                    .map(|(k, v)| (k.clone(), *v))
                    .collect();
            } else if let Some(entity) = self.entities.get(entity_id) {
                entity_domain = entity.dominant_sign().map(crate::wheel::Domain::from_sign);
                entity_properties = entity.values.iter().map(|(k, v)| (k.clone(), *v)).collect();
            } else {
                entity_domain = None;
                entity_properties = Vec::new();
            }

            if let Some(dom) = entity_domain {
                let mut scored: Vec<(&Formula, usize)> = Vec::new();
                for formula in self.formulas.all() {
                    // Domain filter: formula must match entity's domain
                    if formula.domain != dom {
                        continue;
                    }
                    let matches = formula
                        .inputs
                        .iter()
                        .filter(|input| entity_properties.iter().any(|(k, _)| k == *input))
                        .count();
                    if matches > 0 {
                        scored.push((formula, matches));
                    }
                }
                if !scored.is_empty() {
                    // Sort by most matches first
                    scored.sort_by_key(|b| std::cmp::Reverse(b.1));
                    let results: Vec<&Formula> = scored.into_iter().map(|(f, _)| f).collect();
                    return if let Some(lvl) = detected_level {
                        let filtered = self.filter_by_level(results, lvl);
                        Ok(filtered.into_iter().map(|f| f.id.clone()).take(3).collect())
                    } else {
                        Ok(results.into_iter().map(|f| f.id.clone()).take(3).collect())
                    };
                }
            }
        }

        Err(ShikaiError::NoFormulasFound(query.to_string()))
    }

    /// Filter formulas — since all primitives are level-agnostic, just return them all.
    fn filter_by_level<'a>(&'a self, formulas: Vec<&'a Formula>, _level: u8) -> Vec<&'a Formula> {
        if formulas.is_empty() {
            return self.formulas.all();
        }
        formulas
    }

    /// Format a ShikaiQuery as a human-readable string.
    pub fn format_query(query: &ShikaiQuery) -> String {
        let domain_str: Vec<String> = query
            .domains
            .iter()
            .map(|d| format!("{}{}", d.symbol(), d.full_name()))
            .collect();

        let args_str: Vec<String> = query
            .args
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let level_str = match (query.level, query.cycle) {
            (Some(l), Some(c)) => format!("K-12 Level {}/{} (Cycle {})", l, 12, c),
            (Some(l), None) => format!("K-12 Level {}/{}", l, 12),
            (None, Some(c)) => format!("K-12 Cycle {}", c),
            (None, None) => "K-12: not specified".to_string(),
        };

        // Compute Bleach layer from the detected level
        let bleach = query.level.map_or(BleachLayer::Asauchi, |l| {
            UnderstandingAxis::new(l, query.cycle.unwrap_or(0)).bleach_layer()
        });

        format!(
            "Shikai Query: {:?}\n  Original: {}\n  Domains: {}\n  Formulas: {}\n  Args: [{}]\n  {}  Bleach Layer: {} ({})",
            query.intent,
            query.original,
            domain_str.join(", "),
            query.formula_ids.join(", "),
            args_str.join(", "),
            level_str,
            bleach.state(),
            bleach.level(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formula::Formula;
    use crate::formula::FormulaRegistry;
    use crate::zanpakuto::{AccessTier, Zanpakuto};

    fn setup_shikai() -> Shikai {
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
            ])
            .unwrap();
        let entities = EntityRegistry::new();
        Shikai::with_entities(registry, entities)
    }

    #[test]
    fn test_identify_intent_evaluate() {
        let s = setup_shikai();
        assert_eq!(s.identify_intent("2 + 2", None).unwrap(), Intent::Evaluate);
        assert_eq!(
            s.identify_intent("mass * acceleration", None).unwrap(),
            Intent::Evaluate
        );
        assert_eq!(
            s.identify_intent("solve newtons_second mass=5", None)
                .unwrap(),
            Intent::Evaluate
        );
        assert_eq!(
            s.identify_intent("calculate force mass=5", None).unwrap(),
            Intent::Evaluate
        );
        assert_eq!(
            s.identify_intent("eval pythagorean a=3 b=4", None).unwrap(),
            Intent::Evaluate
        );
    }

    #[test]
    fn test_identify_intent_validate() {
        let s = setup_shikai();
        assert_eq!(
            s.identify_intent("validate 2 + 2 = 4", None).unwrap(),
            Intent::Validate
        );
        assert_eq!(
            s.identify_intent("check F = ma", None).unwrap(),
            Intent::Validate
        );
    }

    #[test]
    fn test_identify_intent_traverse() {
        let s = setup_shikai();
        assert_eq!(
            s.identify_intent("traverse from aries", None).unwrap(),
            Intent::Traverse
        );
    }

    #[test]
    fn test_identify_intent_search() {
        let s = setup_shikai();
        assert_eq!(
            s.identify_intent("search momentum", None).unwrap(),
            Intent::Search
        );
    }

    #[test]
    fn test_identify_intent_info() {
        let s = setup_shikai();
        assert_eq!(s.identify_intent("info", None).unwrap(), Intent::Info);
        assert_eq!(s.identify_intent("help", None).unwrap(), Intent::Info);
    }

    #[test]
    fn test_process_query() {
        let s = setup_shikai();
        let mut z = Zanpakuto::new();
        let id = z.register("test", AccessTier::Shikai);
        let query = s
            .process("validate newtons_second mass=5 acceleration=9.8", &id, None)
            .unwrap();
        assert_eq!(query.intent, Intent::Validate);
        assert!(!query.formula_ids.is_empty());
    }

    #[test]
    fn test_extract_args() {
        let s = setup_shikai();
        let args = s.extract_args("mass=5 acceleration=9.8");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], ("mass".to_string(), 5.0));
    }

    #[test]
    fn test_rank_by_arg_coverage_prefers_covered_inputs() {
        let wrong = Formula::atomic(
            "annuity_value",
            Domain::Budha,
            vec!["payment", "rate", "periods"],
            "value",
            "payment * (1 - (1 + rate)^(-periods)) / rate",
            "Annuity present value",
        );
        let right = Formula::atomic(
            "compound_interest",
            Domain::Budha,
            vec!["principal", "rate", "periods"],
            "future_value",
            "principal * (1 + rate)^periods",
            "Compound interest",
        );
        let args = vec![
            ("principal".to_string(), 1000.0),
            ("rate".to_string(), 0.07),
            ("periods".to_string(), 10.0),
        ];
        // Search relevance put the wrong formula first; coverage must flip it.
        let ranked = Shikai::rank_by_arg_coverage(
            vec![&wrong, &right],
            "compound interest principal=1000 rate=0.07 periods=10",
            &args,
        );
        assert_eq!(ranked[0].id, "compound_interest");
    }

    #[test]
    fn test_rank_by_arg_coverage_ties_break_on_id_match() {
        let joule = Formula::atomic(
            "joule_heating",
            Domain::Shukra,
            vec!["current", "resistance"],
            "power",
            "current^2 * resistance",
            "Joule heating",
        );
        let ohm = Formula::atomic(
            "ohms_law",
            Domain::Shukra,
            vec!["current", "resistance"],
            "voltage",
            "current * resistance",
            "Ohm's law",
        );
        let args = vec![
            ("current".to_string(), 2.5),
            ("resistance".to_string(), 470.0),
        ];
        // Both formulas have identical input coverage; the query names one.
        let ranked = Shikai::rank_by_arg_coverage(
            vec![&joule, &ohm],
            "ohms law current=2.5 resistance=470",
            &args,
        );
        assert_eq!(ranked[0].id, "ohms_law");
    }

    #[test]
    fn test_rank_by_arg_coverage_no_args_is_noop() {
        let a = Formula::atomic("f1", Domain::Shukra, vec!["x"], "y", "x", "First");
        let b = Formula::atomic("f2", Domain::Shukra, vec!["z"], "w", "z", "Second");
        let ranked = Shikai::rank_by_arg_coverage(vec![&a, &b], "some query", &[]);
        assert_eq!(ranked[0].id, "f1");
        assert_eq!(ranked[1].id, "f2");
    }

    #[test]
    fn test_extract_domain() {
        let s = setup_shikai();
        let domain = s.extract_domain("calculate momentum in mangala");
        assert_eq!(domain, Some(Domain::Mangala));
    }

    // ─── Domain inference tests ─────────────────────────

    fn setup_shikai_with_grammar() -> Shikai {
        let mut registry = FormulaRegistry::new();
        registry
            .register_all(vec![Formula::atomic(
                "newtons_second",
                Domain::Mangala,
                vec!["mass", "acceleration"],
                "force",
                "mass * acceleration",
                "F = ma",
            )])
            .unwrap();
        let entities = EntityRegistry::new();
        Shikai::with_entities(registry, entities)
    }

    #[test]
    fn test_fallback_topic_as_evaluate() {
        let s = setup_shikai();
        // Topics that don't match any formula/entity/grammar should still
        // resolve to Evaluate via the significant-word fallback
        assert_eq!(
            s.identify_intent("advanced calculus", None).unwrap(),
            Intent::Evaluate
        );
        assert_eq!(
            s.identify_intent("quantum mechanics", None).unwrap(),
            Intent::Evaluate
        );
        assert_eq!(
            s.identify_intent("bipolar disorder", None).unwrap(),
            Intent::Evaluate
        );
        // Short/gibberish queries should still fail
        assert!(s.identify_intent("xyz zzz qqq", None).is_err());
        assert!(s.identify_intent("foo bar baz", None).is_err());
    }

    #[test]
    fn test_infer_domain_from_clinical_keywords() {
        let s = setup_shikai_with_grammar();
        let domain = s.infer_domain_from_grammar("psychology behavior");
        // The new keyword-based inference checks for "psychology" → Pisces
        // or "behavior" → Pisces
        assert!(domain.is_some(), "Should infer some domain from keywords");
    }

    #[test]
    fn test_domain_inference_in_process() {
        let s = setup_shikai_with_grammar();
        let mut z = Zanpakuto::new();
        let id = z.register("test", AccessTier::Shikai);
        let query = s.process("force physics", &id, None).unwrap();
        assert_eq!(query.intent, Intent::Evaluate);
        assert!(
            query.domains.contains(&Domain::Shukra),
            "Physics query should infer Taurus domain, got: {:?}",
            query.domains
        );
    }

    #[test]
    fn test_no_false_positive_common_word() {
        let s = setup_shikai_with_grammar();
        assert_eq!(
            s.infer_domain_from_grammar("queries with example"),
            None,
            "Common words should NOT trigger domain inference"
        );
    }
}
