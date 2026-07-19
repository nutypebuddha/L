use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use thiserror::Error;

/// The 9 Vedic grahas — primary domain nodes on the wheel.
///
/// Each graha represents a fundamental cognitive/knowledge domain.
/// The wheel defines how knowledge composes across domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Domain {
    Surya,
    Chandra,
    Mangala,
    Budha,
    Brihaspati,
    Shukra,
    Shani,
    Rahu,
    Ketu,
}

/// All 9 domains in wheel order (Surya → Ketu at 40° intervals).
pub const ALL_DOMAINS: [Domain; 9] = [
    Domain::Surya,
    Domain::Chandra,
    Domain::Mangala,
    Domain::Budha,
    Domain::Brihaspati,
    Domain::Shukra,
    Domain::Shani,
    Domain::Rahu,
    Domain::Ketu,
];

impl Domain {
    /// Arc index on the wheel (0–8).
    pub fn index(self) -> usize {
        match self {
            Domain::Surya => 0,
            Domain::Chandra => 1,
            Domain::Mangala => 2,
            Domain::Budha => 3,
            Domain::Brihaspati => 4,
            Domain::Shukra => 5,
            Domain::Shani => 6,
            Domain::Rahu => 7,
            Domain::Ketu => 8,
        }
    }

    /// Create a domain from its arc index (0–8).
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Domain::Surya),
            1 => Some(Domain::Chandra),
            2 => Some(Domain::Mangala),
            3 => Some(Domain::Budha),
            4 => Some(Domain::Brihaspati),
            5 => Some(Domain::Shukra),
            6 => Some(Domain::Shani),
            7 => Some(Domain::Rahu),
            8 => Some(Domain::Ketu),
            _ => None,
        }
    }

    /// Astrological symbol.
    pub fn symbol(self) -> &'static str {
        match self {
            Domain::Surya => "☉",
            Domain::Chandra => "☽",
            Domain::Mangala => "♂",
            Domain::Budha => "☿",
            Domain::Brihaspati => "♃",
            Domain::Shukra => "♀",
            Domain::Shani => "♄",
            Domain::Rahu => "☊",
            Domain::Ketu => "☋",
        }
    }

    /// Sanskrit name.
    pub fn name(self) -> &'static str {
        match self {
            Domain::Surya => "Surya",
            Domain::Chandra => "Chandra",
            Domain::Mangala => "Mangala",
            Domain::Budha => "Budha",
            Domain::Brihaspati => "Brihaspati",
            Domain::Shukra => "Shukra",
            Domain::Shani => "Shani",
            Domain::Rahu => "Rahu",
            Domain::Ketu => "Ketu",
        }
    }

    /// English name.
    pub fn full_name(self) -> &'static str {
        match self {
            Domain::Surya => "Sun",
            Domain::Chandra => "Moon",
            Domain::Mangala => "Mars",
            Domain::Budha => "Mercury",
            Domain::Brihaspati => "Jupiter",
            Domain::Shukra => "Venus",
            Domain::Shani => "Saturn",
            Domain::Rahu => "North Node",
            Domain::Ketu => "South Node",
        }
    }

    /// Knowledge domain archetype.
    pub fn archetype(self) -> &'static str {
        match self {
            Domain::Surya => "Self & Leadership",
            Domain::Chandra => "Mind & Emotion",
            Domain::Mangala => "Action & Engineering",
            Domain::Budha => "Logic & Communication",
            Domain::Brihaspati => "Wisdom & Law",
            Domain::Shukra => "Arts & Value",
            Domain::Shani => "Structure & Time",
            Domain::Rahu => "Innovation & Tech",
            Domain::Ketu => "Spirituality & Science",
        }
    }

    /// Offset by `delta` steps on the 9-node wheel (wrapping).
    pub fn offset(self, delta: isize) -> Self {
        let idx = ((self.index() as isize + delta).rem_euclid(9)) as usize;
        Self::from_index(idx).unwrap_or(Domain::Surya)
    }

    /// Opposite graha (index + 4) % 9 — 160° opposition.
    pub fn opposite(self) -> Self {
        self.offset(4)
    }

    /// Adjacent grahas (±1 step).
    pub fn adjacent(self) -> [Self; 2] {
        [self.offset(1), self.offset(-1)]
    }

    /// Trine grahas (index + 3) % 9 and (index + 6) % 9.
    pub fn trines(self) -> [Self; 2] {
        [self.offset(3), self.offset(6)]
    }

    /// Greek letter name (Phase 7 — Greek ontology).
    pub fn greek_name(self) -> &'static str {
        match self {
            Domain::Surya => "Alpha (α)",
            Domain::Chandra => "Beta (β)",
            Domain::Mangala => "Gamma (γ)",
            Domain::Budha => "Delta (δ)",
            Domain::Brihaspati => "Epsilon (ε)",
            Domain::Shukra => "Zeta (ζ)",
            Domain::Shani => "Eta (η)",
            Domain::Rahu => "Theta (θ)",
            Domain::Ketu => "Iota (ι)",
        }
    }

    /// Greek symbol.
    pub fn greek_symbol(self) -> &'static str {
        match self {
            Domain::Surya => "α",
            Domain::Chandra => "β",
            Domain::Mangala => "γ",
            Domain::Budha => "δ",
            Domain::Brihaspati => "ε",
            Domain::Shukra => "ζ",
            Domain::Shani => "η",
            Domain::Rahu => "θ",
            Domain::Ketu => "ι",
        }
    }

    /// Lowercased name — no allocation.
    pub fn name_lower(self) -> &'static str {
        match self {
            Domain::Surya => "surya",
            Domain::Chandra => "chandra",
            Domain::Mangala => "mangala",
            Domain::Budha => "budha",
            Domain::Brihaspati => "brihaspati",
            Domain::Shukra => "shukra",
            Domain::Shani => "shani",
            Domain::Rahu => "rahu",
            Domain::Ketu => "ketu",
        }
    }

    /// Western/English name for this graha.
    pub fn english_name(self) -> &'static str {
        match self {
            Domain::Surya => "sun",
            Domain::Chandra => "moon",
            Domain::Mangala => "mars",
            Domain::Budha => "mercury",
            Domain::Brihaspati => "jupiter",
            Domain::Shukra => "venus",
            Domain::Shani => "saturn",
            Domain::Rahu => "rahu",
            Domain::Ketu => "ketu",
        }
    }

    /// Sanskrit name.
    pub fn sanskrit(self) -> &'static str {
        match self {
            Domain::Surya => "सूर्य",
            Domain::Chandra => "चन्द्र",
            Domain::Mangala => "मङ्गल",
            Domain::Budha => "बुध",
            Domain::Brihaspati => "बृहस्पति",
            Domain::Shukra => "शुक्र",
            Domain::Shani => "शनि",
            Domain::Rahu => "राहु",
            Domain::Ketu => "केतु",
        }
    }

    /// Element affinity (Vedic tattva name as string).
    pub fn element_affinity(self) -> &'static str {
        match self {
            Domain::Surya => "Fire",
            Domain::Chandra => "Water",
            Domain::Mangala => "Fire",
            Domain::Budha => "Earth",
            Domain::Brihaspati => "Ether",
            Domain::Shukra => "Water",
            Domain::Shani => "Air",
            Domain::Rahu => "Air",
            Domain::Ketu => "Ether",
        }
    }

    /// All 9 domains in wheel order.
    pub fn all() -> [Self; 9] {
        ALL_DOMAINS
    }

    /// Alias for `name_lower()` — backward compatibility.
    #[inline]
    pub fn full_name_lower(self) -> &'static str {
        self.name_lower()
    }

    /// Parse a domain from a string (name, symbol, or alias).
    pub fn parse(s: &str) -> Option<Self> {
        let lower = s.to_ascii_lowercase();
        match lower.as_str() {
            "surya" | "☉" | "sun" | "alpha" | "α" => Some(Domain::Surya),
            "chandra" | "☽" | "moon" | "beta" | "β" => Some(Domain::Chandra),
            "mangala" | "♂" | "mars" | "gamma" | "γ" => Some(Domain::Mangala),
            "budha" | "☿" | "mercury" | "delta" | "δ" => Some(Domain::Budha),
            "brihaspati" | "♃" | "jupiter" | "epsilon" | "ε" => Some(Domain::Brihaspati),
            "shukra" | "♀" | "venus" | "zeta" | "ζ" => Some(Domain::Shukra),
            "shani" | "♄" | "saturn" | "eta" | "η" => Some(Domain::Shani),
            "rahu" | "☊" | "north node" | "theta" | "θ" => Some(Domain::Rahu),
            "ketu" | "☋" | "south node" | "iota" | "ι" => Some(Domain::Ketu),
            _ => None,
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.symbol(), self.name())
    }
}

