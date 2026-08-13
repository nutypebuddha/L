//! Benchmark: Shikai query processing performance.
//!
//! Measures intent identification, domain inference, grammar rule matching,
//! and formula search end-to-end for various query types.
//!
//! Note: Only benchmarks the public `process()` API, not internal helpers
//! like `infer_domain_from_grammar` (those have dedicated unit tests).

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use athena::formula::FormulaRegistry;
use athena::shikai::Shikai;
use athena::zanpakuto::{AccessTier, NlpContext, Zanpakuto};

fn build_bench_shikai() -> (Shikai, Zanpakuto, String) {
    // Load real formulas from disk (same as CLI does)
    let mut registry = FormulaRegistry::new();
    let formula_dirs = [
        "formulas/atomic",
        "formulas/bridging",
        "formulas/vortex",
        "formulas/nonmath",
    ];
    for dir in &formula_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "toml") {
                    let _ = registry.load_from_file(&path);
                }
            }
        }
    }

    let entities = athena::entity::EntityRegistry::new();
    let shikai = Shikai::with_entities(registry, entities);
    let mut zanpakuto = Zanpakuto::new();
    let id = zanpakuto.register("bench", AccessTier::Shikai);
    (shikai, zanpakuto, id.session.clone())
}

fn bench_shikai(c: &mut Criterion) {
    let (shikai, mut zanpakuto, _session) = build_bench_shikai();
    let id = zanpakuto.register("bench", AccessTier::Shikai);

    let mut group = c.benchmark_group("shikai_processing");

    group.bench_function("simple_math_query", |b| {
        b.iter(|| {
            let result = shikai.process(
                black_box("force mass=5 acceleration=9.8"),
                &id,
                None::<&NlpContext>,
            );
            black_box(result.is_ok())
        })
    });

    group.bench_function("natural_language_query", |b| {
        b.iter(|| {
            let result = shikai.process(
                black_box("calculate the momentum of a 5kg mass moving at 10m/s"),
                &id,
                None::<&NlpContext>,
            );
            black_box(result.is_ok())
        })
    });

    group.bench_function("domain_mention_query", |b| {
        b.iter(|| {
            let result = shikai.process(
                black_box("traverse from surya to budha"),
                &id,
                None::<&NlpContext>,
            );
            black_box(result.is_ok())
        })
    });

    group.bench_function("grammar_inference_query", |b| {
        b.iter(|| {
            let result = shikai.process(black_box("serial comma rule"), &id, None::<&NlpContext>);
            black_box(result.is_ok())
        })
    });

    group.finish();
}

criterion_group!(benches, bench_shikai);
criterion_main!(benches);
