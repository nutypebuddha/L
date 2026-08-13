// Tanto verify — self-verification and test suite
// Adapted for CID

use crate::tanto::parser::{eval_math, eval_op_format};
use crate::tanto::TantoEnv;

/// Result of a verification check
#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub status: &'static str, // "OK", "CLOSE", or "MISMATCH"
    pub expected: f64,
    pub computed: f64,
    pub diff: f64,
}

/// Verify an expression against an expected value
/// Format: "verify 5 hypot(3, 4)"
pub fn verify(args: &str, env: &TantoEnv) -> Option<VerifyResult> {
    let args = args.trim().as_bytes();
    if !args.starts_with(b"verify") {
        return None;
    }
    let rest = if args.len() > 6 {
        trim(&args[6..])
    } else {
        return None;
    };
    let (expected_str, expr_rest) = split_token(rest)?;
    let expected = parse_f64(expected_str)?;
    let expr = trim(expr_rest);
    if expr.is_empty() {
        return None;
    }

    let computed = eval_math(expr, env).or_else(|| eval_op_format(expr, env))?;

    let diff = (expected - computed).abs();
    let status = if diff < 1e-10 {
        "OK"
    } else if diff < 0.01 {
        "CLOSE"
    } else {
        "MISMATCH"
    };

    Some(VerifyResult {
        status,
        expected,
        computed,
        diff,
    })
}

/// Run self-test on all core math operations
pub fn run_self_test() -> Vec<(&'static str, bool)> {
    vec![
        ("add 2+3", test_expr(b"add 2 3", 5.0)),
        ("sub 10-4", test_expr(b"sub 10 4", 6.0)),
        ("mul 3*4", test_expr(b"mul 3 4", 12.0)),
        ("div 12/4", test_expr(b"div 12 4", 3.0)),
        ("neg 5", test_expr(b"neg 5", -5.0)),
        ("abs -5", test_expr(b"abs -5", 5.0)),
        ("hypot 3,4", test_expr(b"hypot 3 4", 5.0)),
        ("sqrt 9", test_expr(b"sqrt 9", 3.0)),
        ("energy 0", test_expr(b"energy 0", 0.0)),
        ("c", test_expr(b"c", 299792458.0)),
        ("R_air", test_expr(b"R_air", 287.0586949114925)),
        ("pi", test_expr(b"pi", std::f64::consts::PI)),
        ("15% of 240", test_expr_nl(b"15% of 240", 36.0)),
        ("sqrt(144)", test_expr(b"sqrt(144)", 12.0)),
        ("2^10", test_expr(b"2^10", 1024.0)),
        ("10% of 250", test_expr_nl(b"10% of 250", 25.0)),
        ("sin(0)", test_expr(b"sin(0)", 0.0)),
        ("cos(0)", test_expr(b"cos(0)", 1.0)),
        ("ln(1)", test_expr(b"ln(1)", 0.0)),
        ("exp(0)", test_expr(b"exp(0)", 1.0)),
        (
            "add 2 3 | mul 6 _",
            test_pipeline(b"div 1 3 | mul 6 _", 2.0),
        ),
    ]
}

fn test_expr(expr: &[u8], expected: f64) -> bool {
    let env = TantoEnv::new();
    let computed = eval_math(expr, &env).or_else(|| eval_op_format(expr, &env));
    match computed {
        Some(v) => (expected - v).abs() < 1e-6,
        None => false,
    }
}

fn test_expr_nl(expr: &[u8], expected: f64) -> bool {
    let env = TantoEnv::new();
    let processed = crate::tanto::natural::preprocess_line(expr);
    let computed = eval_math(&processed, &env).or_else(|| eval_op_format(&processed, &env));
    match computed {
        Some(v) => (expected - v).abs() < 1e-6,
        None => false,
    }
}

fn test_pipeline(expr: &[u8], expected: f64) -> bool {
    let env = TantoEnv::new();
    let line = std::str::from_utf8(expr).unwrap_or("");
    let computed = crate::tanto::pipeline::evaluate_pipeline(line, &env);
    match computed {
        Some(v) => (expected - v).abs() < 1e-6,
        None => false,
    }
}

fn trim(s: &[u8]) -> &[u8] {
    let mut start = 0;
    while start < s.len() && (s[start] == b' ' || s[start] == b'\t') {
        start += 1;
    }
    let mut end = s.len();
    while end > start && (s[end - 1] == b' ' || s[end - 1] == b'\t') {
        end -= 1;
    }
    &s[start..end]
}

fn split_token(s: &[u8]) -> Option<(&[u8], &[u8])> {
    let s = trim(s);
    if s.is_empty() {
        return None;
    }
    let mut i = 0;
    while i < s.len() && s[i] != b' ' && s[i] != b'\t' {
        i += 1;
    }
    if i >= s.len() {
        return Some((s, &[]));
    }
    Some((&s[..i], &s[i..]))
}

fn parse_f64(s: &[u8]) -> Option<f64> {
    let s = trim(s);
    let s_str = std::str::from_utf8(s).ok()?;
    s_str.parse::<f64>().ok()
}