impl std::str::FromStr for Domain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| format!("unknown domain: {s}"))
    }
}

/// A node on the wheel, combining a domain with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub domain: Domain,
    pub symbol: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub index: usize,
    pub opposite: Domain,
    pub trines: [Domain; 2],
}

/// All 9 nodes with metadata.
pub fn compute_all_nodes() -> Vec<Node> {
    ALL_DOMAINS
        .iter()
        .enumerate()
        .map(|(i, &domain)| Node {
            domain,
            symbol: domain.symbol(),
            name: domain.name(),
            description: domain.archetype(),
            index: i,
            opposite: domain.opposite(),
            trines: domain.trines(),
        })
        .collect()
}

/// Errors from wheel operations.
#[derive(Error, Debug)]
pub enum WheelError {
    #[error("unknown domain: {0}")]
    UnknownDomain(String),

    #[error("no path between {0} and {1}")]
    NoPath(Domain, Domain),

    #[error("cycle detected in traversal")]
    CycleDetected,

    #[error("max depth {0} exceeded")]
    MaxDepthExceeded(usize),
}

/// Result type for wheel operations.
pub type WheelResult<T> = Result<T, WheelError>;

/// A position on the wheel: a domain and an optional formula reference.
#[derive(Debug, Clone)]
pub struct Position {
    pub domain: Domain,
    pub formula_id: Option<String>,
}

