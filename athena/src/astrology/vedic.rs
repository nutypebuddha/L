//! # Vedic Astrology (Jyotisha) — Grahas, Nakshatras, Gunas
//!
//! Vedic astrology adds three deeper dimensions beyond the Western zodiac:
//!
//! - **9 Grahas** (planets): 7 classical + Rahu (North Lunar Node) + Ketu (South Lunar Node)
//! - **27 Nakshatras** (lunar mansions): each 13°20′ of the ecliptic, the Moon's daily station
//! - **3 Gunas** (primordial qualities): Sattva (harmony), Rajas (passion), Tamas (inertia)
//! - **5th Element — Ether (Akasha)**: the void/substrate from which form arises
//!
//! These form the descent matrix axes. Every token in the descent pipeline
//! resolves through Graha → Nakshatra → Guna → Element → NAND.

use serde::{Deserialize, Serialize};
use std::fmt;

// ─── 9 Grahas (Planets) — Primary Wheel Nodes ───────────────────────────────

/// The 9 Vedic grahas (planets + lunar nodes) — primary domain nodes on the wheel.
///
/// Navagraha: 7 classical + Rahu (North Node) + Ketu (South Node).
/// Rahu and Ketu are shadow planets (chaya graha) — mathematical points
/// where the Moon's orbit intersects the ecliptic.
///
/// The grahas serve as Athena's primary domain classification, replacing
/// the Western 12-sign zodiac wheel. Each graha represents a fundamental
/// cognitive/knowledge domain.
///
/// # Serde
///
/// `rename_all = "lowercase"` so TOML/YAML/JSON deserialization uses
/// lowercase names: `"surya"`, `"chandra"`, `"mangala"`, etc.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Graha {
    /// ☉ Surya — Self, Leadership, Governance (Fire, Sattva)
    Surya = 0,
    /// ☽ Chandra — Mind, Emotion, Psychology (Water, Sattva)
    Chandra = 1,
    /// ♂ Mangala — Action, Energy, Engineering (Fire, Rajas)
    Mangala = 2,
    /// ☿ Budha — Intellect, Logic, Communication (Earth, Rajas)
    Budha = 3,
    /// ♃ Brihaspati — Wisdom, Philosophy, Law (Ether, Sattva)
    Brihaspati = 4,
    /// ♀ Shukra — Arts, Beauty, Value, Wealth (Water, Sattva)
    Shukra = 5,
    /// ♄ Shani — Structure, Discipline, Time, Karma (Air, Tamas)
    Shani = 6,
    /// ☊ Rahu — Innovation, Technology, Ambition (Air, Tamas)
    Rahu = 7,
    /// ☋ Ketu — Spirituality, Liberation, Deep Science (Ether, Tamas)
    Ketu = 8,
}

impl Graha {
    /// Number of grahas (9).
    pub const COUNT: usize = 9;

    /// Get the index of this graha on the wheel (0–8).
    #[inline]
    pub fn index(self) -> usize {
        self as usize
    }

    /// Get the graha at the given index (0–8).
    #[inline]
    pub fn from_index(i: usize) -> Self {
        match i % 9 {
            0 => Graha::Surya,
            1 => Graha::Chandra,
            2 => Graha::Mangala,
            3 => Graha::Budha,
            4 => Graha::Brihaspati,
            5 => Graha::Shukra,
            6 => Graha::Shani,
            7 => Graha::Rahu,
            8 => Graha::Ketu,
            _ => unreachable!(),
        }
    }

