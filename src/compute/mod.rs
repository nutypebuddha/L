pub mod convert;
pub mod determinism;
pub mod formulas;
pub mod math;
pub mod natural;
pub mod parser;
pub mod pipeline;
pub mod rational;
pub mod sanity;
pub mod solver;
pub mod thinking;
pub mod verify;

use std::collections::HashMap;

pub type TantoEnv = HashMap<String, f64>;

pub fn create_env() -> TantoEnv {
    let mut env = TantoEnv::new();
    env.insert("pi".to_string(), std::f64::consts::PI);
    env.insert("e".to_string(), std::f64::consts::E);
    env.insert("c".to_string(), 299792458.0);
    env.insert("g".to_string(), 9.80665);
    env.insert("G".to_string(), 6.674e-11);
    env.insert("h".to_string(), 6.626e-34);
    env.insert("kB".to_string(), 1.381e-23);
    env.insert("R_air".to_string(), 287.0586949114925);
    env.insert("R_universal".to_string(), 8.314462618);
    env.insert("c_squared".to_string(), 89875517873681764.0);
    env.insert("avogadro".to_string(), 6.022e23);
    env
}

pub fn evaluate_expr(expr: &str, env: &TantoEnv) -> Option<f64> {
    // Fast path: pure math expressions skip the preprocess_line alloc roundtrip.
    // NL prefixes ("what is", "calculate") start with an alphabetic character
    // and contain spaces. Percentage expressions ("15% of 240") contain " of ".
    // Pure math (identifiers, digits, operators, parens) hits the fast path.
    let first = *expr.as_bytes().first().unwrap_or(&b' ');
    let may_need_preprocess =
        first.is_ascii_alphabetic() && expr.contains(' ') || expr.contains(" of ");
    if !may_need_preprocess {
        let expr_bytes = expr.as_bytes();
        return parser::eval_math(expr_bytes, env)
            .or_else(|| parser::eval_op_format(expr_bytes, env));
    }
    let processed = natural::preprocess_line(expr.as_bytes());
    let expr_str = std::str::from_utf8(&processed).ok()?;
    let expr_bytes = expr_str.as_bytes();
    parser::eval_math(expr_bytes, env).or_else(|| parser::eval_op_format(expr_bytes, env))
}

pub fn evaluate_nl(expr: &str, env: &TantoEnv) -> Option<f64> {
    evaluate_expr(expr, env)
}

/// Pure function: check whether `expr` is syntactically valid Tanto —
/// balanced, well-tokenized, using known function names — without requiring
/// every identifier to be bound. Free variables are tolerated (each bound to
/// `1.0`), so this validates a formula's *expression syntax*, not a concrete
/// evaluation against a particular set of inputs.
///
/// Used by `laverna corpus validate` to flag malformed corpus expressions
/// (unbalanced parens, unknown operators, misspelled function names) without
/// false-positiving on legitimate free variables.
pub fn is_expression_valid(expr: &str) -> bool {
    let mut env = create_env();
    for ident in extract_identifiers(expr) {
        env.entry(ident).or_insert(1.0);
    }
    parser::parse_math(expr.as_bytes(), &env).is_some()
}

/// Pure function: extract bare identifier tokens from an expression, excluding
/// Tanto's reserved function names. Mirrors the tokenizer (`parser::read_ident`)
/// so the validity check observes the same identifiers the evaluator would look
/// up in the environment.
pub fn extract_identifiers(expr: &str) -> Vec<String> {
    const FUNCS: &[&str] = &[
        "sqrt",
        "sin",
        "cos",
        "tan",
        "asin",
        "acos",
        "atan",
        "atan2",
        "abs",
        "exp",
        "ln",
        "log10",
        "log2",
        "log",
        "hypot",
        "pow",
        "round",
        "floor",
        "ceil",
        "min",
        "max",
        "clamp",
        "sum",
        "avg",
        "erf",
        "diff",
        "factorial",
        "gcd",
        "gauss_inv",
        "rad2deg",
        "deg2rad",
        "norm",
        "f_to_c",
        "c_to_f",
        "c_to_k",
        "f_to_k",
        "k_to_c",
        "k_to_f",
        "mi_to_km",
        "km_to_mi",
        "mph_to_kmh",
        "kmh_to_mph",
        "lb_to_kg",
        "kg_to_lb",
        "ft_to_m",
        "in_to_cm",
        "mph_to_ms",
        "ms_to_mph",
    ];
    let bytes = expr.as_bytes();
    let mut idents = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let s = &expr[start..i];
            if !FUNCS.contains(&s) {
                idents.push(s.to_string());
            }
        } else if b.is_ascii_digit() || b == b'.' {
            // Consume the entire numeric literal (including `e`/`E` scientific
            // notation and an optional sign on the exponent) so `1e9` is not
            // mistaken for the identifier `e9`.
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' {
                    i += 1;
                    if (c == b'e' || c == b'E')
                        && i < bytes.len()
                        && (bytes[i] == b'+' || bytes[i] == b'-')
                    {
                        i += 1;
                    }
                } else {
                    break;
                }
            }
        } else {
            i += 1;
        }
    }
    idents
}

pub fn compute_formula(name: &str, args: &str) -> Option<formulas::FormulaResult> {
    let line = format!("{} {}", name, args);
    formulas::compute_formula(&line)
}

pub fn solve_problem(input: &str) -> Option<solver::SolverResult> {
    solver::solve(input)
}

pub fn apply_thinking(framework: &str, problem: &str) -> Option<thinking::ThinkResult> {
    thinking::think(&format!("{} {}", framework, problem))
}

pub fn evaluate_pipeline(line: &str, env: &TantoEnv) -> Option<f64> {
    pipeline::evaluate_pipeline(line, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_env() {
        let env = create_env();
        assert!(env.contains_key("pi"));
        assert!(env.contains_key("e"));
        assert!((env["pi"] - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_expr_basic() {
        let env = create_env();
        assert_eq!(evaluate_expr("2+3", &env), Some(5.0));
        assert_eq!(evaluate_expr("10-4", &env), Some(6.0));
        assert_eq!(evaluate_expr("3*4", &env), Some(12.0));
        assert_eq!(evaluate_expr("12/4", &env), Some(3.0));
    }

    #[test]
    fn test_evaluate_expr_constants() {
        let env = create_env();
        assert!((evaluate_expr("pi", &env).unwrap() - std::f64::consts::PI).abs() < 1e-10);
        assert!((evaluate_expr("e", &env).unwrap() - std::f64::consts::E).abs() < 1e-10);
    }

    #[test]
    fn test_evaluate_nl() {
        let env = create_env();
        assert_eq!(evaluate_nl("2+3", &env), Some(5.0));
    }
}