/// The **structural composition relationship** between two domains on
/// the fixed 9-node wheel. This is NOT an astronomical computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompositionAspect {
    Aligned,
    Adjacent,
    Harmonic,
    Tense,
    Antipodal,
}

/// Precomputed 9×9 lookup table for `CompositionAspect::between(a, b)`.
const COMPOSITION_ASPECT_TABLE: [[CompositionAspect; 9]; 9] = {
    let mut table = [[CompositionAspect::Aligned; 9]; 9];
    let mut i: usize = 0;
    while i < 9 {
        let mut j: usize = 0;
        while j < 9 {
            let diff = i.abs_diff(j);
            let min_diff = if diff < 9 - diff { diff } else { 9 - diff };
            table[i][j] = match min_diff {
                0 => CompositionAspect::Aligned,
                1 => CompositionAspect::Adjacent,
                2 => CompositionAspect::Tense,
                3 => CompositionAspect::Harmonic,
                4 => CompositionAspect::Antipodal,
                _ => CompositionAspect::Aligned,
            };
            j += 1;
        }
        i += 1;
    }
    table
};

impl CompositionAspect {
    /// Determine the structural composition relationship between two domains.
    #[inline]
    pub fn between(a: Domain, b: Domain) -> CompositionAspect {
        COMPOSITION_ASPECT_TABLE[a.index()][b.index()]
    }

    /// The structural arc distance (minimum steps on the 9-node wheel).
    #[inline]
    pub fn arc_distance(self) -> usize {
        match self {
            CompositionAspect::Aligned => 0,
            CompositionAspect::Adjacent => 1,
            CompositionAspect::Harmonic => 3,
            CompositionAspect::Tense => 2,
            CompositionAspect::Antipodal => 4,
        }
    }

    /// Whether formulas compose directly across this relationship.
    #[inline]
    pub fn is_direct(self) -> bool {
        matches!(
            self,
            CompositionAspect::Aligned | CompositionAspect::Adjacent | CompositionAspect::Harmonic
        )
    }

    /// Base confidence for this structural composition relationship.
    #[inline]
    pub fn confidence(self) -> f64 {
        match self {
            CompositionAspect::Aligned => 1.00,
            CompositionAspect::Adjacent => 0.95,
            CompositionAspect::Harmonic => 0.90,
            CompositionAspect::Tense => 0.75,
            CompositionAspect::Antipodal => 0.60,
        }
    }

    /// Whether this aspect represents tension or inversion.
    #[inline]
    pub fn is_tension(self) -> bool {
        matches!(
            self,
            CompositionAspect::Tense | CompositionAspect::Antipodal
        )
    }
}

impl fmt::Display for CompositionAspect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositionAspect::Aligned => write!(f, "Aligned (same domain)"),
            CompositionAspect::Adjacent => write!(f, "Adjacent (1 step, structural)"),
            CompositionAspect::Harmonic => write!(f, "Harmonic (3 steps, structural)"),
            CompositionAspect::Tense => write!(f, "Tense (2 steps, structural)"),
            CompositionAspect::Antipodal => write!(
                f,
                "Antipodal (4 steps, structural — NOT a real 180° opposition)"
            ),
        }
    }
}

/// The direction of traversal along the edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Reverse,
}

/// A typed relationship between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub from: Domain,
    pub to: Domain,
    pub aspect: CompositionAspect,
    pub distance: usize,
    pub direction: Direction,
}

impl Relationship {
    /// Create a relationship between two domains.
    pub fn new(from: Domain, to: Domain) -> Self {
        let aspect = CompositionAspect::between(from, to);
        let distance = aspect.arc_distance();
        let (from_idx, to_idx) = (from.index(), to.index());
        let forward = ((to_idx as isize - from_idx as isize).rem_euclid(9)) <= 4;
        let direction = if forward {
            Direction::Forward
        } else {
            Direction::Reverse
        };
        Relationship {
            from,
            to,
            aspect,
            distance,
            direction,
        }
    }

    /// Reverse the direction of this relationship.
    pub fn reverse(&self) -> Self {
        Relationship {
            from: self.to,
            to: self.from,
            aspect: self.aspect,
            distance: self.distance,
            direction: match self.direction {
                Direction::Forward => Direction::Reverse,
                Direction::Reverse => Direction::Forward,
            },
        }
    }
}

/// Maximum level number (Grade 12).
pub const MAX_LEVEL: u8 = 12;

/// Total levels per cycle (K + 12 grades).
pub const LEVELS_PER_CYCLE: u16 = 13;

/// The four mastery layers mapped to K-12 understanding levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MasteryLayer {
    Unknown,
    Aware,
    Learning,
    Known,
}

impl MasteryLayer {
    /// The K-12 level corresponding to this mastery layer.
    pub fn level(self) -> u8 {
        match self {
            MasteryLayer::Unknown => 0,
            MasteryLayer::Aware => 3,
            MasteryLayer::Learning => 6,
            MasteryLayer::Known => 12,
        }
    }

