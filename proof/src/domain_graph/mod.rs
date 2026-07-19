//! # Domain graph — re-exports from lai-core with proof-specific extensions

pub use lai_core::domain::{
    CompositionAspect, CurriculumBand, Direction, Domain, MasteryLayer, Node, Position,
    Relationship, UnderstandingAxis, WheelError, WheelGraph, WheelResult, ALL_DOMAINS,
    LEVELS_PER_CYCLE, MAX_LEVEL,
};

use crate::astrology::Sign;

/// Map an astrology `Sign` to its ruling `Domain` via planetary rulership.
pub fn from_sign(sign: Sign) -> Domain {
    match sign {
        Sign::Aries => Domain::Mangala,
        Sign::Taurus => Domain::Shukra,
        Sign::Gemini => Domain::Budha,
        Sign::Cancer => Domain::Chandra,
        Sign::Leo => Domain::Surya,
        Sign::Virgo => Domain::Budha,
        Sign::Libra => Domain::Shukra,
        Sign::Scorpio => Domain::Mangala,
        Sign::Sagittarius => Domain::Brihaspati,
        Sign::Capricorn => Domain::Shani,
        Sign::Aquarius => Domain::Shani,
        Sign::Pisces => Domain::Brihaspati,
    }
}
