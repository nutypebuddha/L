//! # Influence — Cross-Domain Property Propagation
//!
//! The Influence Engine implements **neuro-symbolic property propagation** across the
//! 9-graha Vedic wheel. When a query causes tokens to settle in multiple domains, the
//! aspect relationships between those domains determine how properties, confidence,
//! and semantic weight "ripple" from one domain to another.
//!
//! ## Architecture
//!
//! Each domain (graha) on the wheel is connected to every other domain via an
//! **aspect** (conjunction, sextile, trine, square, opposition). Each aspect has a
//! **confidence coefficient** that quantifies how naturally properties flow across
//! that relationship:
//!
//! | Aspect      | Coefficient | Direct? |
//! |-------------|-------------|---------|
//! | Conjunction | 1.00        | Yes     |
//! | Sextile     | 0.95        | Yes     |
//! | Trine       | 0.90        | Yes     |
//! | Square      | 0.75        | No      |
//! | Opposition  | 0.60        | No      |
//!
//! When a source domain activates with weight `w`, it propagates to each target
//! domain with magnitude `w × influence(src, tgt)`. Multi-hop propagation
//! decays exponentially by hop count.
//!
//! ## Usage
//!
//! The engine slots between the `SettlingMatrix` (output of token descent) and
//! the `GyroState` (physical wheel model), providing cross-domain context that
//! influences formula ranking, entity scoring, and gyroscopic precession.
//!
//! ## Research Context
//!
//! This is the first step toward a full neuro-symbolic knowledge graph layer
//! (see research on GNNs over the zodiac wheel, hybrid reasoning layers, and
//! LLM+graph orchestration). The symbolic core stays deterministic and
//! auditable while the influence coefficients can later be learned or weighted
//! by neural components.

use serde::{Deserialize, Serialize};

use crate::wheel::{Aspect, Domain};

// ─── Constants ───────────────────────────────────────────────────────────────

/// Number of grahas on the Vedic wheel.
const GRAHA_COUNT: usize = 9;

/// Maximum propagation hops from source domain.
/// Beyond this, influence is considered negligible (< 1%).
#[allow(dead_code)]
const MAX_HOPS: usize = 4;

/// Minimum influence threshold — anything below this is rounded to zero.
const MIN_INFLUENCE: f64 = 0.01;

// ─── Precomputed Influence Matrix ───────────────────────────────────────────

/// Precomputed 9×9 influence coefficient matrix.
///
/// `INFLUENCE_MATRIX[i][j]` = coefficient for Domain(i) influencing Domain(j).
/// This is a compile-time constant from aspect confidences, providing O(1)
/// lookup during propagation.
const INFLUENCE_MATRIX: [[f64; GRAHA_COUNT]; GRAHA_COUNT] = {
    let mut matrix = [[0.0f64; GRAHA_COUNT]; GRAHA_COUNT];
    let mut i: usize = 0;
    while i < GRAHA_COUNT {
        let mut j: usize = 0;
        while j < GRAHA_COUNT {
            let diff = i.abs_diff(j);
            let min_diff = if diff < GRAHA_COUNT - diff {
                diff
            } else {
                GRAHA_COUNT - diff
            };
            matrix[i][j] = match min_diff {
                0 => 1.00, // Conjunction
                1 => 0.95, // Sextile
                2 => 0.75, // Square
                3 => 0.90, // Trine
                4 => 0.60, // Opposition
                _ => 0.0,  // unreachable
            };
            j += 1;
        }
        i += 1;
    }
    matrix
};

// ─── Core Types ─────────────────────────────────────────────────────────────

/// A single influence propagation from a source domain to a target domain.
///
/// Records what flowed, how strongly, and the path it took through the wheel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfluence {
    /// The source domain where the activation originated.
    pub source: Domain,
    /// The target domain receiving the propagated influence.
    pub target: Domain,
    /// The aspect relationship between source and target.
    pub aspect: Aspect,
    /// The direct influence coefficient (aspect confidence).
    pub coefficient: f64,
    /// The attenuated influence after multi-hop decay.
    pub attenuated_strength: f64,
    /// Number of hops from source to target.
    pub hops: usize,
    /// Whether propagation followed harmonious edges only (direct aspects).
    pub harmonious_path: bool,
}

