//! # Zanpakuto NLP — preprocessing for the Shikai layer
//!
//! Zanpakuto is the gateway. Before passing a query to Shikai for interpretation,
//! it preprocesses the raw text through an NLP pipeline:
//!
//! 1. **Tokenization** — split into words with formula-awareness
//! 2. **Stopword removal** — filter noise words
//! 3. **Stemming** — reduce inflected words to root form
//! 4. **Entity recognition** — fuzzy-match tokens against the entity registry
//! 5. **Intent scoring** — score each intent type by token evidence
//! 6. **Domain scoring** — score each of the 12 domains by token evidence
//! 7. **Query type classification** — math vs grammar vs code vs logic vs unknown
//!
//! The result is an `NlpContext` — a structured, enriched version of the query
//! that Shikai consumes for faster, more accurate interpretation.

use crate::entity::EntityRegistry;
use crate::formula::FormulaRegistry;
use crate::shikai::Intent;
use crate::wheel::Domain;
use crate::wheel::ALL_DOMAINS;

/// The type of query, inferred from token content and structure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum QueryType {
    /// Contains formula operators (= + - * / ^) or numeric args
    Math,
    /// Matches grammar rule patterns (syntax, punctuation, composition)
    Grammar,
    /// Matches code patterns (function, loop, if/else, variable)
    Code,
    /// Matches formal logic patterns (if-then, therefore, implies)
    Logic,
    /// Could not be confidently classified
    Unknown,
}

/// A preprocessed, enriched query ready for Shikai interpretation.
///
/// This is the contract between Zanpakuto (NLP preprocessing) and Shikai (query parsing).
/// Every field is computed deterministically — no LLM, no external API, no randomness.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NlpContext {
    /// Raw query string (as entered)
    pub original: String,

    // ─── Tokenization ──────────────────────────────────────────────
    /// Tokens split on whitespace, punctuation stripped from edges
    pub tokens: Vec<String>,
    /// Tokens with common stopwords removed (for keyword matching)
    pub significant_tokens: Vec<String>,
    /// Stemmed form of each significant token
    pub stems: Vec<String>,

    // ─── Formula structure detection ───────────────────────────────
    /// True if the query contains `key=value` patterns
    pub has_key_value_args: bool,
    /// True if the query contains math operators (+ - * / ^ =)
    pub has_math_operators: bool,
    /// True if the query contains numeric literals
    pub has_numbers: bool,
    /// True if a formula ID appears verbatim in the query
    pub has_exact_formula_id: bool,
    /// The exact formula IDs found verbatim (if any)
    pub exact_formula_ids: Vec<String>,

    // ─── Query type ────────────────────────────────────────────────
    /// Classified query type
    pub query_type: QueryType,

    // ─── Intent candidates ─────────────────────────────────────────
    /// Each intent type with a score 0.0–1.0. Higher = more likely.
    /// At least one entry always present. Sorted by score descending.
    pub intent_scores: Vec<(Intent, f64)>,

    // ─── Domain candidates ─────────────────────────────────────────
    /// Each domain with a score 0.0–1.0. Higher = stronger signal.
    /// Empty if no domain signal detected. Sorted by score descending.
    pub domain_scores: Vec<(Domain, f64)>,

    // ─── Entity matches ────────────────────────────────────────────
    /// Fuzzy-matched entities: (entity_id, entity_name, confidence)
    /// Sorted by confidence descending. Max 5 entries.
    pub entity_matches: Vec<(String, String, f64)>,

    // ─── Performance hints ─────────────────────────────────────────
    /// The most likely single intent (highest-scored)
    pub likely_intent: Intent,
    /// The most likely single domain (highest-scored, or None)
    pub likely_domain: Option<Domain>,
    /// The most likely entity ID (highest-confidence, or None)
    pub likely_entity: Option<String>,
}

/// The Zanpakuto NLP preprocessing engine.
///
/// Uses only the entity registry and formula registry for grounding.
/// No external dependencies, no ML, no randomness.
#[derive(Debug, Clone)]
pub struct NlpEngine {
    entities: EntityRegistry,
    formulas: FormulaRegistry,
}

impl NlpEngine {
    /// Create a new NLP engine.
    pub fn new(entities: EntityRegistry, formulas: FormulaRegistry) -> Self {
        NlpEngine { entities, formulas }
    }

