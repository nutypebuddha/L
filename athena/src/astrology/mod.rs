//! # Astrology — 7-Axis Classification System
//!
//! Every token, formula, and entity in Athena is classified across 7
//! astrological axes simultaneously. No atom has a single domain — every
//! atom has a weighted distribution across all axes.
//!
//! ## The 7 Axes
//!
//! | Axis | Type | Range | Purpose |
//! |------|------|-------|---------|
//! | Signs | 12 f64 | [0.0, 1.0] | Zodiac sign activation (Aries–Pisces) |
//! | Elements | 4 f64 | [0.0, 1.0] | Fire, Earth, Air, Water |
//! | Modalities | 3 f64 | [0.0, 1.0] | Cardinal, Fixed, Mutable |
//! | Rulers | 7 f64 | [0.0, 1.0] | Planetary rulers (Sun–Saturn) |
//! | Houses | 12 f64 | [0.0, 1.0] | House activation (1st–12th) |
//! | Aspects | 5 f64 | [0.0, 1.0] | Aspect affinity |
//! | Polarity | 1 f64 | [0.0, 1.0] | Yang (0) → Yin (1) |
//!
//! ## Usage
//!
//! Classification is NOT a one-hot vector — it's a weighted distribution.
//! A formula can be 0.7 Aries + 0.3 Leo (both fire signs, different modality).

use serde::{Deserialize, Serialize};

mod aspects;
mod classifier;
mod constants;
mod elements;
mod houses;
mod modalities;
mod rulers;
mod signs;
mod vedic;

pub use aspects::*;
pub use classifier::*;
pub use constants::*;
pub use elements::*;
pub use houses::*;
pub use modalities::*;
pub use rulers::*;
pub use signs::*;
pub use vedic::*;

/// The 7-axis classification of any atom (token, formula, entity).
///
/// Every classification is a weighted distribution, not a one-hot vector.
/// All values are in [0.0, 1.0] and do not need to sum to 1.0 — they
/// represent independent activation strengths across each axis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AtomClassification {
    /// Zodiac sign activation: [Aries, Taurus, Gemini, Cancer, Leo, Virgo,
    /// Libra, Scorpio, Sagittarius, Capricorn, Aquarius, Pisces]
    pub signs: [f64; 12],

    /// Element activation: [Fire, Earth, Air, Water]
    pub elements: [f64; 4],

    /// Modality activation: [Cardinal, Fixed, Mutable]
    pub modalities: [f64; 3],

    /// Planetary ruler activation: [Sun, Moon, Mercury, Venus, Mars, Jupiter, Saturn]
    pub rulers: [f64; 7],

    /// House activation: [1st, 2nd, ..., 12th]
    pub houses: [f64; 12],

    /// Aspect affinity: [Conjunction, Sextile, Trine, Square, Opposition]
    pub aspects: [f64; 5],

    /// Polarity: 0.0 = Yang (masculine/positive), 1.0 = Yin (feminine/negative)
    pub polarity: f64,

    /// Vedic classification: grahas (9), nakshatras (27), gunas (3), vedic elements (5)
    pub vedic: VedicClassification,
}

impl Default for AtomClassification {
    fn default() -> Self {
        AtomClassification {
            signs: [0.0; 12],
            elements: [0.0; 4],
            modalities: [0.0; 3],
            rulers: [0.0; 7],
            houses: [0.0; 12],
            aspects: [0.0; 5],
            polarity: 0.5,
            vedic: VedicClassification::new(),
        }
    }
}

