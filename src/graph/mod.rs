//! Graph primitives — T47: Dijkstra, Kruskal, max-flow via `petgraph`.
//!
//! All three algorithms are standard textbook implementations backed by
//! `petgraph`. The critical determinism invariant: **petgraph's
//! `dijkstra`/`bellman_ford` return `HashMap<NodeId, K>` whose iteration
//! order is nondeterministic.** Every consumer of these results must sort
//! by a stable key before any output formatting or aggregation step.
//!
//! This module re-exports the results in deterministic (sorted) order.

use petgraph::algo::{dijkstra as pet_dijkstra, dinics, min_spanning_tree};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, Graph};

// ── Dijkstra ─────────────────────────────────────────────────────────────────

/// A single shortest-path result: source → target with cost.
#[derive(Debug, Clone, PartialEq)]
pub struct ShortestPathResult {
    pub source: usize,
    pub target: usize,
    pub cost: f64,
    pub path: Vec<usize>,
}

/// Compute shortest paths from `source` to all reachable nodes using Dijkstra.
/// Edge weights must be non-negative.
///
/// Returns results sorted by (target node ID) for determinism — this is the
/// critical fix for the T47 HashMap-ordering issue.
pub fn shortest_paths<N, E, F>(
    graph: &Graph<N, E, Directed>,
    source: NodeIndex,
    weight_fn: F,
) -> Vec<ShortestPathResult>
where
    N: core::fmt::Debug,
    E: core::fmt::Debug,
    F: Fn(&E) -> f64,
{
    let distances = pet_dijkstra(graph, source, None, |edge| weight_fn(edge.weight()));

    let mut results: Vec<ShortestPathResult> = distances
        .iter()
        .map(|(&target_idx, &cost)| {
            let path = reconstruct_path(
                graph,
                source,
                target_idx,
                |idx| distances.get(&idx).copied(),
                &weight_fn,
            );
            ShortestPathResult {
                source: source.index(),
                target: target_idx.index(),
                cost,
                path: path.into_iter().map(|n| n.index()).collect(),
            }
        })
        .collect();

    results.sort_by_key(|r| r.target);
    results
}

/// Find the shortest path from `source` to a specific `target`.
pub fn shortest_path_to<N, E, F>(
    graph: &Graph<N, E, Directed>,
    source: NodeIndex,
    target: NodeIndex,
    weight_fn: F,
) -> Option<ShortestPathResult>
where
    N: core::fmt::Debug,
    E: core::fmt::Debug,
    F: Fn(&E) -> f64,
{
    let all = shortest_paths(graph, source, weight_fn);
    all.into_iter().find(|r| r.target == target.index())
}

fn reconstruct_path<N, E, F>(
    graph: &Graph<N, E, Directed>,
    source: NodeIndex,
    target: NodeIndex,
    dist_lookup: impl Fn(NodeIndex) -> Option<f64>,
    weight_fn: F,
) -> Vec<NodeIndex>
where
    N: core::fmt::Debug,
    E: core::fmt::Debug,
    F: Fn(&E) -> f64,
{
    if source == target {
        return vec![source];
    }

    let mut path = vec![target];
    let mut current = target;

    while current != source {
        let mut found = false;
        for edge in graph.edges_directed(current, petgraph::Direction::Incoming) {
            let predecessor = edge.source();
            let edge_weight = weight_fn(edge.weight());
            if let (Some(prev_dist), Some(curr_dist)) =
                (dist_lookup(predecessor), dist_lookup(current))
            {
                if (prev_dist + edge_weight - curr_dist).abs() < 1e-9 {
                    path.push(predecessor);
                    current = predecessor;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            break;
        }
    }

    path.reverse();
    path
}

// ── Kruskal (Minimum Spanning Tree) ────────────────────────────────────────

/// An edge in the MST, with sorted node indices (smaller first).
#[derive(Debug, Clone, PartialEq)]
pub struct MstEdge {
    pub from: usize,
    pub to: usize,
    pub weight: f64,
}

/// Compute the minimum spanning tree using Kruskal's algorithm.
/// Returns edges sorted by (weight, then from, then to) for determinism.
pub fn compute_minimum_spanning_tree<N, E, F>(
    graph: &Graph<N, E, Directed>,
    weight_fn: F,
) -> Vec<MstEdge>
where
    N: core::fmt::Debug + Clone,
    E: core::fmt::Debug + Clone + PartialOrd,
    F: Fn(&E) -> f64,
{
    // min_spanning_tree treats the graph as undirected and returns elements
    // with the original edge type.
    let mst_elements: Vec<_> = min_spanning_tree(graph).collect();

    let mut result: Vec<MstEdge> = Vec::new();
    for element in mst_elements {
        if let petgraph::data::Element::Edge {
            source,
            target,
            weight,
        } = element
        {
            let from = source;
            let to = target;
            let w = weight_fn(&weight);
            let (sorted_from, sorted_to) = if from <= to { (from, to) } else { (to, from) };
            result.push(MstEdge {
                from: sorted_from,
                to: sorted_to,
                weight: w,
            });
        }
    }

    result.sort_by(|a, b| {
        a.weight
            .partial_cmp(&b.weight)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.from.cmp(&b.from))
            .then_with(|| a.to.cmp(&b.to))
    });

    result
}

// ── Max Flow (Dinic's algorithm) ───────────────────────────────────────────

/// A flow edge in the result.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowEdge {
    pub from: usize,
    pub to: usize,
    pub flow: f64,
    pub capacity: f64,
}