    /// Unicode symbol for this graha.
    pub fn symbol(self) -> &'static str {
        match self {
            Graha::Surya => "☉",
            Graha::Chandra => "☽",
            Graha::Mangala => "♂",
            Graha::Budha => "☿",
            Graha::Brihaspati => "♃",
            Graha::Shukra => "♀",
            Graha::Shani => "♄",
            Graha::Rahu => "☊",
            Graha::Ketu => "☋",
        }
    }

    /// Vedic name (title case).
    pub fn name(self) -> &'static str {
        match self {
            Graha::Surya => "Surya",
            Graha::Chandra => "Chandra",
            Graha::Mangala => "Mangala",
            Graha::Budha => "Budha",
            Graha::Brihaspati => "Brihaspati",
            Graha::Shukra => "Shukra",
            Graha::Shani => "Shani",
            Graha::Rahu => "Rahu",
            Graha::Ketu => "Ketu",
        }
    }

    /// Alias for `name()` — backward compatibility with old `Domain::full_name()`.
    #[inline]
    pub fn full_name(self) -> &'static str {
        self.name()
    }

    /// Alias for `name_lower()` — backward compatibility with old `Domain::full_name_lower()`.
    #[inline]
    pub fn full_name_lower(self) -> &'static str {
        self.name_lower()
    }

    /// Lowercased name — no allocation.
    pub fn name_lower(self) -> &'static str {
        match self {
            Graha::Surya => "surya",
            Graha::Chandra => "chandra",
            Graha::Mangala => "mangala",
            Graha::Budha => "budha",
            Graha::Brihaspati => "brihaspati",
            Graha::Shukra => "shukra",
            Graha::Shani => "shani",
            Graha::Rahu => "rahu",
            Graha::Ketu => "ketu",
        }
    }

    /// The knowledge domain this graha represents.
    pub fn knowledge_domain(self) -> &'static str {
        match self {
            Graha::Surya => "Self & Leadership",
            Graha::Chandra => "Mind & Emotion",
            Graha::Mangala => "Action & Engineering",
            Graha::Budha => "Logic & Communication",
            Graha::Brihaspati => "Wisdom & Law",
            Graha::Shukra => "Arts & Value",
            Graha::Shani => "Structure & Time",
            Graha::Rahu => "Innovation & Tech",
            Graha::Ketu => "Spirituality & Science",
        }
    }

    /// Pre-lowercased knowledge domain — no allocation.
    pub fn knowledge_domain_lower(self) -> &'static str {
        match self {
            Graha::Surya => "self & leadership",
            Graha::Chandra => "mind & emotion",
            Graha::Mangala => "action & engineering",
            Graha::Budha => "logic & communication",
            Graha::Brihaspati => "wisdom & law",
            Graha::Shukra => "arts & value",
            Graha::Shani => "structure & time",
            Graha::Rahu => "innovation & tech",
            Graha::Ketu => "spirituality & science",
        }
    }

    /// Archetypal function of this graha.
    pub fn archetype(self) -> &'static str {
        match self {
            Graha::Surya => "The Sovereign — identity, authority, vitality",
            Graha::Chandra => "The Reflector — mind, emotion, nurturing",
            Graha::Mangala => "The Warrior — action, energy, courage",
            Graha::Budha => "The Messenger — intellect, communication, commerce",
            Graha::Brihaspati => "The Guru — wisdom, expansion, dharma",
            Graha::Shukra => "The Sage — love, beauty, wealth, relationships",
            Graha::Shani => "The Ascetic — discipline, karma, limitation, time",
            Graha::Rahu => "The Seeker — innovation, ambition, technology",
            Graha::Ketu => "The Liberated — spirituality, detachment, deep science",
        }
    }

    /// Greek letter associated with this graha.
    ///
    /// Maps each of the 9 grahas to a Greek letter based on position and affinity:
    ///
    /// | Graha      | Greek Letter | Rationale                                |
    /// |------------|-------------|------------------------------------------|
    /// | Surya      | Alpha (α)   | The first, origin, the self              |
    /// | Chandra    | Beta (β)    | The second, duality, reflection          |
    /// | Mangala    | Gamma (γ)   | Energy, action, gamma radiation          |
    /// | Budha      | Delta (δ)   | Change, communication, delta of diff.    |
    /// | Brihaspati | Epsilon (ε) | The fifth, the expansive one             |
    /// | Shukra     | Zeta (ζ)    | The sixth, beauty and harmony            |
    /// | Shani      | Eta (η)     | Structure, the seventh, the gate         |
    /// | Rahu       | Theta (θ)   | Innovation, the eighth, theta waves      |
    /// | Ketu       | Iota (ι)    | The smallest point, liberation           |
    pub fn greek_name(self) -> &'static str {
        match self {
            Graha::Surya => "Alpha (α)",
            Graha::Chandra => "Beta (β)",
            Graha::Mangala => "Gamma (γ)",
            Graha::Budha => "Delta (δ)",
            Graha::Brihaspati => "Epsilon (ε)",
            Graha::Shukra => "Zeta (ζ)",
            Graha::Shani => "Eta (η)",
            Graha::Rahu => "Theta (θ)",
            Graha::Ketu => "Iota (ι)",
        }
    }

    /// Greek letter symbol only (lowercase).
    pub fn greek_symbol(self) -> &'static str {
        match self {
            Graha::Surya => "α",
            Graha::Chandra => "β",
            Graha::Mangala => "γ",
            Graha::Budha => "δ",
            Graha::Brihaspati => "ε",
            Graha::Shukra => "ζ",
            Graha::Shani => "η",
            Graha::Rahu => "θ",
            Graha::Ketu => "ι",
        }
    }

    /// Sanskrit name.
    pub fn sanskrit(self) -> &'static str {
        match self {
            Graha::Surya => "सूर्य",
            Graha::Chandra => "चन्द्र",
            Graha::Mangala => "मङ्गल",
            Graha::Budha => "बुध",
            Graha::Brihaspati => "बृहस्पति",
            Graha::Shukra => "शुक्र",
            Graha::Shani => "शनि",
            Graha::Rahu => "राहु",
            Graha::Ketu => "केतु",
        }
    }

    /// Element affinity (Vedic: each graha has a tattva/element).
    pub fn element_affinity(self) -> &'static str {
        match self {
            Graha::Surya => "Fire",
            Graha::Chandra => "Water",
            Graha::Mangala => "Fire",
            Graha::Budha => "Earth",
            Graha::Brihaspati => "Ether",
            Graha::Shukra => "Water",
            Graha::Shani => "Air",
            Graha::Rahu => "Air",
            Graha::Ketu => "Ether",
        }
    }

    /// Vedic element (tattva) enum for this graha.
    pub fn vedic_element(self) -> VedicElement {
        match self {
            Graha::Surya => VedicElement::Fire,
            Graha::Chandra => VedicElement::Water,
            Graha::Mangala => VedicElement::Fire,
            Graha::Budha => VedicElement::Earth,
            Graha::Brihaspati => VedicElement::Ether,
            Graha::Shukra => VedicElement::Water,
            Graha::Shani => VedicElement::Air,
            Graha::Rahu => VedicElement::Air,
            Graha::Ketu => VedicElement::Ether,
        }
    }

    /// Guna (quality) of this graha.
    pub fn guna(self) -> Guna {
        match self {
            Graha::Surya => Guna::Sattva,
            Graha::Chandra => Guna::Sattva,
            Graha::Mangala => Guna::Rajas,
            Graha::Budha => Guna::Rajas,
            Graha::Brihaspati => Guna::Sattva,
            Graha::Shukra => Guna::Sattva,
            Graha::Shani => Guna::Tamas,
            Graha::Rahu => Guna::Tamas,
            Graha::Ketu => Guna::Tamas,
        }
    }

    /// Get the graha at the position offset by `delta` steps on the 9-node wheel.
    #[inline]
    pub fn offset(self, delta: isize) -> Graha {
        let idx = (self.index() as isize + delta).rem_euclid(9) as usize;
        Graha::from_index(idx)
    }

    /// Get the opposite graha (4 steps away on a 9-node wheel).
    /// 9 is odd, so there's no exact opposite. This returns the graha
    /// 4 steps forward (closest to 180° on a 9-node wheel at 40° intervals:
    /// 4 × 40° = 160°, which is the Saptama/full aspect).
    #[inline]
    pub fn opposite(self) -> Graha {
        self.offset(4)
    }

    /// Get trine grahas (3 and 6 steps away = 120° and 240° on 9-node wheel).
    #[inline]
    pub fn trines(self) -> [Graha; 2] {
        [self.offset(3), self.offset(6)]
    }

    /// Get adjacent grahas (1 step forward and backward).
    #[inline]
    pub fn adjacent(self) -> [Graha; 2] {
        [self.offset(1), self.offset(-1)]
    }

    /// Convert from an astrology `Sign` to a `Graha` via planetary rulership.
    #[inline]
    pub fn from_sign(sign: crate::astrology::Sign) -> Graha {
        match sign {
            crate::astrology::Sign::Aries => Graha::Mangala,
            crate::astrology::Sign::Taurus => Graha::Shukra,
            crate::astrology::Sign::Gemini => Graha::Budha,
            crate::astrology::Sign::Cancer => Graha::Chandra,
            crate::astrology::Sign::Leo => Graha::Surya,
            crate::astrology::Sign::Virgo => Graha::Budha,
            crate::astrology::Sign::Libra => Graha::Shukra,
            crate::astrology::Sign::Scorpio => Graha::Mangala,
            crate::astrology::Sign::Sagittarius => Graha::Brihaspati,
            crate::astrology::Sign::Capricorn => Graha::Shani,
            crate::astrology::Sign::Aquarius => Graha::Shani,
            crate::astrology::Sign::Pisces => Graha::Brihaspati,
        }
    }

    /// All 9 grahas in wheel order (Surya → Ketu at 40° intervals).
    pub fn all() -> [Graha; 9] {
        [
            Graha::Surya,
            Graha::Chandra,
            Graha::Mangala,
            Graha::Budha,
            Graha::Brihaspati,
            Graha::Shukra,
            Graha::Shani,
            Graha::Rahu,
            Graha::Ketu,
        ]
    }

    /// Parse a graha from a string (case-insensitive).
    /// Matches full names, symbols, and common aliases.
    pub fn parse(s: &str) -> Option<Graha> {
        let s = s.trim();
        for &(graha, aliases) in GRAHA_ALIASES {
            if aliases.iter().any(|&alias| alias.eq_ignore_ascii_case(s)) {
                return Some(graha);
            }
        }
        None
    }
}

