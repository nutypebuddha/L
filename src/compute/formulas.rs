#[derive(Debug, Clone)]
pub struct FormulaResult {
    pub name: String,
    pub args: Vec<f64>,
    pub result: f64,
    pub explanation: String,
}

pub fn compute_formula(line: &str) -> Option<FormulaResult> {
    let line = line.trim();
    let (name, rest) = split_token(line.as_bytes())?;
    let name_str = std::str::from_utf8(name).ok()?;
    let args_str = std::str::from_utf8(rest).ok()?;
    let args = parse_f64s(args_str);

    let (result, explanation) = match name_str {
        "circle_area" => {
            if args.is_empty() {
                return None;
            }
            let r = args[0];
            let area = std::f64::consts::PI * r * r;
            (area, format!("A = π * r² = π * {}² = {}", r, area))
        }
        "circle_perimeter" => {
            if args.is_empty() {
                return None;
            }
            let r = args[0];
            let p = 2.0 * std::f64::consts::PI * r;
            (p, format!("P = 2π * r = 2π * {} = {}", r, p))
        }
        "sphere_volume" => {
            if args.is_empty() {
                return None;
            }
            let r = args[0];
            let v = (4.0 / 3.0) * std::f64::consts::PI * r.powi(3);
            (v, format!("V = (4/3)π * r³ = (4/3)π * {}³ = {}", r, v))
        }
        "sphere_surface" => {
            if args.is_empty() {
                return None;
            }
            let r = args[0];
            let a = 4.0 * std::f64::consts::PI * r * r;
            (a, format!("A = 4π * r² = 4π * {}² = {}", r, a))
        }
        "cylinder_volume" => {
            if args.len() < 2 {
                return None;
            }
            let r = args[0];
            let h = args[1];
            let v = std::f64::consts::PI * r * r * h;
            (v, format!("V = π * r² * h = π * {}² * {} = {}", r, h, v))
        }
        "cone_volume" => {
            if args.len() < 2 {
                return None;
            }
            let r = args[0];
            let h = args[1];
            let v = (1.0 / 3.0) * std::f64::consts::PI * r * r * h;
            (
                v,
                format!("V = (1/3)π * r² * h = (1/3)π * {}² * {} = {}", r, h, v),
            )
        }
        "ke" => {
            if args.len() < 2 {
                return None;
            }
            let m = args[0];
            let v = args[1];
            let ke = 0.5 * m * v * v;
            (
                ke,
                format!("KE = 0.5 * m * v² = 0.5 * {} * {}² = {}", m, v, ke),
            )
        }
        "pe" => {
            if args.len() < 2 {
                return None;
            }
            let m = args[0];
            let h = args[1];
            let g = 9.80665;
            let pe = m * g * h;
            (
                pe,
                format!("PE = m * g * h = {} * {} * {} = {}", m, g, h, pe),
            )
        }
        "force" => {
            if args.len() < 2 {
                return None;
            }
            let m = args[0];
            let a = args[1];
            let f = m * a;
            (f, format!("F = m * a = {} * {} = {}", m, a, f))
        }
        "pressure" => {
            if args.len() < 2 {
                return None;
            }
            let f = args[0];
            let a = args[1];
            if a == 0.0 {
                return None;
            }
            let p = f / a;
            (p, format!("P = F / A = {} / {} = {}", f, a, p))
        }
        "work" => {
            if args.len() < 2 {
                return None;
            }
            let f = args[0];
            let d = args[1];
            let w = f * d;
            (w, format!("W = F * d = {} * {} = {}", f, d, w))
        }
        "power" => {
            if args.len() < 2 {
                return None;
            }
            let w = args[0];
            let t = args[1];
            if t == 0.0 {
                return None;
            }
            let p = w / t;
            (p, format!("P = W / t = {} / {} = {}", w, t, p))
        }
        "ohm" => {
            if args.len() < 2 {
                return None;
            }
            let v = args[0];
            let r = args[1];
            if r == 0.0 {
                return None;
            }
            let i = v / r;
            let p = v * i;
            (
                i,
                format!("I = V / R = {} / {} = {} A, P = {} W", v, r, i, p),
            )
        }
        "doppler" => {
            if args.len() < 3 {
                return None;
            }
            let f0 = args[0];
            let vs = args[1];
            let vr = args[2];
            let c = 343.0;
            let f = f0 * (c + vr) / (c - vs);
            (
                f,
                format!(
                    "f = f0 * (c + vr) / (c - vs) = {} * ({} + {}) / ({} - {}) = {}",
                    f0, c, vr, c, vs, f
                ),
            )
        }
        "reynolds" => {
            if args.len() < 4 {
                return None;
            }
            let rho = args[0];
            let v = args[1];
            let l = args[2];
            let mu = args[3];
            if mu == 0.0 {
                return None;
            }
            let re = rho * v * l / mu;
            (
                re,
                format!(
                    "Re = ρ * v * L / μ = {} * {} * {} / {} = {}",
                    rho, v, l, mu, re
                ),
            )
        }
        _ => return None,
    };

    Some(FormulaResult {
        name: name_str.to_string(),
        args,
        result,
        explanation,
    })
}

pub fn list_formulas() -> Vec<(&'static str, &'static str)> {
    vec![
        ("circle_area <r>", "Area of circle"),
        ("circle_perimeter <r>", "Perimeter of circle"),
        ("sphere_volume <r>", "Volume of sphere"),
        ("sphere_surface <r>", "Surface area of sphere"),
        ("cylinder_volume <r> <h>", "Volume of cylinder"),
        ("cone_volume <r> <h>", "Volume of cone"),
        ("ke <m> <v>", "Kinetic energy"),
        ("pe <m> <h>", "Potential energy"),
        ("force <m> <a>", "Newton's second law"),
        ("pressure <F> <A>", "Pressure"),
        ("work <F> <d>", "Work done"),
        ("power <W> <t>", "Power"),
        ("ohm <V> <R>", "Ohm's law"),
        ("doppler <f0> <vs> <vr>", "Doppler effect"),
        ("reynolds <rho> <v> <L> <mu>", "Reynolds number"),
    ]
}

fn parse_f64s(s: &str) -> Vec<f64> {
    let s = s.trim();
    if s.is_empty() {
        return vec![];
    }
    s.split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_area() {
        let result = compute_formula("circle_area 5").unwrap();
        assert!((result.result - 78.53981633974483).abs() < 1e-10);
    }

    #[test]
    fn test_sphere_volume() {
        let result = compute_formula("sphere_volume 3").unwrap();
        assert!((result.result - 113.09733552923255).abs() < 1e-10);
    }

    #[test]
    fn test_ke() {
        let result = compute_formula("ke 10 5").unwrap();
        assert!((result.result - 125.0).abs() < 1e-10);
    }

    #[test]
    fn test_pe() {
        let result = compute_formula("pe 5 9.8").unwrap();
        let expected = 5.0 * 9.80665 * 9.8;
        assert!((result.result - expected).abs() < 1e-10);
    }

    #[test]
    fn test_force() {
        let result = compute_formula("force 10 5").unwrap();
        assert!((result.result - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_pressure() {
        let result = compute_formula("pressure 100 2").unwrap();
        assert!((result.result - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_work() {
        let result = compute_formula("work 10 5").unwrap();
        assert!((result.result - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_power() {
        let result = compute_formula("power 100 10").unwrap();
        assert!((result.result - 10.0).abs() < 1e-10);
    }
}
