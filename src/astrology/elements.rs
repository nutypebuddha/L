use serde::{Deserialize, Serialize};

/// The 4 classical elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Fire,
    Earth,
    Air,
    Water,
}

impl Element {
    pub const COUNT: usize = 4;

    pub fn index(self) -> usize {
        match self {
            Element::Fire => 0,
            Element::Earth => 1,
            Element::Air => 2,
            Element::Water => 3,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i % 4 {
            0 => Element::Fire,
            1 => Element::Earth,
            2 => Element::Air,
            3 => Element::Water,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Element::Fire => "🔥",
            Element::Earth => "🜁",
            Element::Air => "🜄",
            Element::Water => "🜃",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Element::Fire => "Fire",
            Element::Earth => "Earth",
            Element::Air => "Air",
            Element::Water => "Water",
        }
    }

    pub fn quality(self) -> &'static str {
        match self {
            Element::Fire => "Hot & Dry — passion, energy, will",
            Element::Earth => "Cold & Dry — practical, stable, material",
            Element::Air => "Hot & Wet — intellectual, social, communicative",
            Element::Water => "Cold & Wet — emotional, intuitive, reflective",
        }
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_index_roundtrip() {
        for i in 0..4 {
            let elem = Element::from_index(i);
            assert_eq!(elem.index(), i);
        }
    }

    #[test]
    fn test_element_symbols() {
        assert_eq!(Element::Fire.symbol(), "🔥");
        assert_eq!(Element::Water.symbol(), "🜃");
    }
}
