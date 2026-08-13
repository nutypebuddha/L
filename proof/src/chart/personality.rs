use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::astrology::{Graha, Nakshatra, Rashi};
use crate::chart::{Aspect, ChartSnapshot};

// ─── Pillar (Graha → Game Stat Direction) ───────────────────────────────────

/// A game-stat direction derived from a graha.
///
/// Each Pillar represents an axis of strategic capability. The optimizer
/// uses these as objective dimensions — higher weight = stronger emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Pillar {
    Spear = 0,
    Olive = 1,
    Forge = 2,
    Owl = 3,
    Council = 4,
    Loom = 5,
    Stone = 6,
}

impl Pillar {
    pub const COUNT: usize = 7;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i % 7 {
            0 => Pillar::Spear,
            1 => Pillar::Olive,
            2 => Pillar::Forge,
            3 => Pillar::Owl,
            4 => Pillar::Council,
            5 => Pillar::Loom,
            6 => Pillar::Stone,
            _ => unreachable!(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Pillar::Spear => "Spear",
            Pillar::Olive => "Olive",
            Pillar::Forge => "Forge",
            Pillar::Owl => "Owl",
            Pillar::Council => "Council",
            Pillar::Loom => "Loom",
            Pillar::Stone => "Stone",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Pillar::Spear => "Vitality, HP, presence, leadership",
            Pillar::Olive => "Sustain, resource generation, recovery",
            Pillar::Forge => "Offense, damage, craft, action",
            Pillar::Owl => "Perception, utility, tech, information",
            Pillar::Council => "Scaling, growth, range, expansion",
            Pillar::Loom => "Synergy, buff, control, weaving",
            Pillar::Stone => "Defense, durability, cost, endurance",
        }
    }
}

/// Map a Graha to its corresponding Pillar.
fn graha_to_pillar(graha: Graha) -> Option<Pillar> {
    match graha {
        Graha::Surya => Some(Pillar::Spear),
        Graha::Chandra => Some(Pillar::Olive),
        Graha::Mangala => Some(Pillar::Forge),
        Graha::Budha => Some(Pillar::Owl),
        Graha::Brihaspati => Some(Pillar::Council),
        Graha::Shukra => Some(Pillar::Loom),
        Graha::Shani => Some(Pillar::Stone),
        Graha::Rahu => None,
        Graha::Ketu => None,
    }
}

// ─── Watch Archetype (Nakshatra → Strategy Archetype) ───────────────────────

/// A strategy archetype derived from a nakshatra placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchArchetype {
    Stride,
    Root,
    Cut,
    Threshold,
    Trail,
    Storm,
    Renewal,
    Bread,
    Coil,
    Crown,
    Repose,
    Vow,
    Hand,
    Gem,
    Sail,
    Target,
    Altar,
    ElderSpear,
    Foundation,
    Shield,
    Acropolis,
    Ear,
    Rhythm,
    Remedy,
    Blaze,
    Depths,
    Guide,
}

