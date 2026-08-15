//! # Reverse Routing — query → graha forces → strategy
//!
//! The descent engine maps a query's tokens onto the 9-graha wheel
//! (`Domain` nodes). Reverse routing *inverts* that flow: given the dominant
//! graha forces a query activated, synthesize a context-aware strategic
//! framework — without inventing or speculating. The strategy emerges from the
//! query's own semantic structure, so it is deterministic and reproducible.
//!
//! Strategy mapping audited in `laverna_reverse_routing_strategy.md`: each
//! graha is an archetypal force with a standing strategic principle.

use crate::chart::personality::Pillar;
use crate::descent::SettledToken;
use crate::domain_graph::Domain;
use crate::nlp::is_stopword;

/// Strategic principle carried by each graha (archetypal force). This is the
/// "upward" leg of reverse routing: force → recommended action framework.
pub fn principle_of_strategy(graha: Domain) -> &'static str {
    match graha {
        Domain::Surya => "Protect the irreducible core; lead from first principles",
        Domain::Chandra => "Listen, adapt, and respect natural cycles",
        Domain::Mangala => "Build, test, verify, and fail fast",
        Domain::Budha => "Articulate clearly and link ideas precisely",
        Domain::Brihaspati => "Extract principles and scale understanding",
        Domain::Shukra => "Bridge domains and integrate systems harmoniously",
        Domain::Shani => "Honor limits and work within structure",
        Domain::Rahu => "Transcend boundaries and evolve",
        Domain::Ketu => "Let go, detach, and consolidate",
    }
}

/// Default assimilation target for a graha force when a repo has no specific
/// profile. Used by `route --repos` to map an unknown repo's dominant force to
/// where it belongs in the Laverna ecosystem.
pub fn graha_default_target(graha: Domain) -> &'static str {
    match graha {
        Domain::Surya => "Protect core / lead (anchor the reboot)",
        Domain::Chandra => "Listen & adapt (UX / iteration)",
        Domain::Mangala => "Build / test / verify (engineering subsystem)",
        Domain::Budha => "Articulate / link (docs & bridges)",
        Domain::Brihaspati => "Extract principles (validation layer)",
        Domain::Shukra => "Bridge / integrate (cross-domain glue)",
        Domain::Shani => "Honor limits (sandbox / structure)",
        Domain::Rahu => "Transcend boundaries (experimental layer)",
        Domain::Ketu => "Let go / consolidate (prune / archive)",
    }
}

/// A synthesized strategy report for a single query.
#[derive(Debug, Clone)]
pub struct StrategyReport {
    /// The original query text.
    pub query: String,
    /// Dominant graha forces, ranked by accumulated specificity weight (desc),
    /// then wheel index. Tuple is `(graha, weight, share_of_total_weight [0,1])`.
    pub ranked: Vec<(Domain, f64, f64)>,
    /// Strongest force (primary strategy).
    pub primary: Option<Domain>,
    /// Second force (secondary / balancing strategy).
    pub secondary: Option<Domain>,
    /// Third force (tertiary strategy), if present.
    pub tertiary: Option<Domain>,
    /// Content tokens (post-stopword) that mapped to no corpus graha.
    pub unresolved: Vec<String>,
    /// Tokens filtered out as stopwords before scoring.
    pub stopwords: Vec<String>,
    /// Fail-loud diagnostic when routing confidence is too low to trust
    /// (e.g. no forces resolved, or most content tokens unresolved).
    pub warning: Option<String>,
    /// True when a strategy WAS synthesized but on thin evidence (T53): at
    /// least one content token resolved to a graha, yet a majority of content
    /// tokens were unresolved. Callers should surface this as a low-confidence
    /// best-guess rather than refusing outright — refusing the tool's core
    /// competency (metacognitive/reasoning queries) is worse than a flagged
    /// guess. `primary` is still populated in this case; it is only `None` when
    /// *nothing* resolved.
    pub low_confidence: bool,
}

/// Per-token routing classification feeding `synthesize_strategy`.
///
/// Pure: derived from the token text alone (corpus lookup + stopword set),
/// **not** from any other token in the query. This is the T54 fix — routing no
/// longer inherits a neighbor's domain via query-global constraint propagation,
/// and stopwords/unknown words no longer invent a graha.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenForce {
    /// Function word excluded from scoring (T55). Carries no domain signal.
    Stopword,
    /// Survived stopword filtering but maps to no corpus graha.
    Unresolved,
    /// Mapped to a single graha with a corpus-specificity weight in (0, 1].
    Resolved { graha: Domain, weight: f64 },
}