    /// Preprocess a raw query string into a structured NlpContext.
    ///
    /// This is the main entry point. Every query passes through this
    /// pipeline before reaching Shikai.
    pub fn preprocess(&self, query: &str) -> NlpContext {
        let original = query.to_string();

        // Step 1: Tokenize
        let tokens = self.tokenize(query);
        let raw_tokens: Vec<String> = tokens.clone();

        // Step 2: Remove stopwords
        let significant_tokens = self.remove_stopwords(&tokens);

        // Step 3: Stem
        let stems: Vec<String> = significant_tokens.iter().map(|t| self.stem(t)).collect();

        // Step 4: Detect structure
        let has_key_value_args = query.contains('=');
        let has_math_operators = query.contains('+')
            || query.contains('-')
            || query.contains('*')
            || query.contains('/')
            || query.contains('^');
        let has_numbers = tokens.iter().any(|t| t.parse::<f64>().is_ok());

        // Step 5: Detect exact formula IDs
        let exact_formula_ids: Vec<String> = tokens
            .iter()
            .filter_map(|t| {
                let stripped = t.split('=').next().unwrap_or(t);
                if self.formulas.get(stripped).is_some() {
                    Some(stripped.to_string())
                } else {
                    None
                }
            })
            .collect();
        let has_exact_formula_id = !exact_formula_ids.is_empty();

        // Step 6: Classify query type
        let query_type = self.classify_type(query, &tokens, has_math_operators, has_numbers);

        // Step 7: Score intents
        let intent_scores = self.score_intents(query, &tokens, &significant_tokens);
        let likely_intent = intent_scores
            .first()
            .map(|(i, _)| *i)
            .unwrap_or(Intent::Evaluate);

        // Step 8: Score domains
        let domain_scores = self.score_domains(query, &tokens, &significant_tokens, &stems);
        let likely_domain = domain_scores.first().map(|(d, _)| *d);

        // Step 9: Recognize entities (fuzzy)
        let entity_matches = self.recognize_entities(query, &tokens, &stems);
        let likely_entity = entity_matches.first().map(|(id, _, _)| id.clone());

        NlpContext {
            original,
            tokens: raw_tokens,
            significant_tokens,
            stems,
            has_key_value_args,
            has_math_operators,
            has_numbers,
            has_exact_formula_id,
            exact_formula_ids,
            query_type,
            intent_scores,
            domain_scores,
            entity_matches,
            likely_intent,
            likely_domain,
            likely_entity,
        }
    }

    // ─── Tokenization ──────────────────────────────────────────────

