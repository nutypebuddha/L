use serde::{Deserialize, Serialize};
use std::fmt;

use crate::domain_graph::Domain;

/// Graha is a type alias for `Domain` — the 9 Vedic grahas.
pub type Graha = Domain;

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

/// The 12 Vedic rashis (sidereal signs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rashi {
    Mesha = 0,
    Vrishabha = 1,
    Mithuna = 2,
    Karka = 3,
    Simha = 4,
    Kanya = 5,
    Tula = 6,
    Vrishchika = 7,
    Dhanu = 8,
    Makara = 9,
    Kumbha = 10,
    Meena = 11,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Nakshatra {
    Ashwini = 0,
    Bharani = 1,
    Krittika = 2,
    Rohini = 3,
    Mrigashira = 4,
    Ardra = 5,
    Punarvasu = 6,
    Pushya = 7,
    Ashlesha = 8,
    Magha = 9,
    PurvaPhalguni = 10,
    UttaraPhalguni = 11,
    Hasta = 12,
    Chitra = 13,
    Svati = 14,
    Vishakha = 15,
    Anuradha = 16,
    Jyeshtha = 17,
    Mula = 18,
    PurvaAshadha = 19,
    UttaraAshadha = 20,
    Shravana = 21,
    Dhanishtha = 22,
    Shatabhisha = 23,
    PurvaBhadrapada = 24,
    UttaraBhadrapada = 25,
    Revati = 26,
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

    pub fn ruler(self) -> Graha {
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

    pub fn parse(s: &str) -> Option<Nakshatra> {
        use Nakshatra::*;
        let lower = s.trim().to_lowercase();
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Guna {
    Sattva = 0,
    Rajas = 1,
    Tamas = 2,
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

    pub fn quality(self) -> &'static str {
        match self {
            Guna::Sattva => "Harmony — clarity, balance, light",
            Guna::Rajas => "Passion — activity, movement, creativity",
            Guna::Tamas => "Inertia — stability, darkness, form",
        }
    }

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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VedicElement {
    Fire = 0,
    Earth = 1,
    Air = 2,
    Water = 3,
    Ether = 4,
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
            VedicElement::Ether => "◌",
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

    pub fn sanskrit(self) -> &'static str {
        match self {
            VedicElement::Fire => "Tejas",
            VedicElement::Earth => "Prithvi",
            VedicElement::Air => "Vayu",
            VedicElement::Water => "Apas",
            VedicElement::Ether => "Akasha",
        }
    }

    pub fn guna(self) -> Guna {
        match self {
            VedicElement::Ether => Guna::Sattva,
            VedicElement::Air => Guna::Rajas,
            VedicElement::Fire => Guna::Rajas,
            VedicElement::Water => Guna::Sattva,
            VedicElement::Earth => Guna::Tamas,
        }
    }

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VedicClassification {
    pub grahas: [f64; 9],
    pub nakshatras: [f64; 27],
    pub gunas: [f64; 3],
    pub vedic_elements: [f64; 5],
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

    pub fn with_graha(mut self, graha: Graha, value: f64) -> Self {
        self.grahas[graha.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn set_graha(&mut self, graha: Graha, value: f64) {
        self.grahas[graha.index()] = value.clamp(0.0, 1.0);
    }

    pub fn with_nakshatra(mut self, nak: Nakshatra, value: f64) -> Self {
        self.nakshatras[nak.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_guna(mut self, guna: Guna, value: f64) -> Self {
        self.gunas[guna.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_vedic_element(mut self, elem: VedicElement, value: f64) -> Self {
        self.vedic_elements[elem.index()] = value.clamp(0.0, 1.0);
        self
    }

    pub fn with_confidence(mut self, conf: f64) -> Self {
        self.confidence = conf.clamp(0.0, 1.0);
        self
    }

    pub fn dominant_graha(&self) -> Option<Graha> {
        self.grahas
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .and_then(|(i, _)| Graha::from_index(i))
    }

    pub fn dominant_nakshatra(&self) -> Option<Nakshatra> {
        self.nakshatras
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| Nakshatra::from_index(i))
    }

    pub fn dominant_guna(&self) -> Option<Guna> {
        self.gunas
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| Guna::from_index(i))
    }

    pub fn dominant_vedic_element(&self) -> Option<VedicElement> {
        self.vedic_elements
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .filter(|(_, &v)| v > 0.0)
            .map(|(i, _)| VedicElement::from_index(i))
    }

    pub fn merge_max(&self, other: &VedicClassification) -> VedicClassification {
        let mut result = self.clone();
        result.merge_max_into(other);
        result
    }

    pub fn merge_max_into(&mut self, other: &VedicClassification) {
        for i in 0..9 {
            self.grahas[i] = self.grahas[i].max(other.grahas[i]);
        }
        for i in 0..27 {
            self.nakshatras[i] = self.nakshatras[i].max(other.nakshatras[i]);
        }
        for i in 0..3 {
            self.gunas[i] = self.gunas[i].max(other.gunas[i]);
        }
        for i in 0..5 {
            self.vedic_elements[i] = self.vedic_elements[i].max(other.vedic_elements[i]);
        }
        self.confidence = self.confidence.max(other.confidence);
    }
}

impl fmt::Display for VedicClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vedic {{ ")?;
        if let Some(g) = self.dominant_graha() {
            write!(f, "graha: {} {}, ", g.symbol(), g.name())?;
        }
        if let Some(n) = self.dominant_nakshatra() {
            write!(f, "nakshatra: {:?}, ", n)?;
        }
        if let Some(g) = self.dominant_guna() {
            write!(f, "guna: {} {}, ", g.symbol(), g.name())?;
        }
        if let Some(e) = self.dominant_vedic_element() {
            write!(f, "bhuta: {} {}, ", e.symbol(), e.sanskrit())?;
        }
        write!(f, "confidence: {:.2}", self.confidence)?;
        write!(f, " }}")
    }
}

/// Helper: get the VedicElement for a Graha (Domain).
pub fn graha_vedic_element(graha: Graha) -> VedicElement {
    match graha {
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

/// Helper: get the Guna for a Graha (Domain).
pub fn graha_guna(graha: Graha) -> Guna {
    match graha {
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

/// Helper: convert a Sign to a Graha via planetary rulership.
pub fn graha_from_sign(sign: crate::astrology::Sign) -> Graha {
    use crate::astrology::Sign;
    match sign {
        Sign::Aries => Graha::Mangala,
        Sign::Taurus => Graha::Shukra,
        Sign::Gemini => Graha::Budha,
        Sign::Cancer => Graha::Chandra,
        Sign::Leo => Graha::Surya,
        Sign::Virgo => Graha::Budha,
        Sign::Libra => Graha::Shukra,
        Sign::Scorpio => Graha::Mangala,
        Sign::Sagittarius => Graha::Brihaspati,
        Sign::Capricorn => Graha::Shani,
        Sign::Aquarius => Graha::Shani,
        Sign::Pisces => Graha::Brihaspati,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graha_index_roundtrip() {
        for i in 0..9 {
            let g = Graha::from_index(i).expect("valid graha index");
            assert_eq!(g.index(), i);
        }
    }

    #[test]
    fn test_graha_all_count() {
        assert_eq!(Graha::all().len(), 9);
        assert_eq!(ALL_GRAHAS.len(), 9);
    }

    #[test]
    fn test_surya_is_sattva() {
        assert_eq!(graha_guna(Graha::Surya), Guna::Sattva);
    }

    #[test]
    fn test_mangala_is_rajas() {
        assert_eq!(graha_guna(Graha::Mangala), Guna::Rajas);
    }

    #[test]
    fn test_shani_is_tamas() {
        assert_eq!(graha_guna(Graha::Shani), Guna::Tamas);
    }

    #[test]
    fn test_rashi_index_roundtrip() {
        for i in 0..12 {
            let r = Rashi::from_index(i);
            assert_eq!(r.index(), i);
        }
    }

    #[test]
    fn test_rashi_lords() {
        assert_eq!(Rashi::Mesha.lord(), Graha::Mangala);
        assert_eq!(Rashi::Simha.lord(), Graha::Surya);
        assert_eq!(Rashi::Kanya.lord(), Graha::Budha);
        assert_eq!(Rashi::Meena.lord(), Graha::Brihaspati);
    }

    #[test]
    fn test_nakshatra_count() {
        assert_eq!(Nakshatra::all().len(), 27);
    }

    #[test]
    fn test_nakshatra_ruler_cycle() {
        assert_eq!(Nakshatra::Ashwini.ruler(), Graha::Ketu);
        assert_eq!(Nakshatra::Bharani.ruler(), Graha::Shukra);
        assert_eq!(Nakshatra::Krittika.ruler(), Graha::Surya);
        assert_eq!(Nakshatra::Magha.ruler(), Graha::Ketu);
    }

    #[test]
    fn test_guna_all() {
        assert_eq!(Guna::all().len(), 3);
    }

    #[test]
    fn test_vedic_element_5() {
        assert_eq!(VedicElement::all().len(), 5);
    }

    #[test]
    fn test_ether_is_sattva() {
        assert_eq!(VedicElement::Ether.guna(), Guna::Sattva);
    }

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
}