impl StrategyReport {
    /// Human-readable strategy report.
    pub fn format(&self) -> String {
        let mut out = String::new();
        out.push_str("═══ Reverse-Routing Strategy ═══\n");
        out.push_str(&format!("query: \"{}\"\n\n", self.query));

        out.push_str("GRAHA FORCES (by specificity weight):\n");
        if self.ranked.is_empty() {
            out.push_str("  (no graha forces resolved — query is outside the wheel's scope)\n");
        } else {
            for (graha, weight, share) in &self.ranked {
                out.push_str(&format!(
                    "  {} {} ({}) — {} — weight {:.3} ({:.0}%) — {}\n",
                    graha.symbol(),
                    graha.name(),
                    graha.full_name(),
                    graha.archetype(),
                    weight,
                    share * 100.0,
                    principle_of_strategy(*graha),
                ));
            }
        }

        if !self.stopwords.is_empty() {
            out.push_str(&format!(
                "\nstopwords (excluded from scoring): {}\n",
                self.stopwords.join(", ")
            ));
        }
        if !self.unresolved.is_empty() {
            out.push_str(&format!(
                "\nunresolved (no corpus graha): {}\n",
                self.unresolved.join(", ")
            ));
        }
        if let Some(warning) = &self.warning {
            out.push_str(&format!("\n⚠ {warning}\n"));
        }

        out.push_str("\nSYNTHESIZED STRATEGY:\n");
        if self.low_confidence {
            out.push_str("  (low confidence — best-guess from partial resolution)\n");
        }
        match self.primary {
            Some(g) => out.push_str(&format!(
                "  PRIMARY:    {} ({}) — {}\n",
                g.archetype(),
                g.name(),
                principle_of_strategy(g),
            )),
            None => out.push_str("  PRIMARY:    (none)\n"),
        }
        match self.secondary {
            Some(g) => out.push_str(&format!(
                "  SECONDARY:  {} ({}) — {}\n",
                g.archetype(),
                g.name(),
                principle_of_strategy(g),
            )),
            None => out.push_str("  SECONDARY:  (none)\n"),
        }
        match self.tertiary {
            Some(g) => out.push_str(&format!(
                "  TERTIARY:   {} ({}) — {}\n",
                g.archetype(),
                g.name(),
                principle_of_strategy(g),
            )),
            None => out.push_str("  TERTIARY:   (none)\n"),
        }

        out.push_str(
            "\nThis strategy emerges from the query's semantic structure — no speculation.\n",
        );
        out
    }
}

/// Pure: the single strongest graha force a token resolved to, read ONLY from
/// the token's own scored vedic classification. No query-global fallback — a
/// token with no scored graha weight (stopword, unknown word, or a word whose
/// force was only ever propagated from a neighbor) resolves to `None` rather
/// than inheriting a sibling's domain. This is the core T54 fix: routing is now
/// a function of the token text alone, so identical tokens route identically
/// regardless of their neighbors in a query (T54).
pub fn dominant_graha_of(token: &SettledToken) -> Option<Domain> {
    let best = token
        .vedic_classification
        .grahas
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    match best {
        Some((i, &w)) if w > 0.0 => Domain::from_index(i),
        _ => None,
    }
}

/// Display/downstream view of a token's dominant graha. `domains` is the
/// authoritative classification used for unification (keyword→formula
/// `DomainClassification` voting and the FormulaMatch shortcut), whereas
/// `dominant_graha_of` reads the auxiliary `vedic_classification` vector that
/// is populated through a *separate* sign→ruler mapping and is not kept in
/// sync with `domains` (T53: null on FormulaMatch, or disagreement on ties).
/// For any display purpose we prefer the actual winning domain so the shown
/// graha always agrees with `domains`; routing stays on the pure vedic signal
/// via `dominant_graha_of`.
pub fn dominant_graha_display(token: &SettledToken) -> Option<Domain> {
    if let Some(domain) = token.domains.first() {
        return Some(*domain);
    }
    dominant_graha_of(token)
}

/// Build the per-token `TokenForce` classification for a descent matrix token.
///
/// - Stopwords (function words) are filtered out before scoring (T55).
/// - Everything else resolves through the *pure* `dominant_graha_of` — no
///   neighbor-domain inheritance (T54). Tokens with no corpus graha are
///   reported as `Unresolved`.
/// - Resolved tokens carry a corpus-specificity `weight` (higher = rarer /
///   more discriminating). The caller supplies the weight via `specificity`,
///   which keeps this function free of any registry dependency.
pub fn classify_route_token(token: &SettledToken, specificity: f64) -> TokenForce {
    if is_stopword(&token.text) {
        return TokenForce::Stopword;
    }
    match dominant_graha_of(token) {
        Some(graha) => TokenForce::Resolved {
            graha,
            weight: specificity,
        },
        None => TokenForce::Unresolved,
    }
}

/// Fail-loud threshold: if this fraction (or more) of *content* tokens are
/// unresolved, the report carries a warning and the primary/secondary/tertiary
/// forces are left `None` rather than guessed from noise.
const UNRESOLVED_WARN_FRACTION: f64 = 0.5;

/// Pure reverse-routing synthesis: each content token contributes its resolved
/// graha's specificity `weight`; the forces are ranked and the
/// primary/secondary/tertiary are picked. Deterministic — identical inputs
/// yield identical reports. Stopwords and unresolved tokens are recorded but
/// carry no vote (T54/T55).
pub fn synthesize_strategy(query: &str, forces: &[(String, TokenForce)]) -> StrategyReport {
    let mut weights = [0.0f64; 9];
    let mut unresolved = Vec::new();
    let mut stopwords = Vec::new();
    let mut resolved_total = 0.0f64;
    let mut content_count = 0usize;

    for (text, force) in forces {
        match force {
            TokenForce::Stopword => stopwords.push(text.clone()),
            TokenForce::Unresolved => {
                unresolved.push(text.clone());
                content_count += 1;
            }
            TokenForce::Resolved { graha, weight } => {
                weights[graha.index()] += *weight;
                resolved_total += *weight;
                content_count += 1;
            }
        }
    }

    let mut ranked: Vec<(Domain, f64, f64)> = Domain::all()
        .iter()
        .map(|&graha| (graha, weights[graha.index()], 0.0))
        .filter(|(_, weight, _)| *weight > 0.0)
        .collect();

    ranked.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.index().cmp(&b.0.index()))
    });
    for entry in ranked.iter_mut() {
        entry.2 = if resolved_total > 0.0 {
            entry.1 / resolved_total
        } else {
            0.0
        };
    }

    // Two distinct low-confidence conditions, handled differently (T53):
    //   1. NOTHING resolved (resolved_total == 0): genuinely outside scope —
    //      fail loud, assert no strategy (→ caller emits OutOfScope).
    //   2. SOMETHING resolved but a majority of content tokens missed: thin
    //      evidence, not zero evidence. Emit a flagged best-guess rather than
    //      refusing the tool's core competency.
    let nothing_resolved = resolved_total == 0.0;
    let low_confidence = !nothing_resolved
        && content_count > 0
        && unresolved.len() as f64 / content_count as f64 >= UNRESOLVED_WARN_FRACTION;

    let warning = if nothing_resolved {
        Some(
            "no graha forces resolved — query maps to no corpus domain; routing is speculative"
                .to_string(),
        )
    } else if low_confidence {
        Some(format!(
            "{} of {} content tokens unresolved — low-confidence best-guess",
            unresolved.len(),
            content_count
        ))
    } else {
        None
    };

    // Fail loud only when NOTHING resolved: leave primary/secondary/tertiary
    // `None` so the caller refuses (T54). When at least one token resolved we
    // still surface the ranked best-guess — flagged via `low_confidence` — so
    // metacognitive/reasoning queries route instead of dropping to OutOfScope.
    let (primary, secondary, tertiary) = if nothing_resolved {
        (None, None, None)
    } else {
        (
            ranked.first().map(|(g, _, _)| *g),
            ranked.get(1).map(|(g, _, _)| *g),
            ranked.get(2).map(|(g, _, _)| *g),
        )
    };

    StrategyReport {
        query: query.to_string(),
        ranked,
        primary,
        secondary,
        tertiary,
        unresolved,
        stopwords,
        warning,
        low_confidence,
    }
}