/// Static table of graha aliases for allocation-free parsing.
/// Includes Western zodiac sign names, Vedic rashi names, and domain keywords
/// for backward compatibility with existing TOML data files.
const GRAHA_ALIASES: &[(Graha, &[&str])] = &[
    (
        Graha::Surya,
        &[
            "surya",
            "sun",
            "sovereign",
            "self",
            "leadership",
            "☉",
            "leo",
            "simha",
            "alpha",
            "α",
        ],
    ),
    (
        Graha::Chandra,
        &[
            "chandra",
            "moon",
            "mind",
            "emotion",
            "psychology",
            "☽",
            "cancer",
            "karka",
            "beta",
            "β",
        ],
    ),
    (
        Graha::Mangala,
        &[
            "mangala",
            "mars",
            "action",
            "energy",
            "engineering",
            "warrior",
            "♂",
            "aries",
            "mesha",
            "scorpio",
            "vrishchika",
            "gamma",
            "γ",
        ],
    ),
    (
        Graha::Budha,
        &[
            "budha",
            "mercury",
            "logic",
            "intellect",
            "communication",
            "messenger",
            "☿",
            "gemini",
            "mithuna",
            "virgo",
            "kanya",
            "delta",
            "δ",
        ],
    ),
    (
        Graha::Brihaspati,
        &[
            "brihaspati",
            "jupiter",
            "wisdom",
            "philosophy",
            "law",
            "guru",
            "♃",
            "sagittarius",
            "dhanu",
            "pisces",
            "meena",
            "epsilon",
            "ε",
        ],
    ),
    (
        Graha::Shukra,
        &[
            "shukra",
            "venus",
            "arts",
            "beauty",
            "value",
            "wealth",
            "sage",
            "♀",
            "taurus",
            "vrishabha",
            "libra",
            "tula",
            "zeta",
            "ζ",
        ],
    ),
    (
        Graha::Shani,
        &[
            "shani",
            "saturn",
            "structure",
            "discipline",
            "time",
            "karma",
            "ascetic",
            "♄",
            "capricorn",
            "makara",
            "aquarius",
            "kumbha",
            "eta",
            "η",
        ],
    ),
    (
        Graha::Rahu,
        &[
            "rahu",
            "north node",
            "innovation",
            "technology",
            "ambition",
            "seeker",
            "☊",
            "theta",
            "θ",
        ],
    ),
    (
        Graha::Ketu,
        &[
            "ketu",
            "south node",
            "spirituality",
            "liberation",
            "science",
            "detachment",
            "liberated",
            "☋",
            "iota",
            "ι",
        ],
    ),
];

impl fmt::Display for Graha {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

impl std::str::FromStr for Graha {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Graha::parse(s).ok_or_else(|| format!("unknown graha: {s}"))
    }
}

/// All 9 grahas in wheel order.
pub const ALL_GRAHAS: [Graha; 9] = [
    Graha::Surya,
    Graha::Chandra,
    Graha::Mangala,
    Graha::Budha,
    Graha::Brihaspati,
    Graha::Shukra,
    Graha::Shani,
    Graha::Rahu,
    Graha::Ketu,
];

// ─── 12 Rashis (Vedic Signs) ────────────────────────────────────────────────

/// The 12 Vedic rashis (signs) — sidereal zodiac divisions of 30° each.
///
/// Unlike the Western zodiac which is tropical (fixed to equinoxes),
/// Vedic rashis are sidereal (fixed to constellations). The names
/// and symbols overlap with Western but the underlying astronomy differs.
///
/// In the wheel system, rashis provide a secondary classification axis
/// while grahas are the primary 9 domain nodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rashi {
    Mesha = 0,      // ♈ Aries — Mangala rules
    Vrishabha = 1,  // ♉ Taurus — Shukra rules
    Mithuna = 2,    // ♊ Gemini — Budha rules
    Karka = 3,      // ♋ Cancer — Chandra rules
    Simha = 4,      // ♌ Leo — Surya rules
    Kanya = 5,      // ♍ Virgo — Budha rules
    Tula = 6,       // ♎ Libra — Shukra rules
    Vrishchika = 7, // ♏ Scorpio — Mangala rules
    Dhanu = 8,      // ♐ Sagittarius — Brihaspati rules
    Makara = 9,     // ♑ Capricorn — Shani rules
    Kumbha = 10,    // ♒ Aquarius — Shani rules
    Meena = 11,     // ♓ Pisces — Brihaspati rules
}

