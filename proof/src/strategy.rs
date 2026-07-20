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

/// Aggregate a `StrategyReport`'s per-graha shares into a normalized 7-pillar
/// objective vector. Each graha's `share` (already in `[0,1]`, summing to 1.0
/// over resolved grahas) is added into its pillar bucket. Pillars that no graha
/// resolved to stay at 0.0.
///
/// Pure + deterministic. The result feeds the optimizer as `objective.weights`.
pub fn aggregate_pillars(report: &StrategyReport) -> [f64; 7] {
    let mut pillars = [0.0f64; 7];
    for (graha, _weight, share) in &report.ranked {
        if let Some(pillar) = domain_to_pillar(*graha) {
            pillars[pillar.index()] += *share;
        }
    }
    pillars
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
        let pillars = aggregate_pillars(&report);

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
        let pillars = aggregate_pillars(&report);
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
}