    /// The human-readable state name for this layer.
    pub fn state(self) -> &'static str {
        match self {
            MasteryLayer::Unknown => "Unknown",
            MasteryLayer::Aware => "Aware",
            MasteryLayer::Learning => "Learning",
            MasteryLayer::Known => "Known",
        }
    }

    /// Derive the mastery layer from a K-12 level.
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => MasteryLayer::Unknown,
            1..=3 => MasteryLayer::Aware,
            4..=6 => MasteryLayer::Learning,
            _ => MasteryLayer::Known,
        }
    }
}

impl fmt::Display for MasteryLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.state(), self.level())
    }
}

/// The K-12 Spiral Understanding Axis for a single domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnderstandingAxis {
    pub level: u8,
    pub cycle: u8,
}

impl UnderstandingAxis {
    /// Create a new axis at the given level and cycle.
    pub fn new(level: u8, cycle: u8) -> Self {
        UnderstandingAxis {
            level: level.min(MAX_LEVEL),
            cycle,
        }
    }

    /// Kindergarten — the starting point.
    pub const fn kindergarten() -> Self {
        UnderstandingAxis { level: 0, cycle: 0 }
    }

    /// Linear depth value: `depth = cycle × 13 + level`.
    pub fn depth(self) -> u16 {
        self.cycle as u16 * LEVELS_PER_CYCLE + self.level as u16
    }

    /// Advance one level. If at level 12, loop to level 0 at next cycle.
    pub fn advance(&mut self) {
        if self.level >= MAX_LEVEL {
            self.level = 0;
            self.cycle += 1;
        } else {
            self.level += 1;
        }
    }

    /// Advance multiple levels.
    pub fn advance_by(&mut self, steps: u8) {
        for _ in 0..steps {
            self.advance();
        }
    }

    /// Get the next axis state without mutating.
    pub fn next(self) -> Self {
        let mut next = self;
        next.advance();
        next
    }

    /// Human-readable level name.
    pub fn level_name(self) -> &'static str {
        match self.level {
            0 => "Kindergarten",
            1 => "Grade 1",
            2 => "Grade 2",
            3 => "Grade 3",
            4 => "Grade 4",
            5 => "Grade 5",
            6 => "Grade 6",
            7 => "Grade 7",
            8 => "Grade 8",
            9 => "Grade 9",
            10 => "Grade 10",
            11 => "Grade 11",
            12 => "Grade 12",
            _ => "Beyond",
        }
    }

    /// Band name for the current level.
    pub fn band(self) -> &'static str {
        match self.level {
            0 => "Foundation",
            1..=3 => "Elementary",
            4..=6 => "Intermediate",
            7..=9 => "Advanced",
            10..=12 => "Mastery",
            _ => "Transcendent",
        }
    }

    /// The mastery layer corresponding to this understanding level.
    pub fn layer_index(self) -> MasteryLayer {
        MasteryLayer::from_level(self.level)
    }

    /// Full description.
    pub fn describe(self) -> String {
        format!(
            "Cycle {} {} ({}) — {} ({})",
            self.cycle,
            self.level_name(),
            self.band(),
            self.layer_index().state(),
            self.layer_index().level(),
        )
    }

    /// Shorter description.
    pub fn describe_layer(self) -> String {
        let layer = self.layer_index();
        format!(
            "{} ({}) — Level {}/{}",
            layer.state(),
            layer.level(),
            self.level,
            MAX_LEVEL
        )
    }
}

impl Default for UnderstandingAxis {
    fn default() -> Self {
        UnderstandingAxis::kindergarten()
    }
}

impl fmt::Display for UnderstandingAxis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.describe())
    }
}

/// Curriculum bands — broader groupings of K-12 levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CurriculumBand {
    Foundation,
    Elementary,
    Intermediate,
    Advanced,
    Mastery,
}

impl CurriculumBand {
    /// The levels covered by this band.
    pub fn levels(self) -> std::ops::RangeInclusive<u8> {
        match self {
            CurriculumBand::Foundation => 0..=0,
            CurriculumBand::Elementary => 1..=3,
            CurriculumBand::Intermediate => 4..=6,
            CurriculumBand::Advanced => 7..=9,
            CurriculumBand::Mastery => 10..=12,
        }
    }

    /// Detect the band from a level value.
    pub fn from_level(level: u8) -> Self {
        match level {
            0 => CurriculumBand::Foundation,
            1..=3 => CurriculumBand::Elementary,
            4..=6 => CurriculumBand::Intermediate,
            7..=9 => CurriculumBand::Advanced,
            _ => CurriculumBand::Mastery,
        }
    }
}

/// Precomputed edge existence table for O(1) `has_edge` lookup.
const EDGE_TABLE: [[bool; 9]; 9] = {
    let mut table = [[false; 9]; 9];
    let mut i: usize = 0;
    while i < 9 {
        let mut j: usize = 0;
        while j < 9 {
            let diff = i.abs_diff(j);
            let min_diff = if diff < 9 - diff { diff } else { 9 - diff };
            table[i][j] = matches!(min_diff, 0 | 1 | 3 | 4);
            j += 1;
        }
        i += 1;
    }
    table
};

/// The symbolic wheel graph.
#[derive(Debug, Clone)]
pub struct WheelGraph {
    adjacency: HashMap<Domain, Vec<(Domain, CompositionAspect)>>,
    shortest_paths: [[Option<Vec<Domain>>; 9]; 9],
}