impl Rashi {
    pub const COUNT: usize = 12;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i % 12 {
            0 => Rashi::Mesha,
            1 => Rashi::Vrishabha,
            2 => Rashi::Mithuna,
            3 => Rashi::Karka,
            4 => Rashi::Simha,
            5 => Rashi::Kanya,
            6 => Rashi::Tula,
            7 => Rashi::Vrishchika,
            8 => Rashi::Dhanu,
            9 => Rashi::Makara,
            10 => Rashi::Kumbha,
            11 => Rashi::Meena,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Rashi::Mesha => "♈",
            Rashi::Vrishabha => "♉",
            Rashi::Mithuna => "♊",
            Rashi::Karka => "♋",
            Rashi::Simha => "♌",
            Rashi::Kanya => "♍",
            Rashi::Tula => "♎",
            Rashi::Vrishchika => "♏",
            Rashi::Dhanu => "♐",
            Rashi::Makara => "♑",
            Rashi::Kumbha => "♒",
            Rashi::Meena => "♓",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Rashi::Mesha => "Mesha",
            Rashi::Vrishabha => "Vrishabha",
            Rashi::Mithuna => "Mithuna",
            Rashi::Karka => "Karka",
            Rashi::Simha => "Simha",
            Rashi::Kanya => "Kanya",
            Rashi::Tula => "Tula",
            Rashi::Vrishchika => "Vrishchika",
            Rashi::Dhanu => "Dhanu",
            Rashi::Makara => "Makara",
            Rashi::Kumbha => "Kumbha",
            Rashi::Meena => "Meena",
        }
    }

    pub fn sanskrit(self) -> &'static str {
        match self {
            Rashi::Mesha => "मेष",
            Rashi::Vrishabha => "वृषभ",
            Rashi::Mithuna => "मिथुन",
            Rashi::Karka => "कर्क",
            Rashi::Simha => "सिंह",
            Rashi::Kanya => "कन्या",
            Rashi::Tula => "तुला",
            Rashi::Vrishchika => "वृश्चिक",
            Rashi::Dhanu => "धनु",
            Rashi::Makara => "मकर",
            Rashi::Kumbha => "कुम्भ",
            Rashi::Meena => "मीन",
        }
    }

    /// Ruling graha for this rashi.
    pub fn lord(self) -> Graha {
        match self {
            Rashi::Mesha => Graha::Mangala,
            Rashi::Vrishabha => Graha::Shukra,
            Rashi::Mithuna => Graha::Budha,
            Rashi::Karka => Graha::Chandra,
            Rashi::Simha => Graha::Surya,
            Rashi::Kanya => Graha::Budha,
            Rashi::Tula => Graha::Shukra,
            Rashi::Vrishchika => Graha::Mangala,
            Rashi::Dhanu => Graha::Brihaspati,
            Rashi::Makara => Graha::Shani,
            Rashi::Kumbha => Graha::Shani,
            Rashi::Meena => Graha::Brihaspati,
        }
    }

    /// Tattva (element) for this rashi.
    pub fn tattva(self) -> VedicElement {
        match self {
            Rashi::Mesha => VedicElement::Fire,
            Rashi::Vrishabha => VedicElement::Earth,
            Rashi::Mithuna => VedicElement::Air,
            Rashi::Karka => VedicElement::Water,
            Rashi::Simha => VedicElement::Fire,
            Rashi::Kanya => VedicElement::Earth,
            Rashi::Tula => VedicElement::Air,
            Rashi::Vrishchika => VedicElement::Water,
            Rashi::Dhanu => VedicElement::Fire,
            Rashi::Makara => VedicElement::Earth,
            Rashi::Kumbha => VedicElement::Air,
            Rashi::Meena => VedicElement::Water,
        }
    }

    /// Guna for this rashi.
    pub fn guna(self) -> Guna {
        match self {
            Rashi::Mesha => Guna::Rajas,
            Rashi::Vrishabha => Guna::Tamas,
            Rashi::Mithuna => Guna::Rajas,
            Rashi::Karka => Guna::Sattva,
            Rashi::Simha => Guna::Sattva,
            Rashi::Kanya => Guna::Tamas,
            Rashi::Tula => Guna::Rajas,
            Rashi::Vrishchika => Guna::Tamas,
            Rashi::Dhanu => Guna::Sattva,
            Rashi::Makara => Guna::Tamas,
            Rashi::Kumbha => Guna::Sattva,
            Rashi::Meena => Guna::Sattva,
        }
    }

    /// Purushartha (life goal) for this rashi.
    pub fn purushartha(self) -> &'static str {
        match self {
            Rashi::Mesha => "Artha",
            Rashi::Vrishabha => "Kama",
            Rashi::Mithuna => "Artha",
            Rashi::Karka => "Kama",
            Rashi::Simha => "Dharma",
            Rashi::Kanya => "Moksha",
            Rashi::Tula => "Kama",
            Rashi::Vrishchika => "Moksha",
            Rashi::Dhanu => "Dharma",
            Rashi::Makara => "Artha",
            Rashi::Kumbha => "Moksha",
            Rashi::Meena => "Dharma",
        }
    }

    pub fn all() -> [Rashi; 12] {
        [
            Rashi::Mesha,
            Rashi::Vrishabha,
            Rashi::Mithuna,
            Rashi::Karka,
            Rashi::Simha,
            Rashi::Kanya,
            Rashi::Tula,
            Rashi::Vrishchika,
            Rashi::Dhanu,
            Rashi::Makara,
            Rashi::Kumbha,
            Rashi::Meena,
        ]
    }
}

impl fmt::Display for Rashi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

/// All 12 rashis in wheel order.
pub const ALL_RASHIS: [Rashi; 12] = [
    Rashi::Mesha,
    Rashi::Vrishabha,
    Rashi::Mithuna,
    Rashi::Karka,
    Rashi::Simha,
    Rashi::Kanya,
    Rashi::Tula,
    Rashi::Vrishchika,
    Rashi::Dhanu,
    Rashi::Makara,
    Rashi::Kumbha,
    Rashi::Meena,
];

// ─── 27 Nakshatras (Lunar Mansions) ─────────────────────────────────────────

/// The 27 Vedic nakshatras — lunar mansions of 13°20′ each.
///
/// Each nakshatra has a ruling graha, a symbol, and a shakti (power).
/// In the descent system, nakshatras provide the finest-grained
/// classification of a token's "lunar resonance" — where it sits
/// in the 360° cycle of consciousness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nakshatra {
    Ashwini = 0,           // Star: Horse — speed, healing, initiation
    Bharani = 1,           // Star: Yoni — birth, endurance, restraint
    Krittika = 2,          // Star: Razor — cutting, purification, courage
    Rohini = 3,            // Star: Cart — growth, beauty, creation
    Mrigashira = 4,        // Star: Deer — searching, seeking, restlessness
    Ardra = 5,             // Star: Teardrop — storm, transformation, suffering
    Punarvasu = 6,         // Star: Bow — return, renewal, restoration
    Pushya = 7,            // Star: Flower — nourishment, protection, growth
    Ashlesha = 8,          // Star: Serpent — entanglement, healing, depth
    Magha = 9,             // Star: Throne — ancestry, authority, lineage
    PurvaPhalguni = 10,    // Star: Couch — love, enjoyment, creativity
    UttaraPhalguni = 11,   // Star: Bed — union, marriage, stability
    Hasta = 12,            // Star: Hand — skill, dexterity, completion
    Chitra = 13,           // Star: Pearl — beauty, art, weaving
    Svati = 14,            // Star: Coral — independence, flexibility, wind
    Vishakha = 15,         // Star: Archway — purpose, achievement, radiance
    Anuradha = 16,         // Star: Lotus — devotion, friendship, loyalty
    Jyeshtha = 17,         // Star: Earring — protection, courage, seniority
    Mula = 18,             // Star: Root — destruction, depth, foundation
    PurvaAshadha = 19,     // Star: Fan — victory, purification, invigoration
    UttaraAshadha = 20,    // Star: Tusk — permanence, conquest, determination
    Shravana = 21,         // Star: Ear — listening, learning, communication
    Dhanishtha = 22,       // Star: Drum — rhythm, prosperity, fame
    Shatabhisha = 23,      // Star: Circle — healing, secrecy, veiling
    PurvaBhadrapada = 24,  // Star: Sword — transformation, burning, spirituality
    UttaraBhadrapada = 25, // Star: Twins — purification, water, depth
    Revati = 26,           // Star: Fish — nourishment, journey, completion
}

