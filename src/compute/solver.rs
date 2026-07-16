const G_GRAV: f64 = 9.80665;

#[derive(Debug, Clone)]
pub struct SolverResult {
    pub solver: String,
    pub output: String,
}

pub fn solve(args: &str) -> Option<SolverResult> {
    let args = args.trim().as_bytes();
    let (cmd, rest) = split_token(args)?;
    let cmd_str = std::str::from_utf8(cmd).ok()?;
    let rest_str = std::str::from_utf8(trim(rest)).ok()?;

    match cmd {
        b"orbit" => solve_orbit(rest_str),
        b"projectile" => solve_projectile(rest_str),
        b"energy" => solve_energy(rest_str),
        b"fall" => solve_fall(rest_str),
        b"ke" => solve_ke(rest_str),
        b"pe" => solve_pe(rest_str),
        b"ohm" => solve_ohm(rest_str),
        b"compound" => solve_compound(rest_str),
        b"growth" => solve_growth(rest_str),
        b"stats" => solve_stats(rest_str),
        b"kinematics" => solve_kinematics(rest_str),
        b"thermo" => solve_thermo(rest_str),
        b"circuit" => solve_circuit(rest_str),
        b"tvm" => solve_tvm(rest_str),
        b"lens" => solve_lens(rest_str),
        _ => None,
    }
    .map(|output| SolverResult {
        solver: cmd_str.to_string(),
        output,
    })
}

pub fn list_solvers() -> Vec<(&'static str, &'static str)> {
    vec![
        ("orbit <mu> <r>", "Orbital velocity, period, KE"),
        ("projectile <v0> <angle_deg>", "Projectile motion"),
        ("energy <mass_kg>", "E=mc^2 energy in J, kWh, tons TNT"),
        ("fall <height_m>", "Free fall time, impact velocity"),
        ("ke <mass_kg> <vel_mps>", "Kinetic energy + momentum"),
        ("pe <mass_kg> <height_m>", "Potential energy"),
        ("ohm <voltage> <resistance>", "Ohm's law: current + power"),
        ("compound <P> <r> <n> <t>", "Compound interest"),
        ("growth <N0> <r> <t>", "Exponential growth/decay"),
        (
            "stats <v1 v2 ...>",
            "Statistics: mean, median, std_dev, variance",
        ),
        ("kinematics <v0 v t>", "Kinematics: a, displacement, avg_v"),
        (
            "thermo <P V n T>",
            "Ideal gas law (PV=nRT), internal energy",
        ),
        ("circuit <R1 R2>", "Series/parallel resistance & power"),
        ("tvm <PV r n>", "Time value of money (future value, NPV)"),
        ("lens <f do>", "Thin lens: image distance di, magnification"),
    ]
}

fn solve_orbit(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let mu = vals[0];
    let r = vals[1];
    let v = (mu / r).sqrt();
    let t = 2.0 * std::f64::consts::PI * r / v;
    let ke = v * v / 2.0;
    let grav = mu / (r * r);
    let cent = v * v / r;
    let verify = if cent.to_bits() == grav.to_bits() {
        "OK (exact)"
    } else {
        "CLOSE (float rounding)"
    };
    Some(format!(
        "=== ORBITAL SOLUTION ===\n  v (velocity)    = {} m/s\n  T (period)      = {} s\n  KE (per kg)     = {} J/kg\n  grav accel      = {} m/s^2\n  verify: v^2/r == GM/r^2 => {}",
        frmt(v), frmt(t), frmt(ke), frmt(grav), verify
    ))
}

fn solve_projectile(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let v0 = vals[0];
    let angle_deg = vals[1];
    let angle = angle_deg * std::f64::consts::PI / 180.0;
    let vx = v0 * angle.cos();
    let vy = v0 * angle.sin();
    let t_flight = 2.0 * vy / G_GRAV;
    let range = vx * t_flight;
    let max_h = vy * vy / (2.0 * G_GRAV);
    Some(format!(
        "=== PROJECTILE SOLUTION ===\n  vx           = {} m/s\n  vy           = {} m/s\n  flight time  = {} s\n  range        = {} m\n  max height   = {} m",
        frmt(vx), frmt(vy), frmt(t_flight), frmt(range), frmt(max_h)
    ))
}

fn solve_energy(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.is_empty() {
        return None;
    }
    let m = vals[0];
    let e = m * 89875517873681764.0;
    let kwh = e / 3.6e6;
    let tons_tnt = e / 4.184e9;
    Some(format!(
        "=== ENERGY SOLUTION (E=mc^2) ===\n  E (joules)   = {} J\n  E (kWh)      = {} kWh\n  E (tons TNT) = {} tons",
        frmt(e), frmt(kwh), frmt(tons_tnt)
    ))
}

