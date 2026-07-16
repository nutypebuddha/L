//! End-to-end regression tests for the `strategize` command (T59/T60).
//!
//! Exercises the real CLI so the full wired pipeline
//! (reverse-route → 7-pillar aggregation → deterministic Pareto allocation)
//! is covered: determinism, the 7-pillar structure, and the fail-loud
//! "speculative" flag when routing confidence is too low to trust.

use std::process::Command;

use serde_json::Value;

fn strategize_json(query: &str, budget: &str) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_laverna"))
        .args([
            "strategize",
            "--query",
            query,
            "--budget",
            budget,
            "--top-k",
            "3",
            "--format",
            "json",
        ])
        .output()
        .expect("spawn laverna");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("stdout is utf-8");
    serde_json::from_str(&stdout).expect("strategize output is JSON")
}

#[test]
fn strategize_is_deterministic() {
    let query = "how do I build a resilient distributed system with strong guarantees?";
    let a = strategize_json(query, "20");
    let b = strategize_json(query, "20");
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap(),
        "strategize output must be byte-deterministic for a fixed query + budget"
    );
    assert!(a["allocations"].as_array().unwrap().len() >= 1);
    assert_eq!(a["pillars"].as_array().unwrap().len(), 7);
}

#[test]
fn strategize_emits_seven_pillars_and_a_plan() {
    let v = strategize_json("how should we architect for safety and scale?", "12");
    let pillars = v["pillars"].as_array().unwrap();
    assert_eq!(pillars.len(), 7);
    // Weights should sum to ~1.0 (normalized from the route).
    let total: f64 = pillars.iter().map(|p| p["weight"].as_f64().unwrap()).sum();
    assert!((total - 1.0).abs() < 1e-6, "pillar weights sum to {total}");

    let alloc = &v["allocations"].as_array().unwrap()[0];
    let levels = alloc["levels"].as_object().unwrap();
    let spent: u32 = levels.values().map(|l| l.as_u64().unwrap() as u32).sum();
    assert!(spent <= 12, "allocation spent {spent} > budget 12");
}

#[test]
fn strategize_low_confidence_is_speculative() {
    // Gibberish with no corpus graha → routing warning → speculative flag.
    let v = strategize_json("zqxwkj vbnmrt lkjhgf qplzmw", "7");
    assert_eq!(
        v["speculative"],
        Value::Bool(true),
        "low-confidence routing must be flagged speculative"
    );
}