impl Nakshatra {
    pub const COUNT: usize = 27;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i % 27 {
            0 => Nakshatra::Ashwini,
            1 => Nakshatra::Bharani,
            2 => Nakshatra::Krittika,
            3 => Nakshatra::Rohini,
            4 => Nakshatra::Mrigashira,
            5 => Nakshatra::Ardra,
            6 => Nakshatra::Punarvasu,
            7 => Nakshatra::Pushya,
            8 => Nakshatra::Ashlesha,
            9 => Nakshatra::Magha,
            10 => Nakshatra::PurvaPhalguni,
            11 => Nakshatra::UttaraPhalguni,
            12 => Nakshatra::Hasta,
            13 => Nakshatra::Chitra,
            14 => Nakshatra::Svati,
            15 => Nakshatra::Vishakha,
            16 => Nakshatra::Anuradha,
            17 => Nakshatra::Jyeshtha,
            18 => Nakshatra::Mula,
            19 => Nakshatra::PurvaAshadha,
            20 => Nakshatra::UttaraAshadha,
            21 => Nakshatra::Shravana,
            22 => Nakshatra::Dhanishtha,
            23 => Nakshatra::Shatabhisha,
            24 => Nakshatra::PurvaBhadrapada,
            25 => Nakshatra::UttaraBhadrapada,
            26 => Nakshatra::Revati,
            _ => unreachable!(),
        }
    }

    /// Display name (Sanskrit, spaced for the two-word nakshatras).
    pub fn name(self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "Ashwini",
            Nakshatra::Bharani => "Bharani",
            Nakshatra::Krittika => "Krittika",
            Nakshatra::Rohini => "Rohini",
            Nakshatra::Mrigashira => "Mrigashira",
            Nakshatra::Ardra => "Ardra",
            Nakshatra::Punarvasu => "Punarvasu",
            Nakshatra::Pushya => "Pushya",
            Nakshatra::Ashlesha => "Ashlesha",
            Nakshatra::Magha => "Magha",
            Nakshatra::PurvaPhalguni => "Purva Phalguni",
            Nakshatra::UttaraPhalguni => "Uttara Phalguni",
            Nakshatra::Hasta => "Hasta",
            Nakshatra::Chitra => "Chitra",
            Nakshatra::Svati => "Svati",
            Nakshatra::Vishakha => "Vishakha",
            Nakshatra::Anuradha => "Anuradha",
            Nakshatra::Jyeshtha => "Jyeshtha",
            Nakshatra::Mula => "Mula",
            Nakshatra::PurvaAshadha => "Purva Ashadha",
            Nakshatra::UttaraAshadha => "Uttara Ashadha",
            Nakshatra::Shravana => "Shravana",
            Nakshatra::Dhanishtha => "Dhanishtha",
            Nakshatra::Shatabhisha => "Shatabhisha",
            Nakshatra::PurvaBhadrapada => "Purva Bhadrapada",
            Nakshatra::UttaraBhadrapada => "Uttara Bhadrapada",
            Nakshatra::Revati => "Revati",
        }
    }

    /// Ruling graha for this nakshatra (Vedic: each nakshatra is ruled by one graha).
    pub fn ruler(self) -> Graha {
        // Vedic order: Ketu, Shukra, Surya, Chandra, Mangala, Rahu, Brihaspati, Shani, Budha
        // repeating across 27 nakshatras
        const RULER_CYCLE: [Graha; 9] = [
            Graha::Ketu,
            Graha::Shukra,
            Graha::Surya,
            Graha::Chandra,
            Graha::Mangala,
            Graha::Rahu,
            Graha::Brihaspati,
            Graha::Shani,
            Graha::Budha,
        ];
        RULER_CYCLE[self.index() % 9]
    }

    /// Shakti (power/action) of this nakshatra.
    pub fn shakti(self) -> &'static str {
        match self {
            Nakshatra::Ashwini => "Speed — to heal, to bring horses",
            Nakshatra::Bharani => "To bear — endurance through restriction",
            Nakshatra::Krittika => "To cut — purification through fire",
            Nakshatra::Rohini => "To grow — creation through form",
            Nakshatra::Mrigashira => "To seek — searching through curiosity",
            Nakshatra::Ardra => "To storm — transformation through tears",
            Nakshatra::Punarvasu => "To renew — return after dissolution",
            Nakshatra::Pushya => "To nourish — growth through care",
            Nakshatra::Ashlesha => "To entangle — healing through embrace",
            Nakshatra::Magha => "To honor — lineage through authority",
            Nakshatra::PurvaPhalguni => "To enjoy — creativity through love",
            Nakshatra::UttaraPhalguni => "To unite — stability through marriage",
            Nakshatra::Hasta => "To complete — skill through dexterity",
            Nakshatra::Chitra => "To weave — beauty through art",
            Nakshatra::Svati => "To float — independence through wind",
            Nakshatra::Vishakha => "To achieve — purpose through radiance",
            Nakshatra::Anuradha => "To devote — loyalty through friendship",
            Nakshatra::Jyeshtha => "To protect — courage through seniority",
            Nakshatra::Mula => "To root — depth through destruction",
            Nakshatra::PurvaAshadha => "To purify — victory through invigoration",
            Nakshatra::UttaraAshadha => "To conquer — permanence through determination",
            Nakshatra::Shravana => "To hear — wisdom through listening",
            Nakshatra::Dhanishtha => "To prosper — fame through rhythm",
            Nakshatra::Shatabhisha => "To heal — secrecy through veiling",
            Nakshatra::PurvaBhadrapada => "To transform — spirituality through fire",
            Nakshatra::UttaraBhadrapada => "To purify — depth through water",
            Nakshatra::Revati => "To journey — completion through nourishment",
        }
    }

    pub fn all() -> [Nakshatra; 27] {
        use Nakshatra::*;
        [
            Ashwini,
            Bharani,
            Krittika,
            Rohini,
            Mrigashira,
            Ardra,
            Punarvasu,
            Pushya,
            Ashlesha,
            Magha,
            PurvaPhalguni,
            UttaraPhalguni,
            Hasta,
            Chitra,
            Svati,
            Vishakha,
            Anuradha,
            Jyeshtha,
            Mula,
            PurvaAshadha,
            UttaraAshadha,
            Shravana,
            Dhanishtha,
            Shatabhisha,
            PurvaBhadrapada,
            UttaraBhadrapada,
            Revati,
        ]
    }

    /// Parse a nakshatra from a lowercase string name.
    /// Matches both English and Sanskrit-derived names.
    pub fn parse(s: &str) -> Option<Nakshatra> {
        use Nakshatra::*;
        let lower = s.trim().to_lowercase();
        // NB: cannot use as_str() for all-arms match due to sheer count;
        // use chained if-else instead
        let words: Vec<&str> = lower.split_whitespace().collect();
        let compact: String = words.join("_");
        match compact.as_str() {
            "ashwini" | "ashvini" => Some(Ashwini),
            "bharani" => Some(Bharani),
            "krittika" | "kritika" => Some(Krittika),
            "rohini" => Some(Rohini),
            "mrigashira" | "mrigashirsha" | "mrigasira" => Some(Mrigashira),
            "ardra" | "ardaa" => Some(Ardra),
            "punarvasu" | "punarbasu" => Some(Punarvasu),
            "pushya" | "pusya" => Some(Pushya),
            "ashlesha" | "aslesha" => Some(Ashlesha),
            "magha" => Some(Magha),
            "purva_phalguni" | "purvaphalguni" => Some(PurvaPhalguni),
            "uttara_phalguni" | "uttaraphalguni" => Some(UttaraPhalguni),
            "hasta" => Some(Hasta),
            "chitra" | "chitraa" | "citra" => Some(Chitra),
            "svati" | "swati" | "svaati" => Some(Svati),
            "vishakha" | "visakha" | "vishaakha" => Some(Vishakha),
            "anuradha" | "anuraadha" => Some(Anuradha),
            "jyeshtha" | "jyestha" | "jyeshthaa" => Some(Jyeshtha),
            "mula" | "moola" => Some(Mula),
            "purva_ashadha" | "purvashadha" | "purva_asadha" => Some(PurvaAshadha),
            "uttara_ashadha" | "uttarashadha" | "uttara_asadha" => Some(UttaraAshadha),
            "shravana" | "sravana" => Some(Shravana),
            "dhanishtha" | "dhanistha" | "dhanishta" => Some(Dhanishtha),
            "shatabhisha" | "satabhisha" | "satabhishek" => Some(Shatabhisha),
            "purva_bhadrapada" | "purvabhadrapada" => Some(PurvaBhadrapada),
            "uttara_bhadrapada" | "uttarabhadrapada" => Some(UttaraBhadrapada),
            "revati" => Some(Revati),
            _ => None,
        }
    }
}

