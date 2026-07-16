const ITERATIONS: usize = 100;

#[derive(Debug, Clone)]
pub struct DeterminismResult {
    pub description: &'static str,
    pub passed: bool,
    pub variations: usize,
    pub sample_value: String,
}

#[allow(clippy::vec_init_then_push)]
pub fn run_determinism_audit() -> Vec<(&'static str, bool)> {
    let mut results = vec![];

    results.push(("arithmetic: 2+3", check_determinism("2+3", None)));
    results.push(("arithmetic: 10/3", check_determinism("10/3", None)));
    results.push(("arithmetic: 1.5*2.5", check_determinism("1.5*2.5", None)));
    results.push(("arithmetic: 100-37", check_determinism("100-37", None)));
    results.push((
        "arithmetic: (2+3)*(4-1)",
        check_determinism("(2+3)*(4-1)", None),
    ));
    results.push(("arithmetic: 2^10", check_determinism("2^10", None)));
    results.push(("arithmetic: -42", check_determinism("-42", None)));
    results.push(("arithmetic: +42", check_determinism("+42", None)));
    results.push(("arithmetic: 7%3", check_determinism("7%3", None)));
    results.push(("arithmetic: 1e5", check_determinism("1e5", None)));
    results.push(("arithmetic: 1.23e-4", check_determinism("1.23e-4", None)));

    results.push(("func: sqrt(144)", check_determinism("sqrt(144)", None)));
    results.push(("func: sqrt(2)", check_determinism("sqrt(2)", None)));
    results.push(("func: sin(pi/2)", check_determinism("sin(pi/2)", None)));
    results.push(("func: cos(0)", check_determinism("cos(0)", None)));
    results.push(("func: tan(pi/4)", check_determinism("tan(pi/4)", None)));
    results.push(("func: asin(1)", check_determinism("asin(1)", None)));
    results.push(("func: acos(0.5)", check_determinism("acos(0.5)", None)));
    results.push(("func: atan(1)", check_determinism("atan(1)", None)));
    results.push(("func: hypot(3,4)", check_determinism("hypot(3,4)", None)));
    results.push(("func: abs(-5)", check_determinism("abs(-5)", None)));
    results.push(("func: exp(1)", check_determinism("exp(1)", None)));
    results.push(("func: ln(e)", check_determinism("ln(e)", None)));
    results.push(("func: log10(100)", check_determinism("log10(100)", None)));
    results.push(("func: log2(8)", check_determinism("log2(8)", None)));
    results.push(("func: pow(2,10)", check_determinism("pow(2,10)", None)));
    results.push(("func: round(1.5)", check_determinism("round(1.5)", None)));
    results.push(("func: floor(2.7)", check_determinism("floor(2.7)", None)));
    results.push(("func: ceil(2.3)", check_determinism("ceil(2.3)", None)));
    results.push(("func: min(1,2,3)", check_determinism("min(1,2,3)", None)));
    results.push(("func: max(1,2,3)", check_determinism("max(1,2,3)", None)));
    results.push((
        "func: clamp(5,0,10)",
        check_determinism("clamp(5,0,10)", None),
    ));
    results.push((
        "func: sum(1,2,3,4)",
        check_determinism("sum(1,2,3,4)", None),
    ));
    results.push((
        "func: avg(10,20,30)",
        check_determinism("avg(10,20,30)", None),
    ));
    results.push(("func: rad2deg(pi)", check_determinism("rad2deg(pi)", None)));
    results.push((
        "func: deg2rad(180)",
        check_determinism("deg2rad(180)", None),
    ));
    results.push(("func: atan2(0,1)", check_determinism("atan2(0,1)", None)));
    results.push(("func: norm(3,4)", check_determinism("norm(3,4)", None)));

    results.push(("named: add 2 3", check_determinism("add 2 3", None)));
    results.push(("named: sub 10 4", check_determinism("sub 10 4", None)));
    results.push(("named: mul 3 4", check_determinism("mul 3 4", None)));
    results.push(("named: div 12 4", check_determinism("div 12 4", None)));
    results.push(("named: neg 5", check_determinism("neg 5", None)));
    results.push(("named: pct 200 15", check_determinism("pct 200 15", None)));
    results.push((
        "named: pct_change 100 150",
        check_determinism("pct_change 100 150", None),
    ));
    results.push(("named: ratio 4 3 2", check_determinism("ratio 4 3 2", None)));
    results.push(("named: dilute 10 5", check_determinism("dilute 10 5", None)));

    results.push((
        "convert: f_to_c(212)",
        check_determinism("f_to_c(212)", None),
    ));
    results.push(("convert: c_to_f(0)", check_determinism("c_to_f(0)", None)));
    results.push(("convert: c_to_k(0)", check_determinism("c_to_k(0)", None)));

    results.push(("const: pi", check_determinism("pi", None)));
    results.push(("const: e", check_determinism("e", None)));
    results.push(("const: c (speed of light)", check_determinism("c", None)));
    results.push(("const: g (gravity)", check_determinism("g", None)));
    results.push(("const: G (gravitational)", check_determinism("G", None)));
    results.push(("const: h (planck)", check_determinism("h", None)));
    results.push(("const: kB", check_determinism("kB", None)));

    results.push(("nl: 15% of 240", check_determinism_nl("15% of 240")));
    results.push(("nl: 10% of 250", check_determinism_nl("10% of 250")));
    results.push((
        "nl: whats sqrt(144)",
        check_determinism_nl("whats sqrt(144)"),
    ));
    results.push(("nl: calculate 2+3", check_determinism_nl("calculate 2+3")));
    results.push(("nl: how much is 42", check_determinism_nl("how much is 42")));

    results.push((
        "formula: circle_area(5)",
        check_determinism_formula("circle_area", "5"),
    ));
    results.push((
        "formula: sphere_volume(3)",
        check_determinism_formula("sphere_volume", "3"),
    ));
    results.push(("formula: ke(10,5)", check_determinism_formula("ke", "10 5")));
    results.push((
        "formula: pe(5,9.8)",
        check_determinism_formula("pe", "5 9.8"),
    ));
    results.push((
        "formula: force(10,5)",
        check_determinism_formula("force", "10 5"),
    ));
    results.push((
        "formula: pressure(100,2)",
        check_determinism_formula("pressure", "100 2"),
    ));
    results.push((
        "formula: work(10,5)",
        check_determinism_formula("work", "10 5"),
    ));
    results.push((
        "formula: power(100,10)",
        check_determinism_formula("power", "100 10"),
    ));

    results.push((
        "solver: stats 1 2 3 4 5",
        check_determinism_solver("stats", "1 2 3 4 5"),
    ));
    results.push((
        "solver: compound 1000 0.05 12 10",
        check_determinism_solver("compound", "1000 0.05 12 10"),
    ));
    results.push((
        "solver: growth 100 0.1 5",
        check_determinism_solver("growth", "100 0.1 5"),
    ));

    results.push((
        "pipeline: div 1 3 | mul 6 _",
        check_determinism_pipeline("div 1 3 | mul 6 _"),
    ));
    results.push((
        "pipeline: add 2 3 | mul 4 _",
        check_determinism_pipeline("add 2 3 | mul 4 _"),
    ));

    results.push(("chained: 1+2*3+4*5", check_determinism("1+2*3+4*5", None)));
    results.push((
        "chained: sqrt(3^2 + 4^2)",
        check_determinism("sqrt(3^2 + 4^2)", None),
    ));
    results.push((
        "chained: sin(pi/6)^2 + cos(pi/6)^2",
        check_determinism("sin(pi/6)^2 + cos(pi/6)^2", None),
    ));
    results.push(("chained: ln(e^5)", check_determinism("ln(e^5)", None)));
    results.push((
        "chained: 10% of 500 + 5% of 200",
        check_determinism_nl("10% of 500 + 5% of 200"),
    ));

    results
}