/// A map of domain → total influence weight from all sources.
///
/// This is the aggregated output of the influence engine — each domain's
/// total received influence across all propagation paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfluenceMap {
    /// Per-domain total influence weight [0..GRAHA_COUNT].
    pub weights: [f64; GRAHA_COUNT],
    /// All individual influence propagations that produced this map.
    pub influences: Vec<DomainInfluence>,
    /// Whether any propagation exceeded the attenuation threshold.
    pub has_significant_influence: bool,
}

impl InfluenceMap {
    /// Create an empty influence map with zero weights.
    pub fn new() -> Self {
        InfluenceMap {
            weights: [0.0; GRAHA_COUNT],
            influences: Vec::new(),
            has_significant_influence: false,
        }
    }

    /// Get the total influence weight for a domain.
    pub fn weight(&self, domain: Domain) -> f64 {
        self.weights[domain.index()]
    }

    /// Get the dominant domain (highest influence weight).
    pub fn dominant_domain(&self) -> Option<(Domain, f64)> {
        let mut max_idx = 0usize;
        let mut max_val = self.weights[0];
        for i in 1..GRAHA_COUNT {
            if self.weights[i] > max_val {
                max_val = self.weights[i];
                max_idx = i;
            }
        }
        if max_val > MIN_INFLUENCE {
            Some((Domain::from_index(max_idx), max_val))
        } else {
            None
        }
    }

    /// Get all domains with influence above the threshold, sorted descending.
    pub fn active_domains(&self) -> Vec<(Domain, f64)> {
        let mut active: Vec<(Domain, f64)> = self
            .weights
            .iter()
            .enumerate()
            .filter(|(_, w)| **w > MIN_INFLUENCE)
            .map(|(i, &w)| (Domain::from_index(i), w))
            .collect();
        active.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        active
    }
}

impl Default for InfluenceMap {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Influence Engine ───────────────────────────────────────────────────────

/// The engine that computes cross-domain property propagation.
///
/// Given domain activation weights (from a `SettlingMatrix` or direct input),
/// the engine computes how activation "ripples" across the wheel via aspect
/// relationships.
///
/// ## Multi-Hop Propagation
///
/// Influence decays exponentially with each hop:
/// ```text
/// attenuated = coefficient × decay_factor^hops
/// ```
/// where `decay_factor = 0.5` means each hop halves the influence.
///
/// ## Integration
///
/// This engine is called after descent produces a `SettlingMatrix` and before
/// the result is passed to `GyroState::apply_matrix()`. The output `InfluenceMap`
/// provides cross-domain context for formula ranking and entity scoring.
#[derive(Debug, Clone)]
pub struct InfluenceEngine {
    /// Decay factor per hop (0.0 = no propagation, 1.0 = no decay).
    decay_factor: f64,
    /// Whether to only propagate along direct aspects (sextile, trine, conjunction).
    direct_only: bool,
}

impl Default for InfluenceEngine {
    fn default() -> Self {
        Self {
            decay_factor: 0.5,
            direct_only: false,
        }
    }
}

impl InfluenceEngine {
    /// Create a new influence engine with default parameters.
    ///
    /// Defaults:
    /// - `decay_factor`: 0.5 (each hop halves influence)
    /// - `direct_only`: false (all aspects propagate)
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an influence engine with a custom decay factor.
    ///
    /// `decay` should be in (0.0, 1.0]. 0.0 means no propagation beyond
    /// the source domain. 1.0 means no decay (infinite propagation).
    pub fn with_decay(decay: f64) -> Self {
        Self {
            decay_factor: decay.clamp(0.0, 1.0),
            direct_only: false,
        }
    }

    /// Restrict propagation to direct aspects only (conjunction, sextile, trine).
    pub fn direct_only(mut self) -> Self {
        self.direct_only = true;
        self
    }

