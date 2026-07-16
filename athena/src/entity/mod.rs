//! # Entity — every token is an entity
//!
//! Every token the LLM generates becomes an entity. Entities are classified
//! by the Change Sorter across all 7 astrology axes at creation time, then
//! stored in the registry for the gyroscopic wheel to route.
//!
//! ## Two Sources of Entities
//!
//! 1. **Dynamic** (primary): Every token flowing through the pipeline becomes
//!    a `TokenEntity` — classified, timestamped, stored in a ring buffer.
//!
//! 2. **Static** (optional, seed): TOML files can seed known entities with
//!    pre-classified astrology data. These are loaded at startup but the
//!    system works without them.
//!
//! ## Lifecycle
//!
//! Token "force" enters pipeline
//!   -> TokenEntity::new("force", &change_sorter)
//!     -> AtomClassification computed
//!       -> Entity stored in TokenEntityRegistry
//!         -> Gyroscopic wheel reads entity's classification
//!           -> Sign determines which primitive formulas fire

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

pub use crate::astrology::{
    Aspect, AtomClassification, ChangeSorter, Element, Graha, Guna, Modality, Nakshatra,
    PlanetaryRuler, Sign, VedicClassification, VedicElement,
};

// ─── Core Runtime Entity ──────────────────────────────────────────────────

/// A runtime entity — a token flowing through the system.
///
/// Every entity has:
/// - A unique runtime ID (monotonic counter)
/// - The original text token
/// - A full 7-axis astrology classification (from Change Sorter)
/// - Optionally extracted numeric values (from adjacent tokens)
/// - A seed entity ID it was matched to (if any)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique runtime ID (e.g. "ent_000042")
    pub id: String,

    /// The original text token that created this entity
    pub text: String,

    /// Full 7-axis astrology classification from Change Sorter
    pub classification: AtomClassification,

    /// Seed entity ID this token matched to (if any from static registry)
    #[serde(default)]
    pub seed_id: Option<String>,

    /// Numeric values extracted from this token or adjacent tokens
    /// (e.g. "5kg" → {"value": 5.0, "unit": "kg"})
    #[serde(default)]
    pub values: HashMap<String, f64>,

    /// Logical truth value (for logic gate evaluation of token streams)
    /// Defaults to 1.0 (true). Set to 0.0 for negated tokens.
    #[serde(default)]
    pub truth: f64,

    /// Timestamp (monotonic sequence number)
    pub seq: u64,

    /// Tags inherited from seed entity or set dynamically
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Entity {
    /// Create a new runtime entity from a text token.
    ///
    /// This is the PRIMARY constructor — called for every token in the stream.
    pub fn new(text: &str, sorter: &ChangeSorter, seq: u64) -> Self {
        let classification = sorter.classify_token(text);
        Entity {
            id: format!("ent_{:06}", seq),
            text: text.to_string(),
            classification,
            seed_id: None,
            values: HashMap::new(),
            truth: 1.0,
            seq,
            tags: Vec::new(),
        }
    }

    /// Set the numeric values extracted from this token.
    pub fn with_values(mut self, values: HashMap<String, f64>) -> Self {
        self.values = values;
        self
    }

    /// Set the logical truth value.
    pub fn with_truth(mut self, truth: f64) -> Self {
        self.truth = truth.clamp(0.0, 1.0);
        self
    }

    /// Link this token to a seed entity from the static registry.
    pub fn with_seed(mut self, seed_id: &str) -> Self {
        self.seed_id = Some(seed_id.to_string());
        self
    }

    /// Add tags to this entity.
    pub fn with_tags(mut self, tags: Vec<&str>) -> Self {
        self.tags = tags.into_iter().map(String::from).collect();
        self
    }

    /// The dominant sign from the entity's astrology classification.
    pub fn dominant_sign(&self) -> Option<Sign> {
        self.classification.dominant_sign()
    }

    /// The dominant element from the entity's astrology classification.
    pub fn dominant_element(&self) -> Option<Element> {
        self.classification.dominant_element()
    }

    /// The dominant modality from the entity's astrology classification.
    pub fn dominant_modality(&self) -> Option<Modality> {
        self.classification.dominant_modality()
    }

    /// The dominant Vedic graha (planet) from the entity's classification.
    pub fn dominant_graha(&self) -> Option<Graha> {
        self.classification.vedic.dominant_graha()
    }

    /// The dominant Vedic guna (quality) from the entity's classification.
    pub fn dominant_guna(&self) -> Option<Guna> {
        self.classification.vedic.dominant_guna()
    }

    /// The dominant Vedic nakshatra (lunar mansion) from the entity's classification.
    pub fn dominant_nakshatra(&self) -> Option<Nakshatra> {
        self.classification.vedic.dominant_nakshatra()
    }

    /// The dominant Vedic element (tattva) from the entity's classification.
    pub fn dominant_vedic_element(&self) -> Option<VedicElement> {
        self.classification.vedic.dominant_vedic_element()
    }
}