pub fn check_determinism(expr: &str, expected: Option<f64>) -> bool {
    let env = super::create_env();
    let first = match super::evaluate_nl(expr, &env) {
        Some(v) => v,
        None => return false,
    };

    if let Some(exp) = expected {
        if (first - exp).abs() > 1e-6 {
            return false;
        }
    }

    for _ in 1..ITERATIONS {
        let env = super::create_env();
        match super::evaluate_nl(expr, &env) {
            Some(v) => {
                if v.to_bits() != first.to_bits() {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn check_determinism_rational(expr: &str) -> bool {
    let env = super::create_env();
    let first = match super::rational::eval_rational(expr, &env) {
        Some(r) => r,
        None => return false,
    };

    for _ in 1..ITERATIONS {
        let env = super::create_env();
        match super::rational::eval_rational(expr, &env) {
            Some(r) => {
                if r != first {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn check_determinism_nl(expr: &str) -> bool {
    let env = super::create_env();
    let first = match super::evaluate_nl(expr, &env) {
        Some(v) => v,
        None => return false,
    };

    for _ in 1..ITERATIONS {
        let env = super::create_env();
        match super::evaluate_nl(expr, &env) {
            Some(v) => {
                if v.to_bits() != first.to_bits() {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn check_determinism_formula(name: &str, args: &str) -> bool {
    let first = match super::compute_formula(name, args) {
        Some(r) => r.result,
        None => return false,
    };

    for _ in 1..ITERATIONS {
        match super::compute_formula(name, args) {
            Some(r) => {
                if r.result.to_bits() != first.to_bits() {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn check_determinism_solver(name: &str, args: &str) -> bool {
    let line = format!("{} {}", name, args);
    let first = match super::solve_problem(&line) {
        Some(r) => r.output.clone(),
        None => return false,
    };

    for _ in 1..ITERATIONS {
        match super::solve_problem(&line) {
            Some(r) => {
                if r.output != first {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

pub fn check_determinism_pipeline(expr: &str) -> bool {
    let env = super::create_env();
    let first = match super::pipeline::evaluate_pipeline(expr, &env) {
        Some(v) => v,
        None => return false,
    };

    for _ in 1..ITERATIONS {
        let env = super::create_env();
        match super::pipeline::evaluate_pipeline(expr, &env) {
            Some(v) => {
                if v.to_bits() != first.to_bits() {
                    return false;
                }
            }
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determinism_basic_arithmetic() {
        assert!(check_determinism("2+3", None));
        assert!(check_determinism("10/3", None));
        assert!(check_determinism("1.5*2.5", None));
    }

    #[test]
    fn test_determinism_with_expected() {
        assert!(check_determinism("2+3", Some(5.0)));
        assert!(check_determinism("sqrt(144)", Some(12.0)));
    }

    #[test]
    fn test_determinism_nl() {
        assert!(check_determinism_nl("15% of 240"));
    }

    #[test]
    fn test_determinism_formula() {
        assert!(check_determinism_formula("circle_area", "5"));
        assert!(check_determinism_formula("sphere_volume", "3"));
    }

    #[test]
    fn test_determinism_solver() {
        assert!(check_determinism_solver("stats", "1 2 3 4 5"));
    }

    #[test]
    fn test_determinism_pipeline() {
        assert!(check_determinism_pipeline("div 1 3 | mul 6 _"));
    }

    #[test]
    fn test_determinism_audit_all_pass() {
        let results = run_determinism_audit();
        let failures: Vec<_> = results.iter().filter(|(_, ok)| !ok).collect();
        assert!(
            failures.is_empty(),
            "Determinism audit failed on {} tests:\n{}",
            failures.len(),
            failures
                .iter()
                .map(|(desc, _)| format!("  FAIL: {}", desc))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
}
