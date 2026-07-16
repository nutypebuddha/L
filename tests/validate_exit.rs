//! # `validate` fail-loud exit code
//!
//! `laverna validate` must exit non-zero when the expression fails verification,
//! not merely print `passed: false`. This restores the fail-loud contract used
//! by `verify` / corpus commands (a script can detect failure by exit code).
//!
//! Run: `cargo test --test validate_exit`

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_laverna");

fn run_validate(expr: &str) -> std::process::Output {
    Command::new(BIN)
        .args(["validate", expr])
        .output()
        .expect("run laverna validate")
}

#[test]
fn validate_fails_loud_on_bad_expression() {
    let out = run_validate("2 + 3 = 6");
    assert!(
        !out.status.success(),
        "validate must exit non-zero on a non-balancing equation"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("passed: false"),
        "validate output should report passed: false, got:\n{stdout}"
    );
}

#[test]
fn validate_succeeds_on_good_expression() {
    let out = run_validate("2 + 3 = 5");
    assert!(
        out.status.success(),
        "validate must exit 0 on a balancing equation"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("passed: true"),
        "validate output should report passed: true, got:\n{stdout}"
    );
}