impl Default for WheelGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl WheelGraph {
    /// Create a new wheel graph with all 9 Vedic graha nodes.
    pub fn new() -> Self {
        let mut adjacency: HashMap<Domain, Vec<(Domain, CompositionAspect)>> = HashMap::new();

        for &domain in ALL_DOMAINS.iter() {
            let mut edges: Vec<(Domain, CompositionAspect)> = Vec::new();

            edges.push((domain, CompositionAspect::Aligned));

            for adj in domain.adjacent() {
                edges.push((adj, CompositionAspect::Adjacent));
            }

            for trine in domain.trines() {
                edges.push((trine, CompositionAspect::Harmonic));
            }

            edges.push((domain.offset(4), CompositionAspect::Antipodal));

            adjacency.insert(domain, edges);
        }

        let mut shortest_paths: [[Option<Vec<Domain>>; 9]; 9] = Default::default();

        for (i, &from) in ALL_DOMAINS.iter().enumerate() {
            let mut visited = [false; 9];
            let mut prev = [None; 9];
            let mut queue = VecDeque::new();
            visited[i] = true;
            queue.push_back(i);

            while let Some(curr) = queue.pop_front() {
                let curr_domain = ALL_DOMAINS[curr];
                if let Some(neighbors) = adjacency.get(&curr_domain) {
                    for &(neighbor, _) in neighbors {
                        let nidx = neighbor.index();
                        if !visited[nidx] {
                            visited[nidx] = true;
                            prev[nidx] = Some(curr);
                            queue.push_back(nidx);
                        }
                    }
                }
            }

            for (j, &_to) in ALL_DOMAINS.iter().enumerate() {
                if i == j {
                    shortest_paths[i][j] = Some(vec![from]);
                } else if visited[j] {
                    let mut path = Vec::with_capacity(6);
                    let mut cur = j;
                    loop {
                        path.push(ALL_DOMAINS[cur]);
                        match prev[cur] {
                            Some(p) => cur = p,
                            None => break,
                        }
                    }
                    path.reverse();
                    shortest_paths[i][j] = Some(path);
                }
            }
        }

        WheelGraph {
            adjacency,
            shortest_paths,
        }
    }

    /// Get the neighbors of a domain, optionally filtered by aspect.
    #[inline]
    pub fn neighbors(
        &self,
        domain: Domain,
        aspect_filter: Option<CompositionAspect>,
    ) -> Vec<Domain> {
        let mut result: Vec<Domain> = self
            .adjacency
            .get(&domain)
            .map(|edges| {
                edges
                    .iter()
                    .filter(|(_, a)| aspect_filter.is_none_or(|filter| *a == filter))
                    .map(|(d, _)| *d)
                    .collect()
            })
            .unwrap_or_default();
        result.sort_by_key(|d| d.index());
        result
    }

    /// Get the relationship between two domains.
    pub fn relationship(&self, from: Domain, to: Domain) -> Relationship {
        Relationship::new(from, to)
    }

    /// Find all paths between `from` and `to` with maximum depth.
    pub fn find_paths(
        &self,
        from: Domain,
        to: Domain,
        max_depth: usize,
    ) -> WheelResult<Vec<Vec<Domain>>> {
        if max_depth > 12 {
            return Err(WheelError::MaxDepthExceeded(max_depth));
        }

        if from == to {
            return Ok(vec![vec![from]]);
        }

        let mut paths = Vec::new();
        let mut queue: VecDeque<(Vec<Domain>, HashSet<Domain>)> = VecDeque::new();
        queue.push_back((vec![from], HashSet::from([from])));

        while let Some((path, visited)) = queue.pop_front() {
            let current = *path.last().unwrap();
            let current_depth = path.len() - 1;

            if current_depth >= max_depth {
                continue;
            }

            if let Some(neighbors) = self.adjacency.get(&current) {
                for &(next, _) in neighbors {
                    if next == to {
                        let mut full_path = path.clone();
                        full_path.push(next);
                        paths.push(full_path);
                    } else if !visited.contains(&next) && current_depth + 1 < max_depth {
                        let mut new_path = path.clone();
                        new_path.push(next);
                        let mut new_visited = visited.clone();
                        new_visited.insert(next);
                        queue.push_back((new_path, new_visited));
                    }
                }
            }
        }

        if paths.is_empty() {
            return Err(WheelError::NoPath(from, to));
        }

        paths.sort_by_key(|p| p.len());
        paths.dedup();
        Ok(paths)
    }

    /// Find the shortest path between two domains (O(1) lookup).
    pub fn shortest_path(&self, from: Domain, to: Domain) -> WheelResult<Vec<Domain>> {
        self.shortest_paths[from.index()][to.index()]
            .clone()
            .ok_or(WheelError::NoPath(from, to))
    }

    /// Get the node metadata for a domain.
    pub fn node(&self, domain: Domain) -> Node {
        let idx = domain.index();
        Node {
            domain,
            symbol: domain.symbol(),
            name: domain.full_name(),
            description: domain.archetype(),
            index: idx,
            opposite: domain.opposite(),
            trines: domain.trines(),
        }
    }

