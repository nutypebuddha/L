use serde::{Deserialize, Serialize};

/// The 12 astrological houses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum House {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Ninth,
    Tenth,
    Eleventh,
    Twelfth,
}

impl House {
    pub const COUNT: usize = 12;

    pub fn index(self) -> usize {
        match self {
            House::First => 0,
            House::Second => 1,
            House::Third => 2,
            House::Fourth => 3,
            House::Fifth => 4,
            House::Sixth => 5,
            House::Seventh => 6,
            House::Eighth => 7,
            House::Ninth => 8,
            House::Tenth => 9,
            House::Eleventh => 10,
            House::Twelfth => 11,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i % 12 {
            0 => House::First,
            1 => House::Second,
            2 => House::Third,
            3 => House::Fourth,
            4 => House::Fifth,
            5 => House::Sixth,
            6 => House::Seventh,
            7 => House::Eighth,
            8 => House::Ninth,
            9 => House::Tenth,
            10 => House::Eleventh,
            11 => House::Twelfth,
            _ => unreachable!(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            House::First => "1st",
            House::Second => "2nd",
            House::Third => "3rd",
            House::Fourth => "4th",
            House::Fifth => "5th",
            House::Sixth => "6th",
            House::Seventh => "7th",
            House::Eighth => "8th",
            House::Ninth => "9th",
            House::Tenth => "10th",
            House::Eleventh => "11th",
            House::Twelfth => "12th",
        }
    }

    pub fn domain(self) -> &'static str {
        match self {
            House::First => "Self & Identity",
            House::Second => "Resources & Values",
            House::Third => "Communication & Siblings",
            House::Fourth => "Home & Foundation",
            House::Fifth => "Creativity & Children",
            House::Sixth => "Health & Service",
            House::Seventh => "Partnerships & Marriage",
            House::Eighth => "Transformation & Shared Resources",
            House::Ninth => "Philosophy & Travel",
            House::Tenth => "Career & Reputation",
            House::Eleventh => "Friends & Goals",
            House::Twelfth => "Subconscious & Retreat",
        }
    }
}

impl std::fmt::Display for House {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} House", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_house_index_roundtrip() {
        for i in 0..12 {
            let h = House::from_index(i);
            assert_eq!(h.index(), i);
        }
    }

    #[test]
    fn test_house_domains() {
        assert_eq!(House::First.domain(), "Self & Identity");
        assert_eq!(House::Tenth.domain(), "Career & Reputation");
    }
}