    /// Compute the influence map from a set of domain activation weights.
    ///
    /// This is the primary entry point. Given a slice of (domain, weight) pairs
    /// representing how strongly each domain is activated by the current query,
    /// returns an `InfluenceMap` with all propagated weights.
    ///
    /// The algorithm:
    /// 1. For each active source domain with weight `w`:
    /// 2.   For each target domain: compute `w × influence_coefficient(src, tgt)`
    /// 3.   Apply multi-hop attenuation: `attenuated = raw × decay^hops`
    /// 4.   Accumulate all propagated weights into the map
    #[allow(clippy::needless_range_loop)]
    pub fn compute(&self, activations: &[(Domain, f64)]) -> InfluenceMap {
        let mut map = InfluenceMap::new();

        for &(source, weight) in activations {
            if weight <= MIN_INFLUENCE {
                continue;
            }

            // Always propagate to self (conjunction, 0 hops)
            let self_coeff = INFLUENCE_MATRIX[source.index()][source.index()];
            let self_strength = weight * self_coeff;
            map.weights[source.index()] += self_strength;
            map.influences.push(DomainInfluence {
                source,
                target: source,
                aspect: Aspect::Conjunction,
                coefficient: self_coeff,
                attenuated_strength: self_strength,
                hops: 0,
                harmonious_path: true,
            });

            // Propagate to all other domains
            for target_idx in 0..GRAHA_COUNT {
                let target = Domain::from_index(target_idx);
                if target == source {
                    continue;
                }

                let coeff = INFLUENCE_MATRIX[source.index()][target_idx];
                if coeff <= MIN_INFLUENCE {
                    continue;
                }

                let aspect = Aspect::between(source, target);

                // Skip tension aspects if direct_only mode
                if self.direct_only && aspect.is_tension() {
                    continue;
                }

                // Determine hops from arc distance
                let hops = aspect.arc_distance();

                // Multi-hop attenuation
                let attenuated = weight * coeff * self.decay_factor.powi(hops as i32);

                if attenuated > MIN_INFLUENCE {
                    map.weights[target_idx] += attenuated;
                    map.influences.push(DomainInfluence {
                        source,
                        target,
                        aspect,
                        coefficient: coeff,
                        attenuated_strength: attenuated,
                        hops,
                        harmonious_path: !aspect.is_tension(),
                    });
                    map.has_significant_influence = true;
                }
            }
        }

        map
    }

    /// Compute influence from a `SettlingMatrix`'s dominant domains.
    ///
    /// Convenience method that extracts domain activations from a settling
    /// matrix's dominant domains and their approximate weights.
    ///
    /// The weight for each dominant domain is derived from the matrix's
    /// resolution score (overall query confidence) divided equally among
    /// the dominant domains.
    pub fn from_matrix(&self, matrix: &crate::descent::SettlingMatrix) -> InfluenceMap {
        if matrix.dominant_domains.is_empty() {
            return InfluenceMap::new();
        }

        let base_weight = matrix.resolution_score.max(0.1);
        let per_domain = base_weight / matrix.dominant_domains.len() as f64;

        let activations: Vec<(Domain, f64)> = matrix
            .dominant_domains
            .iter()
            .map(|&d| (d, per_domain))
            .collect();

        self.compute(&activations)
    }

    /// Get the raw influence coefficient between two domains (0 hops, no decay).
    #[inline]
    pub fn raw_coefficient(a: Domain, b: Domain) -> f64 {
        INFLUENCE_MATRIX[a.index()][b.index()]
    }

    /// Reset the engine to default parameters.
    pub fn reset(&mut self) {
        self.decay_factor = 0.5;
        self.direct_only = false;
    }
}

// ─── SettlingMatrix Extension ───────────────────────────────────────────────

/// Extension methods for `SettlingMatrix` to compute influence directly.
///
/// This provides a fluent interface so callers can write:
/// ```ignore
/// let influence = matrix.compute_influence(&engine);
/// ```
impl crate::descent::SettlingMatrix {
    /// Compute cross-domain influence from this settling matrix.
    pub fn compute_influence(&self, engine: &InfluenceEngine) -> InfluenceMap {
        engine.from_matrix(self)
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wheel::Domain;

    fn test_engine() -> InfluenceEngine {
        InfluenceEngine::new()
    }

    // ─── Matrix Tests ────────────────────────────────────────────────────────

    #[test]
    fn test_influence_matrix_is_symmetric() {
        for (i, row) in INFLUENCE_MATRIX.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                assert!(
                    (val - INFLUENCE_MATRIX[j][i]).abs() < 1e-12,
                    "matrix not symmetric at [{i}][{j}]"
                );
            }
        }
    }

    #[test]
    fn test_influence_matrix_self_is_1() {
        for (i, row) in INFLUENCE_MATRIX.iter().enumerate() {
            assert!(
                (row[i] - 1.0).abs() < 1e-12,
                "self-influence at [{i}] is not 1.0"
            );
        }
    }

