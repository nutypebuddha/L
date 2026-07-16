//! # Proof-object red-team (T52) + byte-reproducibility (T53)
//!
//! `laverna verify` must be *recomputation*, not an echo. This test:
//!   1. asserts an honest proof verifies (exit 0),
//!   2. asserts proofs are byte-identical across runs (no wall clock in payload),
//!   3. mutates every field of an honest proof in turn and asserts `verify`
//!      rejects each forgery with a non-zero exit and `verified: false`.
//!
//! Run: `cargo test --test proof_redteam -- --nocapture`

use serde_json::Value;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

const BIN: &str = env!("CARGO_BIN_EXE_laverna");
const QUERY: &str = "2 + 3 = 5";

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Process- and call-unique temp path (tests run concurrently in one process).
fn tmp(name: &str) -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("laverna_redteam_{}_{n}_{name}", std::process::id()))
}

fn write_proof(path: &std::path::Path) {
    let status = Command::new(BIN)
        .args(["solve", "--query", QUERY, "--proof-out"])
        .arg(path)
        .status()
        .expect("run solve");
    assert!(status.success(), "solve --proof-out failed");
}

/// Run `verify --format json` on `proof` and return (exit_ok, verified_flag).
fn verify(proof: &Value) -> (bool, bool) {
    let path = tmp("candidate.json");
    std::fs::write(&path, serde_json::to_string_pretty(proof).unwrap()).unwrap();
    let out = Command::new(BIN)
        .args(["verify"])
        .arg(&path)
        .args(["--format", "json"])
        .output()
        .expect("run verify");
    let json: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|_| {
        panic!(
            "verify emitted non-JSON: {:?}",
            String::from_utf8_lossy(&out.stdout)
        )
    });
    let _ = std::fs::remove_file(&path);
    (
        out.status.success(),
        json["verified"].as_bool().unwrap_or(false),
    )
}

#[test]
fn honest_proof_verifies() {
    let p = tmp("honest.json");
    write_proof(&p);
    let proof: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let (ok, verified) = verify(&proof);
    assert!(ok && verified, "honest proof must verify");
    let _ = std::fs::remove_file(&p);
}

#[test]
fn proofs_are_byte_reproducible() {
    let a = tmp("a.json");
    let b = tmp("b.json");
    write_proof(&a);
    write_proof(&b);
    let sa = std::fs::read(&a).unwrap();
    let sb = std::fs::read(&b).unwrap();
    assert_eq!(
        sa, sb,
        "two proofs of the same query must be byte-identical (T53)"
    );
    let _ = std::fs::remove_file(&a);
    let _ = std::fs::remove_file(&b);
}

/// Mutate a JSON value into a definitely-different value of a compatible shape.
fn mutate(v: &Value) -> Value {
    match v {
        Value::String(s) => Value::String(format!("{s}__FORGED")),
        Value::Number(n) => {
            let f = n.as_f64().unwrap_or(0.0);
            serde_json::json!(f + 42.0)
        }
        Value::Bool(b) => Value::Bool(!b),
        Value::Array(a) => {
            // Drop the last element (or inject one if empty) to change the array.
            let mut a = a.clone();
            if a.pop().is_none() {
                a.push(Value::String("FORGED".into()));
            }
            Value::Array(a)
        }
        Value::Object(o) => {
            let mut o = o.clone();
            o.insert("__forged".into(), Value::Bool(true));
            Value::Object(o)
        }
        Value::Null => Value::Bool(true),
    }
}

#[test]
fn every_mutated_field_is_rejected() {
    let p = tmp("base.json");
    write_proof(&p);
    let base: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let _ = std::fs::remove_file(&p);

    let obj = base.as_object().expect("proof is an object");
    let mut checked = 0usize;

    for (key, val) in obj {
        // `computed_at` is intentionally not part of the hashed payload (T53);
        // it is absent by default. Every other field is security-relevant.
        if key == "computed_at" {
            continue;
        }

        // `digest` is the envelope, not trusted content — verify recomputes it.
        // The meaningful forgery is flipping its `value`, which must be rejected.
        if key == "digest" {
            let mut forged = base.clone();
            forged.as_object_mut().unwrap().insert(
                "digest".into(),
                serde_json::json!({ "algorithm": "sha256", "value": "0".repeat(64) }),
            );
            let (ok, verified) = verify(&forged);
            assert!(!ok && !verified, "forging digest.value must be rejected");
            checked += 1;
            continue;
        }

        // Mutate the top-level field.
        let mut forged = base.clone();
        forged
            .as_object_mut()
            .unwrap()
            .insert(key.clone(), mutate(val));
        let (ok, verified) = verify(&forged);
        assert!(
            !ok && !verified,
            "forging top-level field `{key}` must be rejected, got ok={ok} verified={verified}"
        );
        checked += 1;

        // Descend one level into `descent` and mutate each subfield too.
        if key == "descent" {
            if let Some(d) = val.as_object() {
                for (dk, dv) in d {
                    let mut forged = base.clone();
                    let mut descent = d.clone();
                    descent.insert(dk.clone(), mutate(dv));
                    forged
                        .as_object_mut()
                        .unwrap()
                        .insert("descent".into(), Value::Object(descent));
                    let (ok, verified) = verify(&forged);
                    assert!(
                        !ok && !verified,
                        "forging descent.{dk} must be rejected, got ok={ok} verified={verified}"
                    );
                    checked += 1;
                }
            }
        }
    }

    assert!(
        checked >= 8,
        "expected to red-team several fields, only did {checked}"
    );
    eprintln!("red-teamed {checked} field mutations, all rejected");
}
