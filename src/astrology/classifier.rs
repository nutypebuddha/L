use std::collections::HashMap;

use super::{AtomClassification, Element, Modality, PlanetaryRuler, Sign};

/// The Change Sorter — classifies tokens across all 7 astrology axes.
#[derive(Debug, Clone)]
pub struct ChangeSorter {
    sign_keywords: HashMap<String, Vec<(Sign, f64)>>,
    element_keywords: HashMap<String, Vec<(Element, f64)>>,
    modality_keywords: HashMap<String, Vec<(Modality, f64)>>,
    ruler_keywords: HashMap<String, Vec<(PlanetaryRuler, f64)>>,
}

impl Default for ChangeSorter {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeSorter {
    pub fn new() -> Self {
        let mut sorter = ChangeSorter {
            sign_keywords: HashMap::new(),
            element_keywords: HashMap::new(),
            modality_keywords: HashMap::new(),
            ruler_keywords: HashMap::new(),
        };
        sorter.init_default_mappings();
        sorter
    }

    fn init_default_mappings(&mut self) {
        use Element::*;
        use Modality::*;
        use PlanetaryRuler::*;
        use Sign::*;

        let sign_mappings: [(Sign, &str, f64); 108] = [
            (Aries, "add", 0.8),
            (Aries, "subtract", 0.8),
            (Aries, "multiply", 0.8),
            (Aries, "divide", 0.8),
            (Aries, "math", 0.9),
            (Aries, "logic", 0.8),
            (Aries, "number", 0.7),
            (Aries, "calculate", 0.7),
            (Aries, "prove", 0.6),
            (Aries, "theorem", 0.7),
            (Aries, "axiom", 0.6),
            (Taurus, "force", 0.8),
            (Taurus, "mass", 0.8),
            (Taurus, "energy", 0.8),
            (Taurus, "velocity", 0.7),
            (Taurus, "acceleration", 0.8),
            (Taurus, "physics", 0.9),
            (Taurus, "chemistry", 0.8),
            (Taurus, "atom", 0.6),
            (Taurus, "molecule", 0.6),
            (Taurus, "gravity", 0.7),
            (Taurus, "motion", 0.7),
            (Gemini, "star", 0.8),
            (Gemini, "planet", 0.8),
            (Gemini, "galaxy", 0.8),
            (Gemini, "cosmos", 0.8),
            (Gemini, "astronomy", 0.9),
            (Gemini, "orbit", 0.7),
            (Gemini, "space", 0.7),
            (Gemini, "universe", 0.7),
            (Gemini, "light", 0.6),
            (Cancer, "earth", 0.8),
            (Cancer, "water", 0.7),
            (Cancer, "air", 0.6),
            (Cancer, "climate", 0.8),
            (Cancer, "weather", 0.7),
            (Cancer, "ocean", 0.7),
            (Cancer, "environment", 0.8),
            (Cancer, "ecosystem", 0.7),
            (Cancer, "nature", 0.6),
            (Leo, "cell", 0.7),
            (Leo, "dna", 0.8),
            (Leo, "gene", 0.7),
            (Leo, "protein", 0.7),
            (Leo, "biology", 0.9),
            (Leo, "medicine", 0.8),
            (Leo, "organ", 0.6),
            (Leo, "tissue", 0.6),
            (Leo, "evolution", 0.7),
            (Virgo, "price", 0.7),
            (Virgo, "cost", 0.7),
            (Virgo, "market", 0.8),
            (Virgo, "economy", 0.9),
            (Virgo, "finance", 0.8),
            (Virgo, "money", 0.7),
            (Virgo, "trade", 0.7),
            (Virgo, "budget", 0.6),
            (Virgo, "tax", 0.6),
            (Libra, "engineer", 0.8),
            (Libra, "design", 0.7),
            (Libra, "machine", 0.7),
            (Libra, "circuit", 0.7),
            (Libra, "bridge", 0.6),
            (Libra, "structure", 0.7),
            (Libra, "technology", 0.8),
            (Libra, "system", 0.7),
            (Libra, "mechanics", 0.7),
            (Scorpio, "computer", 0.8),
            (Scorpio, "algorithm", 0.8),
            (Scorpio, "data", 0.7),
            (Scorpio, "code", 0.7),
            (Scorpio, "program", 0.7),
            (Scorpio, "software", 0.7),
            (Scorpio, "ai", 0.8),
            (Scorpio, "intelligence", 0.7),
            (Scorpio, "neural", 0.6),
            (Sagittarius, "history", 0.9),
            (Sagittarius, "culture", 0.7),
            (Sagittarius, "ancient", 0.7),
            (Sagittarius, "civilization", 0.7),
            (Sagittarius, "anthropology", 0.8),
            (Sagittarius, "society", 0.6),
            (Sagittarius, "tradition", 0.6),
            (Sagittarius, "origin", 0.6),
            (Capricorn, "word", 0.7),
            (Capricorn, "language", 0.9),
            (Capricorn, "grammar", 0.8),
            (Capricorn, "linguistics", 0.8),
            (Capricorn, "syntax", 0.7),
            (Capricorn, "semantics", 0.7),
            (Capricorn, "speech", 0.6),
            (Capricorn, "text", 0.6),
            (Aquarius, "truth", 0.7),
            (Aquarius, "ethics", 0.8),
            (Aquarius, "philosophy", 0.9),
            (Aquarius, "moral", 0.7),
            (Aquarius, "justice", 0.7),
            (Aquarius, "virtue", 0.6),
            (Aquarius, "reason", 0.7),
            (Aquarius, "knowledge", 0.7),
            (Pisces, "mind", 0.7),
            (Pisces, "brain", 0.8),
            (Pisces, "psychology", 0.9),
            (Pisces, "neuron", 0.7),
            (Pisces, "consciousness", 0.7),
            (Pisces, "emotion", 0.7),
            (Pisces, "behavior", 0.6),
            (Pisces, "cognition", 0.7),
        ];

        for (sign, keyword, weight) in &sign_mappings {
            self.sign_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*sign, *weight));
        }