impl WatchArchetype {
    pub fn from_nakshatra(nak: Nakshatra) -> Self {
        match nak {
            Nakshatra::Ashwini => WatchArchetype::Stride,
            Nakshatra::Bharani => WatchArchetype::Root,
            Nakshatra::Krittika => WatchArchetype::Cut,
            Nakshatra::Rohini => WatchArchetype::Root,
            Nakshatra::Mrigashira => WatchArchetype::Trail,
            Nakshatra::Ardra => WatchArchetype::Storm,
            Nakshatra::Punarvasu => WatchArchetype::Renewal,
            Nakshatra::Pushya => WatchArchetype::Bread,
            Nakshatra::Ashlesha => WatchArchetype::Coil,
            Nakshatra::Magha => WatchArchetype::Crown,
            Nakshatra::PurvaPhalguni => WatchArchetype::Repose,
            Nakshatra::UttaraPhalguni => WatchArchetype::Vow,
            Nakshatra::Hasta => WatchArchetype::Hand,
            Nakshatra::Chitra => WatchArchetype::Gem,
            Nakshatra::Svati => WatchArchetype::Sail,
            Nakshatra::Vishakha => WatchArchetype::Target,
            Nakshatra::Anuradha => WatchArchetype::Altar,
            Nakshatra::Jyeshtha => WatchArchetype::ElderSpear,
            Nakshatra::Mula => WatchArchetype::Foundation,
            Nakshatra::PurvaAshadha => WatchArchetype::Shield,
            Nakshatra::UttaraAshadha => WatchArchetype::Acropolis,
            Nakshatra::Shravana => WatchArchetype::Ear,
            Nakshatra::Dhanishtha => WatchArchetype::Rhythm,
            Nakshatra::Shatabhisha => WatchArchetype::Remedy,
            Nakshatra::PurvaBhadrapada => WatchArchetype::Blaze,
            Nakshatra::UttaraBhadrapada => WatchArchetype::Depths,
            Nakshatra::Revati => WatchArchetype::Guide,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WatchArchetype::Stride => "Stride",
            WatchArchetype::Root => "Root",
            WatchArchetype::Cut => "Cut",
            WatchArchetype::Threshold => "Threshold",
            WatchArchetype::Trail => "Trail",
            WatchArchetype::Storm => "Storm",
            WatchArchetype::Renewal => "Renewal",
            WatchArchetype::Bread => "Bread",
            WatchArchetype::Coil => "Coil",
            WatchArchetype::Crown => "Crown",
            WatchArchetype::Repose => "Repose",
            WatchArchetype::Vow => "Vow",
            WatchArchetype::Hand => "Hand",
            WatchArchetype::Gem => "Gem",
            WatchArchetype::Sail => "Sail",
            WatchArchetype::Target => "Target",
            WatchArchetype::Altar => "Altar",
            WatchArchetype::ElderSpear => "Elder Spear",
            WatchArchetype::Foundation => "Foundation",
            WatchArchetype::Shield => "Shield",
            WatchArchetype::Acropolis => "Acropolis",
            WatchArchetype::Ear => "Ear",
            WatchArchetype::Rhythm => "Rhythm",
            WatchArchetype::Remedy => "Remedy",
            WatchArchetype::Blaze => "Blaze",
            WatchArchetype::Depths => "Depths",
            WatchArchetype::Guide => "Guide",
        }
    }

    pub fn solver_hint(self) -> &'static str {
        match self {
            WatchArchetype::Stride => "greedy-first, fast branching",
            WatchArchetype::Root => "depth-first, persistence",
            WatchArchetype::Cut => "divide-and-conquer, pruning",
            WatchArchetype::Threshold => "boundary search, transitions",
            WatchArchetype::Trail => "breadth-first, exploration",
            WatchArchetype::Storm => "random restart, perturbation",
            WatchArchetype::Renewal => "backtracking, iterative deepening",
            WatchArchetype::Bread => "resource allocation, LP",
            WatchArchetype::Coil => "constraint propagation, AC-3",
            WatchArchetype::Crown => "branch-and-bound, dominance",
            WatchArchetype::Repose => "lazy evaluation, delay",
            WatchArchetype::Vow => "commitment search, BCP",
            WatchArchetype::Hand => "constructive, greedy build",
            WatchArchetype::Gem => "local search, tabu",
            WatchArchetype::Sail => "random walk, escape local optima",
            WatchArchetype::Target => "focused beam search",
            WatchArchetype::Altar => "systematic, complete enumeration",
            WatchArchetype::ElderSpear => "alpha-beta, iterative deepening",
            WatchArchetype::Foundation => "bottom-up DP, tabulation",
            WatchArchetype::Shield => "minimax, defensive evaluation",
            WatchArchetype::Acropolis => "exhaustive, all-pairs",
            WatchArchetype::Ear => "constraint listening, propagation",
            WatchArchetype::Rhythm => "alternating, round-robin",
            WatchArchetype::Remedy => "repair-based, fix violations",
            WatchArchetype::Blaze => "simulated annealing, heat",
            WatchArchetype::Depths => "deep-first, recursive decomposition",
            WatchArchetype::Guide => "heuristic-guided, A*",
        }
    }
}

// ─── Aspect Modifier ─────────────────────────────────────────────────────────

/// A modifier to pillar weights derived from an aspect pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AspectModifier {
    pub graha_a: Graha,
    pub graha_b: Graha,
    pub aspect: Aspect,
    pub modifier: f64,
    pub reason: String,
}

fn aspect_modifier_value(aspect: Aspect) -> f64 {
    match aspect {
        Aspect::Conjunction => 0.15,
        Aspect::Sextile => 0.08,
        Aspect::Trine => 0.12,
        Aspect::Square => -0.05,
        Aspect::Opposition => -0.10,
    }
}

