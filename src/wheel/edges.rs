use serde::{Deserialize, Serialize};
use std::fmt;

use super::nodes::Domain;

/// The **structural composition relationship** between two domains on
/// the fixed 9-node wheel. This is NOT an astronomical computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompositionAspect {
    Aligned,
    Adjacent,
    Harmonic,
    Tense,
    Antipodal,
}

/// Precomputed 9×9 lookup table for `CompositionAspect::between(a, b)`.
const COMPOSITION_ASPECT_TABLE: [[CompositionAspect; 9]; 9] = {
    let mut table = [[CompositionAspect::Aligned; 9]; 9];
    let mut i: usize = 0;
    while i < 9 {
        let mut j: usize = 0;
        while j < 9 {
            let diff = i.abs_diff(j);
            let min_diff = if diff < 9 - diff { diff } else { 9 - diff };
            table[i][j] = match min_diff {
                0 => CompositionAspect::Aligned,
                1 => CompositionAspect::Adjacent,
                2 => CompositionAspect::Tense,
                3 => CompositionAspect::Harmonic,
                4 => CompositionAspect::Antipodal,
                _ => CompositionAspect::Aligned,
            };
            j += 1;
        }
        i += 1;
    }
    table
};

impl CompositionAspect {
    /// Determine the structural composition relationship between two domains.
    #[inline]
    pub fn between(a: Domain, b: Domain) -> CompositionAspect {
        COMPOSITION_ASPECT_TABLE[a.index()][b.index()]
    }

    /// The structural arc distance (minimum steps on the 9-node wheel).
    #[inline]
    pub fn arc_distance(self) -> usize {
        match self {
            CompositionAspect::Aligned => 0,
            CompositionAspect::Adjacent => 1,
            CompositionAspect::Harmonic => 3,
            CompositionAspect::Tense => 2,
            CompositionAspect::Antipodal => 4,
        }
    }

    /// Whether formulas compose directly across this relationship.
    #[inline]
    pub fn is_direct(self) -> bool {
        matches!(
            self,
            CompositionAspect::Aligned | CompositionAspect::Adjacent | CompositionAspect::Harmonic
        )
    }

    /// Base confidence for this structural composition relationship.
    #[inline]
    pub fn confidence(self) -> f64 {
        match self {
            CompositionAspect::Aligned => 1.00,
            CompositionAspect::Adjacent => 0.95,
            CompositionAspect::Harmonic => 0.90,
            CompositionAspect::Tense => 0.75,
            CompositionAspect::Antipodal => 0.60,
        }
    }

    /// Whether this aspect represents tension or inversion.
    #[inline]
    pub fn is_tension(self) -> bool {
        matches!(
            self,
            CompositionAspect::Tense | CompositionAspect::Antipodal
        )
    }
}

/// A typed relationship between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: Domain,
    pub to: Domain,
    pub aspect: CompositionAspect,
    pub distance: usize,
    pub direction: Direction,
}

/// The direction of traversal along the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Reverse,
}

impl Relationship {
    /// Create a relationship between two domains.
    pub fn new(from: Domain, to: Domain) -> Self {
        let aspect = CompositionAspect::between(from, to);
        let distance = aspect.arc_distance();
        let (from_idx, to_idx) = (from.index(), to.index());
        let forward = ((to_idx as isize - from_idx as isize).rem_euclid(9)) <= 4;
        let direction = if forward {
            Direction::Forward
        } else {
            Direction::Reverse
        };
        Relationship {
            from,
            to,
            aspect,
            distance,
            direction,
        }
    }

    /// Reverse the direction of this relationship.
    pub fn reverse(&self) -> Self {
        Relationship {
            from: self.to,
            to: self.from,
            aspect: self.aspect,
            distance: self.distance,
            direction: match self.direction {
                Direction::Forward => Direction::Reverse,
                Direction::Reverse => Direction::Forward,
            },
        }
    }
}

impl fmt::Display for CompositionAspect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositionAspect::Aligned => write!(f, "Aligned (same domain)"),
            CompositionAspect::Adjacent => write!(f, "Adjacent (1 step, structural)"),
            CompositionAspect::Harmonic => write!(f, "Harmonic (3 steps, structural)"),
            CompositionAspect::Tense => write!(f, "Tense (2 steps, structural)"),
            CompositionAspect::Antipodal => write!(
                f,
                "Antipodal (4 steps, structural — NOT a real 180° opposition)"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composition_aligned_self() {
        assert_eq!(
            CompositionAspect::between(Domain::Mangala, Domain::Mangala),
            CompositionAspect::Aligned
        );
    }

    #[test]
    fn test_composition_adjacent() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Chandra),
            CompositionAspect::Adjacent
        );
        assert_eq!(
            CompositionAspect::between(Domain::Ketu, Domain::Surya),
            CompositionAspect::Adjacent
        );
    }

    #[test]
    fn test_composition_antipodal() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Brihaspati),
            CompositionAspect::Antipodal
        );
        assert_eq!(
            CompositionAspect::between(Domain::Brihaspati, Domain::Surya),
            CompositionAspect::Antipodal
        );
    }

    #[test]
    fn test_composition_harmonic() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Budha),
            CompositionAspect::Harmonic
        );
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Shani),
            CompositionAspect::Harmonic
        );
    }

    #[test]
    fn test_composition_tense() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Mangala),
            CompositionAspect::Tense
        );
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Rahu),
            CompositionAspect::Tense
        );
    }

    #[test]
    fn test_relationship_creation() {
        let r = Relationship::new(Domain::Surya, Domain::Chandra);
        assert_eq!(r.aspect, CompositionAspect::Adjacent);
        assert_eq!(r.distance, 1);
    }

    #[test]
    fn test_direct_and_tension() {
        assert!(CompositionAspect::Aligned.is_direct());
        assert!(CompositionAspect::Adjacent.is_direct());
        assert!(!CompositionAspect::Tense.is_direct());
        assert!(!CompositionAspect::Antipodal.is_direct());
        assert!(CompositionAspect::Tense.is_tension());
        assert!(CompositionAspect::Antipodal.is_tension());
    }

    #[test]
    fn t27_rahu_ketu_structural_adjacent_not_astronomical_opposition() {
        let structural = CompositionAspect::between(Domain::Rahu, Domain::Ketu);
        assert_eq!(structural, CompositionAspect::Adjacent);
        let printed = format!("{structural}");
        assert!(
            !printed.contains("Opposition") && !printed.contains("Sextile"),
            "T27: CompositionAspect must never print astrological aspect names; got {printed:?}"
        );
    }
}