/// Map a wheel `Domain` (graha) onto its strategic `Pillar`. Rahu/Ketu carry no
/// pillar (they are boundary/detachment forces, not capability axes), so they
/// return `None`. Mirrors `chart::personality::graha_to_pillar`.
pub fn domain_to_pillar(graha: Domain) -> Option<Pillar> {
    match graha {
        Domain::Surya => Some(Pillar::Spear),
        Domain::Chandra => Some(Pillar::Olive),
        Domain::Mangala => Some(Pillar::Forge),
        Domain::Budha => Some(Pillar::Owl),
        Domain::Brihaspati => Some(Pillar::Council),
        Domain::Shukra => Some(Pillar::Loom),
        Domain::Shani => Some(Pillar::Stone),
        Domain::Rahu | Domain::Ketu => None,
    }
}

/// Threshold for considering a secondary graha in a token's vedic classification
/// as a meaningful co-presence (a "conjunction" proxy).
const CONJUNCTION_THRESHOLD: f64 = 0.2;

/// Bonus applied to a pillar when its graha co-occurs with another graha's
/// pillar within the same token (conjunction proxy).
const CONJUNCTION_BONUS: f64 = 0.04;

/// Smaller bonus when different tokens in the same query map to different
/// grahas (weaker "aspect" proxy).
const CO_OCCURRENCE_BONUS: f64 = 0.02;

/// Apply graha interaction modifiers to an in-progress pillar weight array.
///
/// Since `strategize` has no chart positions, we proxy interactions using:
/// - **Token-level co-presence**: when a single token's `vedic_classification`
///   carries multiple grahas above `CONJUNCTION_THRESHOLD`, we apply a
///   conjunction-like bonus between those grahas' pillars.
/// - **Query-level co-occurrence**: when two different tokens resolve to
///   distinct grahas, we apply a weaker co-occurrence bonus.
///
/// The matrix is passed in so we can read per-token `vedic_classification`
/// vectors beyond `dominant_graha_of`.
fn apply_pillar_interactions(pillars: &mut [f64; 7], matrix: &crate::descent::SettlingMatrix) {
    // Phase 1: token-level co-presence (conjunction proxy).
    for token in &matrix.tokens {
        if crate::nlp::is_stopword(&token.text) {
            continue;
        }
        let vc = &token.vedic_classification;
        // Collect all grahas above threshold.
        let present: Vec<Domain> = Domain::all()
            .iter()
            .filter(|&&g| vc.grahas[g.index()] >= CONJUNCTION_THRESHOLD)
            .copied()
            .collect();
        if present.len() < 2 {
            continue;
        }
        // Every pair of co-present grahas gets a conjunction bonus.
        for i in 0..present.len() {
            for j in (i + 1)..present.len() {
                let ga = present[i];
                let gb = present[j];
                if let (Some(pa), Some(pb)) = (domain_to_pillar(ga), domain_to_pillar(gb)) {
                    pillars[pa.index()] += CONJUNCTION_BONUS;
                    pillars[pb.index()] += CONJUNCTION_BONUS;
                }
                // If one side is Rahu/Ketu, distribute to all pillars.
                if domain_to_pillar(ga).is_none() {
                    let dist = CONJUNCTION_BONUS / 7.0;
                    for w in pillars.iter_mut() {
                        *w += dist;
                    }
                }
                if domain_to_pillar(gb).is_none() {
                    let dist = CONJUNCTION_BONUS / 7.0;
                    for w in pillars.iter_mut() {
                        *w += dist;
                    }
                }
            }
        }
    }

    // Phase 2: query-level co-occurrence (weaker aspect proxy).
    let resolved_pillars: Vec<Option<Pillar>> = matrix
        .tokens
        .iter()
        .filter(|t| !crate::nlp::is_stopword(&t.text))
        .map(|t| {
            let dominant = crate::strategy::dominant_graha_of(t);
            dominant.and_then(domain_to_pillar)
        })
        .collect();
    for i in 0..resolved_pillars.len() {
        for j in (i + 1)..resolved_pillars.len() {
            if let (Some(pa), Some(pb)) = (resolved_pillars[i], resolved_pillars[j]) {
                if pa != pb {
                    pillars[pa.index()] += CO_OCCURRENCE_BONUS;
                    pillars[pb.index()] += CO_OCCURRENCE_BONUS;
                }
            }
        }
    }
}