// ─── 3 Gunas (Primordial Qualities) ─────────────────────────────────────────

/// The 3 Vedic gunas — primordial qualities of all manifest existence.
///
/// Every token in the descent pipeline has a guna balance that determines
/// its fundamental nature:
///
/// - Sattva: harmony, clarity, balance — rises, illuminates
/// - Rajas: passion, activity, movement — expands, drives
/// - Tamas: inertia, darkness, stability — descends, congeals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Guna {
    Sattva = 0, // Harmony — pure, luminous, balanced
    Rajas = 1,  // Passion — active, turbulent, creative
    Tamas = 2,  // Inertia — dark, heavy, stable
}

impl Guna {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i % 3 {
            0 => Guna::Sattva,
            1 => Guna::Rajas,
            2 => Guna::Tamas,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Guna::Sattva => "◯",
            Guna::Rajas => "△",
            Guna::Tamas => "□",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Guna::Sattva => "Sattva",
            Guna::Rajas => "Rajas",
            Guna::Tamas => "Tamas",
        }
    }

    /// Quality description.
    pub fn quality(self) -> &'static str {
        match self {
            Guna::Sattva => "Harmony — clarity, balance, light",
            Guna::Rajas => "Passion — activity, movement, creativity",
            Guna::Tamas => "Inertia — stability, darkness, form",
        }
    }

    /// Direction of movement on the descent axis.
    /// Sattva rises (up), Rajas expands (outward), Tamas descends (down).
    pub fn direction(self) -> &'static str {
        match self {
            Guna::Sattva => "↑ Rise",
            Guna::Rajas => "→ Expand",
            Guna::Tamas => "↓ Descend",
        }
    }

    pub fn all() -> [Guna; 3] {
        [Guna::Sattva, Guna::Rajas, Guna::Tamas]
    }

    /// Parse a guna from a lowercase string.
    /// Accepts: sattva, rajas, tamas.
    pub fn parse(s: &str) -> Option<Guna> {
        match s.trim().to_lowercase().as_str() {
            "sattva" => Some(Guna::Sattva),
            "rajas" => Some(Guna::Rajas),
            "tamas" => Some(Guna::Tamas),
            _ => None,
        }
    }
}

// ─── Vedic Element (5th Element: Ether/Akasha) ─────────────────────────────

/// The Vedic pancha mahabhuta — 5 great elements including Ether (Akasha).
///
/// Ether is the substrate from which the other 4 elements arise.
/// In the descent system, Ether represents pure potential — the void
/// at the top of the descent before any resolution has occurred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VedicElement {
    Fire = 0,
    Earth = 1,
    Air = 2,
    Water = 3,
    Ether = 4, // Akasha — void, substrate, potential
}

impl VedicElement {
    pub const COUNT: usize = 5;

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Self {
        match i % 5 {
            0 => VedicElement::Fire,
            1 => VedicElement::Earth,
            2 => VedicElement::Air,
            3 => VedicElement::Water,
            4 => VedicElement::Ether,
            _ => unreachable!(),
        }
    }

    pub fn symbol(self) -> &'static str {
        match self {
            VedicElement::Fire => "🔥",
            VedicElement::Earth => "🜁",
            VedicElement::Air => "🜄",
            VedicElement::Water => "🜂",
            VedicElement::Ether => "◌", // void circle
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            VedicElement::Fire => "Fire",
            VedicElement::Earth => "Earth",
            VedicElement::Air => "Air",
            VedicElement::Water => "Water",
            VedicElement::Ether => "Ether",
        }
    }

    /// Sanskrit name.
    pub fn sanskrit(self) -> &'static str {
        match self {
            VedicElement::Fire => "Tejas",
            VedicElement::Earth => "Prithvi",
            VedicElement::Air => "Vayu",
            VedicElement::Water => "Apas",
            VedicElement::Ether => "Akasha",
        }
    }

    /// The guna associated with this element in Vedic philosophy.
    pub fn guna(self) -> Guna {
        match self {
            VedicElement::Ether => Guna::Sattva, // pure potential
            VedicElement::Air => Guna::Rajas,    // movement
            VedicElement::Fire => Guna::Rajas,   // transformation
            VedicElement::Water => Guna::Sattva, // flow, harmony
            VedicElement::Earth => Guna::Tamas,  // stability
        }
    }

    /// Corresponding classical element index (None for Ether).
    pub fn classical_index(self) -> Option<usize> {
        match self {
            VedicElement::Fire => Some(0),
            VedicElement::Earth => Some(1),
            VedicElement::Air => Some(2),
            VedicElement::Water => Some(3),
            VedicElement::Ether => None,
        }
    }

    pub fn all() -> [VedicElement; 5] {
        [
            VedicElement::Ether,
            VedicElement::Fire,
            VedicElement::Earth,
            VedicElement::Air,
            VedicElement::Water,
        ]
    }

    /// Parse a vedic element from a lowercase string.
    /// Accepts English ("fire") and Sanskrit ("tejas") names.
    pub fn parse(s: &str) -> Option<VedicElement> {
        match s.trim().to_lowercase().as_str() {
            "fire" | "tejas" | "agni" => Some(VedicElement::Fire),
            "earth" | "prithvi" | "bhumi" => Some(VedicElement::Earth),
            "air" | "vayu" => Some(VedicElement::Air),
            "water" | "apas" | "jal" | "apa" => Some(VedicElement::Water),
            "ether" | "akasha" | "space" | "void" => Some(VedicElement::Ether),
            _ => None,
        }
    }
}

// ─── Vedic Classification ───────────────────────────────────────────────────