    /// Split a query into tokens.
    ///
    /// Handles:
    /// - Whitespace-separated words
    /// - `key=value` pairs (preserved as single tokens)
    /// - Punctuation attached to words (stripped from edges)
    /// - Formula operators preserved in context
    fn tokenize(&self, query: &str) -> Vec<String> {
        query
            .split_whitespace()
            .map(|w| {
                // Strip leading/trailing punctuation, but keep internal
                // punctuation like underscores, hyphens, and dots
                w.trim_matches(|c: char| {
                    c.is_ascii_punctuation() && c != '_' && c != '-' && c != '.'
                })
                .to_string()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    // ─── Stopword removal ──────────────────────────────────────────

    /// Common English stopwords that carry little semantic weight
    /// for intent/domain classification.
    const STOPWORDS: &'static [&'static str] = &[
        "a", "an", "the", "this", "that", "these", "those", "is", "are", "was", "were", "be",
        "been", "being", "have", "has", "had", "having", "do", "does", "did", "doing", "will",
        "would", "could", "should", "can", "may", "might", "shall", "must", "need", "i", "you",
        "he", "she", "it", "we", "they", "me", "him", "her", "us", "them", "my", "your", "his",
        "its", "our", "their", "mine", "yours", "hers", "ours", "theirs", "in", "on", "at", "to",
        "for", "of", "with", "by", "from", "into", "through", "during", "before", "after", "above",
        "below", "between", "under", "over", "and", "or", "but", "nor", "yet", "so", "if", "then",
        "else", "when", "while", "because", "as", "until", "unless", "since", "not", "no", "nor",
        "none", "nothing", "very", "just", "too", "also", "only", "about", "than", "well", "now",
        "here", "there", "what", "which", "who", "whom", "whose", "why", "how", "all", "each",
        "every", "both", "few", "more", "most", "other", "some", "such", "any", "much",
    ];

    /// Remove common stopwords from a token list.
    fn remove_stopwords(&self, tokens: &[String]) -> Vec<String> {
        tokens
            .iter()
            .filter(|t| {
                let lower = t.to_lowercase();
                !Self::STOPWORDS.contains(&lower.as_str())
            })
            .cloned()
            .collect()
    }

    // ─── Stemming ──────────────────────────────────────────────────

    /// A lightweight, rule-based English stemmer.
    ///
    /// Implements the Porter-style suffix stripping algorithm
    /// but simplified — handles the most common inflectional suffixes.
    /// Does NOT use any external dependency.
    fn stem(&self, word: &str) -> String {
        let lower = word.to_lowercase();
        let w = lower.as_str();

        // Handle short words (2 chars or fewer) — never stem
        if w.len() <= 2 {
            return lower;
        }

        let result = self.stem_inner(w);
        // Ensure we never return an empty stem
        if result.is_empty() {
            lower
        } else {
            result
        }
    }

    /// Inner stemmer — applies suffix stripping rules in order.
    fn stem_inner(&self, w: &str) -> String {
        let len = w.len();

        // Rule set: apply in order, stop at first match.
        // Each rule checks for a suffix and returns the stem if matched.

        // ── Step 1: Plural and third-person singular ──
        // -sses → -ss (e.g., "classes" → "class")
        if w.ends_with("sses") {
            return format!("{}ss", &w[..len - 4]);
        }
        // -ies → -i (e.g., "policies" → "polici", but "dies" → "di")
        if w.ends_with("ies") && len > 4 {
            return format!("{}i", &w[..len - 3]);
        }
        // -ves → -f (e.g., "wolves" → "wol")
        if w.ends_with("ves") && len > 4 {
            return format!("{}f", &w[..len - 3]);
        }
        // -es → -e (e.g., "boxes" → "boxe", but not for short words)
        if w.ends_with("es") && len > 4 && !w.ends_with("sses") && !w.ends_with("ies") {
            // Only if the preceding char is a sibilant (s, x, z, sh, ch)
            let pre = &w[..len - 2];
            if pre.ends_with('s')
                || pre.ends_with('x')
                || pre.ends_with('z')
                || pre.ends_with("sh")
                || pre.ends_with("ch")
            {
                return w[..len - 1].to_string();
            }
        }
        // -s → (remove) (e.g., "dogs" → "dog"), but not "ss" → "s"
        if w.ends_with('s') && len > 3 && !w.ends_with("ss") && !w.ends_with("us") {
            return w[..len - 1].to_string();
        }

        // ── Step 2: Past tense and participles ──
        // -ied → -i (e.g., "studied" → "studi")
        if w.ends_with("ied") && len > 4 {
            return format!("{}i", &w[..len - 3]);
        }
        // -ingly → (remove) (e.g., "amazingly" → "amaz")
        if w.ends_with("ingly") && len > 6 {
            return w[..len - 5].to_string();
        }
        // -ingly → but if ends with -yingly, handle specially
        // -ingly and -edly are common adverbial forms
        // General: if word ends with -ing, try stripping it
        if w.ends_with("ing") && len > 5 {
            let candidate = &w[..len - 3];
            // Only strip if the remaining part is at least 2 chars
            if candidate.len() >= 2 {
                return candidate.to_string();
            }
        }
        // -ed → (remove) (e.g., "walked" → "walk")
        if w.ends_with("ed") && len > 4 {
            return w[..len - 2].to_string();
        }

        // ── Step 3: Noun suffixes ──
        // -tion → -t (e.g., "action" → "act")
        if w.ends_with("tion") && len >= 6 {
            return format!("{}t", &w[..len - 4]);
        }
        // -ment → (remove) (e.g., "agreement" → "agree")
        if w.ends_with("ment") && len > 6 {
            return w[..len - 4].to_string();
        }
        // -ness → (remove) (e.g., "happiness" → "happi")
        if w.ends_with("ness") && len > 5 {
            return w[..len - 4].to_string();
        }
        // -ity → (remove) (e.g., "activity" → "activ")
        if w.ends_with("ity") && len > 5 {
            return w[..len - 3].to_string();
        }

        // ── Step 4: Adjective suffixes ──
        // -able → (remove) (e.g., "comfortable" → "comfort")
        if w.ends_with("able") && len > 6 {
            return w[..len - 4].to_string();
        }
        // -ible → (remove) (e.g., "possible" → "poss")
        if w.ends_with("ible") && len > 6 {
            return w[..len - 4].to_string();
        }
        // -ful → (remove) (e.g., "helpful" → "help")
        if w.ends_with("ful") && len > 5 {
            return w[..len - 3].to_string();
        }
        // -ous → (remove) (e.g., "dangerous" → "danger")
        if w.ends_with("ous") && len > 5 {
            return w[..len - 3].to_string();
        }
        // -al → (remove) if word is long enough (e.g., "global" → "glob")
        if w.ends_with("al") && len > 5 {
            return w[..len - 2].to_string();
        }
        // -ic → (remove) if word is long enough (e.g., "atomic" → "atom")
        if w.ends_with("ic") && len > 5 {
            return w[..len - 2].to_string();
        }

        // ── Step 5: Adverbial suffixes ──
        // -ly → (remove) (e.g., "quickly" → "quick")
        if w.ends_with("ly") && len > 4 {
            return w[..len - 2].to_string();
        }

        // No rule matched — return as-is (lowercased, via `w`)
        w.to_string()
    }

    // ─── Query type classification ─────────────────────────────────

    /// Classify the overall type of query.
    fn classify_type(
        &self,
        query: &str,
        tokens: &[String],
        has_ops: bool,
        has_nums: bool,
    ) -> QueryType {
        // Check for code patterns
        let code_keywords = [
            "function", "fn", "loop", "if", "else", "match", "return", "variable", "let", "const",
            "mut", "impl", "struct", "enum", "trait", "pub", "use", "mod", "async", "await",
        ];
        if code_keywords
            .iter()
            .any(|k| tokens.iter().any(|t| t.to_lowercase() == *k))
        {
            return QueryType::Code;
        }

        // Check for logic patterns
        let logic_keywords = [
            "modus",
            "ponens",
            "tollens",
            "therefore",
            "implies",
            "syllogism",
            "premise",
            "conclusion",
            "deduct",
            "induct",
            "abduct",
            "iff",
            "valid",
            "sound",
        ];
        if logic_keywords
            .iter()
            .any(|k| tokens.iter().any(|t| t.to_lowercase() == *k))
        {
            return QueryType::Logic;
        }

        // Check for grammar patterns
        let grammar_keywords = [
            "grammar",
            "syntax",
            "punctuation",
            "comma",
            "semicolon",
            "clause",
            "phrase",
            "noun",
            "verb",
            "adjective",
            "adverb",
            "subject",
            "predicate",
            "tense",
            "plural",
            "singular",
            "agreement",
            "serial",
            "oxford",
        ];
        if grammar_keywords
            .iter()
            .any(|k| tokens.iter().any(|t| t.to_lowercase() == *k))
        {
            return QueryType::Grammar;
        }

        // Check for math patterns
        if has_ops || has_nums {
            return QueryType::Math;
        }

        // Check if query matches any formula description
        if !self.formulas.search(query).is_empty() {
            return QueryType::Math;
        }

        // Check for any seed entity name match
        for id in self.entities.list_seeds() {
            if let Some(seed) = self.entities.get_seed(id) {
                let ename = seed.name.to_lowercase();
                if tokens.iter().any(|t| t.to_lowercase() == ename) {
                    return QueryType::Math; // Entities typically connect to math formulas
                }
            }
        }

        QueryType::Unknown
    }

    // ─── Intent scoring ────────────────────────────────────────────

    /// Score each intent type based on token evidence.
    ///
    /// Returns all 6 intent types with scores 0.0–1.0, sorted descending.
    fn score_intents(
        &self,
        query: &str,
        tokens: &[String],
        _significant: &[String],
    ) -> Vec<(Intent, f64)> {
        let lower = query.to_lowercase();
        let mut scores: Vec<(Intent, f64)> = Vec::new();

        // ── Intent: Validate ──
        let mut val_score: f64 = 0.0;
        if lower.starts_with("validate")
            || lower.starts_with("check")
            || lower.starts_with("verify")
            || lower.starts_with("confirm")
        {
            val_score = 1.0;
        } else if self.has_word_anycase(tokens, &["validate", "check", "verify", "confirm"]) {
            val_score = 0.8;
        } else if query.contains('=') && !query.contains("==") {
            // "F = ma" pattern — equation validation
            val_score = 0.6;
        }
        scores.push((Intent::Validate, val_score));

        // ── Intent: Traverse ──
        let mut trav_score: f64 = 0.0;
        if lower.starts_with("traverse")
            || lower.starts_with("explore")
            || lower.starts_with("follow")
            || lower.starts_with("navigate")
        {
            trav_score = 1.0;
        } else if self.has_word_anycase(
            tokens,
            &[
                "traverse", "explore", "follow", "navigate", "path", "walk", "from", "to",
                "through",
            ],
        ) {
            trav_score = 0.7;
        }
        // "from X to Y" pattern
        if lower.contains(" from ") && lower.contains(" to ") {
            trav_score = trav_score.max(0.8);
        }
        scores.push((Intent::Traverse, trav_score));

        // ── Intent: Compose ──
        let mut comp_score: f64 = 0.0;
        if lower.starts_with("compose")
            || lower.starts_with("chain")
            || lower.starts_with("connect")
        {
            comp_score = 1.0;
        } else if self.has_word_anycase(
            tokens,
            &[
                "compose", "chain", "connect", "bridge", "link", "combine", "sequence",
            ],
        ) {
            comp_score = 0.7;
        }
        scores.push((Intent::Compose, comp_score));

        // ── Intent: Search ──
        let mut search_score: f64 = 0.0;
        if lower.starts_with("search")
            || lower.starts_with("find")
            || lower.starts_with("lookup")
            || lower.starts_with("locate")
        {
            search_score = 1.0;
        } else if self.has_word_anycase(tokens, &["search", "find", "lookup", "locate", "discover"])
        {
            search_score = 0.7;
        }
        scores.push((Intent::Search, search_score));

        // ── Intent: Info ──
        let mut info_score: f64 = 0.0;
        if lower.starts_with("info")
            || lower.starts_with("help")
            || lower.starts_with("status")
            || lower.starts_with("about")
        {
            info_score = 1.0;
        } else if self.has_word_anycase(
            tokens,
            &[
                "info", "help", "status", "about", "what", "who", "explain", "describe",
            ],
        ) {
            info_score = 0.5;
        }
        scores.push((Intent::Info, info_score));

        // ── Intent: Evaluate (the default — compute something) ──
        let mut eval_score: f64 = 0.3; // Baseline — Evaluate is the default fallback
        if lower.starts_with("solve")
            || lower.starts_with("calculate")
            || lower.starts_with("eval")
            || lower.starts_with("compute")
        {
            eval_score = 1.0;
        } else if lower.starts_with("what")
            || lower.starts_with("how")
            || lower.starts_with("find")
            || self.has_word_anycase(
                tokens,
                &[
                    "solve",
                    "calculate",
                    "eval",
                    "compute",
                    "formula",
                    "equation",
                    "value",
                    "result",
                ],
            )
        {
            eval_score = 0.7;
        }

        // Boost Evaluate if:
        // - Has key=value args
        if self.has_word_anycase(tokens, &["="]) || query.contains('=') {
            eval_score = eval_score.max(0.8);
        }
        // - Has formula operators
        if query.contains('+') || query.contains('*') || query.contains('/') || query.contains('^')
        {
            eval_score = eval_score.max(0.85);
        }
        // - Has an exact formula ID
        if self.formulas.search(query).iter().any(|_| true) {
            eval_score = eval_score.max(0.75);
        }
        scores.push((Intent::Evaluate, eval_score));

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// Check if any of the given tokens match any of the target words (case-insensitive).
    fn has_word_anycase(&self, tokens: &[String], targets: &[&str]) -> bool {
        tokens.iter().any(|t| {
            let lower = t.to_lowercase();
            targets.iter().any(|target| lower == *target)
        })
    }

    // ─── Domain scoring ────────────────────────────────────────────

    /// Domain-specific keyword maps.
    /// Each domain has a list of associated terms that increase the score
    /// when found in the query.
    const DOMAIN_KEYWORDS: &'static [(Domain, &'static [&'static str])] = &[
        (
            Domain::Mangala,
            &[
                "math",
                "algebra",
                "geometry",
                "calculus",
                "arithmetic",
                "number",
                "equation",
                "function",
                "derivative",
                "integral",
                "logic",
                "proof",
                "theorem",
                "matrix",
                "vector",
            ],
        ),
        (
            Domain::Shukra,
            &[
                "physics",
                "force",
                "mass",
                "acceleration",
                "energy",
                "momentum",
                "velocity",
                "gravity",
                "friction",
                "wave",
                "chemistry",
                "atom",
                "molecule",
                "reaction",
                "element",
                "compound",
                "bond",
                "acid",
                "base",
                "thermo",
            ],
        ),
        (
            Domain::Budha,
            &[
                "astronomy",
                "star",
                "planet",
                "galaxy",
                "orbit",
                "moon",
                "solar",
                "cosmic",
                "telescope",
                "nebula",
                "cosmology",
                "universe",
                "space",
                "gravity",
                "light",
                "mars",
                "venus",
                "jupiter",
                "saturn",
                "mercury",
            ],
        ),
        (
            Domain::Chandra,
            &[
                "earth",
                "environment",
                "climate",
                "weather",
                "ocean",
                "atmosphere",
                "ecosystem",
                "geology",
                "forest",
                "water",
                "pollution",
                "carbon",
                "energy",
                "sustainable",
                "green",
            ],
        ),
        (
            Domain::Surya,
            &[
                "biology",
                "cell",
                "dna",
                "gene",
                "protein",
                "organism",
                "evolution",
                "species",
                "medicine",
                "disease",
                "drug",
                "symptom",
                "diagnosis",
                "treatment",
                "patient",
            ],
        ),
        (
            Domain::Budha,
            &[
                "economics",
                "finance",
                "market",
                "price",
                "cost",
                "revenue",
                "profit",
                "investment",
                "bank",
                "trade",
                "inflation",
                "gdp",
                "tax",
                "budget",
                "capital",
            ],
        ),
        (
            Domain::Shukra,
            &[
                "engineering",
                "design",
                "circuit",
                "system",
                "structure",
                "mechanical",
                "electrical",
                "software",
                "hardware",
                "bridge",
                "machine",
                "robot",
                "manufacturing",
            ],
        ),
        (
            Domain::Mangala,
            &[
                "computer",
                "algorithm",
                "data",
                "program",
                "code",
                "software",
                "network",
                "ai",
                "artificial intelligence",
                "machine learning",
                "neural",
                "comput",
                "binary",
            ],
        ),
        (
            Domain::Brihaspati,
            &[
                "history",
                "ancient",
                "civilization",
                "culture",
                "war",
                "revolution",
                "empire",
                "kingdom",
                "anthropology",
                "society",
                "tribe",
                "artifact",
            ],
        ),
        (
            Domain::Shani,
            &[
                "language",
                "linguistic",
                "word",
                "sentence",
                "grammar",
                "syntax",
                "semantic",
                "speech",
                "translation",
                "alphabet",
                "letter",
                "vowel",
            ],
        ),
        (
            Domain::Shani,
            &[
                "philosophy",
                "ethics",
                "moral",
                "justice",
                "right",
                "virtue",
                "truth",
                "knowledge",
                "existence",
                "consciousness",
                "logic",
                "reason",
                "argument",
            ],
        ),
        (
            Domain::Brihaspati,
            &[
                "psychology",
                "mind",
                "brain",
                "neuron",
                "behavior",
                "emotion",
                "memory",
                "perception",
                "cognition",
                "mental",
                "therapy",
                "disorder",
                "personality",
            ],
        ),
    ];

    /// Score each of the 12 domains based on token evidence.
    fn score_domains(
        &self,
        query: &str,
        _tokens: &[String],
        significant: &[String],
        _stems: &[String],
    ) -> Vec<(Domain, f64)> {
        let lower = query.to_lowercase();
        let mut scores: Vec<(Domain, f64)> = ALL_DOMAINS.iter().map(|d| (*d, 0.0)).collect();

        for (domain, keywords) in Self::DOMAIN_KEYWORDS {
            for kw in *keywords {
                // Exact word match in significant tokens (highest weight)
                if significant.iter().any(|t| t.eq_ignore_ascii_case(kw)) {
                    if let Some(entry) = scores.iter_mut().find(|(d, _)| d == domain) {
                        entry.1 += 0.25;
                    }
                }
                // Partial match in query (lower weight, but catches compounds)
                else if lower.contains(kw) {
                    if let Some(entry) = scores.iter_mut().find(|(d, _)| d == domain) {
                        entry.1 += 0.10;
                    }
                }
            }
        }

        // Domain name mention — strongest single signal
        for d in ALL_DOMAINS.iter() {
            let dname = d.full_name_lower();
            if lower.contains(dname) {
                if let Some(entry) = scores.iter_mut().find(|(sd, _)| sd == d) {
                    entry.1 += 1.0; // Direct domain mention = near-certain
                }
            }
            // Also check short alias
            let dalias = d.symbol().to_lowercase();
            if significant.iter().any(|t| t.eq_ignore_ascii_case(&dalias)) {
                if let Some(entry) = scores.iter_mut().find(|(sd, _)| sd == d) {
                    entry.1 = entry.1.max(1.0);
                }
            }
        }

        // Entity match boost — if a seed entity matches, boost its domain
        for id in self.entities.list_seeds() {
            if let Some(seed) = self.entities.get_seed(id) {
                let ename = seed.name.to_lowercase();
                if significant
                    .iter()
                    .any(|t| t.eq_ignore_ascii_case(&ename) || ename.contains(&t.to_lowercase()))
                {
                    // Determine domain from seed's classification
                    if let Some(sign) = seed.classification.as_ref().and_then(|c| c.dominant_sign())
                    {
                        let seed_domain = crate::wheel::Domain::from_sign(sign);
                        if let Some(entry) = scores.iter_mut().find(|(d, _)| *d == seed_domain) {
                            entry.1 += 0.5;
                        }
                    }
                }
            }
        }

        // Normalize: cap at 1.0
        for (_, score) in &mut scores {
            *score = score.min(1.0);
        }

        // Remove zero-score domains and sort
        scores.retain(|(_, s)| *s > 0.0);
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    // ─── Entity recognition ────────────────────────────────────────

    /// Fuzzy-match tokens against seed entities.
    ///
    /// Returns up to 5 matches, sorted by confidence descending.
    fn recognize_entities(
        &self,
        query: &str,
        tokens: &[String],
        stems: &[String],
    ) -> Vec<(String, String, f64)> {
        let lower = query.to_lowercase();
        let mut matches: Vec<(String, String, f64)> = Vec::new();

        for id in self.entities.list_seeds() {
            let Some(seed) = self.entities.get_seed(id) else {
                continue;
            };
            let ename_lower = seed.name.to_lowercase();
            let mut best_confidence: f64 = 0.0;

            // Level 1: Exact match (query IS the entity name)
            if lower.trim() == ename_lower || lower.trim() == seed.id.to_lowercase() {
                best_confidence = 1.0;
            }
            // Level 2: Token-level exact match
            else if tokens
                .iter()
                .any(|t| t.eq_ignore_ascii_case(&seed.name) || t.eq_ignore_ascii_case(&seed.id))
            {
                best_confidence = 0.95;
            }
            // Level 3: Entity name contains a token (multi-word entity matching)
            else {
                for token in tokens {
                    let tlower = token.to_lowercase();
                    // Token is part of entity name
                    if ename_lower.contains(&tlower) && tlower.len() > 3 {
                        best_confidence = best_confidence.max(0.7);
                    }
                    // Entity name starts with token (abbreviation)
                    if ename_lower.starts_with(&tlower) && tlower.len() >= 2 {
                        best_confidence = best_confidence.max(0.6);
                    }
                    // Stem match
                    let stemmed = self.stem(&token.to_lowercase());
                    if stems.iter().any(|s| s == &stemmed) {
                        best_confidence = best_confidence.max(0.5);
                    }
                }
            }

            if best_confidence > 0.0 {
                matches.push((seed.id.clone(), seed.name.clone(), best_confidence));
            }
        }

        // Sort by confidence descending, take top 5
        matches.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        matches.truncate(5);
        matches
    }
}

// ─── Unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityRegistry, SeedEntity};
    use crate::formula::{Formula, FormulaRegistry};
    use crate::wheel::Domain;

    fn setup_engine() -> NlpEngine {
        let mut formulas = FormulaRegistry::new();
        formulas
            .register_all(vec![Formula::atomic(
                "newtons_second",
                Domain::Shukra,
                vec!["mass", "acceleration"],
                "force",
                "mass * acceleration",
                "F = ma",
            )])
            .unwrap();
        let entities = EntityRegistry::new();
        NlpEngine::new(entities, formulas)
    }

    fn setup_engine_with_entities() -> NlpEngine {
        use crate::astrology::ChangeSorter;
        let formulas = FormulaRegistry::new();
        let mut entities = EntityRegistry::new();
        let sorter = ChangeSorter::new();
        entities.register_seed(SeedEntity {
            id: "mars".into(),
            name: "Mars".into(),
            description: "The red planet".into(),
            classification: Some(sorter.classify_token("mars")),
            ..Default::default()
        });
        entities.register_seed(SeedEntity {
            id: "dopamine".into(),
            name: "Dopamine".into(),
            description: "Neurotransmitter".into(),
            classification: Some(sorter.classify_token("dopamine")),
            ..Default::default()
        });
        NlpEngine::new(entities, formulas)
    }

    #[test]
    fn test_tokenize_simple() {
        let engine = setup_engine();
        let tokens = engine.tokenize("hello world");
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_tokenize_key_value() {
        let engine = setup_engine();
        let tokens = engine.tokenize("mass=5 acceleration=9.8");
        assert_eq!(tokens, vec!["mass=5", "acceleration=9.8"]);
    }

    #[test]
    fn test_tokenize_punctuation() {
        let engine = setup_engine();
        let tokens = engine.tokenize("what's newton's law?");
        // apostrophe and hyphen are kept, trailing punctuation stripped
        assert_eq!(tokens, vec!["what's", "newton's", "law"]);
    }

    #[test]
    fn test_stopword_removal() {
        let engine = setup_engine();
        let tokens = vec![
            "the".to_string(),
            "force".to_string(),
            "of".to_string(),
            "an".to_string(),
            "object".to_string(),
        ];
        let significant = engine.remove_stopwords(&tokens);
        assert_eq!(significant, vec!["force", "object"]);
    }

    #[test]
    fn test_stem_plural() {
        let engine = setup_engine();
        assert_eq!(engine.stem("forces"), "force");
        assert_eq!(engine.stem("boxes"), "boxe");
        assert_eq!(engine.stem("classes"), "class");
        assert_eq!(engine.stem("studies"), "studi");
    }

    #[test]
    fn test_stem_ing() {
        let engine = setup_engine();
        assert_eq!(engine.stem("calculating"), "calculat");
        assert_eq!(engine.stem("running"), "runn");
        assert_eq!(engine.stem("being"), "being"); // short word, no stem
    }

    #[test]
    fn test_stem_ed() {
        let engine = setup_engine();
        assert_eq!(engine.stem("calculated"), "calculat");
        assert_eq!(engine.stem("walked"), "walk");
    }

    #[test]
    fn test_stem_tion() {
        let engine = setup_engine();
        assert_eq!(engine.stem("acceleration"), "accelerat");
        assert_eq!(engine.stem("action"), "act");
    }

    #[test]
    fn test_stem_ly() {
        let engine = setup_engine();
        assert_eq!(engine.stem("quickly"), "quick");
        assert_eq!(engine.stem("slowly"), "slow");
    }

    #[test]
    fn test_intent_scoring_validate() {
        let engine = setup_engine();
        let ctx = engine.preprocess("validate F = ma");
        assert_eq!(ctx.likely_intent, Intent::Validate);
        assert!(ctx.intent_scores[0].1 >= 0.8);
    }

    #[test]
    fn test_intent_scoring_evaluate() {
        let engine = setup_engine();
        let ctx = engine.preprocess("solve newtons_second mass=5 acceleration=9.8");
        assert_eq!(ctx.likely_intent, Intent::Evaluate);
    }

    #[test]
    fn test_intent_scoring_search() {
        let engine = setup_engine();
        let ctx = engine.preprocess("search momentum");
        assert_eq!(ctx.likely_intent, Intent::Search);
    }

    #[test]
    fn test_intent_scoring_traverse() {
        let engine = setup_engine();
        let ctx = engine.preprocess("traverse from aries to taurus");
        assert_eq!(ctx.likely_intent, Intent::Traverse);
    }

    #[test]
    fn test_domain_scoring() {
        let engine = setup_engine();
        let ctx = engine.preprocess("calculate force in physics");
        let top_domain = ctx.likely_domain;
        assert_eq!(top_domain, Some(Domain::Shukra));
        // At least one domain should match
        assert!(!ctx.domain_scores.is_empty());
    }

    #[test]
    fn test_domain_scoring_math() {
        let engine = setup_engine();
        let ctx = engine.preprocess("solve derivative in math");
        let top_domain = ctx.likely_domain;
        assert_eq!(top_domain, Some(Domain::Mangala));
    }

    #[test]
    fn test_entity_recognition_exact() {
        let engine = setup_engine_with_entities();
        let ctx = engine.preprocess("tell me about Mars");
        assert!(!ctx.entity_matches.is_empty());
        assert_eq!(ctx.entity_matches[0].0, "mars");
        assert!(ctx.entity_matches[0].2 > 0.9);
    }

    #[test]
    fn test_entity_recognition_fuzzy() {
        let engine = setup_engine_with_entities();
        let ctx = engine.preprocess("dopamine levels");
        assert!(!ctx.entity_matches.is_empty());
        assert_eq!(ctx.entity_matches[0].0, "dopamine");
    }

    #[test]
    fn test_query_type_math() {
        let engine = setup_engine();
        let ctx = engine.preprocess("F = ma mass=5 acceleration=9.8");
        assert_eq!(ctx.query_type, QueryType::Math);
    }

    #[test]
    fn test_query_type_numbers() {
        let engine = setup_engine();
        let ctx = engine.preprocess("calculate 5 * 9.8");
        assert_eq!(ctx.query_type, QueryType::Math);
    }

    #[test]
    fn test_exact_formula_id_detection() {
        let engine = setup_engine();
        let ctx = engine.preprocess("newtons_second mass=5");
        assert!(ctx.has_exact_formula_id);
        assert!(!ctx.exact_formula_ids.is_empty());
    }

    #[test]
    fn test_nlp_pipeline_runs_without_entities() {
        let engine = setup_engine();
        let ctx = engine.preprocess("hello world");
        // Should not crash even with empty entity registry
        assert_eq!(ctx.likely_intent, Intent::Evaluate); // default fallback
        assert!(ctx.domain_scores.is_empty()); // no domain detected
        assert!(ctx.entity_matches.is_empty()); // no entities
    }
}