/// Aggregate a `StrategyReport`'s per-graha shares into a 7-pillar objective
/// vector. Each graha's `share` (already in `[0,1]`, summing to 1.0 over
/// resolved grahas) is added into its pillar bucket. Pillars that no graha
/// resolved to stay at their base value (0.0 before interactions).
///
/// Then applies graha interaction modifiers (conjunction/co-occurrence proxies)
/// as additive bonuses. The result is NOT re-normalized — the caller's optimizer
/// receives the raw weight vector as-is.
///
/// Pure + deterministic. The result feeds the optimizer as `objective.weights`.
pub fn aggregate_pillars(
    report: &StrategyReport,
    matrix: &crate::descent::SettlingMatrix,
) -> [f64; 7] {
    let mut pillars = [0.0f64; 7];
    for (graha, _weight, share) in &report.ranked {
        if let Some(pillar) = domain_to_pillar(*graha) {
            pillars[pillar.index()] += *share;
        }
    }

    // Apply graha interaction modifiers (conjunction/co-occurrence proxies).
    apply_pillar_interactions(&mut pillars, matrix);

    pillars
}

/// Parse an external sensor-force TOML file into a name→weight map.
///
/// Accepts either a `[forces]` table (`[forces]\nForge = 1.5`) or a flat
/// top-level table (`Forge = 1.5`). Fail-loud on any parse error.
pub fn parse_sensor_forces(
    toml_str: &str,
) -> Result<std::collections::HashMap<String, f64>, String> {
    #[derive(serde::Deserialize)]
    struct Wrapped {
        forces: std::collections::HashMap<String, f64>,
    }
    if let Ok(w) = toml::from_str::<Wrapped>(toml_str) {
        Ok(w.forces)
    } else {
        toml::from_str::<std::collections::HashMap<String, f64>>(toml_str)
            .map_err(|e| format!("sensor-force parse error: {e}"))
    }
}

/// Match a lowercased pillar name (Spear/Forge/…) to its `Pillar`.
fn pillar_by_name(lower: &str) -> Option<Pillar> {
    for i in 0..Pillar::COUNT {
        if Pillar::from_index(i).name().to_lowercase() == lower {
            return Some(Pillar::from_index(i));
        }
    }
    None
}

/// Match a lowercased graha name (surya/mangala/…) to its `Pillar`.
fn graha_name_to_pillar(lower: &str) -> Option<Pillar> {
    for graha in Domain::all() {
        if graha.name().to_lowercase() == lower {
            return domain_to_pillar(graha);
        }
    }
    None
}

/// Blend external sensor/graha forces into the pillar weight vector (the
/// "real-time pillar reweight from sensor/query forces" path in `strategize`).
///
/// Each key is matched against a pillar name (e.g. `Forge`) or a graha name
/// (e.g. `mangala`); the matching pillar receives the added weight. Keys that
/// match neither are ignored (the caller surfaces which keys were applied).
/// Pure + deterministic.
pub fn reweight_pillars_with_sensor_forces(
    pillars: &mut [f64; 7],
    forces: &std::collections::HashMap<String, f64>,
) {
    for (name, weight) in forces {
        let lower = name.to_lowercase();
        if let Some(pillar) = pillar_by_name(&lower).or_else(|| graha_name_to_pillar(&lower)) {
            pillars[pillar.index()] += *weight;
        }
    }
}

// ── L 0.5 strategy-engine foundation (L Elevation Directive) ──────────────────
//
// These helpers build the explicit world/state/strategy models and wire the
// existing deterministic optimizer into the strategy pipeline. They are kept
// small and independently testable; the full pipeline (§2) is assembled in the
// `lai strategy` CLI command.

use crate::entity::EntityRegistry;
use crate::optimize::{Allocation, Schema};
use lai_core::{
    Claim, EntityResolution, EpistemicStatus, Observation, Strategy, StrategyIR, WorldState,
};

/// Resolve the surface mentions in `text` against the embedded corpus entity
/// registry. Builds n-grams (1..=3 words) so multi-word concepts like
/// "distributed systems" are treated as one concept rather than blind tokens.
///
/// Precision rules (the directive's highest-priority semantic fix):
/// - single words match only an exact entity `id`/`name` (not free-text
///   descriptions, which would silently resolve stopwords to facts);
/// - multi-word phrases may match a description substring (a real concept);
/// - stopwords and bare numbers never resolve.
///
/// Low-confidence or unresolved mentions are surfaced as concepts, never
/// silently upgraded to authoritative facts.
pub fn resolve_entities(text: &str, reg: &EntityRegistry) -> Vec<EntityResolution> {
    let words: Vec<String> = text
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    let mut consumed: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out: Vec<EntityResolution> = Vec::new();
    // Longest n-grams first so "distributed systems" wins over "systems".
    for n in (1..=3).rev() {
        for window in words.windows(n) {
            let phrase = window.join(" ");
            if consumed.contains(&phrase) {
                continue;
            }
            if n == 1 {
                if crate::nlp::is_stopword(&phrase) {
                    continue;
                }
                if phrase.chars().all(|c| c.is_ascii_digit()) {
                    continue;
                }
            }
            // Exact id/name match first (high precision). Corpus ids are
            // snake_case while surface text has spaces, so normalize both sides.
            let phrase_norm = phrase.replace('_', " ");
            let mut hits: Vec<&crate::entity::SeedEntity> = reg
                .seeds()
                .filter(|s| {
                    s.id.replace('_', " ").to_lowercase() == phrase_norm
                        || s.name.replace('_', " ").to_lowercase() == phrase_norm
                })
                .collect();
            // Multi-word concepts may match a description substring.
            if hits.is_empty() && n >= 2 {
                hits = reg
                    .seeds()
                    .filter(|s| s.description.to_lowercase().contains(&phrase))
                    .collect();
            }
            if hits.is_empty() {
                continue;
            }
            consumed.insert(phrase.clone());
            let best = hits[0];
            let exact = best.id.replace('_', " ").to_lowercase() == phrase_norm
                || best.name.replace('_', " ").to_lowercase() == phrase_norm;
            let confidence = if exact { 0.95 } else { 0.8 };
            let alternatives: Vec<String> =
                hits.iter().skip(1).take(3).map(|h| h.id.clone()).collect();
            out.push(EntityResolution {
                mention: phrase.clone(),
                canonical_name: best.id.clone(),
                entity_type: "seed_entity".into(),
                domain: best.dominant_graha().map(|g| format!("{g:?}")),
                confidence,
                evidence: format!("matched '{phrase}' in corpus ({} hit(s))", hits.len()),
                alternatives,
            });
        }
    }
    out
}