fn aspect_modifier_reason(aspect: Aspect, a: Graha, b: Graha) -> String {
    let a_name = pillar_name_for_graha(a);
    let b_name = pillar_name_for_graha(b);
    match aspect {
        Aspect::Conjunction => {
            format!("{a_name} fused with {b_name} — mutual reinforcement")
        }
        Aspect::Sextile => {
            format!("{a_name} adjacent to {b_name} — cooperative flow")
        }
        Aspect::Trine => {
            format!("{a_name} triangulated with {b_name} — strong synergy")
        }
        Aspect::Square => {
            format!("{a_name} crossed with {b_name} — productive tension")
        }
        Aspect::Opposition => {
            format!("{a_name} opposite {b_name} — trade-off, forces specialization")
        }
    }
}

fn pillar_name_for_graha(graha: Graha) -> &'static str {
    graha_to_pillar(graha)
        .map(|p| p.name())
        .unwrap_or_else(|| match graha {
            Graha::Rahu => "Storm",
            Graha::Ketu => "Shadow",
            _ => "Unknown",
        })
}

// ─── Combustion (graha too close to Surya → penalised) ──────────────────────

/// BPHS/Surya Siddhanta standard combustion orbs for each graha.
fn combust_orb(graha: Graha) -> f64 {
    match graha {
        Graha::Chandra => 12.0,
        Graha::Mangala => 17.0,
        Graha::Budha => 14.0,
        Graha::Brihaspati => 11.0,
        Graha::Shukra => 10.0,
        Graha::Shani => 15.0,
        _ => 0.0,
    }
}

/// Angular separation between two grahas in degrees (sidereal longitude).
/// Uses `rem_euclid` because Rust's `%` is truncated remainder (wrong for
/// negative floats on a circular domain).
fn graha_separation(chart: &ChartSnapshot, a: Graha, b: Graha) -> Option<f64> {
    let pos_a = chart.graha_position(a)?;
    let pos_b = chart.graha_position(b)?;
    let diff = (pos_a.sidereal - pos_b.sidereal).rem_euclid(360.0);
    Some(if diff <= 180.0 { diff } else { 360.0 - diff })
}

/// True if `graha` is combust — conjunct Surya within its classical orb.
fn is_combust(chart: &ChartSnapshot, graha: Graha) -> bool {
    if graha == Graha::Surya {
        return false;
    }
    let orb = combust_orb(graha);
    graha_separation(chart, graha, Graha::Surya)
        .map(|sep| sep <= orb)
        .unwrap_or(false)
}

// ─── Uccha Bala (Shadbala positional strength) ───────────────────────────────

/// Deep-debilitation longitude for each graha (BPHS Shadbala chapter).
/// Uccha bala: strength is proportional to angular distance from this point.
fn debilitation_longitude(graha: Graha) -> f64 {
    match graha {
        Graha::Surya => 190.0,      // 10° Tula (180 + 10) — debilitation
        Graha::Chandra => 213.0,    // 3° Vrishchika (180 + 33)
        Graha::Mangala => 118.0,    // 28° Karka (90 + 28) — debilitation (298 = exalt)
        Graha::Budha => 345.0,      // 15° Meena (330 + 15)
        Graha::Brihaspati => 275.0, // 5° Makara (270 + 5)
        Graha::Shukra => 177.0,     // 27° Kanya (150 + 27) — debilitation (357 = exalt)
        Graha::Shani => 20.0,       // 20° Mesha
        _ => 0.0,
    }
}

/// Uccha bala: continuous positional strength from Shadbala (BPHS).
/// Returns 0.0 at deep debilitation, 1.0 at deep exaltation.
/// `dist` is the angular distance (0..180) from the debilitation point;
/// uccha bala grows linearly with that distance.
fn uccha_bala(graha: Graha, sidereal_longitude: f64) -> f64 {
    let debil = debilitation_longitude(graha);
    let diff = (sidereal_longitude - debil).abs().rem_euclid(360.0);
    let dist = if diff <= 180.0 { diff } else { 360.0 - diff };
    dist / 180.0
}

// ─── Personality Profile ─────────────────────────────────────────────────────

