use serde::{Deserialize, Serialize};

/// The 3 modalities (qualities) of the zodiac.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Cardinal,
    Fixed,
    Mutable,
}

impl Modality {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        match self {
            Modality::Cardinal => 0,
            Modality::Fixed => 1,
            Modality::Mutable => 2,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i % 3 {
            0 => Modality::Cardinal,
            1 => Modality::Fixed,
            2 => Modality::Mutable,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Modality::Cardinal => "🅲",
            Modality::Fixed => "🅵",
            Modality::Mutable => "🅼",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Modality::Cardinal => "Cardinal",
            Modality::Fixed => "Fixed",
            Modality::Mutable => "Mutable",
        }
    }

    pub fn quality(self) -> &'static str {
        match self {
            Modality::Cardinal => "Initiating, leadership, action-oriented",
            Modality::Fixed => "Stabilizing, sustaining, determined",
            Modality::Mutable => "Adapting, flexible, versatile",
        }
    }
}

impl std::fmt::Display for Modality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modality_index_roundtrip() {
        for i in 0..3 {
            let m = Modality::from_index(i);
            assert_eq!(m.index(), i);
        }
    }
}
