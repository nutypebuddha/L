//! Benchmark: formula registry search performance.
//!
//! Measures the O(1) word-indexed search against the O(N×fields) linear-scan
//! fallback. With the word index, single-token lookups should be ~100× faster.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use athena::formula::{Formula, FormulaRegistry};
use athena::wheel::Domain;

fn build_bench_registry() -> FormulaRegistry {
    let mut r = FormulaRegistry::new();
    // Register a mix of Vedic-aligned formulas similar to the real dataset
    //
    // Domain mappings (Western sign → Vedic graha):
    //   Taurus (physics)  → Shukra
    //   Aries (math)      → Mangala
    //   Scorpio (CS)      → Rahu
    //   Virgo (econ)      → Budha
    //   Libra (eng)       → Shukra
    let formulas = vec![
        Formula::atomic(
            "newtons_second",
            Domain::Shukra,
            vec!["mass", "acceleration"],
            "force",
            "mass * acceleration",
            "F = ma",
        ),
        Formula::atomic(
            "pythagorean",
            Domain::Mangala,
            vec!["a", "b"],
            "c",
            "sqrt(a^2 + b^2)",
            "a² + b² = c²",
        ),
        Formula::atomic(
            "momentum",
            Domain::Shukra,
            vec!["mass", "velocity"],
            "momentum",
            "mass * velocity",
            "p = mv",
        ),
        Formula::atomic(
            "ke",
            Domain::Mangala,
            vec!["mass", "velocity"],
            "energy",
            "0.5 * mass * velocity^2",
            "KE = ½mv²",
        ),
        Formula::atomic(
            "hookes_law",
            Domain::Shukra,
            vec!["k", "x"],
            "force",
            "k * x",
            "F = -kx",
        ),
        Formula::atomic(
            "pressure",
            Domain::Shukra,
            vec!["force", "area"],
            "pressure",
            "force / area",
            "P = F/A",
        ),
        Formula::atomic(
            "work",
            Domain::Shukra,
            vec!["force", "distance"],
            "work",
            "force * distance",
            "W = Fd",
        ),
        Formula::atomic(
            "ohms_law",
            Domain::Rahu,
            vec!["v", "r"],
            "current",
            "v / r",
            "I = V/R",
        ),
        Formula::atomic(
            "power",
            Domain::Rahu,
            vec!["v", "i"],
            "power",
            "v * i",
            "P = VI",
        ),
        Formula::atomic(
            "momentum_to_ke",
            Domain::Shukra,
            vec!["mass", "velocity"],
            "energy",
            "0.5 * mass * velocity^2",
            "Momentum to kinetic energy",
        ),
        Formula::atomic(
            "work_energy",
            Domain::Shukra,
            vec!["work"],
            "energy",
            "work",
            "Work-Energy Theorem",
        ),
        Formula::atomic(
            "ohms_power",
            Domain::Rahu,
            vec!["v", "i"],
            "power",
            "v * i",
            "Ohm's law to power",
        ),
        Formula::atomic(
            "compound_interest",
            Domain::Budha,
            vec!["principal", "rate", "time"],
            "amount",
            "principal * (1 + rate)^time",
            "A = P(1+r)^t",
        ),
        Formula::atomic(
            "area_circle",
            Domain::Mangala,
            vec!["radius"],
            "area",
            "pi * radius^2",
            "A = πr²",
        ),
        Formula::atomic(
            "force_gravity",
            Domain::Shukra,
            vec!["mass1", "mass2", "distance"],
            "force",
            "6.674e-11 * mass1 * mass2 / distance^2",
            "F = Gm₁m₂/r²",
        ),
    ];
    r.register_all(formulas).unwrap();
    r
}

fn bench_word_index_search(c: &mut Criterion) {
    let r = build_bench_registry();
    let mut group = c.benchmark_group("formula_search");

    group.bench_function("word_index_exact_match", |b| {
        b.iter(|| {
            let results = r.search(black_box("momentum"));
            black_box(results.len())
        })
    });

    group.bench_function("word_index_common_word", |b| {
        b.iter(|| {
            let results = r.search(black_box("mass"));
            black_box(results.len())
        })
    });

    group.bench_function("word_index_partial_substring", |b| {
        // "calc" is a partial match for "calculation" → falls to linear scan
        b.iter(|| {
            let results = r.search(black_box("calc"));
            black_box(results.len())
        })
    });

    group.bench_function("word_index_multi_token", |b| {
        // Multi-token query → falls to linear scan
        b.iter(|| {
            let results = r.search(black_box("force mass"));
            black_box(results.len())
        })
    });

    group.finish();
}

criterion_group!(benches, bench_word_index_search);
criterion_main!(benches);
