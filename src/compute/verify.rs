use super::parser::{eval_math, eval_op_format};
use super::TantoEnv;

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub status: &'static str,
    pub expected: f64,
    pub computed: f64,
    pub diff: f64,
}

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

#[allow(
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::vec_init_then_push
)]
pub fn run_self_test() -> Vec<(&'static str, bool)> {
    let mut results = vec![];

    results.push(("add 2+3", test_expr(b"add 2 3", 5.0)));
    results.push(("sub 10-4", test_expr(b"sub 10 4", 6.0)));
    results.push(("mul 3*4", test_expr(b"mul 3 4", 12.0)));
    results.push(("div 12/4", test_expr(b"div 12 4", 3.0)));
    results.push(("neg 5", test_expr(b"neg 5", -5.0)));
    results.push(("abs -5", test_expr(b"abs -5", 5.0)));
    results.push(("hypot 3,4", test_expr(b"hypot 3 4", 5.0)));
    results.push(("sqrt 9", test_expr(b"sqrt 9", 3.0)));
    results.push(("energy 0", test_expr(b"energy 0", 0.0)));
    results.push(("c", test_expr(b"c", 299792458.0)));
    results.push(("R_air", test_expr(b"R_air", 287.0586949114925)));
    results.push(("pi", test_expr(b"pi", 3.141592653589793)));
    results.push(("15% of 240", test_expr_nl(b"15% of 240", 36.0)));
    results.push(("sqrt(144)", test_expr(b"sqrt(144)", 12.0)));
    results.push(("2^10", test_expr(b"2^10", 1024.0)));
    results.push(("10% of 250", test_expr_nl(b"10% of 250", 25.0)));
    results.push(("sin(0)", test_expr(b"sin(0)", 0.0)));
    results.push(("cos(0)", test_expr(b"cos(0)", 1.0)));
    results.push(("ln(1)", test_expr(b"ln(1)", 0.0)));
    results.push(("exp(0)", test_expr(b"exp(0)", 1.0)));
    results.push((
        "add 2 3 | mul 6 _",
        test_pipeline(b"div 1 3 | mul 6 _", 2.0),
    ));

    results
}

fn test_expr(expr: &[u8], expected: f64) -> bool {
    let env = super::create_env();
    let computed = eval_math(expr, &env).or_else(|| eval_op_format(expr, &env));
    match computed {
        Some(v) => (expected - v).abs() < 1e-6,
        None => false,
    }
}

fn test_expr_nl(expr: &[u8], expected: f64) -> bool {
    let env = super::create_env();
    let processed = super::natural::preprocess_line(expr);
    let computed = eval_math(&processed, &env).or_else(|| eval_op_format(&processed, &env));
    match computed {
        Some(v) => (expected - v).abs() < 1e-6,
        None => false,
    }
}

fn test_pipeline(expr: &[u8], expected: f64) -> bool {
    let env = super::create_env();
    let line = std::str::from_utf8(expr).unwrap_or("");
    let computed = super::pipeline::evaluate_pipeline(line, &env);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_self_test() {
        let results = run_self_test();
        let failures: Vec<_> = results.iter().filter(|(_, ok)| !ok).collect();
        assert!(
            failures.is_empty(),
            "Self-test failed on {} tests:\n{}",
            failures.len(),
            failures
                .iter()
                .map(|(desc, _)| format!("  FAIL: {}", desc))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn test_verify_ok() {
        let env = TantoEnv::new();
        let result = verify("verify 5 add 2 3", &env).unwrap();
        assert_eq!(result.status, "OK");
    }
}