/// Extract resource constraints and a numeric budget from free text.
/// Recognizes `budget <= N`, `budget < N`, and `time <= N days` style phrases.
pub fn parse_resource_constraints(text: &str) -> (f64, Vec<String>) {
    let lower = text.to_lowercase();
    let mut budget = 0.0f64;
    let mut constraints = Vec::new();

    // budget (<= or <) — spaced forms only, to avoid double-matching.
    for (pat, le) in [("budget <= ", true), ("budget < ", false)] {
        if let Some(pos) = lower.find(pat) {
            let rest = &lower[pos + pat.len()..];
            if let Some(num) = rest.split_whitespace().next().and_then(|t| {
                t.trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
                    .parse::<f64>()
                    .ok()
            }) {
                budget = num;
                constraints.push(format!("budget {} {num}", if le { "<=" } else { "<" }));
            }
        }
    }
    // time constraint
    if let Some(pos) = lower.find("time <=") {
        let rest = &lower[pos + "time <=".len()..];
        if let Some(num) = rest.split_whitespace().next().and_then(|t| {
            t.trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
                .parse::<f64>()
                .ok()
        }) {
            constraints.push(format!("time <= {num} days"));
        }
    }
    (budget, constraints)
}

/// Build a [`WorldState`] from a free-text situation: resolve entities, record
/// the observation as a fact (epistemic `Observed`), and capture any parsed
/// resource constraints.
pub fn build_world_state(text: &str, reg: &EntityRegistry, ts: Option<String>) -> WorldState {
    let mut ws = WorldState::new();
    let res = resolve_entities(text, reg);
    let resolved_mentions: std::collections::HashSet<&str> =
        res.iter().map(|r| r.mention.as_str()).collect();
    for r in &res {
        if r.confidence >= 0.5 {
            ws.entities
                .insert(r.mention.clone(), r.canonical_name.clone());
        } else {
            ws.concepts.push(r.mention.clone());
        }
    }
    // Surface unresolved multi-word phrases as concepts (never silently dropped).
    let words: Vec<String> = text
        .split(|c: char| !(c.is_alphanumeric() || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_lowercase())
        .collect();
    for n in 2..=3 {
        for window in words.windows(n) {
            let phrase = window.join(" ");
            if resolved_mentions.contains(phrase.as_str()) {
                continue;
            }
            if phrase.split_whitespace().all(crate::nlp::is_stopword) {
                continue;
            }
            if !ws.concepts.contains(&phrase) {
                ws.concepts.push(phrase);
            }
        }
    }
    ws.observations.push(Observation {
        id: "obs_1".into(),
        content: text.to_string(),
        source: "UserInput".into(),
        timestamp: ts.clone(),
        reliability: 1.0,
        context: None,
    });
    ws.claims.push(Claim {
        id: "claim_1".into(),
        statement: text.to_string(),
        status: EpistemicStatus::Observed,
        evidence_refs: res.iter().map(|r| r.canonical_name.clone()).collect(),
        dependencies: vec![],
        source: Some("observation".into()),
        timestamp: ts.clone(),
        confidence: 1.0,
    });
    let (budget, constraints) = parse_resource_constraints(text);
    if budget > 0.0 {
        ws.resources.insert("budget".into(), budget);
    }
    ws.constraints.extend(constraints);
    ws.timestamp = ts;
    ws.provenance.push("lai strategy ingest".into());
    ws
}

/// Convert a deterministic optimizer [`Allocation`] into a structured [`Strategy`],
/// grounded in the supplied [`StrategyIR`] (objective, constraints, assumptions,
/// entities, evidence, unknowns). The allocation's chosen item levels become the
/// actions. This is Slice 4: the existing optimizer *consumes* the IR rather than
/// re-deriving the world from raw text.
pub fn allocation_to_strategy(
    alloc: &Allocation,
    schema: &Schema,
    ir: &StrategyIR,
    idx: usize,
) -> Strategy {
    let actions: Vec<String> = alloc
        .levels
        .iter()
        .map(|(id, lvl)| format!("{id} x {lvl}"))
        .collect();
    let confidence = if alloc.objective > 0.0 {
        (alloc.objective / (alloc.objective + 1.0)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    Strategy {
        id: format!("S{idx:02}"),
        objective: ir.objective.clone(),
        assumptions: if ir.assumptions.is_empty() {
            vec!["world model derived from ingested text".into()]
        } else {
            ir.assumptions.clone()
        },
        constraints: if ir.hard_constraints.is_empty() {
            schema
                .budget
                .iter()
                .map(|(k, v)| format!("{k} <= {v}"))
                .collect()
        } else {
            ir.hard_constraints.clone()
        },
        actions,
        resources: schema.budget.iter().map(|(k, v)| (k.clone(), *v)).collect(),
        expected_outcomes: vec![format!("objective value {:.3}", alloc.objective)],
        risks: if ir.risks.is_empty() {
            vec!["depends on correctness of the ingested world model".into()]
        } else {
            ir.risks.clone()
        },
        dependencies: ir.dependencies.clone(),
        evidence: {
            let mut e = ir.evidence.clone();
            e.push("deterministic optimizer (Pareto frontier)".into());
            e
        },
        confidence,
        robustness: 0.5,
    }
}

/// Build a [`StrategyIR`] from a resolved [`WorldState`]. This is the compiler-IR
/// stage of the pipeline: natural language -> semantic grounding -> StrategyIR ->
/// planner/optimizer. Keeps objective / hard constraints / soft constraints /
/// assumptions / unknowns strictly separated (directive §15).
pub fn build_strategy_ir(ws: &WorldState, objective: &str) -> StrategyIR {
    let mut ir = StrategyIR::new(objective, ws.version);
    ir.entities = ws.entities.clone();
    ir.assumptions = ws.assumptions.clone();
    ir.hard_constraints = ws.constraints.clone();
    ir.resources = ws.resources.clone();
    ir.goal_conditions = ws.goals.clone();
    ir.unknowns = ws
        .uncertainties
        .iter()
        .map(|u| u.question.clone())
        .collect();
    ir.evidence = ws
        .observations
        .iter()
        .map(|o| format!("observation:{}", o.id))
        .collect();
    for c in &ws.claims {
        ir.evidence.extend(c.evidence_refs.iter().cloned());
    }
    for con in &ws.contradictions {
        ir.risks.push(format!(
            "contradiction {} (severity {}) unresolved",
            con.id, con.severity
        ));
    }
    ir
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain_graph::Domain;

    /// A resolved content token carrying `weight`.
    fn resolved(graha: Domain, weight: f64) -> (String, TokenForce) {
        ("x".to_string(), TokenForce::Resolved { graha, weight })
    }
    fn unresolved(text: &str) -> (String, TokenForce) {
        (text.to_string(), TokenForce::Unresolved)
    }
    fn stopword(text: &str) -> (String, TokenForce) {
        (text.to_string(), TokenForce::Stopword)
    }

    #[test]
    fn synthesizes_primary_secondary_from_dominant_grahas() {
        let forces = vec![
            resolved(Domain::Mangala, 1.0),
            resolved(Domain::Mangala, 1.0),
            resolved(Domain::Mangala, 1.0),
            resolved(Domain::Mangala, 1.0),
            resolved(Domain::Brihaspati, 1.0),
        ];
        let report = synthesize_strategy("how to architect safely", &forces);

        assert_eq!(report.primary, Some(Domain::Mangala));
        assert_eq!(report.secondary, Some(Domain::Brihaspati));
        assert_eq!(report.tertiary, None);

        // Mangala = 4/5 = 80%, Brihaspati = 1/5 = 20% (weight-weighted).
        let mangala = report
            .ranked
            .iter()
            .find(|(g, _, _)| *g == Domain::Mangala)
            .unwrap();
        assert!((mangala.1 - 4.0).abs() < 1e-9);
        assert!((mangala.2 - 0.8).abs() < 1e-9);
    }

    #[test]
    fn empty_matrix_yields_no_forces() {
        let report = synthesize_strategy("???", &[]);
        assert!(report.ranked.is_empty());
        assert!(report.primary.is_none());
        assert!(report.format().contains("no graha forces resolved"));
    }

    #[test]
    fn determinism_same_input_same_report() {
        let forces = vec![
            resolved(Domain::Shukra, 0.5),
            resolved(Domain::Shukra, 0.5),
            resolved(Domain::Budha, 1.0),
        ];
        let a = synthesize_strategy("x", &forces);
        let b = synthesize_strategy("x", &forces);
        assert_eq!(a.ranked, b.ranked);
        assert_eq!(a.primary, b.primary);
    }

    #[test]
    fn stopwords_excluded_unresolved_recorded() {
        let forces = vec![
            stopword("how"),
            stopword("i"),
            resolved(Domain::Budha, 1.0),
            resolved(Domain::Budha, 1.0),
            unresolved("xyzzy"),
        ];
        let report = synthesize_strategy("how do i know xyzzy", &forces);
        assert_eq!(report.primary, Some(Domain::Budha));
        assert!(report.stopwords.contains(&"how".to_string()));
        assert!(report.unresolved.contains(&"xyzzy".to_string()));
    }

    #[test]
    fn unresolved_majority_yields_low_confidence_best_guess() {
        // T53: 3 of 4 content tokens unresolved, but one DID resolve → keep a
        // flagged best-guess rather than refusing. `primary` is populated and
        // `low_confidence` is set (no longer a hard null / OutOfScope).
        let forces = vec![
            resolved(Domain::Budha, 1.0),
            unresolved("aaa"),
            unresolved("bbb"),
            unresolved("ccc"),
        ];
        let report = synthesize_strategy("query", &forces);
        assert!(report.warning.is_some());
        assert!(report.low_confidence);
        assert_eq!(report.primary, Some(Domain::Budha));
        assert!(report.format().contains("low confidence"));
    }

    #[test]
    fn nothing_resolved_still_fails_loud() {
        // When *no* content token resolves, primary stays None so the caller
        // emits OutOfScope. low_confidence is false (there is no guess).
        let forces = vec![unresolved("aaa"), unresolved("bbb")];
        let report = synthesize_strategy("query", &forces);
        assert!(report.warning.is_some());
        assert!(!report.low_confidence);
        assert_eq!(report.primary, None);
    }

    #[test]
    fn graha_default_target_covers_all_grahas() {
        for graha in Domain::all() {
            assert!(!graha_default_target(graha).is_empty());
        }
    }

    #[test]
    fn domain_to_pillar_matches_graha_map() {
        assert_eq!(domain_to_pillar(Domain::Surya), Some(Pillar::Spear));
        assert_eq!(domain_to_pillar(Domain::Chandra), Some(Pillar::Olive));
        assert_eq!(domain_to_pillar(Domain::Mangala), Some(Pillar::Forge));
        assert_eq!(domain_to_pillar(Domain::Budha), Some(Pillar::Owl));
        assert_eq!(domain_to_pillar(Domain::Brihaspati), Some(Pillar::Council));
        assert_eq!(domain_to_pillar(Domain::Shukra), Some(Pillar::Loom));
        assert_eq!(domain_to_pillar(Domain::Shani), Some(Pillar::Stone));
        assert_eq!(domain_to_pillar(Domain::Rahu), None);
        assert_eq!(domain_to_pillar(Domain::Ketu), None);
    }

    #[test]
    fn aggregate_pillars_normalizes_from_report() {
        // Mangala=Forge 0.8, Brihaspati=Council 0.2 → pillar vector.
        let forces = vec![
            resolved(Domain::Mangala, 4.0),
            resolved(Domain::Brihaspati, 1.0),
        ];
        let report = synthesize_strategy("how to architect", &forces);
        let empty_matrix = crate::descent::SettlingMatrix::new(Vec::new());
        let pillars = aggregate_pillars(&report, &empty_matrix);

        let total: f64 = pillars.iter().sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "pillars must sum to 1.0, got {total}"
        );

        assert!((pillars[Pillar::Forge.index()] - 0.8).abs() < 1e-9);
        assert!((pillars[Pillar::Council.index()] - 0.2).abs() < 1e-9);
        // Unmapped pillars stay at zero.
        assert_eq!(pillars[Pillar::Spear.index()], 0.0);
        assert_eq!(pillars[Pillar::Olive.index()], 0.0);
    }

    #[test]
    fn aggregate_pillars_ignores_rahu_ketu() {
        let forces = vec![
            resolved(Domain::Rahu, 1.0),
            resolved(Domain::Ketu, 1.0),
            resolved(Domain::Shani, 2.0),
        ];
        let report = synthesize_strategy("x", &forces);
        let empty_matrix = crate::descent::SettlingMatrix::new(Vec::new());
        let pillars = aggregate_pillars(&report, &empty_matrix);
        // Shani=Stone is 2/4 = 0.5; Rahu/Ketu contribute nothing.
        assert!((pillars[Pillar::Stone.index()] - 0.5).abs() < 1e-9);
        let total: f64 = pillars.iter().sum();
        assert!((total - 0.5).abs() < 1e-9);
    }

    #[test]
    fn t53_dominant_graha_display_prefers_resolved_domains() {
        // T53: when a token resolves via the FormulaMatch shortcut its
        // `vedic_classification.grahas` vector is never populated, so a
        // vedic-only `dominant_graha_of` (used for routing) comes back null.
        // The display view `dominant_graha_display` must fall back to
        // `domains[0]` — the actual unification result.
        let mut token = crate::descent::SettledToken::new("electrical_power");
        token.domains.push(Domain::Shukra);
        // Empty vedic vector (the bug condition): pure routing signal is null.
        assert!(dominant_graha_of(&token).is_none());
        // Display view reflects the resolved domain.
        let dg = dominant_graha_display(&token);
        assert_eq!(dg, Some(Domain::Shukra));
        assert!(token.domains.contains(&dg.expect("resolved")));
    }

    #[test]
    fn t53_dominant_graha_display_agrees_with_tie_domains() {
        // T53 mode 1: a 3-way DomainClassification tie yields multiple domains;
        // the display `dominant_graha` must report `domains[0]`, never a graha
        // absent from `domains`, even when a stale vedic signal would win.
        let mut token = crate::descent::SettledToken::new("ecosystem_resilience");
        token.domains.push(Domain::Rahu);
        token.domains.push(Domain::Brihaspati);
        token.domains.push(Domain::Chandra);
        // Simulate a competing (stale) vedic signal that would otherwise win.
        token.vedic_classification.set_graha(Domain::Mangala, 0.9);
        // Pure routing signal follows the vedic vector (Mangala) — that's the
        // T54 purity contract and must NOT change.
        assert_eq!(dominant_graha_of(&token), Some(Domain::Mangala));
        // Display view follows the resolved domain.
        let dg = dominant_graha_display(&token);
        assert_eq!(dg, Some(Domain::Rahu));
        assert!(token.domains.contains(&dg.expect("resolved")));
    }

    // --- Sensor-force reweighting (#4) -----------------------------------------

    #[test]
    fn parse_sensor_forces_reads_forces_table() {
        let toml = "[forces]\nForge = 1.5\nmangala = 0.5\n";
        let f = parse_sensor_forces(toml).expect("parse [forces] table");
        assert_eq!(f.get("Forge"), Some(&1.5));
        assert_eq!(f.get("mangala"), Some(&0.5));
    }

    #[test]
    fn parse_sensor_forces_reads_flat_table() {
        let toml = "Stone = 2.0\n";
        let f = parse_sensor_forces(toml).expect("parse flat table");
        assert_eq!(f.get("Stone"), Some(&2.0));
    }

    #[test]
    fn parse_sensor_forces_fails_on_garbage() {
        assert!(parse_sensor_forces("this is not = toml {").is_err());
    }

    #[test]
    fn reweight_pillars_accepts_pillar_and_graha_names() {
        let mut pillars = [0.0f64; 7];
        pillars[Pillar::Spear.index()] = 0.4;
        let mut forces = std::collections::HashMap::new();
        forces.insert("Forge".to_string(), 1.5); // pillar name
        forces.insert("mangala".to_string(), 0.5); // graha name → Forge
        reweight_pillars_with_sensor_forces(&mut pillars, &forces);
        // Forge receives both contributions; others untouched.
        assert!((pillars[Pillar::Forge.index()] - 2.0).abs() < 1e-9);
        assert!((pillars[Pillar::Spear.index()] - 0.4).abs() < 1e-9);
        assert_eq!(pillars[Pillar::Olive.index()], 0.0);
    }

    #[test]
    fn reweight_pillars_ignores_unknown_keys() {
        let mut pillars = [1.0f64; 7];
        let mut forces = std::collections::HashMap::new();
        forces.insert("nonexistent_axis".to_string(), 5.0);
        reweight_pillars_with_sensor_forces(&mut pillars, &forces);
        // No pillar changed (unknown key ignored).
        for w in &pillars {
            assert_eq!(*w, 1.0);
        }
    }
}

#[cfg(test)]
mod foundation_tests {
    use super::*;
    use crate::entity::{EntityRegistry, SeedEntity};
    use std::collections::HashMap;

    fn seed(id: &str, desc: &str) -> SeedEntity {
        SeedEntity {
            id: id.to_string(),
            name: id.to_string(),
            description: desc.to_string(),
            classification: None,
            properties: HashMap::new(),
            constants: HashMap::new(),
            tags: vec![],
            formula: None,
            birth_time: None,
            bija: None,
            mantra: None,
            day: None,
            ruled_nakshatras: vec![],
        }
    }

    #[test]
    fn resolve_exact_id_high_confidence() {
        let mut reg = EntityRegistry::new();
        reg.register_seed(seed(
            "surveillance_capitalism",
            "theory of surveillance capitalism and distributed systems",
        ));
        let res = resolve_entities("limit surveillance capitalism now", &reg);
        assert!(
            res.iter()
                .any(|r| r.canonical_name == "surveillance_capitalism" && r.confidence >= 0.95),
            "expected exact id resolution"
        );
    }

    #[test]
    fn resolve_skips_stopwords_and_bare_numbers() {
        let mut reg = EntityRegistry::new();
        reg.register_seed(seed("budget", "financial budget concept"));
        let res = resolve_entities("we must secure 100 systems", &reg);
        // "we"/"must"/"100"/"systems" are stopword/number/non-exact -> not resolved
        assert!(
            !res.iter().any(|r| r.canonical_name == "budget"),
            "stopwords/numbers must not resolve"
        );
    }

    #[test]
    fn multiword_unresolved_surfaces_as_concept() {
        let reg = EntityRegistry::new();
        let ws = build_world_state("manage the distributed systems risk", &reg, None);
        assert!(
            ws.concepts
                .iter()
                .any(|c| c.contains("distributed systems")),
            "unresolved multi-word phrase should surface as a concept"
        );
    }

    #[test]
    fn parse_budget_once_no_duplicate() {
        let (b, c) = parse_resource_constraints("spend with budget <= 100 please");
        assert_eq!(b, 100.0);
        assert_eq!(c, vec!["budget <= 100".to_string()]);
    }

    #[test]
    fn world_state_captures_budget_resource() {
        let mut reg = EntityRegistry::new();
        reg.register_seed(seed(
            "surveillance_capitalism",
            "surveillance capitalism theory",
        ));
        let ws = build_world_state(
            "limit surveillance capitalism with budget <= 50",
            &reg,
            None,
        );
        assert_eq!(ws.resources.get("budget"), Some(&50.0));
        assert!(ws.entities.contains_key("surveillance capitalism"));
    }
}

#[cfg(test)]
mod slice3_tests {
    use super::*;
    use lai_core::{Observation, StrategyIR, Unknown, WorldState};
    use std::collections::BTreeMap;

    #[test]
    fn build_strategy_ir_carries_world_model() {
        let mut ws = WorldState::new();
        ws.observations.push(Observation {
            id: "obs_1".into(),
            content: "limit surveillance capitalism".into(),
            source: "UserInput".into(),
            timestamp: None,
            reliability: 1.0,
            context: None,
        });
        ws.entities.insert(
            "surveillance capitalism".into(),
            "surveillance_capitalism".into(),
        );
        ws.constraints.push("budget <= 100".into());
        ws.resources.insert("budget".into(), 100.0);
        ws.uncertainties.push(Unknown {
            question: "is monitoring active?".into(),
            impact: "high".into(),
            urgency: "high".into(),
            strategies_affected: 1,
            information_gain: "high".into(),
        });
        let ir = build_strategy_ir(&ws, "limit exposure");
        assert_eq!(ir.objective, "limit exposure");
        assert_eq!(ir.initial_state_version, 1);
        assert_eq!(
            ir.entities.get("surveillance capitalism").unwrap(),
            "surveillance_capitalism"
        );
        assert_eq!(ir.hard_constraints, vec!["budget <= 100".to_string()]);
        assert_eq!(ir.resources.get("budget"), Some(&100.0));
        assert_eq!(ir.unknowns, vec!["is monitoring active?".to_string()]);
        assert!(ir.evidence.iter().any(|e| e.starts_with("observation:")));
    }

    #[test]
    fn strategy_ir_conforms_to_directive_shape() {
        // Sanity: StrategyIR serializes with the directive's field vocabulary.
        let ir = StrategyIR::new("demo", 3);
        let j = serde_json::to_string(&ir).unwrap();
        for key in [
            "objective",
            "initial_state_version",
            "goal_conditions",
            "entities",
            "assumptions",
            "hard_constraints",
            "soft_constraints",
            "actions",
            "resources",
            "risks",
            "dependencies",
            "unknowns",
            "evidence",
        ] {
            assert!(j.contains(&format!("\"{key}\"")), "missing key {key}");
        }
    }
}
