use serde::{Deserialize, Serialize};

/// The 7 classical planetary rulers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlanetaryRuler {
    Sun,
    Moon,
    Mercury,
    Venus,
    Mars,
    Jupiter,
    Saturn,
}

impl PlanetaryRuler {
    pub const COUNT: usize = 7;

    pub fn index(self) -> usize {
        match self {
            PlanetaryRuler::Sun => 0,
            PlanetaryRuler::Moon => 1,
            PlanetaryRuler::Mercury => 2,
            PlanetaryRuler::Venus => 3,
            PlanetaryRuler::Mars => 4,
            PlanetaryRuler::Jupiter => 5,
            PlanetaryRuler::Saturn => 6,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i % 7 {
            0 => PlanetaryRuler::Sun,
            1 => PlanetaryRuler::Moon,
            2 => PlanetaryRuler::Mercury,
            3 => PlanetaryRuler::Venus,
            4 => PlanetaryRuler::Mars,
            5 => PlanetaryRuler::Jupiter,
            6 => PlanetaryRuler::Saturn,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            PlanetaryRuler::Sun => "☉",
            PlanetaryRuler::Moon => "☽",
            PlanetaryRuler::Mercury => "☿",
            PlanetaryRuler::Venus => "♀",
            PlanetaryRuler::Mars => "♂",
            PlanetaryRuler::Jupiter => "♃",
            PlanetaryRuler::Saturn => "♄",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PlanetaryRuler::Sun => "Sun",
            PlanetaryRuler::Moon => "Moon",
            PlanetaryRuler::Mercury => "Mercury",
            PlanetaryRuler::Venus => "Venus",
            PlanetaryRuler::Mars => "Mars",
            PlanetaryRuler::Jupiter => "Jupiter",
            PlanetaryRuler::Saturn => "Saturn",
        }
    }
}

impl std::fmt::Display for PlanetaryRuler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ruler_index_roundtrip() {
        for i in 0..7 {
            let r = PlanetaryRuler::from_index(i);
            assert_eq!(r.index(), i);
        }
    }
}