        let element_mappings: [(Element, &str, f64); 16] = [
            (Fire, "action", 0.7),
            (Fire, "energy", 0.6),
            (Fire, "force", 0.6),
            (Fire, "heat", 0.7),
            (Earth, "structure", 0.7),
            (Earth, "solid", 0.6),
            (Earth, "material", 0.7),
            (Earth, "ground", 0.6),
            (Air, "thought", 0.6),
            (Air, "idea", 0.6),
            (Air, "communication", 0.7),
            (Air, "information", 0.6),
            (Water, "feeling", 0.7),
            (Water, "emotion", 0.7),
            (Water, "flow", 0.6),
            (Water, "intuition", 0.6),
        ];

        for (elem, keyword, weight) in &element_mappings {
            self.element_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*elem, *weight));
        }

        let modality_mappings: [(Modality, &str, f64); 12] = [
            (Cardinal, "initiate", 0.8),
            (Cardinal, "start", 0.7),
            (Cardinal, "lead", 0.7),
            (Cardinal, "begin", 0.6),
            (Fixed, "stable", 0.7),
            (Fixed, "endure", 0.7),
            (Fixed, "persist", 0.7),
            (Fixed, "steady", 0.6),
            (Mutable, "change", 0.7),
            (Mutable, "adapt", 0.7),
            (Mutable, "flexible", 0.7),
            (Mutable, "transform", 0.6),
        ];

        for (modality, keyword, weight) in &modality_mappings {
            self.modality_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*modality, *weight));
        }

        let ruler_mappings: [(PlanetaryRuler, &str, f64); 21] = [
            (Sun, "sun", 0.9),
            (Sun, "light", 0.6),
            (Sun, "shine", 0.5),
            (Moon, "moon", 0.9),
            (Moon, "lunar", 0.7),
            (Moon, "night", 0.5),
            (Mercury, "mercury", 0.9),
            (Mercury, "message", 0.5),
            (Mercury, "communicate", 0.6),
            (Venus, "venus", 0.9),
            (Venus, "beauty", 0.6),
            (Venus, "love", 0.5),
            (Mars, "mars", 0.9),
            (Mars, "war", 0.6),
            (Mars, "battle", 0.5),
            (Jupiter, "jupiter", 0.9),
            (Jupiter, "luck", 0.5),
            (Jupiter, "expand", 0.6),
            (Saturn, "saturn", 0.9),
            (Saturn, "time", 0.6),
            (Saturn, "structure", 0.6),
        ];

        for (ruler, keyword, weight) in &ruler_mappings {
            self.ruler_keywords
                .entry(keyword.to_string())
                .or_default()
                .push((*ruler, *weight));
        }
    }

    pub fn classify_token(&self, token: &str) -> AtomClassification {
        let lower = token.to_lowercase();
        let mut result = AtomClassification::new();

        if let Some(activations) = self.sign_keywords.get(&lower) {
            for &(sign, weight) in activations {
                result.signs[sign.index()] = result.signs[sign.index()].max(weight);
            }
        }

        if let Some(activations) = self.element_keywords.get(&lower) {
            for &(elem, weight) in activations {
                result.elements[elem.index()] = result.elements[elem.index()].max(weight);
            }
        }

        if let Some(activations) = self.modality_keywords.get(&lower) {
            for &(modality, weight) in activations {
                result.modalities[modality.index()] =
                    result.modalities[modality.index()].max(weight);
            }
        }

        if let Some(activations) = self.ruler_keywords.get(&lower) {
            for &(ruler, weight) in activations {
                result.rulers[ruler.index()] = result.rulers[ruler.index()].max(weight);
            }
        }

        result
    }

    pub fn classify_query(&self, query: &str) -> AtomClassification {
        let mut result = AtomClassification::new();
        let tokens: Vec<&str> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| !t.is_empty())
            .collect();

        for token in &tokens {
            let token_class = self.classify_token(token);
            result = result.merge_sum(&token_class);
        }

        result
    }

    pub fn is_empty(&self) -> bool {
        self.sign_keywords.is_empty()
            && self.element_keywords.is_empty()
            && self.modality_keywords.is_empty()
            && self.ruler_keywords.is_empty()
    }

    pub fn mapping_count(&self) -> usize {
        self.sign_keywords.len()
            + self.element_keywords.len()
            + self.modality_keywords.len()
            + self.ruler_keywords.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::astrology::Sign;

    #[test]
    fn test_classify_math_token() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("math");
        assert!(
            result.signs[Sign::Aries.index()] > 0.5,
            "math should activate Aries"
        );
    }

    #[test]
    fn test_classify_physics_token() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("force");
        assert!(
            result.signs[Sign::Taurus.index()] > 0.5,
            "force should activate Taurus"
        );
    }

    #[test]
    fn test_classify_query() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_query("calculate kinetic energy");
        assert!(result.signs.iter().any(|&v| v > 0.0));
    }

    #[test]
    fn test_unknown_token() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("xyznonexistent");
        assert_eq!(result, AtomClassification::new());
    }

    #[test]
    fn test_case_insensitive() {
        let sorter = ChangeSorter::new();
        let a = sorter.classify_token("MATH");
        let b = sorter.classify_token("math");
        assert_eq!(a, b);
    }

    #[test]
    fn test_is_empty_after_new() {
        let sorter = ChangeSorter::new();
        assert!(!sorter.is_empty());
    }

    #[test]
    fn test_element_classification() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("action");
        assert!(result.elements[Element::Fire.index()] > 0.5);
    }

    #[test]
    fn test_modality_classification() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("initiate");
        assert!(result.modalities[Modality::Cardinal.index()] > 0.5);
    }

    #[test]
    fn test_ruler_classification() {
        let sorter = ChangeSorter::new();
        let result = sorter.classify_token("mars");
        assert!(result.rulers[PlanetaryRuler::Mars.index()] > 0.5);
    }

    #[test]
    fn test_mapping_count() {
        let sorter = ChangeSorter::new();
        assert!(sorter.mapping_count() > 20);
    }
}