// ─── Token Entity Registry ────────────────────────────────────────────────

/// A ring-buffer registry of runtime token entities.
///
/// The registry is the runtime store for all tokens flowing through the system.
/// It has a maximum capacity (ring buffer) and optionally loads seed entities
/// from static TOML files.
#[derive(Debug, Clone)]
pub struct EntityRegistry {
    /// Runtime entities by ID (ring buffer, newest first)
    entities: HashMap<String, Entity>,

    /// Ordered list of entity IDs (newest first) for ring buffer eviction
    order: VecDeque<String>,

    /// Seed entities loaded from TOML (optional, for grounding known concepts)
    seeds: HashMap<String, SeedEntity>,

    /// Maximum number of runtime entities before eviction
    max_entities: usize,

    /// Monotonic sequence counter for entity ID generation
    next_seq: u64,
}

/// A seed entity loaded from static TOML — an optional grounding set
/// of known concepts with pre-classified astrology data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeedEntity {
    /// Unique identifier (e.g. "mars", "dopamine", "kinetic_energy")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Description of this concept
    pub description: String,

    /// Astrology classification (pre-computed)
    #[serde(default)]
    pub classification: Option<AtomClassification>,

    /// Numeric properties for formula evaluation
    #[serde(default)]
    pub properties: HashMap<String, f64>,

    /// Grounded physical constants
    #[serde(default)]
    pub constants: HashMap<String, f64>,

    /// Tags for search
    #[serde(default)]
    pub tags: Vec<String>,

    /// The expression to compute this entity from inputs
    /// (optional — for compound entities like kinetic_energy = 0.5 * m * v^2)
    #[serde(default)]
    pub formula: Option<String>,

    /// K-12 level on the understanding axis
    #[serde(default)]
    pub level: u8,

    /// Spiral cycle
    #[serde(default)]
    pub cycle: u8,

    // ─── Vedic Classification (Human-Friendly) ────────────────────────
    //
    // These fields populate `VedicClassification` inside `AtomClassification`.
    // Accepts string names (e.g. "surya", "simha", "sattva", "fire") for
    // ergonomic TOML entry. Parsed at seed load time.
    /// Primary graha (Vedic planet) name. E.g. "surya", "mangala", "ketu".
    #[serde(default)]
    pub graha: Option<String>,

    /// Primary rashi (Vedic sign) name. E.g. "simha", "mesha", "kanya".
    #[serde(default)]
    pub rashi: Option<String>,

    /// Primary nakshatra (lunar mansion) name. E.g. "magha", "ashwini".
    #[serde(default)]
    pub nakshatra: Option<String>,

    /// Primary guna (quality) name. E.g. "sattva", "rajas", "tamas".
    #[serde(default)]
    pub guna: Option<String>,

    /// Primary tattva (Vedic element) name. E.g. "fire", "water", "ether".
    #[serde(default)]
    pub tattva: Option<String>,

    /// Purushartha (life goal). E.g. "dharma", "artha", "kama", "moksha".
    #[serde(default)]
    pub purushartha: Option<String>,

    /// Vedic day association. E.g. "sunday", "monday".
    #[serde(default)]
    pub day: Option<String>,

    /// Nakshatras ruled by this entity (for grahas).
    #[serde(default)]
    pub ruled_nakshatras: Vec<String>,

    /// Mantra text (Devanagari).
    #[serde(default)]
    pub mantra: Option<String>,

    /// Bija (seed mantra).
    #[serde(default)]
    pub bija: Option<String>,
}