    /// Get all nodes with their metadata.
    pub fn all_nodes(&self) -> Vec<Node> {
        ALL_DOMAINS.iter().map(|&d| self.node(d)).collect()
    }

    /// Get the structural composition relationship between two domains.
    #[inline]
    pub fn aspect_between(&self, a: Domain, b: Domain) -> CompositionAspect {
        CompositionAspect::between(a, b)
    }

    /// Check if a direct edge exists between two domains (O(1) lookup).
    #[inline]
    pub fn has_edge(&self, from: Domain, to: Domain) -> bool {
        EDGE_TABLE[from.index()][to.index()]
    }

    /// Describe the relationship between two domains.
    pub fn describe_relationship(&self, from: Domain, to: Domain) -> String {
        let rel = self.relationship(from, to);
        let nature = if rel.aspect.is_direct() {
            "direct flow"
        } else {
            "requires mediation"
        };
        format!(
            "{} {} → {}: {} (distance {}, {})",
            from.symbol(),
            from.full_name(),
            to.full_name(),
            rel.aspect,
            rel.distance,
            nature
        )
    }

    /// Render the Vedic wheel as an ASCII diagram.
    pub fn render_wheel(&self) -> String {
        let mut s = String::new();
        s.push_str("            ☉ Surya\n");
        s.push_str("          /        \\\n");
        s.push_str("  ☋ Ketu            ☽ Chandra\n");
        s.push_str(" /                      \\\n");
        s.push_str("☊ Rahu                 ♂ Mangala\n");
        s.push_str("|                        |\n");
        s.push_str("♄ Shani                ☿ Budha\n");
        s.push_str(" \\                      /\n");
        s.push_str("  ♀ Shukra ——— ♃ Brihaspati\n");
        s.push('\n');
        s.push_str("9 Vedic Grahas at 40° intervals:\n");
        s.push_str("  0°  ☉ Surya     — Self & Leadership\n");
        s.push_str("  40° ☽ Chandra   — Mind & Emotion\n");
        s.push_str("  80° ♂ Mangala   — Action & Engineering\n");
        s.push_str("  120°☿ Budha     — Logic & Communication\n");
        s.push_str("  160°♃ Brihaspati — Wisdom & Law\n");
        s.push_str("  200°♀ Shukra    — Arts & Value\n");
        s.push_str("  240°♄ Shani     — Structure & Time\n");
        s.push_str("  280°☊ Rahu      — Innovation & Tech\n");
        s.push_str("  320°☋ Ketu      — Spirituality & Science\n");
        s.push('\n');
        s.push_str("Aspects (9-node wheel):\n");
        s.push_str("  Conjunction (0 steps):    self\n");
        s.push_str("  Sextile   (1 step, 40°):  adjacent flow\n");
        s.push_str("  Square    (2 steps, 80°): tension\n");
        s.push_str("  Trine     (3 steps, 120°): harmonic\n");
        s.push_str("  Opposition(4 steps, 160°): full aspect\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_index_roundtrip() {
        for d in &ALL_DOMAINS {
            assert_eq!(Domain::from_index(d.index()), Some(*d));
        }
    }

    #[test]
    fn test_opposite_is_4_steps() {
        for d in &ALL_DOMAINS {
            let opp = d.opposite();
            let diff = (d.index() as isize - opp.index() as isize).abs();
            assert!(diff == 4 || diff == 5);
        }
    }

    #[test]
    fn test_parse_variants() {
        assert_eq!(Domain::parse("surya"), Some(Domain::Surya));
        assert_eq!(Domain::parse("☉"), Some(Domain::Surya));
        assert_eq!(Domain::parse("sun"), Some(Domain::Surya));
        assert_eq!(Domain::parse("mangala"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("MARS"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("rahu"), Some(Domain::Rahu));
        assert_eq!(Domain::parse("unknown"), None);
    }

    #[test]
    fn test_greek_aliases() {
        assert_eq!(Domain::parse("alpha"), Some(Domain::Surya));
        assert_eq!(Domain::parse("β"), Some(Domain::Chandra));
        assert_eq!(Domain::parse("gamma"), Some(Domain::Mangala));
        assert_eq!(Domain::parse("δ"), Some(Domain::Budha));
        assert_eq!(Domain::parse("epsilon"), Some(Domain::Brihaspati));
        assert_eq!(Domain::parse("zeta"), Some(Domain::Shukra));
        assert_eq!(Domain::parse("η"), Some(Domain::Shani));
        assert_eq!(Domain::parse("theta"), Some(Domain::Rahu));
        assert_eq!(Domain::parse("ι"), Some(Domain::Ketu));
    }

    #[test]
    fn test_greek_names() {
        assert_eq!(Domain::Surya.greek_name(), "Alpha (α)");
        assert_eq!(Domain::Chandra.greek_name(), "Beta (β)");
        assert_eq!(Domain::Mangala.greek_name(), "Gamma (γ)");
        assert_eq!(Domain::Budha.greek_name(), "Delta (δ)");
        assert_eq!(Domain::Brihaspati.greek_name(), "Epsilon (ε)");
        assert_eq!(Domain::Shukra.greek_name(), "Zeta (ζ)");
        assert_eq!(Domain::Shani.greek_name(), "Eta (η)");
        assert_eq!(Domain::Rahu.greek_name(), "Theta (θ)");
        assert_eq!(Domain::Ketu.greek_name(), "Iota (ι)");
    }

    #[test]
    fn test_domain_format() {
        let s = format!("{}", Domain::Surya);
        assert!(s.contains("☉"));
        assert!(s.contains("Surya"));
    }

    #[test]
    fn test_adjacent() {
        let adj = Domain::Surya.adjacent();
        assert_eq!(adj[0], Domain::Chandra);
        assert_eq!(adj[1], Domain::Ketu);
    }

    #[test]
    fn test_trines() {
        let trines = Domain::Surya.trines();
        assert_eq!(trines[0], Domain::Budha);
        assert_eq!(trines[1], Domain::Shani);
    }

    #[test]
    fn test_node_metadata() {
        let nodes = compute_all_nodes();
        assert_eq!(nodes.len(), 9);
        for n in &nodes {
            assert!(!n.symbol.is_empty());
            assert!(!n.name.is_empty());
            assert!(!n.description.is_empty());
        }
    }

    #[test]
    fn test_composition_aligned_self() {
        assert_eq!(
            CompositionAspect::between(Domain::Mangala, Domain::Mangala),
            CompositionAspect::Aligned
        );
    }

    #[test]
    fn test_composition_adjacent() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Chandra),
            CompositionAspect::Adjacent
        );
        assert_eq!(
            CompositionAspect::between(Domain::Ketu, Domain::Surya),
            CompositionAspect::Adjacent
        );
    }

