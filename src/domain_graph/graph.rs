use std::collections::{HashMap, HashSet, VecDeque};

use super::edges::{CompositionAspect, Relationship};
use super::nodes::{Domain, ALL_DOMAINS};
use super::{Node, WheelError, WheelResult};

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