impl SeedEntity {
    /// Build a `VedicClassification` from this seed entity's human-friendly fields.
    pub fn to_vedic_classification(&self) -> VedicClassification {
        let mut vc = VedicClassification::new();
        if let Some(ref g) = self.graha {
            if let Some(graha) = Graha::parse(g) {
                vc = vc.with_graha(graha, 0.9);
            }
        }
        if let Some(ref n) = self.nakshatra {
            if let Some(nak) = Nakshatra::parse(n) {
                vc = vc.with_nakshatra(nak, 0.8);
            }
        }
        if let Some(ref g) = self.guna {
            if let Some(guna) = Guna::parse(g) {
                vc = vc.with_guna(guna, 0.8);
            }
        }
        if let Some(ref t) = self.tattva {
            if let Some(elem) = VedicElement::parse(t) {
                vc = vc.with_vedic_element(elem, 0.8);
            }
        }
        vc.confidence = 0.85;
        vc
    }

    /// Get the dominant graha for this seed entity, if set.
    pub fn dominant_graha(&self) -> Option<Graha> {
        self.graha.as_ref().and_then(|g| Graha::parse(g))
    }

    /// Get the dominant guna for this seed entity, if set.
    pub fn dominant_guna(&self) -> Option<Guna> {
        self.guna.as_ref().and_then(|g| Guna::parse(g))
    }
}

impl Default for EntityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl EntityRegistry {
    /// Create a new empty registry with default capacity (10,000 entities).
    pub fn new() -> Self {
        EntityRegistry {
            entities: HashMap::new(),
            order: VecDeque::new(),
            seeds: HashMap::new(),
            max_entities: 10_000,
            next_seq: 0,
        }
    }

    /// Create a new registry with a custom maximum capacity.
    pub fn with_capacity(max: usize) -> Self {
        EntityRegistry {
            entities: HashMap::new(),
            order: VecDeque::new(),
            seeds: HashMap::new(),
            max_entities: max,
            next_seq: 0,
        }
    }

    /// Record a new token in the registry, creating a classified entity.
    ///
    /// This is the MAIN entry point — called for every token in the LLM stream.
    pub fn record_token(&mut self, text: &str, sorter: &ChangeSorter) -> &Entity {
        let seq = self.next_seq;
        self.next_seq += 1;

        // Check if this token matches a seed entity
        let seed_match = self.match_seed(text);

        let mut entity = Entity::new(text, sorter, seq);
        if let Some(seed_id) = &seed_match {
            entity = entity.with_seed(seed_id);
            // Copy tags and Vedic classification from seed entity
            if let Some(seed) = self.seeds.get(seed_id) {
                entity.tags = seed.tags.clone();
                // Merge seed's Vedic classification into the entity's classification
                let seed_vedic = seed.to_vedic_classification();
                entity.classification.vedic = entity.classification.vedic.merge_max(&seed_vedic);
            }
        }

        let id = entity.id.clone();
        self.order.push_front(id.clone());
        self.entities.insert(id.clone(), entity);

        // Evict oldest if over capacity
        while self.order.len() > self.max_entities && self.order.len() > 1 {
            if let Some(old_id) = self.order.pop_back() {
                self.entities.remove(&old_id);
            }
        }

        self.entities.get(&id).unwrap()
    }

    /// Record multiple tokens at once.
    pub fn record_tokens(&mut self, tokens: &[&str], sorter: &ChangeSorter) -> Vec<String> {
        tokens
            .iter()
            .map(|t| self.record_token(t, sorter).id.clone())
            .collect()
    }

    /// Get an entity by its runtime ID.
    pub fn get(&self, id: &str) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// Search runtime entities by text substring match.
    pub fn search(&self, keyword: &str) -> Vec<&Entity> {
        let kw_lower = keyword.to_lowercase();
        let mut results: Vec<&Entity> = self
            .entities
            .values()
            .filter(|e| {
                e.text.to_lowercase().contains(&kw_lower)
                    || e.tags.iter().any(|t| t.to_lowercase().contains(&kw_lower))
                    || e.seed_id
                        .as_ref()
                        .is_some_and(|s| s.to_lowercase().contains(&kw_lower))
            })
            .collect();
        results.sort_by_key(|e| &e.id);
        results
    }