/// The complete Vedic classification of a token.
///
/// Every token in the descent pipeline ultimately resolves to a
/// Graha × Nakshatra × Guna × VedicElement × confidence score.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VedicClassification {
    /// Graha (planet) activation array [Surya..Ketu].
    pub grahas: [f64; 9],
    /// Nakshatra (lunar mansion) activation array [Ashwini..Revati].
    pub nakshatras: [f64; 27],
    /// Guna (quality) activation [Sattva, Rajas, Tamas].
    pub gunas: [f64; 3],
    /// Vedic element activation [Fire..Ether].
    pub vedic_elements: [f64; 5],
    /// Overall descent confidence [0, 1].
    pub confidence: f64,
}

impl Default for VedicClassification {
    fn default() -> Self {
        VedicClassification {
            grahas: [0.0; 9],
            nakshatras: [0.0; 27],
            gunas: [0.0; 3],
            vedic_elements: [0.0; 5],
            confidence: 0.5,
        }
    }
}

impl VedicClassification {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a graha's activation.
    pub fn with_graha(mut self, graha: Graha, value: f64) -> Self {
        self.grahas[graha.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set a nakshatra's activation.
    pub fn with_nakshatra(mut self, nak: Nakshatra, value: f64) -> Self {
        self.nakshatras[nak.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set a guna's activation.
    pub fn with_guna(mut self, guna: Guna, value: f64) -> Self {
        self.gunas[guna.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set a vedic element's activation.
    pub fn with_vedic_element(mut self, elem: VedicElement, value: f64) -> Self {
        self.vedic_elements[elem.index()] = value.clamp(0.0, 1.0);
        self
    }

    /// Set confidence.
    pub fn with_confidence(mut self, conf: f64) -> Self {
        self.confidence = conf.clamp(0.0, 1.0);
        self
    }

    /// Dominant graha.
    pub fn dominant_graha(&self) -> Option<Graha> {
        self.grahas
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| Graha::from_index(i))
    }

    /// Dominant nakshatra.
    pub fn dominant_nakshatra(&self) -> Option<Nakshatra> {
        self.nakshatras
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| Nakshatra::from_index(i))
    }

    /// Dominant guna.
    pub fn dominant_guna(&self) -> Option<Guna> {
        self.gunas
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| Guna::from_index(i))
    }

    /// Dominant vedic element.
    pub fn dominant_vedic_element(&self) -> Option<VedicElement> {
        self.vedic_elements
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .filter(|(_, v)| **v > 0.0)
            .map(|(i, _)| VedicElement::from_index(i))
    }

    /// Merge two Vedic classifications by element-wise max (union).
    pub fn merge_max(&self, other: &VedicClassification) -> VedicClassification {
        let mut result = self.clone();
        for i in 0..9 {
            result.grahas[i] = result.grahas[i].max(other.grahas[i]);
        }
        for i in 0..27 {
            result.nakshatras[i] = result.nakshatras[i].max(other.nakshatras[i]);
        }
        for i in 0..3 {
            result.gunas[i] = result.gunas[i].max(other.gunas[i]);
        }
        for i in 0..5 {
            result.vedic_elements[i] = result.vedic_elements[i].max(other.vedic_elements[i]);
        }
        result.confidence = result.confidence.max(other.confidence);
        result
    }
}

impl std::fmt::Display for VedicClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vedic {{ ")?;
        if let Some(g) = self.dominant_graha() {
            write!(f, "graha: {} {}, ", g.symbol(), g.name())?;
        }
        if let Some(n) = self.dominant_nakshatra() {
            write!(f, "nakṣatra: {:?}, ", n)?;
        }
        if let Some(g) = self.dominant_guna() {
            write!(f, "guṇa: {} {}, ", g.symbol(), g.name())?;
        }
        if let Some(e) = self.dominant_vedic_element() {
            write!(f, "bhūta: {} {}, ", e.symbol(), e.sanskrit())?;
        }
        write!(f, "confidence: {:.2}", self.confidence)?;
        write!(f, " }}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Graha tests ────────────────────────────────────────────────────────

    #[test]
    fn test_graha_index_roundtrip() {
        for i in 0..9 {
            let g = Graha::from_index(i);
            assert_eq!(g.index(), i);
        }
    }

    #[test]
    fn test_graha_all_count() {
        assert_eq!(Graha::all().len(), 9);
        assert_eq!(ALL_GRAHAS.len(), 9);
    }

    #[test]
    fn test_graha_symbols() {
        for g in Graha::all() {
            assert!(!g.symbol().is_empty());
            assert!(!g.name().is_empty());
        }
    }

    #[test]
    fn test_surya_is_sattva() {
        assert_eq!(Graha::Surya.guna(), Guna::Sattva);
    }

    #[test]
    fn test_mangala_is_rajas() {
        assert_eq!(Graha::Mangala.guna(), Guna::Rajas);
    }

    #[test]
    fn test_shani_is_tamas() {
        assert_eq!(Graha::Shani.guna(), Guna::Tamas);
    }

    #[test]
    fn test_graha_adjacent() {
        let adj = Graha::Surya.adjacent();
        assert_eq!(adj[0], Graha::Chandra);
        assert_eq!(adj[1], Graha::Ketu);
    }

    #[test]
    fn test_graha_opposite() {
        // On a 9-node wheel, opposite is 4 steps forward (160°)
        assert_eq!(Graha::Surya.opposite(), Graha::Brihaspati); // 0→4
        assert_eq!(Graha::Chandra.opposite(), Graha::Shukra); // 1→5
        assert_eq!(Graha::Mangala.opposite(), Graha::Shani); // 2→6
        assert_eq!(Graha::Budha.opposite(), Graha::Rahu); // 3→7
        assert_eq!(Graha::Brihaspati.opposite(), Graha::Ketu); // 4→8
        assert_eq!(Graha::Shukra.opposite(), Graha::Surya); // 5→0
    }

    #[test]
    fn test_graha_trines() {
        // 3 and 6 steps forward (120° and 240°)
        let trines = Graha::Surya.trines();
        assert_eq!(trines[0], Graha::Budha);
        assert_eq!(trines[1], Graha::Shani);
    }

    #[test]
    fn test_graha_offset_wraparound() {
        assert_eq!(Graha::Ketu.offset(1), Graha::Surya);
        assert_eq!(Graha::Surya.offset(-1), Graha::Ketu);
    }

    #[test]
    fn test_graha_parse_variants() {
        assert_eq!(Graha::parse("surya"), Some(Graha::Surya));
        assert_eq!(Graha::parse("☉"), Some(Graha::Surya));
        assert_eq!(Graha::parse("sun"), Some(Graha::Surya));
        assert_eq!(Graha::parse("Mangala"), Some(Graha::Mangala));
        assert_eq!(Graha::parse("MARS"), Some(Graha::Mangala));
        assert_eq!(Graha::parse("rahu"), Some(Graha::Rahu));
        assert_eq!(Graha::parse("ketu"), Some(Graha::Ketu));
        assert_eq!(Graha::parse("unknown"), None);
    }

    #[test]
    fn test_graha_display() {
        let s = format!("{}", Graha::Surya);
        assert!(s.contains("☉"));
        assert!(s.contains("Surya"));
    }

    #[test]
    fn test_graha_knowledge_domain() {
        assert_eq!(Graha::Surya.knowledge_domain(), "Self & Leadership");
        assert_eq!(Graha::Chandra.knowledge_domain(), "Mind & Emotion");
        assert_eq!(Graha::Mangala.knowledge_domain(), "Action & Engineering");
        assert_eq!(Graha::Budha.knowledge_domain(), "Logic & Communication");
        assert_eq!(Graha::Brihaspati.knowledge_domain(), "Wisdom & Law");
        assert_eq!(Graha::Shukra.knowledge_domain(), "Arts & Value");
        assert_eq!(Graha::Shani.knowledge_domain(), "Structure & Time");
        assert_eq!(Graha::Rahu.knowledge_domain(), "Innovation & Tech");
        assert_eq!(Graha::Ketu.knowledge_domain(), "Spirituality & Science");
    }

    #[test]
    fn test_graha_sanskrit() {
        assert_eq!(Graha::Surya.sanskrit(), "सूर्य");
        assert_eq!(Graha::Chandra.sanskrit(), "चन्द्र");
    }

    #[test]
    fn test_graha_name_lower() {
        assert_eq!(Graha::Surya.name_lower(), "surya");
        assert_eq!(Graha::Ketu.name_lower(), "ketu");
    }

    // ─── Rashi tests ────────────────────────────────────────────────────────

    #[test]
    fn test_rashi_index_roundtrip() {
        for i in 0..12 {
            let r = Rashi::from_index(i);
            assert_eq!(r.index(), i);
        }
    }

    #[test]
    fn test_rashi_all_count() {
        assert_eq!(Rashi::all().len(), 12);
        assert_eq!(ALL_RASHIS.len(), 12);
    }

    #[test]
    fn test_rashi_lords() {
        assert_eq!(Rashi::Mesha.lord(), Graha::Mangala);
        assert_eq!(Rashi::Simha.lord(), Graha::Surya);
        assert_eq!(Rashi::Kanya.lord(), Graha::Budha);
        assert_eq!(Rashi::Meena.lord(), Graha::Brihaspati);
    }

    #[test]
    fn test_rashi_tattva() {
        assert_eq!(Rashi::Mesha.tattva(), VedicElement::Fire);
        assert_eq!(Rashi::Karka.tattva(), VedicElement::Water);
        assert_eq!(Rashi::Kumbha.tattva(), VedicElement::Air);
        assert_eq!(Rashi::Kanya.tattva(), VedicElement::Earth);
    }

    #[test]
    fn test_rashi_guna() {
        assert_eq!(Rashi::Simha.guna(), Guna::Sattva);
        assert_eq!(Rashi::Mesha.guna(), Guna::Rajas);
        assert_eq!(Rashi::Vrishabha.guna(), Guna::Tamas);
    }

    #[test]
    fn test_rashi_purushartha() {
        assert_eq!(Rashi::Simha.purushartha(), "Dharma");
        assert_eq!(Rashi::Vrishabha.purushartha(), "Kama");
        assert_eq!(Rashi::Kanya.purushartha(), "Moksha");
        assert_eq!(Rashi::Makara.purushartha(), "Artha");
    }

    #[test]
    fn test_rashi_display() {
        let s = format!("{}", Rashi::Mesha);
        assert!(s.contains("♈"));
        assert!(s.contains("Mesha"));
    }

    // ─── Nakshatra tests ────────────────────────────────────────────────────

    #[test]
    fn test_nakshatra_count() {
        assert_eq!(Nakshatra::all().len(), 27);
    }

    #[test]
    fn test_nakshatra_index_roundtrip() {
        for i in 0..27 {
            let n = Nakshatra::from_index(i);
            assert_eq!(n.index(), i);
        }
    }

    #[test]
    fn test_nakshatra_shakti() {
        for n in Nakshatra::all() {
            assert!(!n.shakti().is_empty());
        }
    }

    #[test]
    fn test_nakshatra_ruler_cycle() {
        // First 9 nakshatras cycle through 9 grahas
        assert_eq!(Nakshatra::Ashwini.ruler(), Graha::Ketu);
        assert_eq!(Nakshatra::Bharani.ruler(), Graha::Shukra);
        assert_eq!(Nakshatra::Krittika.ruler(), Graha::Surya);
        assert_eq!(Nakshatra::Rohini.ruler(), Graha::Chandra);
        assert_eq!(Nakshatra::Mrigashira.ruler(), Graha::Mangala);
        assert_eq!(Nakshatra::Ardra.ruler(), Graha::Rahu);
        assert_eq!(Nakshatra::Punarvasu.ruler(), Graha::Brihaspati);
        assert_eq!(Nakshatra::Pushya.ruler(), Graha::Shani);
        assert_eq!(Nakshatra::Ashlesha.ruler(), Graha::Budha);
        // Cycle repeats
        assert_eq!(Nakshatra::Magha.ruler(), Graha::Ketu);
    }

    // ─── Guna tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_guna_all() {
        assert_eq!(Guna::all().len(), 3);
    }

    #[test]
    fn test_guna_direction() {
        assert_eq!(Guna::Sattva.direction(), "↑ Rise");
        assert_eq!(Guna::Tamas.direction(), "↓ Descend");
    }

    // ─── VedicElement tests ─────────────────────────────────────────────────

    #[test]
    fn test_vedic_element_5() {
        assert_eq!(VedicElement::all().len(), 5);
    }

    #[test]
    fn test_ether_is_sattva() {
        assert_eq!(VedicElement::Ether.guna(), Guna::Sattva);
    }

    // ─── VedicClassification tests ──────────────────────────────────────────

    #[test]
    fn test_vedic_classification_default() {
        let v = VedicClassification::new();
        assert_eq!(v.grahas, [0.0; 9]);
        assert_eq!(v.confidence, 0.5);
    }

    #[test]
    fn test_vedic_classification_dominants() {
        let v = VedicClassification::new()
            .with_graha(Graha::Surya, 0.9)
            .with_graha(Graha::Chandra, 0.3)
            .with_nakshatra(Nakshatra::Ashwini, 0.8)
            .with_guna(Guna::Sattva, 0.9)
            .with_vedic_element(VedicElement::Ether, 0.7);
        assert_eq!(v.dominant_graha(), Some(Graha::Surya));
        assert_eq!(v.dominant_nakshatra(), Some(Nakshatra::Ashwini));
        assert_eq!(v.dominant_guna(), Some(Guna::Sattva));
        assert_eq!(v.dominant_vedic_element(), Some(VedicElement::Ether));
    }

    #[test]
    fn test_vedic_classification_display() {
        let v = VedicClassification::new().with_graha(Graha::Surya, 0.9);
        let s = format!("{}", v);
        assert!(s.contains("Surya"));
        assert!(s.contains("☉"));
    }
}