/// Compute maximum flow from `source` to `sink` using Dinic's algorithm.
/// Returns the max flow value and the per-edge flows, sorted deterministically.
pub fn compute_max_flow<N, E, F>(
    graph: &Graph<N, E, Directed>,
    source: NodeIndex,
    sink: NodeIndex,
    capacity_fn: F,
) -> (f64, Vec<FlowEdge>)
where
    N: core::fmt::Debug,
    E: core::fmt::Debug,
    F: Fn(&E) -> f64,
{
    let mut cap_graph =
        Graph::<(), f64, Directed>::with_capacity(graph.node_count(), graph.edge_count());
    for _ in graph.node_indices() {
        cap_graph.add_node(());
    }
    for edge in graph.edge_indices() {
        if let Some((s, t)) = graph.edge_endpoints(edge) {
            let cap = capacity_fn(&graph[edge]);
            cap_graph.add_edge(s, t, cap);
        }
    }

    let (flow_value, edge_flows) = dinics(&cap_graph, source, sink);

    let mut flow_edges: Vec<FlowEdge> = Vec::new();
    for (i, edge) in graph.edge_indices().enumerate() {
        if let Some((s, t)) = graph.edge_endpoints(edge) {
            let capacity = capacity_fn(&graph[edge]);
            let flow = edge_flows[i];
            flow_edges.push(FlowEdge {
                from: s.index(),
                to: t.index(),
                flow,
                capacity,
            });
        }
    }

    flow_edges.sort_by(|a, b| a.from.cmp(&b.from).then_with(|| a.to.cmp(&b.to)));

    (flow_value, flow_edges)
}

/// Compute max flow and return just the value.
pub fn compute_max_flow_value<N, E, F>(
    graph: &Graph<N, E, Directed>,
    source: NodeIndex,
    sink: NodeIndex,
    capacity_fn: F,
) -> f64
where
    N: core::fmt::Debug,
    E: core::fmt::Debug,
    F: Fn(&E) -> f64,
{
    compute_max_flow(graph, source, sink, capacity_fn).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::DiGraph;

    fn test_graph() -> DiGraph<String, f64> {
        let mut g = DiGraph::new();
        let a = g.add_node("A".into());
        let b = g.add_node("B".into());
        let c = g.add_node("C".into());
        let d = g.add_node("D".into());

        g.add_edge(a, b, 1.0);
        g.add_edge(a, c, 4.0);
        g.add_edge(b, c, 2.0);
        g.add_edge(b, d, 6.0);
        g.add_edge(c, d, 3.0);

        g
    }

    #[test]
    fn dijkstra_finds_shortest_path() {
        let g = test_graph();
        let source = NodeIndex::new(0);
        let target = NodeIndex::new(3);

        let result = shortest_path_to(&g, source, target, |w| *w).unwrap();
        assert_eq!(result.cost, 6.0);
        assert_eq!(result.path, vec![0, 1, 2, 3]);
    }

    #[test]
    fn dijkstra_determinism() {
        let g = test_graph();
        let source = NodeIndex::new(0);

        let r1 = shortest_paths(&g, source, |w| *w);
        let r2 = shortest_paths(&g, source, |w| *w);
        assert_eq!(r1, r2);
    }

    #[test]
    fn dijkstra_sorted_output() {
        let g = test_graph();
        let source = NodeIndex::new(0);
        let results = shortest_paths(&g, source, |w| *w);

        for window in results.windows(2) {
            assert!(window[0].target <= window[1].target);
        }
    }

    #[test]
    fn mst_basic() {
        let g = test_graph();
        let mst = compute_minimum_spanning_tree(&g, |w| *w);

        assert_eq!(mst.len(), 3);
        let total: f64 = mst.iter().map(|e| e.weight).sum();
        assert!((total - 6.0).abs() < 1e-9);
    }

    #[test]
    fn mst_determinism() {
        let g = test_graph();
        let r1 = compute_minimum_spanning_tree(&g, |w| *w);
        let r2 = compute_minimum_spanning_tree(&g, |w| *w);
        assert_eq!(r1, r2);
    }

    #[test]
    fn max_flow_basic() {
        let mut g = DiGraph::new();
        let s = g.add_node("S");
        let a = g.add_node("A");
        let b = g.add_node("B");
        let t = g.add_node("T");

        g.add_edge(s, a, 10.0);
        g.add_edge(s, b, 10.0);
        g.add_edge(a, b, 2.0);
        g.add_edge(a, t, 10.0);
        g.add_edge(b, t, 10.0);

        let flow = compute_max_flow_value(&g, s, t, |w| *w);
        assert!((flow - 20.0).abs() < 1e-9);
    }

    #[test]
    fn max_flow_determinism() {
        let mut g = DiGraph::new();
        let s = g.add_node("S");
        let a = g.add_node("A");
        let b = g.add_node("B");
        let t = g.add_node("T");

        g.add_edge(s, a, 10.0);
        g.add_edge(s, b, 10.0);
        g.add_edge(a, t, 10.0);
        g.add_edge(b, t, 10.0);

        let (v1, e1) = compute_max_flow(&g, s, t, |w| *w);
        let (v2, e2) = compute_max_flow(&g, s, t, |w| *w);
        assert_eq!(v1, v2);
        assert_eq!(e1, e2);
    }
}