/// A personality derived from a birth chart.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    /// Weight for each pillar (0.0–1.0, normalized to sum to 1.0).
    pub pillar_weights: [f64; 7],
    /// The dominant pillar (highest weight). `None` when the signal is empty
    /// (all weights collapsed to 0) — no pillar should be presented as dominant.
    pub dominant: Option<Pillar>,
    /// The Watch archetype from the dominant nakshatra (or lagna).
    pub archetype: WatchArchetype,
    /// The nakshatra the archetype was derived from.
    pub source_nakshatra: Nakshatra,
    /// Aspect-derived modifiers applied to pillar weights.
    pub aspect_modifiers: Vec<AspectModifier>,
    /// Optional lagna-based archetype (if location was provided).
    pub lagna_archetype: Option<WatchArchetype>,
    /// The lagna rashi (if available).
    pub lagna_rashi: Option<Rashi>,
}

impl PersonalityProfile {
    pub fn to_optimizer_weights(&self) -> HashMap<String, f64> {
        let mut weights = HashMap::new();
        for i in 0..7 {
            let pillar = Pillar::from_index(i);
            weights.insert(pillar.name().to_lowercase(), self.pillar_weights[i]);
        }
        weights
    }
}

/// Apply an aspect `modifier_val` to the affected grahas. A pillar graha gets
/// the full value on its own pillar; a non-pillar graha (Rahu/Ketu) distributes
/// it evenly across all 7 pillars. The *total* weight added is `modifier_val`
/// per endpoint, so a node–pillar conjunction adds the same total as a
/// pillar–pillar conjunction — i.e. no node-specific 2× reweighting (T82 pin).
fn apply_aspect_modifier(weights: &mut [f64; 7], applied_to: &[Graha], modifier_val: f64) {
    for &graha in applied_to {
        if let Some(p) = graha_to_pillar(graha) {
            weights[p.index()] += modifier_val;
        } else {
            let distributed = modifier_val / 7.0;
            for w in weights.iter_mut() {
                *w += distributed;
            }
        }
    }
}

// ─── Derive Personality ──────────────────────────────────────────────────────

