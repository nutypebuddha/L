//! End-to-end regression tests for the `strategize` command (T59/T60).
//!
//! Exercises the real CLI so the full wired pipeline
//! (reverse-route → 7-pillar aggregation → deterministic Pareto allocation)
//! is covered: determinism, the 7-pillar structure, and the fail-loud
//! "speculative" flag when routing confidence is too low to trust.

use std::process::Command;

use serde_json::Value;

fn strategize_json(query: &str, budget: &str) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_lai"))
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
    assert!(!a["allocations"].as_array().unwrap().is_empty());
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

#[test]
fn strategize_zero_budget_returns_no_allocations() {
    // T-NEW-1: a 0 / sub-unit budget must yield an empty allocation list,
    // never a fabricated 1-unit spend.
    for budget in ["0.0", "0.01"] {
        let v = strategize_json("how do I build a resilient distributed system?", budget);
        assert!(
            v["allocations"].as_array().unwrap().is_empty(),
            "--budget {budget} must yield an empty allocation list"
        );
    }
}

#[test]
fn strategize_negative_budget_is_rejected() {
    // T-NEW-1: negative budgets are a clean CLI error (non-zero exit, no JSON
    // stdout), not a silent clamp.
    for budget in ["-5", "-0.01"] {
        let out = Command::new(env!("CARGO_BIN_EXE_lai"))
            .args([
                "strategize",
                "--query",
                "how do I build a resilient distributed system?",
                &format!("--budget={budget}"),
                "--format",
                "json",
            ])
            .output()
            .expect("spawn laverna");
        assert!(
            !out.status.success(),
            "--budget {budget} must be rejected, not accepted"
        );
        assert!(
            out.stdout.is_empty(),
            "--budget {budget} must not print JSON stdout"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("--budget must be >= 0"),
            "stderr must name the budget constraint, got: {stderr}"
        );
    }
}

#[test]
fn strategize_fractional_budget_never_exceeds() {
    // T-NEW-1: the integer allocation must never exceed the stated budget
    // (20.5 units may spend at most 20).
    let v = strategize_json(
        "how do I build a resilient distributed system with strong guarantees?",
        "20.5",
    );
    let allocs = v["allocations"].as_array().unwrap();
    assert!(!allocs.is_empty(), "positive budget must still allocate");
    for a in allocs {
        let levels = a["levels"].as_object().unwrap();
        let spent: u32 = levels.values().map(|l| l.as_u64().unwrap() as u32).sum();
        assert!(
            spent as f64 <= 20.5,
            "allocation spent {spent} > budget 20.5"
        );
    }
}

fn generate_json(text: &str, budget: &str, strict: bool) -> Value {
    let mut args = vec![
        "strategy", "generate", text, "--budget", budget, "--top-k", "3", "--format", "json",
    ];
    if strict {
        args.push("--strict");
    }
    let out = Command::new(env!("CARGO_BIN_EXE_lai"))
        .args(&args)
        .output()
        .expect("spawn laverna");
    let stdout = String::from_utf8(out.stdout).expect("stdout is utf-8");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "generate failed (status={:?}) stdout={stdout:?} stderr={stderr}",
        out.status.code()
    );
    serde_json::from_str(&stdout).expect("generate output is JSON")
}

#[test]
fn generate_unresolved_input_is_flagged_speculative() {
    // T-NEW-3: default mode on an input with no real corpus match must clearly
    // flag degraded confidence (speculative + warning), never silently emit a
    // bare "verified". This is the audit repro input.
    let v = generate_json(
        "A competitor just launched a cheaper product. Morale is low due to recent layoffs.",
        "20",
        false,
    );
    assert_eq!(
        v["speculative"],
        Value::Bool(true),
        "unresolved input must be flagged speculative"
    );
    assert!(
        v["warning"].is_string() && !v["warning"].as_str().unwrap().is_empty(),
        "speculative output must carry a non-null warning, got {:?}",
        v["warning"]
    );
    assert!(
        !v["unresolved_concepts"].as_array().unwrap().is_empty(),
        "unresolved concepts must be enumerated"
    );
    // T-NEW-2 gate: no phantom "due to" entity anywhere in the strategy IR or
    // evidence of a "verified" output.
    let ir = &v["strategy_ir"];
    assert_eq!(
        ir["entities"].as_object().map(|m| m.len()).unwrap_or(0),
        0,
        "strategy_ir.entities must be empty for this input, got {:?}",
        ir["entities"]
    );
    for s in v["strategies"].as_array().unwrap() {
        for ev in s["evidence"].as_array().unwrap() {
            assert!(
                !ev.as_str().unwrap().contains("disability_insurance"),
                "no spurious disability_insurance in evidence, got {ev:?}"
            );
        }
    }
}

#[test]
fn generate_strict_still_blocks_on_unresolved() {
    // T-NEW-3: --strict must still refuse on the same input (blocked_by_unknowns).
    let args = vec![
        "strategy",
        "generate",
        "A competitor just launched a cheaper product. Morale is low due to recent layoffs.",
        "--budget",
        "20",
        "--top-k",
        "3",
        "--format",
        "json",
        "--strict",
    ];
    let out = Command::new(env!("CARGO_BIN_EXE_lai"))
        .args(&args)
        .output()
        .expect("spawn laverna");
    assert!(out.status.success(), "strict refusal must still exit 0");
    let v: Value = serde_json::from_slice(&out.stdout).expect("strict output is JSON");
    assert_eq!(
        v["status"],
        Value::String("blocked_by_unknowns".to_string()),
        "--strict must block on unresolved concepts, got {:?}",
        v
    );
    assert_eq!(v["verified"], Value::Bool(false));
}

#[test]
fn generate_fully_grounded_is_not_speculative() {
    // A fully-grounded input (no unresolved n-grams, no weak resolutions) must
    // be clean: speculative:false, warning:null.
    let v = generate_json("budget", "10", false);
    assert_eq!(
        v["speculative"],
        Value::Bool(false),
        "fully-grounded input must not be speculative, got {:?}",
        v
    );
    assert!(
        v["warning"].is_null(),
        "clean output must carry warning: null, got {:?}",
        v["warning"]
    );
}
