//! # T58 — `optimize --top-k` Pareto tie-breaking must be deterministic (HIGH)
//!
//! When several allocations tie on the objective, the selection and ordering of
//! the top-k must NOT depend on `HashMap` iteration order (the T43 class). The
//! previous tiebreaker sorted by `format!("{:?}", a.levels)` where `levels` was
//! a `HashMap` — its Debug order is randomized per process, so a tied front
//! produced a different top-k membership *and* order on every run.
//!
//! This test builds a schema whose Pareto front is fully tied (every allocation
//! scores identically) and asserts the top-k is byte-identical across 100 runs.
//! It also locks the greedy-trap ground truth (optimizer must see through a
//! gate that a myopic marginal-value solver would miss).
//!
//! Run: `cargo test --test optimizer_determinism`

use laverna::optimize::{parse_schema, solve, Allocation, Schema};

/// Two attributes share one 20-point pool; each level yields +1 to its own
/// stat. Objective weights atk/def at 0.5/0.5, so `0.5*atk + 0.5*def` is
/// constant (= 10.0000) for every split → the entire candidate set sits on the
/// Pareto front and every point is tied. This is the worst case for a leaky
/// tiebreaker.
fn tied_schema() -> Schema {
    parse_schema(
        r#"
[meta]
domain = "tie_test"

[objective]
maximize = ["atk", "def"]
weights = { atk = 0.5, def = 0.5 }

[budget]
pts = 20

[[items]]
id = "att_a"
type = "attribute"
cost = { pts = 1 }
max_level = 20
effects = { atk = 1.0 }

[[items]]
id = "att_b"
type = "attribute"
cost = { pts = 1 }
max_level = 20
effects = { def = 1.0 }

[scoring.atk]
terms = { atk = 1.0 }

[scoring.def]
terms = { def = 1.0 }
"#,
    )
    .unwrap()
}

/// `att_b` pays 1.5 power/point; `att_a` pays only 1.0 — but `att_a >= 8` is
/// the sole gate to a perk worth 20. A myopic greedy solver dumps all 10
/// points into `att_b` (score 15) and never sees the gate. The true front
/// peaks at `att_a=8, att_b=2, gated_perk=1` → 31.0000.
fn greedy_trap_schema() -> Schema {
    parse_schema(
        r#"
[meta]
domain = "greedy_trap"

[objective]
maximize = ["power"]

[budget]
pts = 10

[[items]]
id = "att_a"
type = "attribute"
cost = { pts = 1 }
max_level = 10
effects = { a = 1.0 }

[[items]]
id = "att_b"
type = "attribute"
cost = { pts = 1 }
max_level = 10
effects = { b = 1.5 }

[[items]]
id = "gated_perk"
type = "perk"
requires = ["att_a>=8"]
cost = { pts = 0 }
max_level = 1
effects = { p = 20.0 }

[scoring.power]
terms = { a = 1.0, b = 1.0, p = 1.0 }
"#,
    )
    .unwrap()
}

/// Stable fingerprint of a solution set. `levels` is a `BTreeMap`, so it
/// iterates in sorted-key order regardless of insertion order — the fingerprint
/// is deterministic by construction and catches both membership and order drift.
fn fingerprint(sols: &[Allocation]) -> String {
    let mut s = String::new();
    for a in sols {
        s.push_str(&format!("{:.6}|", a.objective));
        for (k, v) in &a.levels {
            s.push_str(&format!("{}={}.", k, v));
        }
        s.push(';');
    }
    s
}

#[test]
fn top_k_is_deterministic_across_100_runs() {
    let schema = tied_schema();
    let first = solve(&schema, 4).unwrap();
    assert!(first.len() >= 4, "tied front must expose >=4 points");
    let reference = fingerprint(&first);
    for _ in 0..99 {
        let again = solve(&schema, 4).unwrap();
        assert_eq!(
            fingerprint(&again),
            reference,
            "top-k nondeterministic across runs"
        );
    }
}

#[test]
fn optimizer_sees_through_greedy_trap() {
    let schema = greedy_trap_schema();
    let sol = solve(&schema, 1).unwrap();
    assert_eq!(sol.len(), 1);
    let a = &sol[0];
    assert_eq!(a.levels.get("att_a").copied().unwrap_or(0), 8);
    assert_eq!(a.levels.get("att_b").copied().unwrap_or(0), 2);
    assert_eq!(a.levels.get("gated_perk").copied().unwrap_or(0), 1);
    assert!(
        (a.objective - 31.0).abs() < 1e-6,
        "expected objective 31.0, got {}",
        a.objective
    );

    // Stable across 100 runs (single-objective front is a singleton).
    let reference = fingerprint(&sol);
    for _ in 0..99 {
        let again = solve(&schema, 1).unwrap();
        assert_eq!(fingerprint(&again), reference);
    }
}