fn solve_fall(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.is_empty() {
        return None;
    }
    let h = vals[0];
    let t = (2.0 * h / G_GRAV).sqrt();
    let v = G_GRAV * t;
    let ke = v * v / 2.0;
    let verify = {
        let v_expected = (2.0 * G_GRAV * h).sqrt();
        let diff = (v - v_expected).abs();
        if diff < 0.0001 {
            "CLOSE"
        } else {
            "MISMATCH"
        }
    };
    Some(format!(
        "=== FREE FALL SOLUTION ===\n  formula: t = sqrt(2h/g), v = g*t, KE = 0.5*v^2\n  height       = {} m\n  fall time    = {} s\n  impact vel   = {} m/s\n  KE per kg    = {} J/kg\n  verify: v == sqrt(2gh) => {}",
        frmt(h), frmt(t), frmt(v), frmt(ke), verify
    ))
}

fn solve_ke(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let m = vals[0];
    let v = vals[1];
    let ke = 0.5 * m * v * v;
    let p = m * v;
    Some(format!(
        "=== KINETIC ENERGY SOLUTION ===\n  formula: KE = 0.5*m*v^2, p = m*v\n  mass     = {} kg\n  velocity = {} m/s\n  KE       = {} J\n  momentum = {} kg*m/s",
        frmt(m), frmt(v), frmt(ke), frmt(p)
    ))
}

fn solve_pe(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let m = vals[0];
    let h = vals[1];
    let pe = m * G_GRAV * h;
    Some(format!(
        "=== POTENTIAL ENERGY SOLUTION ===\n  formula: PE = m*g*h (g = 9.80665 m/s^2)\n  mass   = {} kg\n  height = {} m\n  PE     = {} J",
        frmt(m), frmt(h), frmt(pe)
    ))
}

fn solve_ohm(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let v = vals[0];
    let r = vals[1];
    let i = v / r;
    let p = v * i;
    Some(format!(
        "=== OHM'S LAW SOLUTION ===\n  formula: I = V/R, P = V*I\n  voltage   = {} V\n  resistance = {} ohm\n  current   = {} A\n  power     = {} W",
        frmt(v), frmt(r), frmt(i), frmt(p)
    ))
}

fn solve_compound(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 4 {
        return None;
    }
    let principal = vals[0];
    let rate = vals[1];
    let periods = vals[2];
    let years = vals[3];
    let amount = principal * (1.0 + rate / periods).powf(periods * years);
    let interest = amount - principal;
    Some(format!(
        "=== COMPOUND INTEREST ===\n  formula: A = P*(1 + r/n)^(n*t)\n  principal = {}\n  rate      = {}%/year\n  compounds = {}/year\n  years     = {}\n  final     = {}\n  interest  = {}",
        frmt(principal), frmt(rate * 100.0), frmt(periods), frmt(years), frmt(amount), frmt(interest)
    ))
}

fn solve_growth(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 3 {
        return None;
    }
    let initial = vals[0];
    let rate = vals[1];
    let t = vals[2];
    let final_val = initial * (rate * t).exp();
    let doubling_time = (2.0_f64).ln() / rate;
    Some(format!(
        "=== EXPONENTIAL GROWTH/DECAY ===\n  formula: N = N0 * e^(r*t)\n  initial (N0) = {}\n  rate (r)     = {} /time\n  time (t)     = {}\n  final (N)    = {}\n  doubling time = {}",
        frmt(initial), frmt(rate), frmt(t), frmt(final_val), frmt(doubling_time)
    ))
}

fn solve_stats(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.is_empty() {
        return None;
    }
    let n = vals.len() as f64;
    let sum: f64 = vals.iter().sum();
    let mean = sum / n;
    let mut sorted = vals.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        let m = sorted.len() / 2;
        (sorted[m - 1] + sorted[m]) / 2.0
    };
    let variance = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let std_dev = variance.sqrt();
    let min = sorted.first().copied().unwrap_or(0.0);
    let max = sorted.last().copied().unwrap_or(0.0);
    Some(format!(
        "=== STATISTICS ===\n  n            = {}\n  sum          = {}\n  mean         = {}\n  median       = {}\n  std_dev      = {}\n  variance     = {}\n  min          = {}\n  max          = {}",
        frmt(n), frmt(sum), frmt(mean), frmt(median), frmt(std_dev), frmt(variance), frmt(min), frmt(max)
    ))
}

fn solve_kinematics(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 3 {
        return None;
    }
    let v0 = vals[0];
    let v = vals[1];
    let t = vals[2];
    if t == 0.0 {
        return None;
    }
    let a = (v - v0) / t;
    let d = (v0 + v) * t / 2.0;
    let d2 = v0 * t + 0.5 * a * t * t;
    Some(format!(
        "=== KINEMATICS SOLUTION ===\n  formulas: a = (v-v0)/t, d = (v0+v)*t/2\n  v0           = {} m/s\n  v            = {} m/s\n  t            = {} s\n  a            = {} m/s^2\n  d            = {} m\n  d (alt)      = {} m",
        frmt(v0), frmt(v), frmt(t), frmt(a), frmt(d), frmt(d2)
    ))
}