/// Derive a PersonalityProfile from a ChartSnapshot.
///
/// Pure computation — same input, same output.
///
/// 1. Base weights: each classical graha contributes 0.1 to its pillar.
/// 2. Placement modifier: uccha bala (Shadbala positional strength).
/// 3. Conjunction count bonus.
/// 4. Aspect modifiers: conjunction strengthens, opposition forces trade-off.
///    Combustion: conjunct Surya within classical orb → penalty on combust graha only.
/// 5. Normalize to sum = 1.0.
/// 6. Select dominant pillar.
/// 7. Select archetype from dominant graha's nakshatra (or lagna).
pub fn derive_personality(chart: &ChartSnapshot) -> PersonalityProfile {
    let mut weights = [0.1_f64; 7];

    // Placement modifier: uccha bala (continuous Shadbala strength per graha).
    // Replaces the old 4-tier dignity + synthetic rashi_offset approach (T62 fix).
    // Uccha bala is a continuous function of real longitude, so ties are measure-zero.
    for pos in &chart.graha_positions {
        if let Some(pillar) = graha_to_pillar(pos.graha) {
            weights[pillar.index()] += uccha_bala(pos.graha, pos.sidereal) * 0.06;
        }
    }

    // Conjunction count: grahas conjuncting a given graha add a small bonus.
    for pos in &chart.graha_positions {
        if let Some(pillar) = graha_to_pillar(pos.graha) {
            let conj_count = chart
                .graha_positions
                .iter()
                .filter(|other| other.graha != pos.graha)
                .filter(|other| {
                    matches!(
                        chart.aspect_between(pos.graha, other.graha),
                        Some(Aspect::Conjunction)
                    )
                })
                .count();
            weights[pillar.index()] += conj_count as f64 * 0.03;
        }
    }

    let mut modifiers = Vec::new();
    let mut seen_pairs = std::collections::HashSet::new();

    for i in 0..chart.graha_positions.len() {
        for j in (i + 1)..chart.graha_positions.len() {
            let a = chart.graha_positions[i].graha;
            let b = chart.graha_positions[j].graha;

            if seen_pairs.contains(&(a, b)) {
                continue;
            }
            seen_pairs.insert((a, b));

            if let Some(aspect) = chart.aspect_between(a, b) {
                // Determine modifier value and which grahas are affected.
                let (modifier_val, applied_to, reason) = if aspect == Aspect::Conjunction {
                    // Combustion: only Surya conjunctions can trigger combustion logic.
                    // If a non-Surya graha is conjunct Surya within its classical orb,
                    // the combust graha is penalised, not Surya.
                    if a == Graha::Surya || b == Graha::Surya {
                        let combust_target = if a == Graha::Surya { b } else { a };
                        if is_combust(chart, combust_target) {
                            let reason = format!(
                                "{} combust — conjunct Surya within classical orb → penalty",
                                pillar_name_for_graha(combust_target)
                            );
                            (-0.06, vec![combust_target], reason)
                        } else {
                            (
                                aspect_modifier_value(aspect),
                                vec![a, b],
                                aspect_modifier_reason(aspect, a, b),
                            )
                        }
                    } else {
                        // Non-Surya conjunction: normal conjunction bonus.
                        (
                            aspect_modifier_value(aspect),
                            vec![a, b],
                            aspect_modifier_reason(aspect, a, b),
                        )
                    }
                } else {
                    (
                        aspect_modifier_value(aspect),
                        vec![a, b],
                        aspect_modifier_reason(aspect, a, b),
                    )
                };

                // Apply modifier to affected grahas only.
                apply_aspect_modifier(&mut weights, &applied_to, modifier_val);

                modifiers.push(AspectModifier {
                    graha_a: a,
                    graha_b: b,
                    aspect,
                    modifier: modifier_val,
                    reason,
                });
            }
        }
    }

    // Clamp weights to [0, 1] before normalizing to prevent inflation (T64 fix).
    for w in &mut weights {
        *w = w.clamp(0.0, 1.0);
    }
    let total: f64 = weights.iter().sum();
    if total > 0.0 {
        for w in &mut weights {
            *w /= total;
        }
    } else {
        // Empty signal (all weights collapsed to 0): emit uniform weights and
        // report no dominant pillar. A uniform profile would otherwise let
        // argmax pick an arbitrary pillar (max_by keeps the *last* tie, so it
        // would crown Stone) and present it with false confidence.
        for w in &mut weights {
            *w = 1.0 / 7.0;
        }
    }

    let dominant_idx = weights
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let dominant = if total > 0.0 {
        Some(Pillar::from_index(dominant_idx))
    } else {
        None
    };

    let (source_nakshatra, archetype, lagna_archetype, lagna_rashi) =
        if let Some(lagna) = chart.lagna {
            let lagna_nak = rashi_to_dominant_nakshatra(lagna);
            let lagna_arch = WatchArchetype::from_nakshatra(lagna_nak);

            let dom_graha = graha_of_pillar(dominant.unwrap_or(Pillar::Spear), chart);
            let dom_nak = chart
                .graha_position(dom_graha)
                .map(|p| p.nakshatra)
                .unwrap_or(Nakshatra::Ashwini);
            let arch = WatchArchetype::from_nakshatra(dom_nak);

            (dom_nak, arch, Some(lagna_arch), Some(lagna))
        } else {
            let dom_graha = graha_of_pillar(dominant.unwrap_or(Pillar::Spear), chart);
            let dom_nak = chart
                .graha_position(dom_graha)
                .map(|p| p.nakshatra)
                .unwrap_or(Nakshatra::Ashwini);
            let arch = WatchArchetype::from_nakshatra(dom_nak);

            (dom_nak, arch, None, None)
        };

    PersonalityProfile {
        pillar_weights: weights,
        dominant,
        archetype,
        source_nakshatra,
        aspect_modifiers: modifiers,
        lagna_archetype,
        lagna_rashi,
    }
}

fn graha_of_pillar(pillar: Pillar, _chart: &ChartSnapshot) -> Graha {
    let candidates: Vec<Graha> = [
        Graha::Surya,
        Graha::Chandra,
        Graha::Mangala,
        Graha::Budha,
        Graha::Brihaspati,
        Graha::Shukra,
        Graha::Shani,
    ]
    .iter()
    .copied()
    .filter(|g| graha_to_pillar(*g) == Some(pillar))
    .collect();

    candidates.into_iter().next().unwrap_or(Graha::Surya)
}

