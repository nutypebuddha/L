use crate::astrology::Sign;

/// A single primitive mapping entry for a zodiac sign.
#[derive(Debug, Clone)]
pub struct PrimitiveEntry {
    /// The primitive label (e.g. "nand", "not", "and").
    pub primitive: &'static str,
    /// Semantic hint for this primitive in the context of this sign.
    pub hint: &'static str,
}

/// Maps 12 zodiac signs to their NAND primitive routing table.
///
/// Each sign encodes a relationship between two operands. The primitives
/// describe the logical operations that characterize that relationship.
#[derive(Debug, Clone)]
pub struct PrimitiveMapping {
    /// Inner table: 12 signs × primitive entries.
    table: Vec<Vec<PrimitiveEntry>>,
}

impl Default for PrimitiveMapping {
    fn default() -> Self {
        Self::new()
    }
}

impl PrimitiveMapping {
    /// Build the default routing table from Vedic sign semantics.
    pub fn new() -> Self {
        let table = vec![
            // Aries: self / identity → direct, inverse, conjunction
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "self-assertion vs denial",
                },
                PrimitiveEntry {
                    primitive: "not",
                    hint: "reversal of will",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "conjunction of intent",
                },
            ],
            // Taurus: possession / stability → holding, releasing, merging
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "attachment vs release",
                },
                PrimitiveEntry {
                    primitive: "or",
                    hint: "abundance or scarcity",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "stable foundation",
                },
            ],
            // Gemini: communication / duality → signal, noise, channel
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "signal filtered by noise",
                },
                PrimitiveEntry {
                    primitive: "xor",
                    hint: "either/or exchange",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "synchronous dialogue",
                },
            ],
            // Cancer: nourishment / protection → shelter, exposure, nurture
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "protective shell vs vulnerability",
                },
                PrimitiveEntry {
                    primitive: "not",
                    hint: "withdrawal",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "nurturing bond",
                },
            ],
            // Leo: expression / authority → performance, humility, radiance
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "stage vs backstage",
                },
                PrimitiveEntry {
                    primitive: "or",
                    hint: "applause or silence",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "authentic expression",
                },
            ],
            // Virgo: analysis / refinement → filter, purge, optimize
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "imperfection filtered",
                },
                PrimitiveEntry {
                    primitive: "not",
                    hint: "rejection of flaw",
                },
                PrimitiveEntry {
                    primitive: "xnor",
                    hint: "exact match",
                },
            ],
            // Libra: balance / harmony → weighing, counterweight, equilibrium
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "imbalance detected",
                },
                PrimitiveEntry {
                    primitive: "xor",
                    hint: "opposing forces",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "harmonious union",
                },
            ],
            // Scorpio: transformation / depth → dissolution, rebirth, intensification
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "ego dissolution",
                },
                PrimitiveEntry {
                    primitive: "not",
                    hint: "shadow revealed",
                },
                PrimitiveEntry {
                    primitive: "or",
                    hint: "merge or sever",
                },
            ],
            // Sagittarius: expansion / philosophy → projection, retreat, synthesis
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "belief challenged",
                },
                PrimitiveEntry {
                    primitive: "or",
                    hint: "exploration or dogma",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "integrated wisdom",
                },
            ],
            // Capricorn: structure / ambition → constraint, liberation, mastery
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "limitation tested",
                },
                PrimitiveEntry {
                    primitive: "not",
                    hint: "rebellion",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "enduring structure",
                },
            ],
            // Aquarius: innovation / collectivity → disruption, conformity, network
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "orthodoxy broken",
                },
                PrimitiveEntry {
                    primitive: "xor",
                    hint: "individual vs group",
                },
                PrimitiveEntry {
                    primitive: "and",
                    hint: "collective vision",
                },
            ],
            // Pisces: dissolution / transcendence → surrender, grounding, ecstasy
            vec![
                PrimitiveEntry {
                    primitive: "nand",
                    hint: "boundary dissolved",
                },
                PrimitiveEntry {
                    primitive: "not",
                    hint: "world negated",
                },
                PrimitiveEntry {
                    primitive: "or",
                    hint: "spirit or matter",
                },
            ],
        ];
        Self { table }
    }

    /// Return the primitive entries for a given sign.
    pub fn for_sign(&self, sign: Sign) -> &[PrimitiveEntry] {
        let idx = sign as usize;
        if idx < self.table.len() {
            &self.table[idx]
        } else {
            &[]
        }
    }

    /// Return the primitive entries for a sign index (0–11).
    pub fn for_sign_index(&self, idx: usize) -> &[PrimitiveEntry] {
        if idx < self.table.len() {
            &self.table[idx]
        } else {
            &[]
        }
    }
}
