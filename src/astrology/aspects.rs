use serde::{Deserialize, Serialize};

/// Ptolemaic aspects on the 12-sign zodiac wheel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignAspect {
    Conjunction,
    Sextile,
    Trine,
    Square,
    Opposition,
}

impl SignAspect {
    pub fn between_sign_indices(a: usize, b: usize) -> Self {
        let diff = (a as isize - b as isize).abs();
        let min_diff = if diff < 12 - diff { diff } else { 12 - diff };
        match min_diff {
            0 => SignAspect::Conjunction,
            2 => SignAspect::Sextile,
            3 => SignAspect::Trine,
            4 => SignAspect::Square,
            6 => SignAspect::Opposition,
            _ => SignAspect::Conjunction,
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            SignAspect::Conjunction => "☌",
            SignAspect::Sextile => "⚹",
            SignAspect::Trine => "△",
            SignAspect::Square => "□",
            SignAspect::Opposition => "☍",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SignAspect::Conjunction => "Conjunction",
            SignAspect::Sextile => "Sextile",
            SignAspect::Trine => "Trine",
            SignAspect::Square => "Square",
            SignAspect::Opposition => "Opposition",
        }
    }
}

impl std::fmt::Display for SignAspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conjunction() {
        assert_eq!(
            SignAspect::between_sign_indices(0, 0),
            SignAspect::Conjunction
        );
        assert_eq!(
            SignAspect::between_sign_indices(5, 5),
            SignAspect::Conjunction
        );
    }

    #[test]
    fn test_opposition() {
        assert_eq!(
            SignAspect::between_sign_indices(0, 6),
            SignAspect::Opposition
        );
        assert_eq!(
            SignAspect::between_sign_indices(3, 9),
            SignAspect::Opposition
        );
    }

    #[test]
    fn test_trine() {
        assert_eq!(SignAspect::between_sign_indices(0, 3), SignAspect::Trine);
        assert_eq!(SignAspect::between_sign_indices(0, 9), SignAspect::Trine);
    }
}
