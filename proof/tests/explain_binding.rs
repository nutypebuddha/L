// Behavioral verification of the `--explain-binding` trace (the "binder hijack"
// guard made observable). These run the actual `lai` binary so the flag's stderr
// trace is exercised end-to-end, not just the in-process binder.
use std::process::Command;

fn run_explain(query: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_lai"))
        .args(["solve", "--query", query, "--explain-binding"])
        .output()
        .expect("spawn lai");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn explain_binding_emits_trace_on_valid_bind() {
    let stderr = run_explain("gravity force between mass1 5 and mass2 5 at distance 5");
    assert!(
        stderr.contains("[bind]"),
        "--explain-binding must print a [bind] trace on a successful bind:\n{stderr}"
    );
    assert!(
        stderr.contains("gravity_force"),
        "trace should name the bound formula:\n{stderr}"
    );
}

#[test]
fn explain_binding_shows_refusal_on_unit_mismatch() {
    // `celsius` cannot fill a `bytes` input: the binder must REFUSE loudly rather
    // than silently binding the wrong unit — the core anti-hijack guarantee.
    let stderr = run_explain("pad a 13 celsius struct aligned 8 bytes");
    assert!(
        stderr.contains("[bind]"),
        "--explain-binding must print a [bind] trace even on refusal:\n{stderr}"
    );
    assert!(
        stderr.contains("REFUSED"),
        "--explain-binding must surface the unit-mismatch refusal:\n{stderr}"
    );
}