impl AtomClassification {
    /// Create a new classification with all zeros (neutral).
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a single sign's activation.
    pub fn with_sign(mut self, sign: Sign, value: f64) -> Self {
        self.signs[sign.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set a single element's activation.
    pub fn with_element(mut self, element: Element, value: f64) -> Self {
        self.elements[element.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set a single modality's activation.
    pub fn with_modality(mut self, modality: Modality, value: f64) -> Self {
        self.modalities[modality.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set a single planetary ruler's activation.
    pub fn with_ruler(mut self, ruler: PlanetaryRuler, value: f64) -> Self {
        self.rulers[ruler.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set polarity (0 = Yang, 1 = Yin).
    pub fn with_polarity(mut self, value: f64) -> Self {
        self.polarity = value.clamp(0.0, 1.0);
        self
    }

    /// Set a graha's activation in Vedic classification.
    pub fn with_graha(mut self, graha: Graha, value: f64) -> Self {
        self.vedic = self.vedic.with_graha(graha, value);
        self
    }

    /// Set a nakshatra's activation in Vedic classification.
    pub fn with_nakshatra(mut self, nak: Nakshatra, value: f64) -> Self {
        self.vedic = self.vedic.with_nakshatra(nak, value);
        self
    }

    /// Set a guna's activation in Vedic classification.
    pub fn with_guna(mut self, guna: Guna, value: f64) -> Self {
        self.vedic = self.vedic.with_guna(guna, value);
        self
    }

    /// Set a vedic element's activation in Vedic classification.
    pub fn with_vedic_element(mut self, elem: VedicElement, value: f64) -> Self {
        self.vedic = self.vedic.with_vedic_element(elem, value);
        self
    }

    /// Compute the dot product similarity between two classifications.
    /// Higher = more similar (max 7.0 if all axes match perfectly).
    pub fn similarity(&self, other: &AtomClassification) -> f64 {
        let mut sim = 0.0;
        for i in 0..12 {
            sim += self.signs[i] * other.signs[i];
        }
        for i in 0..4 {
            sim += self.elements[i] * other.elements[i];
        }
        for i in 0..3 {
            sim += self.modalities[i] * other.modalities[i];
        }
        for i in 0..7 {
            sim += self.rulers[i] * other.rulers[i];
        }
        for i in 0..12 {
            sim += self.houses[i] * other.houses[i];
        }
        for i in 0..5 {
            sim += self.aspects[i] * other.aspects[i];
        }
        sim + self.polarity * other.polarity
    }

    /// Merge two classifications by element-wise max (union).
    /// Used when combining multiple token classifications.
    pub fn merge_max(&self, other: &AtomClassification) -> AtomClassification {
        let mut result = self.clone();
        for i in 0..12 {
            result.signs[i] = result.signs[i].max(other.signs[i]);
        }
        for i in 0..4 {
            result.elements[i] = result.elements[i].max(other.elements[i]);
        }
        for i in 0..3 {
            result.modalities[i] = result.modalities[i].max(other.modalities[i]);
        }
        for i in 0..7 {
            result.rulers[i] = result.rulers[i].max(other.rulers[i]);
        }
        for i in 0..12 {
            result.houses[i] = result.houses[i].max(other.houses[i]);
        }
        for i in 0..5 {
            result.aspects[i] = result.aspects[i].max(other.aspects[i]);
        }
        result.polarity = result.polarity.max(other.polarity);
        // Merge Vedic
        for i in 0..9 {
            result.vedic.grahas[i] = result.vedic.grahas[i].max(other.vedic.grahas[i]);
        }
        for i in 0..27 {
            result.vedic.nakshatras[i] = result.vedic.nakshatras[i].max(other.vedic.nakshatras[i]);
        }
        for i in 0..3 {
            result.vedic.gunas[i] = result.vedic.gunas[i].max(other.vedic.gunas[i]);
        }
        for i in 0..5 {
            result.vedic.vedic_elements[i] =
                result.vedic.vedic_elements[i].max(other.vedic.vedic_elements[i]);
        }
        result.vedic.confidence = result.vedic.confidence.max(other.vedic.confidence);
        result
    }

    /// Merge two classifications by element-wise sum (accumulation).
    /// Used for token mass accumulation in the gyroscope.
    pub fn merge_sum(&self, other: &AtomClassification) -> AtomClassification {
        let mut result = self.clone();
        for i in 0..12 {
            result.signs[i] = (result.signs[i] + other.signs[i]).min(1.0);
        }
        for i in 0..4 {
            result.elements[i] = (result.elements[i] + other.elements[i]).min(1.0);
        }
        for i in 0..3 {
            result.modalities[i] = (result.modalities[i] + other.modalities[i]).min(1.0);
        }
        for i in 0..7 {
            result.rulers[i] = (result.rulers[i] + other.rulers[i]).min(1.0);
        }
        for i in 0..12 {
            result.houses[i] = (result.houses[i] + other.houses[i]).min(1.0);
        }
        for i in 0..5 {
            result.aspects[i] = (result.aspects[i] + other.aspects[i]).min(1.0);
        }
        result.polarity = (result.polarity + other.polarity).min(1.0);
        // Merge Vedic (sum, clamped)
        for i in 0..9 {
            result.vedic.grahas[i] = (result.vedic.grahas[i] + other.vedic.grahas[i]).min(1.0);
        }
        for i in 0..27 {
            result.vedic.nakshatras[i] =
                (result.vedic.nakshatras[i] + other.vedic.nakshatras[i]).min(1.0);
        }
        for i in 0..3 {
            result.vedic.gunas[i] = (result.vedic.gunas[i] + other.vedic.gunas[i]).min(1.0);
        }
        for i in 0..5 {
            result.vedic.vedic_elements[i] =
                (result.vedic.vedic_elements[i] + other.vedic.vedic_elements[i]).min(1.0);
        }
        result.vedic.confidence = (result.vedic.confidence + other.vedic.confidence).min(1.0);
        result
    }

    /// Find the dominant sign (highest activation).
    pub fn dominant_sign(&self) -> Option<Sign> {
        self.signs
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| Sign::from_index(i))
    }

    /// Find the dominant element (highest activation).
    pub fn dominant_element(&self) -> Option<Element> {
        self.elements
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| Element::from_index(i))
    }

    /// Find the dominant modality (highest activation).
    pub fn dominant_modality(&self) -> Option<Modality> {
        self.modalities
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| Modality::from_index(i))
    }
}

impl std::fmt::Display for AtomClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AtomClassification {{ ")?;
        if let Some(sign) = self.dominant_sign() {
            write!(f, "sign: {}, ", sign.symbol())?;
        }
        if let Some(elem) = self.dominant_element() {
            write!(f, "element: {}, ", elem.symbol())?;
        }
        if let Some(modality) = self.dominant_modality() {
            write!(f, "modality: {}, ", modality.symbol())?;
        }
        write!(f, "polarity: {:.2}", self.polarity)?;
        if let Some(g) = self.vedic.dominant_graha() {
            write!(f, ", graha: {}", g.name())?;
        }
        if let Some(g) = self.vedic.dominant_guna() {
            write!(f, ", guṇa: {}", g.name())?;
        }
        write!(f, " }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_classification() {
        let c = AtomClassification::new();
        assert_eq!(c.signs, [0.0; 12]);
        assert_eq!(c.polarity, 0.5);
    }

    #[test]
    fn test_with_sign() {
        let c = AtomClassification::new()
            .with_sign(Sign::Aries, 0.9)
            .with_sign(Sign::Leo, 0.3);
        assert!((c.signs[0] - 0.9).abs() < 1e-12);
        assert!((c.signs[4] - 0.3).abs() < 1e-12);
        assert!((c.signs[1] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn test_dominant_sign() {
        let c = AtomClassification::new()
            .with_sign(Sign::Taurus, 0.8)
            .with_sign(Sign::Gemini, 0.2);
        assert_eq!(c.dominant_sign(), Some(Sign::Taurus));
    }

    #[test]
    fn test_dominant_sign_none() {
        let c = AtomClassification::new();
        assert_eq!(c.dominant_sign(), None);
    }

    #[test]
    fn test_similarity() {
        // Same sign: signs match (1.0*1.0 = 1.0), polarity default (0.5*0.5 = 0.25)
        let a = AtomClassification::new().with_sign(Sign::Aries, 1.0);
        let b = AtomClassification::new().with_sign(Sign::Aries, 1.0);
        // signs[0]=1 + polarity=0.25 + all others 0 = 1.25
        assert!((a.similarity(&b) - 1.25).abs() < 1e-12);
    }

    #[test]
    fn test_similarity_different() {
        // Different signs: signs don't overlap, only polarity matches (0.5*0.5)
        let a = AtomClassification::new().with_sign(Sign::Aries, 1.0);
        let b = AtomClassification::new().with_sign(Sign::Taurus, 1.0);
        assert!((a.similarity(&b) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_merge_max() {
        let a = AtomClassification::new().with_sign(Sign::Aries, 0.8);
        let b = AtomClassification::new().with_sign(Sign::Taurus, 0.6);
        let merged = a.merge_max(&b);
        assert!((merged.signs[0] - 0.8).abs() < 1e-12);
        assert!((merged.signs[1] - 0.6).abs() < 1e-12);
    }

    #[test]
    fn test_merge_sum() {
        let a = AtomClassification::new().with_sign(Sign::Aries, 0.6);
        let b = AtomClassification::new().with_sign(Sign::Aries, 0.5);
        let merged = a.merge_sum(&b);
        assert!((merged.signs[0] - 1.0).abs() < 1e-12); // clamped
    }

    #[test]
    fn test_classification_display() {
        let c = AtomClassification::new()
            .with_sign(Sign::Aries, 0.9)
            .with_element(Element::Fire, 0.8)
            .with_modality(Modality::Cardinal, 0.7);
        let display = format!("{}", c);
        assert!(display.contains("♈"));
        assert!(display.contains("🔥"));
        assert!(display.contains("🅲"));
    }

    #[test]
    fn test_clamp_values() {
        let c = AtomClassification::new().with_sign(Sign::Aries, 1.5);
        assert!((c.signs[0] - 1.0).abs() < 1e-12);

        let c = AtomClassification::new().with_sign(Sign::Aries, -0.5);
        assert!((c.signs[0] - 0.0).abs() < 1e-12);
    }
}