fn solve_thermo(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 4 {
        return None;
    }
    let p = vals[0];
    let v = vals[1];
    let n = vals[2];
    let t = vals[3];
    if n == 0.0 || t == 0.0 {
        return None;
    }
    let r_gas = 8.314462618;
    let p_calc = if p == 0.0 { n * r_gas * t / v } else { p };
    let v_calc = if v == 0.0 { n * r_gas * t / p } else { v };
    let n_calc = if n == 0.0 { p * v / (r_gas * t) } else { n };
    let t_calc = if t == 0.0 { p * v / (n * r_gas) } else { t };
    let u = 1.5 * n_calc * r_gas * t_calc;
    Some(format!(
        "=== THERMODYNAMICS (Ideal Gas) ===\n  formula: PV = nRT, U = 3/2 nRT\n  P            = {} Pa\n  V            = {} m^3\n  n            = {} mol\n  T            = {} K\n  R            = {} J/(mol*K)\n  verify: PV   = {} J\n  verify: nRT  = {} J\n  U (internal) = {} J",
        frmt(p_calc), frmt(v_calc), frmt(n_calc), frmt(t_calc), frmt(r_gas),
        frmt(p_calc * v_calc), frmt(n_calc * r_gas * t_calc), frmt(u)
    ))
}

fn solve_circuit(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let r1 = vals[0];
    let r2 = vals[1];
    if r1 == 0.0 || r2 == 0.0 {
        return None;
    }
    let r_series = r1 + r2;
    let r_parallel = 1.0 / (1.0 / r1 + 1.0 / r2);
    let v_ref = 12.0;
    let i_series = v_ref / r_series;
    let i_parallel = v_ref / r_parallel;
    let p_series = v_ref * i_series;
    let p_parallel = v_ref * i_parallel;
    Some(format!(
        "=== CIRCUIT SOLUTION (2 resistors) ===\n  formulas: R_series = R1+R2, R_parallel = 1/(1/R1+1/R2)\n  R1           = {} ohm\n  R2           = {} ohm\n  R_series     = {} ohm\n  R_parallel   = {} ohm\n  (at V={}V)\n  I_series     = {} A\n  I_parallel   = {} A\n  P_series     = {} W\n  P_parallel   = {} W",
        frmt(r1), frmt(r2), frmt(r_series), frmt(r_parallel),
        frmt(v_ref), frmt(i_series), frmt(i_parallel), frmt(p_series), frmt(p_parallel)
    ))
}

fn solve_tvm(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 3 {
        return None;
    }
    let pv = vals[0];
    let r = vals[1];
    let n = vals[2];
    let fv = pv * (1.0 + r).powf(n);
    let npv = fv - pv;
    Some(format!(
        "=== TIME VALUE OF MONEY ===\n  formula: FV = PV * (1+r)^n\n  PV (present) = {} USD\n  r (periodic) = {}%\n  n (periods)  = {}\n  FV (future)  = {} USD\n  NPV          = {} USD",
        frmt(pv), frmt(r * 100.0), frmt(n), frmt(fv), frmt(npv)
    ))
}

fn solve_lens(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 2 {
        return None;
    }
    let f = vals[0];
    let do_ = vals[1];
    if f == 0.0 || do_ == 0.0 {
        return None;
    }
    let di = 1.0 / (1.0 / f - 1.0 / do_);
    let mag = -di / do_;
    Some(format!(
        "=== THIN LENS SOLUTION ===\n  formula: 1/f = 1/do + 1/di, m = -di/do\n  f (focal)    = {} m\n  do (object)  = {} m\n  di (image)   = {} m\n  m (mag)      = {}x",
        frmt(f), frmt(do_), frmt(di), frmt(mag)
    ))
}

fn frmt(val: f64) -> String {
    format!("{}", val)
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

fn parse_f64s(s: &str) -> Option<Vec<f64>> {
    let s = s.trim();
    if s.is_empty() {
        return Some(vec![]);
    }
    let mut vals = vec![];
    for token in s.split_whitespace() {
        vals.push(token.parse::<f64>().ok()?);
    }
    Some(vals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solve_orbit() {
        let result = solve("orbit 3.986e14 6771000").unwrap();
        assert_eq!(result.solver, "orbit");
        assert!(result.output.contains("ORBITAL SOLUTION"));
    }

    #[test]
    fn test_solve_projectile() {
        let result = solve("projectile 100 45").unwrap();
        assert_eq!(result.solver, "projectile");
        assert!(result.output.contains("PROJECTILE SOLUTION"));
    }

    #[test]
    fn test_solve_energy() {
        let result = solve("energy 1").unwrap();
        assert_eq!(result.solver, "energy");
        assert!(result.output.contains("ENERGY SOLUTION"));
    }

    #[test]
    fn test_solve_stats() {
        let result = solve("stats 1 2 3 4 5").unwrap();
        assert_eq!(result.solver, "stats");
        assert!(result.output.contains("STATISTICS"));
    }

    #[test]
    fn test_list_solvers() {
        let solvers = list_solvers();
        assert!(!solvers.is_empty());
    }
}