    #[test]
    fn test_composition_antipodal() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Brihaspati),
            CompositionAspect::Antipodal
        );
        assert_eq!(
            CompositionAspect::between(Domain::Brihaspati, Domain::Surya),
            CompositionAspect::Antipodal
        );
    }

    #[test]
    fn test_composition_harmonic() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Budha),
            CompositionAspect::Harmonic
        );
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Shani),
            CompositionAspect::Harmonic
        );
    }

    #[test]
    fn test_composition_tense() {
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Mangala),
            CompositionAspect::Tense
        );
        assert_eq!(
            CompositionAspect::between(Domain::Surya, Domain::Rahu),
            CompositionAspect::Tense
        );
    }

    #[test]
    fn test_relationship_creation() {
        let r = Relationship::new(Domain::Surya, Domain::Chandra);
        assert_eq!(r.aspect, CompositionAspect::Adjacent);
        assert_eq!(r.distance, 1);
    }

    #[test]
    fn test_direct_and_tension() {
        assert!(CompositionAspect::Aligned.is_direct());
        assert!(CompositionAspect::Adjacent.is_direct());
        assert!(!CompositionAspect::Tense.is_direct());
        assert!(!CompositionAspect::Antipodal.is_direct());
        assert!(CompositionAspect::Tense.is_tension());
        assert!(CompositionAspect::Antipodal.is_tension());
    }

    #[test]
    fn test_kindergarten_default() {
        let axis = UnderstandingAxis::kindergarten();
        assert_eq!(axis.level, 0);
        assert_eq!(axis.cycle, 0);
        assert_eq!(axis.depth(), 0);
    }

    #[test]
    fn test_level_name() {
        assert_eq!(UnderstandingAxis::new(0, 0).level_name(), "Kindergarten");
        assert_eq!(UnderstandingAxis::new(6, 0).level_name(), "Grade 6");
        assert_eq!(UnderstandingAxis::new(12, 0).level_name(), "Grade 12");
    }

    #[test]
    fn test_band_detection() {
        assert_eq!(UnderstandingAxis::new(0, 0).band(), "Foundation");
        assert_eq!(UnderstandingAxis::new(2, 0).band(), "Elementary");
        assert_eq!(UnderstandingAxis::new(5, 0).band(), "Intermediate");
        assert_eq!(UnderstandingAxis::new(8, 0).band(), "Advanced");
        assert_eq!(UnderstandingAxis::new(11, 0).band(), "Mastery");
    }

    #[test]
    fn test_advance_loops_to_next_cycle() {
        let mut axis = UnderstandingAxis::new(12, 0);
        assert_eq!(axis.depth(), 12);
        axis.advance();
        assert_eq!(axis.level, 0);
        assert_eq!(axis.cycle, 1);
        assert_eq!(axis.depth(), 13);
    }

    #[test]
    fn test_advance_by_multiple_steps() {
        let mut axis = UnderstandingAxis::new(11, 0);
        axis.advance_by(3);
        assert_eq!(axis.level, 1);
        assert_eq!(axis.cycle, 1);
        assert_eq!(axis.depth(), 14);
    }

    #[test]
    fn test_next_without_mutation() {
        let axis = UnderstandingAxis::new(5, 0);
        let next = axis.next();
        assert_eq!(axis.level, 5);
        assert_eq!(next.level, 6);
    }

    #[test]
    fn test_depth_ordering() {
        let l12c0 = UnderstandingAxis::new(12, 0).depth();
        let l0c1 = UnderstandingAxis::new(0, 1).depth();
        assert!(l12c0 < l0c1);
    }

    #[test]
    fn test_band_level_ranges() {
        assert!(CurriculumBand::Foundation.levels().contains(&0));
        assert!(CurriculumBand::Elementary.levels().contains(&2));
        assert!(CurriculumBand::Intermediate.levels().contains(&5));
        assert!(CurriculumBand::Advanced.levels().contains(&8));
        assert!(CurriculumBand::Mastery.levels().contains(&12));
    }

    #[test]
    fn test_describe() {
        let axis = UnderstandingAxis::new(0, 0);
        assert_eq!(
            axis.describe(),
            "Cycle 0 Kindergarten (Foundation) — Unknown (0)"
        );
        let axis = UnderstandingAxis::new(7, 2);
        assert_eq!(axis.describe(), "Cycle 2 Grade 7 (Advanced) — Known (12)");
    }

    #[test]
    fn test_layer_index_mapping() {
        assert_eq!(
            UnderstandingAxis::new(0, 0).layer_index(),
            MasteryLayer::Unknown
        );
        assert_eq!(
            UnderstandingAxis::new(3, 0).layer_index(),
            MasteryLayer::Aware
        );
        assert_eq!(
            UnderstandingAxis::new(6, 0).layer_index(),
            MasteryLayer::Learning
        );
        assert_eq!(
            UnderstandingAxis::new(12, 0).layer_index(),
            MasteryLayer::Known
        );
        assert_eq!(MasteryLayer::from_level(10), MasteryLayer::Known);
    }

    #[test]
    fn test_describe_layer() {
        let axis = UnderstandingAxis::new(6, 0);
        assert_eq!(axis.describe_layer(), "Learning (6) — Level 6/12");
        let axis = UnderstandingAxis::new(12, 0);
        assert_eq!(axis.describe_layer(), "Known (12) — Level 12/12");
    }

    #[test]
    fn test_graph_creation() {
        let g = WheelGraph::new();
        let nodes = g.all_nodes();
        assert_eq!(nodes.len(), 9);
    }

    #[test]
    fn test_graph_has_self_edges() {
        let g = WheelGraph::new();
        assert!(g.has_edge(Domain::Mangala, Domain::Mangala));
    }

    #[test]
    fn test_graph_has_adjacent_edges() {
        let g = WheelGraph::new();
        assert!(g.has_edge(Domain::Surya, Domain::Chandra));
        assert!(g.has_edge(Domain::Chandra, Domain::Mangala));
    }

    #[test]
    fn test_graph_has_trines() {
        let g = WheelGraph::new();
        assert!(g.has_edge(Domain::Mangala, Domain::Shukra));
        assert!(g.has_edge(Domain::Mangala, Domain::Ketu));
    }

    #[test]
    fn test_shortest_path_adjacent() {
        let g = WheelGraph::new();
        let path = g.shortest_path(Domain::Surya, Domain::Chandra).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], Domain::Surya);
        assert_eq!(path[1], Domain::Chandra);
    }

    #[test]
    fn test_shortest_path_opposite() {
        let g = WheelGraph::new();
        let path = g.shortest_path(Domain::Surya, Domain::Brihaspati).unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], Domain::Surya);
        assert_eq!(path[1], Domain::Brihaspati);
    }

    #[test]
    fn test_shortest_path_complex() {
        let g = WheelGraph::new();
        let path = g.shortest_path(Domain::Mangala, Domain::Budha);
        assert!(path.is_ok());
    }

    #[test]
    fn test_neighbors_filtered() {
        let g = WheelGraph::new();
        let trine_neighbors = g.neighbors(Domain::Mangala, Some(CompositionAspect::Harmonic));
        assert_eq!(trine_neighbors.len(), 2);
        assert!(trine_neighbors.contains(&Domain::Shukra));
        assert!(trine_neighbors.contains(&Domain::Ketu));
    }

    #[test]
    fn test_describe_relationship() {
        let g = WheelGraph::new();
        let desc = g.describe_relationship(Domain::Surya, Domain::Chandra);
        assert!(!desc.is_empty());
        assert!(desc.contains("Adjacent"));
    }

    #[test]
    fn test_render_wheel() {
        let g = WheelGraph::new();
        let rendered = g.render_wheel();
        assert!(rendered.contains("Surya"));
        assert!(rendered.contains("Ketu"));
        assert!(rendered.contains("Grahas"));
        assert!(rendered.contains("Aspects"));
    }

    #[test]
    fn test_find_paths_max_depth() {
        let g = WheelGraph::new();
        assert!(g.find_paths(Domain::Surya, Domain::Chandra, 0).is_err());
        let paths = g.find_paths(Domain::Surya, Domain::Chandra, 3).unwrap();
        assert!(!paths.is_empty());
    }

    #[test]
    fn test_no_path_same() {
        let g = WheelGraph::new();
        let paths = g.find_paths(Domain::Mangala, Domain::Mangala, 5).unwrap();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0], vec![Domain::Mangala]);
    }
}
