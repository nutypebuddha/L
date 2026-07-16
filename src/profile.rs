//! Alternative classifier profiles over the formula corpus.
//!
//! The default classifier is the Vedic Navagraha wheel (`Domain`). A `Profile`
//! is an orthogonal lens — a different cultural/structural taxonomy mapped
//! onto the same knowledge base, so the corpus can be viewed through, e.g., the
//! Chinese Five Elements (Wu Xing) or the four temperaments. This is the
//! "astrology is a *profile*, not the product" posture from the elevation
//! roadmap: the proof cascade is invariant; only the classifier changes.

use crate::domain_graph::Domain;

/// A classifier profile: an alternative mapping from a formula's primary graha
/// domain to a category in this profile's own taxonomy.
pub trait Profile {
    /// Stable identifier (e.g. `"wuxing"`).
    fn id(&self) -> &'static str;
    /// Human-readable description of the profile's taxonomy.
    fn description(&self) -> &'static str;
    /// Map a Vedic graha domain to this profile's category name.
    fn categorize(&self, domain: Domain) -> &'static str;
}

/// Wu Xing — the Chinese Five Elements (Wood, Fire, Earth, Metal, Water).
///
/// Maps each Navagraha to its traditional Five-Element correspondence. This is
/// a stub: the mapping is a reasonable correspondence, not a claimed identity.
pub struct WuXingProfile;

impl Profile for WuXingProfile {
    fn id(&self) -> &'static str {
        "wuxing"
    }
    fn description(&self) -> &'static str {
        "Wu Xing — the Five Elements: Wood, Fire, Earth, Metal, Water"
    }
    fn categorize(&self, domain: Domain) -> &'static str {
        match domain {
            Domain::Surya => "Fire",
            Domain::Chandra => "Water",
            Domain::Mangala => "Fire",
            Domain::Budha => "Water",
            Domain::Brihaspati => "Wood",
            Domain::Shukra => "Metal",
            Domain::Shani => "Earth",
            Domain::Rahu => "Metal",
            Domain::Ketu => "Fire",
        }
    }
}

/// Four temperaments — the medieval humoral typology (Sanguine, Choleric,
/// Melancholic, Phlegmatic), mapped from each graha's characteristic quality.
pub struct TemperamentProfile;

impl Profile for TemperamentProfile {
    fn id(&self) -> &'static str {
        "temperament"
    }
    fn description(&self) -> &'static str {
        "Four temperaments: Sanguine, Choleric, Melancholic, Phlegmatic"
    }
    fn categorize(&self, domain: Domain) -> &'static str {
        match domain {
            Domain::Surya => "Choleric",
            Domain::Chandra => "Phlegmatic",
            Domain::Mangala => "Choleric",
            Domain::Budha => "Sanguine",
            Domain::Brihaspati => "Sanguine",
            Domain::Shukra => "Phlegmatic",
            Domain::Shani => "Melancholic",
            Domain::Rahu => "Melancholic",
            Domain::Ketu => "Melancholic",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wuxing_covers_all_domains() {
        let p = WuXingProfile;
        for d in [
            Domain::Surya,
            Domain::Chandra,
            Domain::Mangala,
            Domain::Budha,
            Domain::Brihaspati,
            Domain::Shukra,
            Domain::Shani,
            Domain::Rahu,
            Domain::Ketu,
        ] {
            assert!(!p.categorize(d).is_empty());
        }
        assert_eq!(p.categorize(Domain::Surya), "Fire");
        assert_eq!(p.id(), "wuxing");
    }

    #[test]
    fn temperament_covers_all_domains() {
        let p = TemperamentProfile;
        assert_eq!(p.categorize(Domain::Shani), "Melancholic");
        assert_eq!(p.categorize(Domain::Budha), "Sanguine");
        assert_eq!(p.id(), "temperament");
    }
}