/// Map a Rashi to the dominant nakshatra within that rashi.
fn rashi_to_dominant_nakshatra(rashi: Rashi) -> Nakshatra {
    match rashi {
        Rashi::Mesha => Nakshatra::Krittika,
        Rashi::Vrishabha => Nakshatra::Rohini,
        Rashi::Mithuna => Nakshatra::Ardra,
        Rashi::Karka => Nakshatra::Pushya,
        Rashi::Simha => Nakshatra::Magha,
        Rashi::Kanya => Nakshatra::Hasta,
        Rashi::Tula => Nakshatra::Svati,
        Rashi::Vrishchika => Nakshatra::Anuradha,
        Rashi::Dhanu => Nakshatra::Mula,
        Rashi::Makara => Nakshatra::UttaraAshadha,
        Rashi::Kumbha => Nakshatra::Shatabhisha,
        Rashi::Meena => Nakshatra::Revati,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::ALL_GRAHAS;
    use crate::ephemeris;

    #[test]
    fn pillar_count() {
        assert_eq!(Pillar::COUNT, 7);
    }

    #[test]
    fn pillar_roundtrip() {
        for i in 0..7 {
            let p = Pillar::from_index(i);
            assert_eq!(p.index(), i);
        }
    }

    #[test]
    fn graha_to_pillar_coverage() {
        assert_eq!(graha_to_pillar(Graha::Surya), Some(Pillar::Spear));
        assert_eq!(graha_to_pillar(Graha::Chandra), Some(Pillar::Olive));
        assert_eq!(graha_to_pillar(Graha::Mangala), Some(Pillar::Forge));
        assert_eq!(graha_to_pillar(Graha::Budha), Some(Pillar::Owl));
        assert_eq!(graha_to_pillar(Graha::Brihaspati), Some(Pillar::Council));
        assert_eq!(graha_to_pillar(Graha::Shukra), Some(Pillar::Loom));
        assert_eq!(graha_to_pillar(Graha::Shani), Some(Pillar::Stone));
        assert_eq!(graha_to_pillar(Graha::Rahu), None);
        assert_eq!(graha_to_pillar(Graha::Ketu), None);
    }

    #[test]
    fn watch_archetype_from_nakshatra() {
        for i in 0..27 {
            let nak = Nakshatra::from_index(i);
            let arch = WatchArchetype::from_nakshatra(nak);
            assert!(!arch.name().is_empty());
            assert!(!arch.solver_hint().is_empty());
        }
    }

    #[test]
    fn derive_personality_deterministic() {
        let jd = ephemeris::julian_day(1994, 4, 15, 1.15);
        let chart = ChartSnapshot::new(jd);
        let a = derive_personality(&chart);
        let b = derive_personality(&chart);

        for (wa, wb) in a.pillar_weights.iter().zip(b.pillar_weights.iter()) {
            assert_eq!(wa.to_bits(), wb.to_bits());
        }
        assert_eq!(a.dominant, b.dominant);
        assert_eq!(a.archetype, b.archetype);
    }

    #[test]
    fn derive_personality_weights_sum_to_one() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd);
        let profile = derive_personality(&chart);

        let sum: f64 = profile.pillar_weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 0.001,
            "pillar weights should sum to 1.0, got {}",
            sum
        );
    }

    #[test]
    fn derive_personality_weights_in_range() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd);
        let profile = derive_personality(&chart);

        for (i, w) in profile.pillar_weights.iter().enumerate() {
            assert!(
                *w >= 0.0 && *w <= 1.0,
                "pillar {} weight {} out of range",
                i,
                w
            );
        }
    }

    #[test]
    fn derive_personality_has_dominant() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd);
        let profile = derive_personality(&chart);
        assert!(
            profile.dominant.is_some(),
            "non-empty signal must have a dominant"
        );

        let max_weight = profile
            .pillar_weights
            .iter()
            .cloned()
            .fold(0.0_f64, f64::max);
        let dominant_weight = profile.pillar_weights[profile.dominant.unwrap().index()];
        assert_eq!(dominant_weight, max_weight);
    }

    // ─── T82 pin ───────────────────────────────────────────────────────────
    // A Rahu–pillar conjunction must add the same *total* weight as a
    // pillar–pillar conjunction. The node distributes modifier_val/7 across all
    // 7 pillars (summing to modifier_val) and the pillar endpoint also gets
    // modifier_val, so the total equals an ordinary conjunction (2 × modifier).
    // If a future change reweights nodes 2× relative to ordinary aspects, this
    // fails — that is the T82 regression to keep locked.
    #[test]
    fn node_and_pillar_conjunction_add_equal_total() {
        let mut node_case = [0.0_f64; 7];
        apply_aspect_modifier(&mut node_case, &[Graha::Rahu, Graha::Surya], 0.06);
        let mut pillar_case = [0.0_f64; 7];
        apply_aspect_modifier(&mut pillar_case, &[Graha::Surya, Graha::Chandra], 0.06);

        let node_total: f64 = node_case.iter().sum();
        let pillar_total: f64 = pillar_case.iter().sum();
        assert!(
            (node_total - pillar_total).abs() < 1e-12,
            "node–pillar total {} != pillar–pillar total {} (T82 regression)",
            node_total,
            pillar_total
        );
        assert!(
            (node_total - 0.12).abs() < 1e-12,
            "expected 2 × modifier_val = 0.12, got {}",
            node_total
        );
    }

    #[test]
    fn to_optimizer_weights() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd);
        let profile = derive_personality(&chart);
        let opt = profile.to_optimizer_weights();

        assert_eq!(opt.len(), 7);
        assert!(opt.contains_key("spear"));
        assert!(opt.contains_key("forge"));
        assert!(opt.contains_key("stone"));
    }

    #[test]
    fn aspect_modifiers_populated() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd);
        let profile = derive_personality(&chart);
        assert!(
            !profile.aspect_modifiers.is_empty(),
            "expected aspect modifiers, got none"
        );
    }

    #[test]
    fn with_location_adds_lagna() {
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd).with_location(40.7, -74.0);
        let profile = derive_personality(&chart);

        assert!(profile.lagna_rashi.is_some());
        assert!(profile.lagna_archetype.is_some());
    }

    #[test]
    fn rashi_to_nakshatra_coverage() {
        for i in 0..12 {
            let rashi = Rashi::from_index(i);
            let nak = rashi_to_dominant_nakshatra(rashi);
            assert!(!nak.name().is_empty());
        }
    }

    // ─── T79 + T80 regression lock ──────────────────────────────────────────
    // uccha_bala must be 0.0 at deep debilitation and 1.0 at the opposite
    // (exaltation) longitude, for *every* graha. This pins the T79 formula fix
    // and the T80 corrected debilitation degrees together — fixing one without
    // the other is what previously left Mangala/Shukra scored backwards.
    #[test]
    fn uccha_bala_debilitation_and_exaltation() {
        for &g in ALL_GRAHAS.iter() {
            let debil = debilitation_longitude(g);
            let exalt = (debil + 180.0) % 360.0;
            assert!(
                (uccha_bala(g, debil) - 0.0).abs() < 1e-9,
                "{:?} uccha_bala at debilitation should be 0.0, got {}",
                g,
                uccha_bala(g, debil)
            );
            assert!(
                (uccha_bala(g, exalt) - 1.0).abs() < 1e-9,
                "{:?} uccha_bala at exaltation should be 1.0, got {}",
                g,
                uccha_bala(g, exalt)
            );
        }
    }

    #[test]
    fn debilitation_longitude_correct() {
        // Corrected BPHS deep-debilitation degrees (T80).
        assert!((debilitation_longitude(Graha::Mangala) - 118.0).abs() < 1e-9);
        assert!((debilitation_longitude(Graha::Shukra) - 177.0).abs() < 1e-9);
        assert!((debilitation_longitude(Graha::Surya) - 190.0).abs() < 1e-9);
        assert!((debilitation_longitude(Graha::Chandra) - 213.0).abs() < 1e-9);
        assert!((debilitation_longitude(Graha::Budha) - 345.0).abs() < 1e-9);
        assert!((debilitation_longitude(Graha::Brihaspati) - 275.0).abs() < 1e-9);
        assert!((debilitation_longitude(Graha::Shani) - 20.0).abs() < 1e-9);
    }

    #[test]
    fn combust_orb_known_grahas() {
        // Rahu/Ketu have no classical combustion orb (the `_ => 0.0` arm).
        assert_eq!(combust_orb(Graha::Rahu), 0.0);
        assert_eq!(combust_orb(Graha::Ketu), 0.0);
        assert!(combust_orb(Graha::Chandra) > 0.0);
        assert!(combust_orb(Graha::Mangala) > 0.0);
    }

    #[test]
    fn is_combust_self_case() {
        // Surya can never be "combust" (self-case guard in is_combust).
        let jd = ephemeris::julian_day(2026, 7, 7, 12.0);
        let chart = ChartSnapshot::new(jd);
        assert!(!is_combust(&chart, Graha::Surya));
    }
}
