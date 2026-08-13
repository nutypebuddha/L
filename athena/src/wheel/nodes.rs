//! # Domain nodes on the zodiac wheel
//!
//! The 9 grahas (Vedic planets) are the primary domain nodes on the wheel,
//! arranged in order: Surya → Chandra → Mangala → Budha → Brihaspati →
//! Shukra → Shani → Rahu → Ketu → (back to Surya).
//!
//! Each graha represents a fundamental cognitive/knowledge domain.
//! The wheel defines how knowledge composes across domains.
//!
//! # Migration
//!
//! `Domain` is a type alias for `Graha` (the Vedic graha enum).
//! All existing code using `Domain` continues to work with `Graha`.
//! All `Graha` methods (index, symbol, name, offset, opposite, adjacent,
//! trines, knowledge_domain, archetype, parse, from_sign, etc.) are
//! available on `Domain` values directly.

use crate::astrology::Graha;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// Domain is a type alias for Graha — the Vedic planetary domain enum.
///
/// All existing code using `Domain` continues to compile. New code should
/// use `Graha` directly for clarity during and after the Vedic alignment.
pub type Domain = Graha;

/// A node on the wheel, combining a graha with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub domain: Domain,
    pub symbol: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    /// Arc index on the wheel (0–8)
    pub index: usize,
    /// Opposite graha (index + 4) % 9 — 160° opposition
    pub opposite: Domain,
    /// Trine grahas (index + 3) % 9 and (index + 6) % 9 — 120° and 240°
    pub trines: [Domain; 2],
}

/// All 9 domains in wheel order (Surya → Ketu at 40° intervals).
pub const ALL_DOMAINS: [Domain; 9] = [
    Domain::Surya,
    Domain::Chandra,
    Domain::Mangala,
    Domain::Budha,
    Domain::Brihaspati,
    Domain::Shukra,
    Domain::Shani,
    Domain::Rahu,
    Domain::Ketu,
];

/// All 9 nodes with metadata, lazily initialized on first access.
pub fn all_nodes() -> &'static Vec<Node> {
    ALL_NODES.get_or_init(|| {
        ALL_DOMAINS
            .iter()
            .enumerate()
            .map(|(i, &domain)| Node {
                domain,
                symbol: domain.symbol(),
                name: domain.name(),
                description: domain.archetype(),
                index: i,
                opposite: domain.opposite(),
                trines: domain.trines(),
            })
            .collect()
    })
}

/// Pre-computed node list.
pub static ALL_NODES: OnceLock<Vec<Node>> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_index_roundtrip() {
        for d in &ALL_DOMAINS {
            assert_eq!(Graha::from_index(d.index()), *d);
        }
    }

    #[test]
    fn test_opposite_is_4_steps() {
        for d in &ALL_DOMAINS {
            let opp = d.opposite();
            let diff = (d.index() as isize - opp.index() as isize).abs();
            assert!(diff == 4 || diff == 5);
        }
    }

    #[test]
    fn test_parse_variants() {
        assert_eq!(Domain::parse("surya"), Some(Domain::Surya));
        assert_eq!(Domain::parse("☉"), Some(Domain::Surya));
        assert_eq!(Domain::parse("sun"), Some(Domain::Surya));
        assert_eq!(Domain::parse("mangala"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("MARS"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("rahu"), Some(Domain::Rahu));
        assert_eq!(Domain::parse("unknown"), None);
    }

    #[test]
    fn test_greek_aliases() {
        // Greek-letter aliases should resolve (Phase 7 — Greek ontology)
        assert_eq!(Domain::parse("alpha"), Some(Domain::Surya));
        assert_eq!(Domain::parse("β"), Some(Domain::Chandra));
        assert_eq!(Domain::parse("gamma"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("δ"), Some(Domain::Budha));
        assert_eq!(Domain::parse("epsilon"), Some(Domain::Brihaspati));
        assert_eq!(Domain::parse("zeta"), Some(Domain::Shukra));
        assert_eq!(Domain::parse("η"), Some(Domain::Shani));
        assert_eq!(Domain::parse("theta"), Some(Domain::Rahu));
        assert_eq!(Domain::parse("ι"), Some(Domain::Ketu));
    }

    #[test]
    fn test_greek_names() {
        assert_eq!(Domain::Surya.greek_name(), "Alpha (α)");
        assert_eq!(Domain::Chandra.greek_name(), "Beta (β)");
        assert_eq!(Domain::Mangala.greek_name(), "Gamma (γ)");
        assert_eq!(Domain::Budha.greek_name(), "Delta (δ)");
        assert_eq!(Domain::Brihaspati.greek_name(), "Epsilon (ε)");
        assert_eq!(Domain::Shukra.greek_name(), "Zeta (ζ)");
        assert_eq!(Domain::Shani.greek_name(), "Eta (η)");
        assert_eq!(Domain::Rahu.greek_name(), "Theta (θ)");
        assert_eq!(Domain::Ketu.greek_name(), "Iota (ι)");
    }

    #[test]
    fn test_greek_symbols() {
        assert_eq!(Domain::Surya.greek_symbol(), "α");
        assert_eq!(Domain::Chandra.greek_symbol(), "β");
        assert_eq!(Domain::Mangala.greek_symbol(), "γ");
        assert_eq!(Domain::Ketu.greek_symbol(), "ι");
    }

    #[test]
    fn test_domain_format() {
        let s = format!("{}", Domain::Surya);
        assert!(s.contains("☉"));
        assert!(s.contains("Surya"));
    }

    #[test]
    fn test_adjacent() {
        let adj = Domain::Surya.adjacent();
        assert_eq!(adj[0], Domain::Chandra);
        assert_eq!(adj[1], Domain::Ketu);
    }

    #[test]
    fn test_trines() {
        let trines = Domain::Surya.trines();
        assert_eq!(trines[0], Domain::Budha);
        assert_eq!(trines[1], Domain::Shani);
    }

    #[test]
    fn test_node_metadata() {
        let nodes = all_nodes();
        assert_eq!(nodes.len(), 9);
        for n in nodes {
            assert!(!n.symbol.is_empty());
            assert!(!n.name.is_empty());
            assert!(!n.description.is_empty());
        }
    }
}
