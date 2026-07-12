use serde::{Deserialize, Serialize};

use super::elements::Element;
use super::modalities::Modality;
use super::rulers::PlanetaryRuler;

/// The 12 zodiac signs, ordered from Aries (0) to Pisces (11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Sign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

impl Sign {
    pub const COUNT: usize = 12;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i % 12 {
            0 => Sign::Aries,
            1 => Sign::Taurus,
            2 => Sign::Gemini,
            3 => Sign::Cancer,
            4 => Sign::Leo,
            5 => Sign::Virgo,
            6 => Sign::Libra,
            7 => Sign::Scorpio,
            8 => Sign::Sagittarius,
            9 => Sign::Capricorn,
            10 => Sign::Aquarius,
            11 => Sign::Pisces,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Sign::Aries => "♈",
            Sign::Taurus => "♉",
            Sign::Gemini => "♊",
            Sign::Cancer => "♋",
            Sign::Leo => "♌",
            Sign::Virgo => "♍",
            Sign::Libra => "♎",
            Sign::Scorpio => "♏",
            Sign::Sagittarius => "♐",
            Sign::Capricorn => "♑",
            Sign::Aquarius => "♒",
            Sign::Pisces => "♓",
        }
    }

    pub fn element(self) -> Element {
        match self {
            Sign::Aries | Sign::Leo | Sign::Sagittarius => Element::Fire,
            Sign::Taurus | Sign::Virgo | Sign::Capricorn => Element::Earth,
            Sign::Gemini | Sign::Libra | Sign::Aquarius => Element::Air,
            Sign::Cancer | Sign::Scorpio | Sign::Pisces => Element::Water,
        }
    }

    pub fn modality(self) -> Modality {
        match self {
            Sign::Aries | Sign::Cancer | Sign::Libra | Sign::Capricorn => Modality::Cardinal,
            Sign::Taurus | Sign::Leo | Sign::Scorpio | Sign::Aquarius => Modality::Fixed,
            Sign::Gemini | Sign::Virgo | Sign::Sagittarius | Sign::Pisces => Modality::Mutable,
        }
    }

    pub fn ruler(self) -> PlanetaryRuler {
        match self {
            Sign::Aries => PlanetaryRuler::Mars,
            Sign::Taurus => PlanetaryRuler::Venus,
            Sign::Gemini => PlanetaryRuler::Mercury,
            Sign::Cancer => PlanetaryRuler::Moon,
            Sign::Leo => PlanetaryRuler::Sun,
            Sign::Virgo => PlanetaryRuler::Mercury,
            Sign::Libra => PlanetaryRuler::Venus,
            Sign::Scorpio => PlanetaryRuler::Mars,
            Sign::Sagittarius => PlanetaryRuler::Jupiter,
            Sign::Capricorn => PlanetaryRuler::Saturn,
            Sign::Aquarius => PlanetaryRuler::Saturn,
            Sign::Pisces => PlanetaryRuler::Jupiter,
        }
    }

    pub fn polarity(self) -> f64 {
        match self {
            Sign::Aries
            | Sign::Gemini
            | Sign::Leo
            | Sign::Libra
            | Sign::Sagittarius
            | Sign::Aquarius => 0.0,
            Sign::Taurus
            | Sign::Cancer
            | Sign::Virgo
            | Sign::Scorpio
            | Sign::Capricorn
            | Sign::Pisces => 1.0,
        }
    }

    pub fn all() -> [Sign; 12] {
        [
            Sign::Aries,
            Sign::Taurus,
            Sign::Gemini,
            Sign::Cancer,
            Sign::Leo,
            Sign::Virgo,
            Sign::Libra,
            Sign::Scorpio,
            Sign::Sagittarius,
            Sign::Capricorn,
            Sign::Aquarius,
            Sign::Pisces,
        ]
    }
}

impl std::fmt::Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.symbol(), self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_index_roundtrip() {
        for i in 0..12 {
            let sign = Sign::from_index(i);
            assert_eq!(sign.index(), i);
        }
    }

    #[test]
    fn test_sign_has_symbol() {
        for sign in Sign::all() {
            assert!(!sign.symbol().is_empty());
        }
    }

    #[test]
    fn test_aries_properties() {
        assert_eq!(Sign::Aries.element(), Element::Fire);
        assert_eq!(Sign::Aries.modality(), Modality::Cardinal);
        assert_eq!(Sign::Aries.ruler(), PlanetaryRuler::Mars);
        assert!((Sign::Aries.polarity() - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_taurus_properties() {
        assert_eq!(Sign::Taurus.element(), Element::Earth);
        assert_eq!(Sign::Taurus.modality(), Modality::Fixed);
        assert_eq!(Sign::Taurus.ruler(), PlanetaryRuler::Venus);
        assert!((Sign::Taurus.polarity() - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_twelve_signs() {
        assert_eq!(Sign::all().len(), 12);
    }
}
