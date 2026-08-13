use crate::astrology::{Element, Modality, PlanetaryRuler, Sign};

/// Get the Element for a given Sign.
pub const fn sign_to_element(sign: Sign) -> Element {
    match sign {
        Sign::Aries | Sign::Leo | Sign::Sagittarius => Element::Fire,
        Sign::Taurus | Sign::Virgo | Sign::Capricorn => Element::Earth,
        Sign::Gemini | Sign::Libra | Sign::Aquarius => Element::Air,
        Sign::Cancer | Sign::Scorpio | Sign::Pisces => Element::Water,
    }
}

/// Get the Modality for a given Sign.
pub const fn sign_to_modality(sign: Sign) -> Modality {
    match sign {
        Sign::Aries | Sign::Cancer | Sign::Libra | Sign::Capricorn => Modality::Cardinal,
        Sign::Taurus | Sign::Leo | Sign::Scorpio | Sign::Aquarius => Modality::Fixed,
        Sign::Gemini | Sign::Virgo | Sign::Sagittarius | Sign::Pisces => Modality::Mutable,
    }
}

/// Get the PlanetaryRuler for a given Sign.
pub const fn sign_to_ruler(sign: Sign) -> PlanetaryRuler {
    match sign {
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

/// Get the polarity for a given sign: 0.0 = Yang, 1.0 = Yin.
pub const fn sign_to_polarity(sign: Sign) -> f64 {
    match sign {
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

/// Precomputed arc distance between any two sign indices (0–11).
pub const fn arc_distance(a: usize, b: usize) -> usize {
    let diff = a.abs_diff(b);
    if diff > 6 {
        12 - diff
    } else {
        diff
    }
}

/// Check if two sign indices form a valid Ptolemaic aspect.
pub const fn has_aspect(a: usize, b: usize) -> bool {
    let dist = arc_distance(a, b);
    matches!(dist, 0 | 1 | 3 | 4 | 6)
}

/// The signs that a given element includes.
pub const fn element_signs(element: Element) -> &'static [Sign] {
    match element {
        Element::Fire => &[Sign::Aries, Sign::Leo, Sign::Sagittarius],
        Element::Earth => &[Sign::Taurus, Sign::Virgo, Sign::Capricorn],
        Element::Air => &[Sign::Gemini, Sign::Libra, Sign::Aquarius],
        Element::Water => &[Sign::Cancer, Sign::Scorpio, Sign::Pisces],
    }
}

/// The signs that a given modality includes.
pub const fn modality_signs(modality: Modality) -> &'static [Sign] {
    match modality {
        Modality::Cardinal => &[Sign::Aries, Sign::Cancer, Sign::Libra, Sign::Capricorn],
        Modality::Fixed => &[Sign::Taurus, Sign::Leo, Sign::Scorpio, Sign::Aquarius],
        Modality::Mutable => &[Sign::Gemini, Sign::Virgo, Sign::Sagittarius, Sign::Pisces],
    }
}

/// Convert a word to a Sign if it matches a zodiac sign name or symbol.
pub fn word_to_sign(word: &str) -> Option<Sign> {
    let lower = word.to_lowercase();
    match lower.as_str() {
        "aries" | "♈" => Some(Sign::Aries),
        "taurus" | "♉" => Some(Sign::Taurus),
        "gemini" | "♊" => Some(Sign::Gemini),
        "cancer" | "♋" => Some(Sign::Cancer),
        "leo" | "♌" => Some(Sign::Leo),
        "virgo" | "♍" => Some(Sign::Virgo),
        "libra" | "♎" => Some(Sign::Libra),
        "scorpio" | "♏" => Some(Sign::Scorpio),
        "sagittarius" | "♐" => Some(Sign::Sagittarius),
        "capricorn" | "♑" => Some(Sign::Capricorn),
        "aquarius" | "♒" => Some(Sign::Aquarius),
        "pisces" | "♓" => Some(Sign::Pisces),
        _ => None,
    }
}

/// Convert a word to an Element if it matches.
pub fn word_to_element(word: &str) -> Option<Element> {
    let lower = word.to_lowercase();
    match lower.as_str() {
        "fire" | "🔥" => Some(Element::Fire),
        "earth" | "🜁" => Some(Element::Earth),
        "air" | "🜄" => Some(Element::Air),
        "water" | "🜃" => Some(Element::Water),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_to_element_mappings() {
        assert_eq!(sign_to_element(Sign::Aries), Element::Fire);
        assert_eq!(sign_to_element(Sign::Leo), Element::Fire);
        assert_eq!(sign_to_element(Sign::Sagittarius), Element::Fire);
        assert_eq!(sign_to_element(Sign::Taurus), Element::Earth);
        assert_eq!(sign_to_element(Sign::Virgo), Element::Earth);
        assert_eq!(sign_to_element(Sign::Gemini), Element::Air);
        assert_eq!(sign_to_element(Sign::Libra), Element::Air);
        assert_eq!(sign_to_element(Sign::Cancer), Element::Water);
        assert_eq!(sign_to_element(Sign::Scorpio), Element::Water);
    }

    #[test]
    fn test_sign_to_ruler_mappings() {
        assert_eq!(sign_to_ruler(Sign::Aries), PlanetaryRuler::Mars);
        assert_eq!(sign_to_ruler(Sign::Leo), PlanetaryRuler::Sun);
    }

    #[test]
    fn test_arc_distance_self() {
        assert_eq!(arc_distance(0, 0), 0);
    }

    #[test]
    fn test_arc_distance_adjacent() {
        assert_eq!(arc_distance(0, 1), 1);
        assert_eq!(arc_distance(0, 11), 1);
    }

    #[test]
    fn test_arc_distance_opposite() {
        assert_eq!(arc_distance(0, 6), 6);
    }

    #[test]
    fn test_has_aspect_valid() {
        assert!(has_aspect(0, 0)); // conjunction
        assert!(has_aspect(0, 1)); // sextile (adjacent)
        assert!(has_aspect(0, 3)); // trine
        assert!(has_aspect(0, 4)); // square
        assert!(has_aspect(0, 6)); // opposition
    }

    #[test]
    fn test_has_aspect_invalid() {
        assert!(!has_aspect(0, 2));
        assert!(!has_aspect(0, 5));
    }

    #[test]
    fn test_word_to_sign() {
        assert_eq!(word_to_sign("aries"), Some(Sign::Aries));
        assert_eq!(word_to_sign("♈"), Some(Sign::Aries));
        assert_eq!(word_to_sign("pisces"), Some(Sign::Pisces));
        assert_eq!(word_to_sign("unknown"), None);
    }

    #[test]
    fn test_word_to_element() {
        assert_eq!(word_to_element("fire"), Some(Element::Fire));
        assert_eq!(word_to_element("🔥"), Some(Element::Fire));
        assert_eq!(word_to_element("unknown"), None);
    }

    #[test]
    fn test_element_signs() {
        assert_eq!(
            element_signs(Element::Fire),
            &[Sign::Aries, Sign::Leo, Sign::Sagittarius]
        );
    }

    #[test]
    fn test_modality_signs() {
        assert_eq!(
            modality_signs(Modality::Cardinal),
            &[Sign::Aries, Sign::Cancer, Sign::Libra, Sign::Capricorn]
        );
    }
}
