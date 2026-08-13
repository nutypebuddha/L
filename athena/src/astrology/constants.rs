//! # Astrological Constants — Precomputed Lookup Tables
//!
//! Static mappings between the 7 astrology axes for fast lookup.
//! These are used by the Change Sorter, Gyroscopic Wheel, and
//! entity/formula classification systems.

/// Get the element index for a given sign index (0–11).
pub const fn sign_to_element(sign: usize) -> usize {
    match sign {
        0 | 4 | 8 => 0,  // Fire: Aries, Leo, Sagittarius
        1 | 5 | 9 => 1,  // Earth: Taurus, Virgo, Capricorn
        2 | 6 | 10 => 2, // Air: Gemini, Libra, Aquarius
        3 | 7 | 11 => 3, // Water: Cancer, Scorpio, Pisces
        _ => 0,
    }
}

/// Get the modality index for a given sign index (0–11).
pub const fn sign_to_modality(sign: usize) -> usize {
    match sign {
        0 | 3 | 6 | 9 => 0,  // Cardinal: Aries, Cancer, Libra, Capricorn
        1 | 4 | 7 | 10 => 1, // Fixed: Taurus, Leo, Scorpio, Aquarius
        2 | 5 | 8 | 11 => 2, // Mutable: Gemini, Virgo, Sagittarius, Pisces
        _ => 0,
    }
}

/// Get the planetary ruler index for a given sign index (0–11).
pub const fn sign_to_ruler(sign: usize) -> usize {
    match sign {
        0 => 4,  // Aries → Mars
        1 => 3,  // Taurus → Venus
        2 => 2,  // Gemini → Mercury
        3 => 1,  // Cancer → Moon
        4 => 0,  // Leo → Sun
        5 => 2,  // Virgo → Mercury
        6 => 3,  // Libra → Venus
        7 => 4,  // Scorpio → Mars
        8 => 5,  // Sagittarius → Jupiter
        9 => 6,  // Capricorn → Saturn
        10 => 6, // Aquarius → Saturn
        11 => 5, // Pisces → Jupiter
        _ => 0,
    }
}

/// Get the polarity for a given sign index (0–11): 0 = Yang, 1 = Yin.
pub const fn sign_to_polarity(sign: usize) -> f64 {
    match sign {
        0 | 2 | 4 | 6 | 8 | 10 => 0.0, // Yang
        1 | 3 | 5 | 7 | 9 | 11 => 1.0, // Yin
        _ => 0.5,
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
pub const fn element_signs(element: usize) -> &'static [usize] {
    match element {
        0 => &[0, 4, 8],  // Fire: Aries, Leo, Sagittarius
        1 => &[1, 5, 9],  // Earth: Taurus, Virgo, Capricorn
        2 => &[2, 6, 10], // Air: Gemini, Libra, Aquarius
        3 => &[3, 7, 11], // Water: Cancer, Scorpio, Pisces
        _ => &[],
    }
}

/// The signs that a given modality includes.
pub const fn modality_signs(modality: usize) -> &'static [usize] {
    match modality {
        0 => &[0, 3, 6, 9],  // Cardinal: Aries, Cancer, Libra, Capricorn
        1 => &[1, 4, 7, 10], // Fixed: Taurus, Leo, Scorpio, Aquarius
        2 => &[2, 5, 8, 11], // Mutable: Gemini, Virgo, Sagittarius, Pisces
        _ => &[],
    }
}

/// Convert a word to a sign index if it matches a zodiac sign name or symbol.
pub fn word_to_sign(word: &str) -> Option<usize> {
    let lower = word.to_lowercase();
    match lower.as_str() {
        "aries" | "♈" => Some(0),
        "taurus" | "♉" => Some(1),
        "gemini" | "♊" => Some(2),
        "cancer" | "♋" => Some(3),
        "leo" | "♌" => Some(4),
        "virgo" | "♍" => Some(5),
        "libra" | "♎" => Some(6),
        "scorpio" | "♏" => Some(7),
        "sagittarius" | "♐" => Some(8),
        "capricorn" | "♑" => Some(9),
        "aquarius" | "♒" => Some(10),
        "pisces" | "♓" => Some(11),
        _ => None,
    }
}

/// Convert a word to an element index if it matches.
pub fn word_to_element(word: &str) -> Option<usize> {
    let lower = word.to_lowercase();
    match lower.as_str() {
        "fire" | "🔥" => Some(0),
        "earth" | "🜁" => Some(1),
        "air" | "🜄" => Some(2),
        "water" | "🜂" => Some(3),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_to_element_mappings() {
        // Fire signs
        assert_eq!(sign_to_element(0), 0); // Aries → Fire
        assert_eq!(sign_to_element(4), 0); // Leo → Fire
        assert_eq!(sign_to_element(8), 0); // Sagittarius → Fire
                                           // Earth
        assert_eq!(sign_to_element(1), 1); // Taurus → Earth
        assert_eq!(sign_to_element(5), 1); // Virgo → Earth
                                           // Air
        assert_eq!(sign_to_element(2), 2); // Gemini → Air
        assert_eq!(sign_to_element(6), 2); // Libra → Air
                                           // Water
        assert_eq!(sign_to_element(3), 3); // Cancer → Water
        assert_eq!(sign_to_element(7), 3); // Scorpio → Water
    }

    #[test]
    fn test_sign_to_ruler_mappings() {
        assert_eq!(sign_to_ruler(0), 4); // Aries → Mars
        assert_eq!(sign_to_ruler(4), 0); // Leo → Sun
    }

    #[test]
    fn test_arc_distance_self() {
        assert_eq!(arc_distance(0, 0), 0);
    }

    #[test]
    fn test_arc_distance_adjacent() {
        assert_eq!(arc_distance(0, 1), 1);
        assert_eq!(arc_distance(0, 11), 1); // wrap around
    }

    #[test]
    fn test_arc_distance_opposite() {
        assert_eq!(arc_distance(0, 6), 6);
    }

    #[test]
    fn test_has_aspect_valid() {
        assert!(has_aspect(0, 0)); // conjunction
        assert!(has_aspect(0, 1)); // sextile
        assert!(has_aspect(0, 3)); // square
        assert!(has_aspect(0, 4)); // trine
        assert!(has_aspect(0, 6)); // opposition
    }

    #[test]
    fn test_has_aspect_invalid() {
        assert!(!has_aspect(0, 2)); // distance 2
        assert!(!has_aspect(0, 5)); // distance 5
    }

    #[test]
    fn test_word_to_sign() {
        assert_eq!(word_to_sign("aries"), Some(0));
        assert_eq!(word_to_sign("♈"), Some(0));
        assert_eq!(word_to_sign("pisces"), Some(11));
        assert_eq!(word_to_sign("unknown"), None);
    }

    #[test]
    fn test_word_to_element() {
        assert_eq!(word_to_element("fire"), Some(0));
        assert_eq!(word_to_element("🔥"), Some(0));
        assert_eq!(word_to_element("unknown"), None);
    }

    #[test]
    fn test_element_signs() {
        assert_eq!(element_signs(0), &[0, 4, 8]); // Fire
    }

    #[test]
    fn test_modality_signs() {
        assert_eq!(modality_signs(0), &[0, 3, 6, 9]); // Cardinal
    }
}