    #[test]
    fn test_influence_matrix_all_nonzero() {
        for (i, row) in INFLUENCE_MATRIX.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                assert!(val > 0.0, "zero coefficient at [{i}][{j}]");
            }
        }
    }

    #[test]
    fn test_influence_matrix_values_match_aspect() {
        for (i, row) in INFLUENCE_MATRIX.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                let a = Domain::from_index(i);
                let b = Domain::from_index(j);
                let aspect = Aspect::between(a, b);
                let expected = aspect.confidence();
                assert!(
                    (val - expected).abs() < 1e-12,
                    "matrix[{i}][{j}] = {}, expected {} for {:?}",
                    val,
                    expected,
                    aspect
                );
            }
        }
    }

    // ─── InfluenceMap Tests ──────────────────────────────────────────────────

    #[test]
    fn test_empty_map_has_zero_weights() {
        let map = InfluenceMap::new();
        for &w in &map.weights {
            assert!((w - 0.0).abs() < 1e-12);
        }
        assert!(!map.has_significant_influence);
        assert!(map.dominant_domain().is_none());
        assert!(map.active_domains().is_empty());
    }

    #[test]
    fn test_influence_map_weight_accessor() {
        let mut map = InfluenceMap::new();
        map.weights[Domain::Surya.index()] = 0.85;
        assert!((map.weight(Domain::Surya) - 0.85).abs() < 1e-12);
        assert!((map.weight(Domain::Mangala) - 0.0).abs() < 1e-12);
    }

    // ─── Engine Tests ────────────────────────────────────────────────────────

    #[test]
    fn test_single_domain_self_influence() {
        let engine = test_engine();
        let activations = [(Domain::Surya, 1.0)];
        let map = engine.compute(&activations);

        // Self-influence should be 1.0
        assert!((map.weight(Domain::Surya) - 1.0).abs() < 1e-12);

        // Should have exactly 9 entries (self + 8 targets)
        assert_eq!(map.influences.len(), GRAHA_COUNT);
    }

    #[test]
    fn test_single_domain_propagates_to_all() {
        let engine = test_engine();
        let activations = [(Domain::Surya, 1.0)];
        let map = engine.compute(&activations);

        // All domains should have some influence (> threshold)
        let active = map.active_domains();
        assert_eq!(active.len(), GRAHA_COUNT);

        // Surya should be dominant
        let (dom, _) = map.dominant_domain().unwrap();
        assert_eq!(dom, Domain::Surya);
    }

    #[test]
    fn test_influence_strength_decreases_with_distance() {
        let engine = test_engine();
        let activations = [(Domain::Surya, 1.0)];
        let map = engine.compute(&activations);

        // Adjacent (sextile, 1 hop) should be stronger than opposite (4 hops)
        let chandra = map.weight(Domain::Chandra); // Surya+1 = sextile
        let brihaspati = map.weight(Domain::Brihaspati); // Surya+4 = opposition

        assert!(
            chandra > brihaspati,
            "adjacent influence ({}) should exceed opposition ({})",
            chandra,
            brihaspati
        );
    }

    #[test]
    fn test_zero_weight_activation_produces_no_propagation() {
        let engine = test_engine();
        let activations = [(Domain::Surya, 0.0), (Domain::Mangala, 0.005)];
        let map = engine.compute(&activations);

        // Both below MIN_INFLUENCE, so nothing propagates
        assert!(!map.has_significant_influence);
    }

    #[test]
    fn test_multiple_sources_accumulate() {
        let engine = test_engine();
        let activations = [(Domain::Surya, 0.5), (Domain::Chandra, 0.5)];
        let map = engine.compute(&activations);

        // Surya self (0.5 * 1.0 = 0.5) + Chandra→Surya (0.5 * 0.95 * 0.5^1 = 0.2375)
        let surya = map.weight(Domain::Surya);
        assert!(
            (surya - 0.7375).abs() < 1e-12,
            "Surya weight: expected 0.7375, got {}",
            surya
        );

        // Chandra self (0.5) + Surya→Chandra (0.5 * 0.95 * 0.5^1 = 0.2375)
        let chandra = map.weight(Domain::Chandra);
        assert!(
            (chandra - 0.7375).abs() < 1e-12,
            "Chandra weight: expected 0.7375, got {}",
            chandra
        );

        // Mangala: Surya→Mangala square (0.5 * 0.75 * 0.5^2 = 0.09375) + Chandra→Mangala sextile (0.5 * 0.95 * 0.5 = 0.2375)
        let mangala = map.weight(Domain::Mangala);
        assert!(
            (mangala - 0.33125).abs() < 1e-12,
            "Mangala weight: expected 0.33125, got {}",
            mangala
        );
    }

    #[test]
    fn test_direct_only_blocks_tension() {
        let engine = InfluenceEngine::new().direct_only();
        let activations = [(Domain::Surya, 1.0)];
        let map = engine.compute(&activations);

        // Surya(0) → Brihaspati(4) is opposition (tension) — should be blocked
        let brihaspati = map.weight(Domain::Brihaspati);
        assert!(
            brihaspati < MIN_INFLUENCE,
            "opposition influence should be blocked in direct_only mode, got {}",
            brihaspati
        );

        // Surya(0) → Chandra(1) is sextile (direct) — should propagate
        let chandra = map.weight(Domain::Chandra);
        assert!(
            chandra > MIN_INFLUENCE,
            "sextile should propagate in direct_only mode"
        );
    }

    #[test]
    fn test_custom_decay_factor() {
        let fast_decay = InfluenceEngine::with_decay(0.3);
        let slow_decay = InfluenceEngine::with_decay(0.9);

        let activations = [(Domain::Surya, 1.0)];
        let fast_map = fast_decay.compute(&activations);
        let slow_map = slow_decay.compute(&activations);

        // With slower decay, far domains get more influence
        let brihaspati_fast = fast_map.weight(Domain::Brihaspati); // 4 hops
        let brihaspati_slow = slow_map.weight(Domain::Brihaspati);

        assert!(
            brihaspati_slow > brihaspati_fast,
            "slower decay should give more influence to distant domains"
        );
    }

    #[test]
    fn test_active_domains_sorted_by_weight() {
        let engine = test_engine();
        // Activate two domains with different weights
        let activations = [(Domain::Mangala, 1.0), (Domain::Budha, 0.3)];
        let map = engine.compute(&activations);
        let active = map.active_domains();

        // Should be sorted descending by weight
        for window in active.windows(2) {
            assert!(
                window[0].1 >= window[1].1,
                "active domains not sorted descending: {:?} >= {:?}",
                window[0],
                window[1]
            );
        }

        // Mangala should be dominant (highest source weight)
        assert_eq!(active[0].0, Domain::Mangala);
    }

    // ─── Integration-Style Tests ─────────────────────────────────────────────

    #[test]
    fn test_raw_coefficient_matches_aspect() {
        for i in 0..GRAHA_COUNT {
            for j in 0..GRAHA_COUNT {
                let a = Domain::from_index(i);
                let b = Domain::from_index(j);
                let coeff = InfluenceEngine::raw_coefficient(a, b);
                let aspect = Aspect::between(a, b);
                assert!((coeff - aspect.confidence()).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn test_symmetric_propagation() {
        let engine = test_engine();

        let surya_activations = [(Domain::Surya, 0.8)];
        let mangala_activations = [(Domain::Mangala, 0.8)];

        let map_sm = engine.compute(&surya_activations);
        let map_ms = engine.compute(&mangala_activations);

        // Surya→Mangala influence should equal Mangala→Surya influence
        // (matrix is symmetric, same source weight)
        let sm = map_sm.weight(Domain::Mangala);
        let ms = map_ms.weight(Domain::Surya);
        assert!(
            (sm - ms).abs() < 1e-12,
            "symmetric propagation violated: {} vs {}",
            sm,
            ms
        );
    }

    #[test]
    fn test_trine_harmony_propagates_strongly() {
        let engine = test_engine();
        // Surya(0) → Budha(3) = trine (coefficient 0.90)
        // Surya(0) → Mangala(2) = square (coefficient 0.75)
        let activations = [(Domain::Surya, 1.0)];
        let map = engine.compute(&activations);

        let budha = map.weight(Domain::Budha); // trine, 3 hops
        let mangala = map.weight(Domain::Mangala); // square, 2 hops

        // Despite being farther (3 hops vs 2), trine's higher coefficient
        // should make Budha comparable to Mangala
        assert!(budha > 0.0, "trine should propagate strongly");
        assert!(mangala > 0.0, "square should propagate");
    }

    #[test]
    fn test_influence_map_format_is_consistent() {
        let engine = test_engine();
        let activations = [(Domain::Surya, 0.75), (Domain::Chandra, 0.25)];
        let map = engine.compute(&activations);

        // Sum of all weights should account for all propagation
        let total: f64 = map.weights.iter().sum();
        assert!(total > 0.0, "total influence should be positive");
        assert!(total < 9.0, "total influence should be bounded"); // max ~1 per domain

        // All influences should have valid sources and targets
        for inf in &map.influences {
            assert!(inf.coefficient > 0.0);
            assert!(inf.coefficient <= 1.0);
            assert!(inf.attenuated_strength >= 0.0);
            assert!(inf.hops <= MAX_HOPS);
        }
    }
}
