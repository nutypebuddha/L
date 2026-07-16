//! `laverna build` — chain chart → graha weight mapping → optimize into one
//! command. Uses a `DomainProfile` (TOML) to map chart pillar weights to
//! optimization objective weights, then delegates to the existing solver.

use std::collections::HashMap;

use serde::Deserialize;

use crate::chart::personality::{PersonalityProfile, Pillar};
use crate::chart::ChartSnapshot;
use crate::optimize;

// ── Domain Profile ──────────────────────────────────────────────────────────

/// A domain-specific profile that maps chart pillar weights to optimizer
/// objective weights, plus the budget/items/objective/scoring definitions.
#[derive(Debug, Clone, Deserialize)]
pub struct DomainProfile {
    pub meta: DomainMeta,
    /// Graha → score splits. Each graha's pillar_weight is distributed
    /// across the named scores by the given fractions. Fractions per
    /// graha MUST sum to 1.0.
    pub graha_map: Vec<GrahaMapEntry>,
    /// Budget pools (same shape as `optimize::Schema`).
    pub budget: HashMap<String, f64>,
    /// Items to allocate (same shape as `optimize::Schema`).
    pub items: Vec<optimize::Item>,
    /// Objective definition — `weights` is OMITTED here; it gets computed
    /// from graha_map × chart pillar_weights at runtime.
    pub objective: DomainObjective,
    /// Score definitions (same shape as `optimize::Schema`).
    pub scoring: HashMap<String, optimize::ScoreTerm>,
    /// Optional practicality caveats (not fed to solver, informational only).
    #[serde(default)]
    pub caveats: Vec<Caveat>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct DomainMeta {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrahaMapEntry {
    /// Graha name (lowercase): "surya", "chandra", "mangala", "budha",
    /// "brihaspati", "shukra", "shani". Rahu/Ketu are excluded (no pillar).
    pub graha: String,
    /// Score name → fraction of this graha's pillar_weight. Must sum to 1.0.
    pub splits: HashMap<String, f64>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DomainObjective {
    /// Which scores to maximize (same as `optimize::Objective::maximize`).
    pub maximize: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Caveat {
    pub applies_to: String,
    pub note: String,
    #[serde(default)]
    pub source: String,
}

// ── Validation ──────────────────────────────────────────────────────────────

/// Validate a domain profile. Returns `Err` on the first structural violation.
pub fn validate_domain_profile(profile: &DomainProfile) -> Result<(), String> {
    for entry in &profile.graha_map {
        let sum: f64 = entry.splits.values().sum();
        if (sum - 1.0).abs() > 1e-9 {
            return Err(format!(
                "graha_map graha '{}' splits sum to {:.6}, expected 1.0",
                entry.graha, sum
            ));
        }
    }

    let _graha_names: std::collections::HashSet<&str> =
        profile.graha_map.iter().map(|e| e.graha.as_str()).collect();
    for entry in &profile.graha_map {
        validate_graha_name(&entry.graha)?;
        for score_name in entry.splits.keys() {
            if !profile.scoring.contains_key(score_name) {
                return Err(format!(
                    "graha '{}' references unknown score '{}' (not in [scoring.*])",
                    entry.graha, score_name
                ));
            }
        }
    }

    let all_score_names: std::collections::HashSet<&str> =
        profile.scoring.keys().map(|s| s.as_str()).collect();
    for name in &profile.objective.maximize {
        if !all_score_names.contains(name.as_str()) {
            return Err(format!(
                "objective maximizes unknown score '{name}' (not in [scoring.*])"
            ));
        }
    }

    Ok(())
}

fn validate_graha_name(name: &str) -> Result<(), String> {
    match name {
        "surya" | "chandra" | "mangala" | "budha" | "brihaspati" | "shukra" | "shani" => Ok(()),
        _ => Err(format!(
            "unknown graha '{name}' (expected one of: surya, chandra, mangala, budha, brihaspati, shukra, shani)"
        )),
    }
}

fn graha_name_to_pillar(name: &str) -> Option<Pillar> {
    match name {
        "surya" => Some(Pillar::Spear),
        "chandra" => Some(Pillar::Olive),
        "mangala" => Some(Pillar::Forge),
        "budha" => Some(Pillar::Owl),
        "brihaspati" => Some(Pillar::Council),
        "shukra" => Some(Pillar::Loom),
        "shani" => Some(Pillar::Stone),
        _ => None,
    }
}

// ── Parse ───────────────────────────────────────────────────────────────────

/// Parse a domain profile from a TOML string.
pub fn parse_domain_profile(toml_str: &str) -> Result<DomainProfile, String> {
    toml::from_str(toml_str).map_err(|e| format!("domain profile parse error: {e}"))
}

// ── Weight Computation ──────────────────────────────────────────────────────

/// Compute optimizer objective weights from chart pillar weights × graha_map.
///
/// For each score name:
///   weight[score] = Σ (pillar_weight[graha] × split[fraction])
///
/// Pure function — deterministic given identical inputs.
pub fn compute_objective_weights(
    profile: &DomainProfile,
    pillar_weights: &[f64; 7],
) -> Result<HashMap<String, f64>, String> {
    let mut weights: HashMap<String, f64> = HashMap::new();

    for entry in &profile.graha_map {
        let pillar = graha_name_to_pillar(&entry.graha)
            .ok_or_else(|| format!("unknown graha '{}' in graha_map", entry.graha))?;
        let pw = pillar_weights[pillar.index()];
        for (score_name, fraction) in &entry.splits {
            *weights.entry(score_name.clone()).or_insert(0.0) += pw * fraction;
        }
    }

    Ok(weights)
}

// ── Build (full pipeline) ───────────────────────────────────────────────────

/// Build result — everything needed for output.
pub struct BuildResult {
    pub chart: ChartSnapshot,
    pub personality: PersonalityProfile,
    pub objective_weights: HashMap<String, f64>,
    pub allocations: Vec<optimize::Allocation>,
}

/// Run the full build pipeline:
/// 1. Cast chart from datetime + location
/// 2. Derive personality (pillar weights)
/// 3. Compute objective weights from graha_map × pillar weights
/// 4. Construct an optimize::Schema with computed weights
/// 5. Run the solver
pub fn build(
    profile: &DomainProfile,
    julian_day: f64,
    latitude: Option<f64>,
    longitude: Option<f64>,
    top_k: usize,
) -> Result<BuildResult, String> {
    build_with(
        profile,
        julian_day,
        latitude,
        longitude,
        top_k,
        crate::ephemeris::AyanamsaSystem::default(),
    )
}

/// Like [`build`], but with an explicit ayanamsa system (Part 1.4) so the
/// embedded chart's sidereal longitudes match the caller's selection.
pub fn build_with(
    profile: &DomainProfile,
    julian_day: f64,
    latitude: Option<f64>,
    longitude: Option<f64>,
    top_k: usize,
    ayanamsa_system: crate::ephemeris::AyanamsaSystem,
) -> Result<BuildResult, String> {
    validate_domain_profile(profile)?;

    let mut chart = ChartSnapshot::with_ayanamsa(julian_day, ayanamsa_system);
    if let Some((lat, lon)) = latitude.zip(longitude) {
        chart = chart.with_location(lat, lon);
    }

    let personality = crate::chart::personality::derive_personality(&chart);

    let objective_weights = compute_objective_weights(profile, &personality.pillar_weights)?;

    let schema = profile_to_schema(profile, &objective_weights)?;
    let allocations = optimize::solve(&schema, top_k)?;

    Ok(BuildResult {
        chart,
        personality,
        objective_weights,
        allocations,
    })
}

/// Generic 7-pillar allocation: distribute `budget` resource points across the
/// 7 strategic pillars according to a pillar-weight vector, via the shared
/// deterministic optimizer. Each pillar is one item with unit cost; the solver
/// returns the top-K Pareto-optimal integer allocations.
///
/// Pure + deterministic. Used by `strategize` when no concrete `--domain`
/// profile is supplied.
/// Build the generic 7-pillar allocation schema (no chart needed). Each pillar
/// is one unit-cost attribute item; the solver distributes `budget` points
/// across them per the pillar-weight vector.
pub fn pillar_schema(pillars: &[f64; 7], budget: f64) -> optimize::Schema {
    let total: u32 = (budget.max(1.0)).round() as u32;
    let mut items: Vec<optimize::Item> = Vec::with_capacity(Pillar::COUNT);
    let mut scoring: HashMap<String, optimize::ScoreTerm> = HashMap::new();
    let mut maximize: Vec<String> = Vec::with_capacity(Pillar::COUNT);

    for i in 0..Pillar::COUNT {
        let pillar = Pillar::from_index(i);
        let score = format!("score_{}", pillar.name().to_lowercase());
        items.push(optimize::Item {
            id: pillar.name().to_lowercase(),
            kind: optimize::ItemKind::Attribute,
            requires: None,
            cost: {
                let mut c = HashMap::new();
                c.insert("unit".to_string(), 1.0);
                c
            },
            max_level: Some(total),
            effects: {
                let mut e = HashMap::new();
                e.insert(score.clone(), 1.0);
                e
            },
        });
        scoring.insert(
            score.clone(),
            optimize::ScoreTerm {
                terms: {
                    let mut t = HashMap::new();
                    t.insert(score.clone(), 1.0);
                    t
                },
            },
        );
        maximize.push(score);
    }

    let mut weights: HashMap<String, f64> = HashMap::new();
    for i in 0..Pillar::COUNT {
        let pillar = Pillar::from_index(i);
        let score = format!("score_{}", pillar.name().to_lowercase());
        weights.insert(score, pillars[pillar.index()]);
    }

    optimize::Schema {
        meta: optimize::Meta {
            domain: "generic_pillars".to_string(),
            schema_version: 0,
            shape: None,
        },
        budget: {
            let mut b = HashMap::new();
            b.insert("unit".to_string(), total as f64);
            b
        },
        items,
        objective: optimize::Objective { maximize, weights },
        scoring,
    }
}

/// Generic 7-pillar allocation: distribute `budget` resource points across the
/// 7 strategic pillars according to a pillar-weight vector, via the shared
/// deterministic optimizer. Each pillar is one item with unit cost; the solver
/// returns the top-K Pareto-optimal integer allocations.
///
/// Pure + deterministic. Used by `strategize` when no concrete `--domain`
/// profile is supplied.
pub fn solve_pillar_allocation(
    pillars: &[f64; 7],
    budget: f64,
    top_k: usize,
) -> Result<Vec<optimize::Allocation>, String> {
    let schema = pillar_schema(pillars, budget);
    optimize::solve(&schema, top_k)
}

/// Convert a DomainProfile + computed weights into an optimize::Schema.
pub fn profile_to_schema(
    profile: &DomainProfile,
    weights: &HashMap<String, f64>,
) -> Result<optimize::Schema, String> {
    Ok(optimize::Schema {
        meta: optimize::Meta {
            domain: profile.meta.domain.clone(),
            schema_version: 0,
            shape: None,
        },
        budget: profile.budget.clone(),
        items: profile.items.clone(),
        objective: optimize::Objective {
            maximize: profile.objective.maximize.clone(),
            weights: weights.clone(),
        },
        scoring: profile.scoring.clone(),
    })
}

// ── Domain Profile Template ─────────────────────────────────────────────────

/// Canonical domain profile template. Printed by `laverna schema domain`.
pub const DOMAIN_PROFILE_TEMPLATE: &str = r#"# Laverna `build` domain profile — copy, edit, run:
#   laverna build --domain this.toml --datetime "YYYY-MM-DD HH:MM" --lat 45.4 --lon -92.9
#
# Chains chart → graha weight mapping → optimize into one command.
# The solver gets `objective.weights` computed from graha_map × chart
# pillar_weights, so you only need to specify the splits, not the weights.

[meta]
domain = "my_domain"
description = "describe what this profile is for"

# Graha → score splits: each graha's pillar_weight is distributed across
# the named scores. Fractions per graha MUST sum to 1.0.
# Pillar mapping: surya→Spear, chandra→Olive, mangala→Forge, budha→Owl,
# brihaspati→Council, shukra→Loom, shani→Stone.
[[graha_map]]
graha = "mangala"
splits = { score_body = 0.7, score_reflexes = 0.3 }
note = "Mars governs both raw force and decisive quick action"

[[graha_map]]
graha = "surya"
splits = { score_body = 1.0 }
note = "Sun — pure vitality and presence"

[[graha_map]]
graha = "budha"
splits = { score_intelligence = 1.0 }
note = "Mercury — pure perception and analysis"

[[graha_map]]
graha = "shani"
splits = { score_technical = 1.0 }
note = "Saturn — pure craft and endurance"

[[graha_map]]
graha = "chandra"
splits = { score_cool = 1.0 }
note = "Moon — pure adaptability and flow"

[[graha_map]]
graha = "brihaspati"
splits = { score_cool = 1.0 }
note = "Jupiter — pure growth and expansion"

[[graha_map]]
graha = "shukra"
splits = { score_cool = 1.0 }
note = "Venus — pure synergy and aesthetics"

# Budget: same shape as optimize schema.
[budget]
attribute_points = 7
perk_points = 0

# Items: same shape as optimize schema.
[[items]]
id = "body"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { body_points = 1.0 }

[[items]]
id = "reflexes"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { reflex_points = 1.0 }

[[items]]
id = "intelligence"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { intel_points = 1.0 }

[[items]]
id = "technical"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { tech_points = 1.0 }

# Objective: maximize — weights are computed at runtime from graha_map.
[objective]
maximize = ["score_body", "score_reflexes", "score_intelligence", "score_technical"]

# Scoring: each score is a linear combination of stats produced by item effects.
[scoring.score_body]
terms = { body_points = 1.0 }

[scoring.score_reflexes]
terms = { reflex_points = 1.0 }

[scoring.score_intelligence]
terms = { intel_points = 1.0 }

[scoring.score_technical]
terms = { tech_points = 1.0 }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_domain_profile() {
        let toml = r#"
[meta]
domain = "test"

[[graha_map]]
graha = "surya"
splits = { score_x = 1.0 }

[budget]
pts = 5.0

[[items]]
id = "a"
type = "attribute"
cost = { pts = 1.0 }
max_level = 5
effects = { x = 1.0 }

[objective]
maximize = ["score_x"]

[scoring.score_x]
terms = { x = 1.0 }
"#;
        let profile = parse_domain_profile(toml).unwrap();
        assert_eq!(profile.meta.domain, "test");
        assert_eq!(profile.graha_map.len(), 1);
        validate_domain_profile(&profile).unwrap();
    }

    #[test]
    fn validate_rejects_splits_not_summing_to_one() {
        let toml = r#"
[meta]
domain = "test"

[[graha_map]]
graha = "surya"
splits = { score_x = 0.6 }

[budget]
pts = 5.0

[[items]]
id = "a"
type = "attribute"
cost = { pts = 1.0 }
max_level = 5
effects = { x = 1.0 }

[objective]
maximize = ["score_x"]

[scoring.score_x]
terms = { x = 1.0 }
"#;
        let profile = parse_domain_profile(toml).unwrap();
        assert!(validate_domain_profile(&profile).is_err());
    }

    #[test]
    fn validate_rejects_unknown_graha() {
        let toml = r#"
[meta]
domain = "test"

[[graha_map]]
graha = "pluto"
splits = { score_x = 1.0 }

[budget]
pts = 5.0

[[items]]
id = "a"
type = "attribute"
cost = { pts = 1.0 }
max_level = 5
effects = { x = 1.0 }

[objective]
maximize = ["score_x"]

[scoring.score_x]
terms = { x = 1.0 }
"#;
        let profile = parse_domain_profile(toml).unwrap();
        assert!(validate_domain_profile(&profile).is_err());
    }

    #[test]
    fn compute_weights_single_graha() {
        let toml = r#"
[meta]
domain = "test"

[[graha_map]]
graha = "surya"
splits = { score_body = 0.7, score_mind = 0.3 }

[budget]
pts = 5.0

[[items]]
id = "a"
type = "attribute"
cost = { pts = 1.0 }
max_level = 5
effects = { x = 1.0 }

[objective]
maximize = ["score_body"]

[scoring.score_body]
terms = { x = 1.0 }

[scoring.score_mind]
terms = { x = 1.0 }
"#;
        let profile = parse_domain_profile(toml).unwrap();
        let mut pillar_weights = [0.0_f64; 7];
        pillar_weights[0] = 0.3; // Spear (Surya)
        let weights = compute_objective_weights(&profile, &pillar_weights).unwrap();
        assert!((weights["score_body"] - 0.21).abs() < 1e-9);
        assert!((weights["score_mind"] - 0.09).abs() < 1e-9);
    }

    #[test]
    fn graha_name_to_pillar_coverage() {
        assert_eq!(graha_name_to_pillar("surya"), Some(Pillar::Spear));
        assert_eq!(graha_name_to_pillar("chandra"), Some(Pillar::Olive));
        assert_eq!(graha_name_to_pillar("mangala"), Some(Pillar::Forge));
        assert_eq!(graha_name_to_pillar("budha"), Some(Pillar::Owl));
        assert_eq!(graha_name_to_pillar("brihaspati"), Some(Pillar::Council));
        assert_eq!(graha_name_to_pillar("shukra"), Some(Pillar::Loom));
        assert_eq!(graha_name_to_pillar("shani"), Some(Pillar::Stone));
        assert_eq!(graha_name_to_pillar("rahu"), None);
        assert_eq!(graha_name_to_pillar("ketu"), None);
    }

    #[test]
    fn build_is_deterministic() {
        use crate::ephemeris;
        let toml = r#"
[meta]
domain = "determinism_test"

[[graha_map]]
graha = "mangala"
splits = { score_body = 0.7, score_reflexes = 0.3 }

[[graha_map]]
graha = "surya"
splits = { score_body = 1.0 }

[[graha_map]]
graha = "budha"
splits = { score_intelligence = 1.0 }

[[graha_map]]
graha = "shani"
splits = { score_technical = 1.0 }

[[graha_map]]
graha = "chandra"
splits = { score_cool = 1.0 }

[[graha_map]]
graha = "brihaspati"
splits = { score_cool = 1.0 }

[[graha_map]]
graha = "shukra"
splits = { score_cool = 1.0 }

[budget]
attribute_points = 7

[[items]]
id = "body"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { body_points = 1.0 }

[[items]]
id = "reflexes"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { reflex_points = 1.0 }

[[items]]
id = "intelligence"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { intel_points = 1.0 }

[[items]]
id = "technical"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { tech_points = 1.0 }

[[items]]
id = "cool"
type = "attribute"
cost = { attribute_points = 1 }
max_level = 7
effects = { cool_points = 1.0 }

[objective]
maximize = ["score_body", "score_reflexes", "score_intelligence", "score_technical", "score_cool"]

[scoring.score_body]
terms = { body_points = 1.0 }

[scoring.score_reflexes]
terms = { reflex_points = 1.0 }

[scoring.score_intelligence]
terms = { intel_points = 1.0 }

[scoring.score_technical]
terms = { tech_points = 1.0 }

[scoring.score_cool]
terms = { cool_points = 1.0 }
"#;
        let profile = parse_domain_profile(toml).unwrap();
        validate_domain_profile(&profile).unwrap();
        let jd = ephemeris::julian_day(1994, 4, 15, 1.15);

        let a = build(&profile, jd, Some(45.41), Some(-92.64), 3).unwrap();
        let b = build(&profile, jd, Some(45.41), Some(-92.64), 3).unwrap();

        assert_eq!(
            a.allocations.len(),
            b.allocations.len(),
            "same number of solutions"
        );
        for (i, (sa, sb)) in a.allocations.iter().zip(b.allocations.iter()).enumerate() {
            assert_eq!(
                sa.objective.to_bits(),
                sb.objective.to_bits(),
                "solution {i} objective differs"
            );
            for key in sa.levels.keys() {
                assert_eq!(
                    sa.levels.get(key),
                    sb.levels.get(key),
                    "solution {i} level '{key}' differs"
                );
            }
        }
    }

    #[test]
    fn pillar_allocation_is_deterministic_and_respects_budget() {
        let pillars = [0.1, 0.1, 0.4, 0.1, 0.1, 0.1, 0.1];
        let a = solve_pillar_allocation(&pillars, 7.0, 3).unwrap();
        let b = solve_pillar_allocation(&pillars, 7.0, 3).unwrap();
        assert_eq!(a.len(), b.len());
        for (sa, sb) in a.iter().zip(b.iter()) {
            assert_eq!(sa.levels, sb.levels, "allocation not deterministic");
        }
        assert!(!a.is_empty());
        for alloc in &a {
            let spent: u32 = alloc.levels.values().sum();
            assert!(spent <= 7, "spent {spent} exceeds budget 7");
        }
        // Heaviest pillar (Forge, index 2) should dominate the top allocation.
        let top = &a[0];
        let forge = top.levels.get("forge").copied().unwrap_or(0);
        let spear = top.levels.get("spear").copied().unwrap_or(0);
        assert!(
            forge >= spear,
            "heaviest pillar should dominate top allocation"
        );
    }

    #[test]
    fn pillar_allocation_single_dominant_pillar() {
        let pillars = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0];
        let allocs = solve_pillar_allocation(&pillars, 5.0, 1).unwrap();
        let stone = allocs[0].levels.get("stone").copied().unwrap_or(0);
        assert_eq!(stone, 5, "all budget should go to dominant pillar");
    }
}