    /// Find entities by dominant sign.
    pub fn by_sign(&self, sign: Sign) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.dominant_sign() == Some(sign))
            .collect()
    }

    /// Find entities by dominant element.
    pub fn by_element(&self, element: Element) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.dominant_element() == Some(element))
            .collect()
    }

    /// Find entities by dominant Vedic graha (planet).
    pub fn by_graha(&self, graha: Graha) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.dominant_graha() == Some(graha))
            .collect()
    }

    /// Find seed entities by dominant Vedic graha.
    pub fn seeds_by_graha(&self, graha: Graha) -> Vec<&SeedEntity> {
        self.seeds
            .values()
            .filter(|s| s.dominant_graha() == Some(graha))
            .collect()
    }

    /// Search seed entities by Vedic field (graha, guna, tattva, rashi name).
    pub fn search_seeds_vedic(&self, keyword: &str) -> Vec<&SeedEntity> {
        let kw = keyword.trim().to_lowercase();
        let graha_match = Graha::parse(&kw);
        let guna_match = Guna::parse(&kw);
        let mut results: Vec<&SeedEntity> = self
            .seeds
            .values()
            .filter(|s| {
                // Check string fields
                s.graha.as_ref().is_some_and(|g| g.to_lowercase().contains(&kw))
                    || s.rashi.as_ref().is_some_and(|r| r.to_lowercase().contains(&kw))
                    || s.guna.as_ref().is_some_and(|g| g.to_lowercase().contains(&kw))
                    || s.tattva.as_ref().is_some_and(|t| t.to_lowercase().contains(&kw))
                    || s.nakshatra.as_ref().is_some_and(|n| n.to_lowercase().contains(&kw))
                    || s.purushartha.as_ref().is_some_and(|p| p.to_lowercase().contains(&kw))
                    // Check enum matches
                    || graha_match.is_some_and(|g| s.dominant_graha() == Some(g))
                    || guna_match.is_some_and(|g| s.dominant_guna() == Some(g))
            })
            .collect();
        results.sort_by_key(|s| &s.id);
        results
    }

    /// Compute the aspect between two runtime entities (via their dominant signs).
    pub fn aspect_between(&self, id_a: &str, id_b: &str) -> Option<(Aspect, &Entity, &Entity)> {
        let a = self.entities.get(id_a)?;
        let b = self.entities.get(id_b)?;

        let sign_a = a.dominant_sign()?;
        let sign_b = b.dominant_sign()?;

        let aspect = Aspect::between_sign_indices(sign_a.index(), sign_b.index())?;
        Some((aspect, a, b))
    }

    /// Total runtime entities.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// List all runtime entity IDs (sorted).
    pub fn list(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.entities.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Get the next sequence number (for external tracking).
    pub fn next_seq(&self) -> u64 {
        self.next_seq
    }

    // ─── Seed Entity Management (Optional Static TOML) ──────────────────

    /// Register a seed entity from TOML data.
    pub fn register_seed(&mut self, mut seed: SeedEntity) {
        let id = seed.id.clone();
        if let Some(existing) = self.seeds.get(&id) {
            eprintln!(
                "Warning: duplicate seed entity id '{}' — merging tags/properties instead of overwriting",
                id
            );
            // Union the tag sets so the losing file's classification isn't
            // silently discarded (e.g. entities/libra.toml vs entities/shukra.toml
            // both declaring the same id with different tags).
            for tag in &existing.tags {
                if !seed.tags.contains(tag) {
                    seed.tags.push(tag.clone());
                }
            }
            // Properties/constants: keep the new file's values, but backfill
            // any keys the new file didn't set from the existing entry.
            for (k, v) in &existing.properties {
                seed.properties.entry(k.clone()).or_insert(*v);
            }
            for (k, v) in &existing.constants {
                seed.constants.entry(k.clone()).or_insert(*v);
            }
        }
        self.seeds.insert(id, seed);
    }

    /// Load seed entities from a TOML file.
    pub fn load_seeds_from_file(&mut self, path: &std::path::Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {}", path.display(), e))?;

        self.load_seeds_from_str(&content, Some(&path.display().to_string()))
    }

    /// Load seed entities from a TOML string (for embedded corpus).
    pub fn load_seeds_from_str(&mut self, toml_str: &str, source: Option<&str>) -> Result<(), String> {
        #[derive(Deserialize)]
        struct EntityFile {
            entity: Option<Vec<SeedEntity>>,
        }

        let parsed: EntityFile = toml::from_str(toml_str)
            .map_err(|e| format!("{}: TOML parse error: {}", source.unwrap_or("<str>"), e))?;

        if let Some(entities) = parsed.entity {
            for entity in entities {
                self.register_seed(entity);
            }
        }

        Ok(())
    }

    /// Find a seed entity by ID.
    pub fn get_seed(&self, id: &str) -> Option<&SeedEntity> {
        self.seeds.get(id)
    }

    /// Search seed entities by keyword.
    pub fn search_seeds(&self, keyword: &str) -> Vec<&SeedEntity> {
        let kw_lower = keyword.to_lowercase();
        let mut results: Vec<&SeedEntity> = self
            .seeds
            .values()
            .filter(|s| {
                s.id.to_lowercase().contains(&kw_lower)
                    || s.name.to_lowercase().contains(&kw_lower)
                    || s.description.to_lowercase().contains(&kw_lower)
                    || s.tags.iter().any(|t| t.to_lowercase().contains(&kw_lower))
            })
            .collect();
        results.sort_by_key(|s| &s.id);
        results
    }

    /// Match a token text against seed entities.
    /// Returns the ID of the best-matching seed, if any.
    fn match_seed(&self, text: &str) -> Option<String> {
        let lower = text.to_lowercase();
        // Direct match against seed ID or name
        for (id, seed) in &self.seeds {
            if id == &lower || seed.name.to_lowercase() == lower {
                return Some(id.clone());
            }
        }
        // Partial match: check if any seed's ID or name contains the token
        // or vice versa (the token contains a seed name)
        for (id, seed) in &self.seeds {
            let seed_lower = seed.name.to_lowercase();
            if lower.contains(&seed_lower) || seed_lower.contains(&lower) {
                return Some(id.clone());
            }
        }
        None
    }

    /// List all seed entity IDs.
    pub fn list_seeds(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.seeds.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Number of seed entities.
    pub fn seed_count(&self) -> usize {
        self.seeds.len()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_sorter() -> ChangeSorter {
        ChangeSorter::new()
    }

    #[test]
    fn test_create_token_entity() {
        let sorter = test_sorter();
        let entity = Entity::new("force", &sorter, 0);
        assert_eq!(entity.text, "force");
        assert_eq!(entity.id, "ent_000000");
        assert!(entity.dominant_sign().is_some());
    }

    #[test]
    fn test_entity_classified() {
        let sorter = test_sorter();
        let entity = Entity::new("math", &sorter, 1);
        // "math" should activate Aries (index 0)
        assert!(
            entity.classification.signs[0] > 0.5,
            "math should activate Aries"
        );
    }

    #[test]
    fn test_entity_with_values() {
        let sorter = test_sorter();
        let mut values = HashMap::new();
        values.insert("mass".to_string(), 5.0);
        let entity = Entity::new("5kg", &sorter, 2).with_values(values);
        assert!((entity.values.get("mass").unwrap() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_registry_record_token() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        let entity = reg.record_token("force", &sorter);
        assert_eq!(entity.text, "force");
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn test_registry_record_tokens() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        let ids = reg.record_tokens(&["calculate", "kinetic", "energy"], &sorter);
        assert_eq!(ids.len(), 3);
        assert_eq!(reg.len(), 3);
        // Verify entities are retrievable
        assert!(reg.get(&ids[0]).is_some());
        assert!(reg.get(&ids[1]).is_some());
        assert!(reg.get(&ids[2]).is_some());
    }

    #[test]
    fn test_registry_search() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("kinetic", &sorter);
        reg.record_token("energy", &sorter);
        reg.record_token("mass", &sorter);

        let results = reg.search("kinetic");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "kinetic");
    }

    #[test]
    fn test_registry_by_sign() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("math", &sorter); // → Aries
        reg.record_token("force", &sorter); // → Taurus
        reg.record_token("history", &sorter); // → Sagittarius

        let aries = reg.by_sign(Sign::Aries);
        assert!(!aries.is_empty(), "math should be in Aries");
    }

    #[test]
    fn test_registry_aspect_between() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("math", &sorter); // Aries
        reg.record_token("history", &sorter); // Sagittarius

        let ids: Vec<String> = reg.list().iter().map(|s| s.to_string()).collect();
        if ids.len() >= 2 {
            let result = reg.aspect_between(&ids[0], &ids[1]);
            assert!(result.is_some());
        }
    }

    #[test]
    fn test_registry_ring_buffer_eviction() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::with_capacity(5);
        for i in 0..10 {
            reg.record_token(&format!("token_{}", i), &sorter);
        }
        // Should have evicted the first 5
        assert_eq!(reg.len(), 5);
        assert!(reg.search("token_0").is_empty());
        assert!(!reg.search("token_9").is_empty());
    }

    #[test]
    fn test_seed_entity_registration() {
        let mut reg = EntityRegistry::new();
        let seed = SeedEntity {
            id: "mars".to_string(),
            name: "Mars".to_string(),
            description: "Fourth planet from the Sun".to_string(),
            properties: HashMap::from([("mass_kg".to_string(), 6.39e23)]),
            tags: vec!["planet".to_string(), "mars".to_string()],
            ..Default::default()
        };
        reg.register_seed(seed);
        assert_eq!(reg.seed_count(), 1);
        assert!(reg.get_seed("mars").is_some());
    }

    #[test]
    fn test_duplicate_seed_merges_tags_and_properties() {
        let mut reg = EntityRegistry::new();
        let first = SeedEntity {
            id: "lithium".to_string(),
            name: "Lithium".to_string(),
            description: "Mood stabilizer".to_string(),
            tags: vec!["psychiatry".to_string(), "element".to_string()],
            properties: HashMap::from([
                ("atomic_number".to_string(), 3.0),
                ("half_life_hours".to_string(), 24.0),
            ]),
            ..Default::default()
        };
        let second = SeedEntity {
            id: "lithium".to_string(),
            name: "Lithium".to_string(),
            description: "Alkali metal".to_string(),
            tags: vec!["chemistry".to_string(), "element".to_string()],
            properties: HashMap::from([("atomic_number".to_string(), 3.0)]),
            ..Default::default()
        };
        reg.register_seed(first);
        reg.register_seed(second);

        assert_eq!(reg.seed_count(), 1);
        let merged = reg.get_seed("lithium").unwrap();
        // Tags are unioned, without duplicating the shared "element" tag.
        assert!(merged.tags.contains(&"chemistry".to_string()));
        assert!(merged.tags.contains(&"psychiatry".to_string()));
        assert_eq!(
            merged.tags.iter().filter(|t| *t == "element").count(),
            1,
            "shared tag must not be duplicated"
        );
        // Properties missing from the second file are backfilled from the first.
        assert_eq!(merged.properties.get("half_life_hours"), Some(&24.0));
        assert_eq!(merged.properties.get("atomic_number"), Some(&3.0));
    }

    #[test]
    fn test_seed_match() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        let seed = SeedEntity {
            id: "kinetic_energy".to_string(),
            name: "Kinetic Energy".to_string(),
            description: "Energy of motion".to_string(),
            tags: vec!["physics".to_string(), "energy".to_string()],
            formula: Some("0.5 * m * v^2".to_string()),
            ..Default::default()
        };
        reg.register_seed(seed);

        // Token "kinetic energy" should match the seed
        let entity = reg.record_token("kinetic", &sorter);
        assert_eq!(entity.seed_id, Some("kinetic_energy".to_string()));
    }

    #[test]
    fn test_entity_dominant_element() {
        let sorter = test_sorter();
        let entity = Entity::new("force", &sorter, 0);
        // "force" activates Taurus (Earth) strongly
        let element = entity.dominant_element();
        assert!(element.is_some());
    }

    #[test]
    fn test_empty_registry() {
        let reg = EntityRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_registry_list_sorted() {
        let sorter = test_sorter();
        let mut reg = EntityRegistry::new();
        reg.record_token("zebra", &sorter);
        reg.record_token("apple", &sorter);
        let list = reg.list();
        assert_eq!(list.len(), 2);
        // IDs are "ent_000000", "ent_000001" — sorted lexicographically
        assert_eq!(list[0], "ent_000000");
        assert_eq!(list[1], "ent_000001");
    }
}
