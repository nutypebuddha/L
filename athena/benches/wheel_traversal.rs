//! Benchmark: wheel graph traversal performance.
//!
//! Measures shortest-path finding, aspect computation, and node lookup
//! across the 9-node Vedic graha wheel.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use athena::wheel::{Domain, WheelGraph};

fn bench_wheel(c: &mut Criterion) {
    let wheel = WheelGraph::new();

    let mut group = c.benchmark_group("wheel_traversal");

    // On 9-node Vedic wheel: Surya (0) ↔ Chandra (1) are adjacent (1 step = 40°)
    group.bench_function("shortest_path_adjacent", |b| {
        b.iter(|| {
            let path = wheel.shortest_path(black_box(Domain::Surya), black_box(Domain::Chandra));
            black_box(path.map(|p| p.len()).unwrap_or(0))
        })
    });

    // Surya (0) ↔ Brihaspati (4) are opposite (4 steps = 160°)
    group.bench_function("shortest_path_opposite", |b| {
        b.iter(|| {
            let path = wheel.shortest_path(black_box(Domain::Surya), black_box(Domain::Brihaspati));
            black_box(path.map(|p| p.len()).unwrap_or(0))
        })
    });

    // Surya (0) ↔ Mangala (2): no direct edge (distance 2 = square)
    // Path goes through Chandra (0→1→2, 3 hops)
    group.bench_function("shortest_path_two_apart", |b| {
        b.iter(|| {
            let path = wheel.shortest_path(black_box(Domain::Surya), black_box(Domain::Mangala));
            black_box(path.map(|p| p.len()).unwrap_or(0))
        })
    });

    // Surya (0) ↔ Chandra (1) = Sextile
    group.bench_function("aspect_between_adjacent", |b| {
        b.iter(|| {
            let aspect = wheel.aspect_between(black_box(Domain::Surya), black_box(Domain::Chandra));
            black_box(aspect)
        })
    });

    // Surya (0) ↔ Brihaspati (4) = Opposition
    group.bench_function("aspect_between_opposite", |b| {
        b.iter(|| {
            let aspect =
                wheel.aspect_between(black_box(Domain::Surya), black_box(Domain::Brihaspati));
            black_box(aspect)
        })
    });

    // Also benchmark aspect between non-adjacent, non-opposite pair
    // Surya (0) ↔ Budha (3) = Trine (3 steps)
    group.bench_function("aspect_between_trine", |b| {
        b.iter(|| {
            let aspect = wheel.aspect_between(black_box(Domain::Surya), black_box(Domain::Budha));
            black_box(aspect)
        })
    });

    group.bench_function("all_nodes_iteration", |b| {
        b.iter(|| {
            let nodes = wheel.all_nodes();
            black_box(nodes.len())
        })
    });

    group.bench_function("node_lookup_by_domain", |b| {
        b.iter(|| {
            let node = wheel.node(black_box(Domain::Brihaspati));
            black_box(node.symbol)
        })
    });

    group.bench_function("precomputed_path_random_pairs", |b| {
        b.iter(|| {
            let mut total = 0usize;
            for a in [Domain::Surya, Domain::Mangala, Domain::Shani, Domain::Rahu] {
                for b in [Domain::Chandra, Domain::Budha, Domain::Shukra, Domain::Ketu] {
                    if let Ok(path) = wheel.shortest_path(black_box(a), black_box(b)) {
                        total += path.len();
                    }
                }
            }
            black_box(total)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_wheel);
criterion_main!(benches);
