// Tanto solvers — 9 multi-step solver templates
// Orbit, projectile, energy, fall, KE, PE, Ohm, compound, growth

const G_GRAV: f64 = 9.80665;

/// Result of a solver computation with detailed explanation
#[derive(Debug, Clone)]
pub struct SolverResult {
    pub solver: String,
    pub output: String,
}

/// Process a solve command: "solve orbit 3.986e14 6771000"
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
        // New solvers (Phase 2)
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

/// List available solvers
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
        // New solvers (Phase 2)
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
        "=== ORBITAL SOLUTION ===
  v (velocity)    = {} m/s
  T (period)      = {} s
  KE (per kg)     = {} J/kg
  grav accel      = {} m/s^2
  verify: v^2/r == GM/r^2 => {}",
        frmt(v),
        frmt(t),
        frmt(ke),
        frmt(grav),
        verify
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
        "=== PROJECTILE SOLUTION ===
  vx           = {} m/s
  vy           = {} m/s
  flight time  = {} s
  range        = {} m
  max height   = {} m",
        frmt(vx),
        frmt(vy),
        frmt(t_flight),
        frmt(range),
        frmt(max_h)
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
        "=== ENERGY SOLUTION (E=mc^2) ===
  E (joules)   = {} J
  E (kWh)      = {} kWh
  E (tons TNT) = {} tons",
        frmt(e),
        frmt(kwh),
        frmt(tons_tnt)
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
        "=== FREE FALL SOLUTION ===
  formula: t = sqrt(2h/g), v = g*t, KE = 0.5*v^2
  height       = {} m
  fall time    = {} s
  impact vel   = {} m/s
  KE per kg    = {} J/kg
  verify: v == sqrt(2gh) => {}",
        frmt(h),
        frmt(t),
        frmt(v),
        frmt(ke),
        verify
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
        "=== KINETIC ENERGY SOLUTION ===
  formula: KE = 0.5*m*v^2, p = m*v
  mass     = {} kg
  velocity = {} m/s
  KE       = {} J
  momentum = {} kg*m/s",
        frmt(m),
        frmt(v),
        frmt(ke),
        frmt(p)
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
        "=== POTENTIAL ENERGY SOLUTION ===
  formula: PE = m*g*h (g = 9.80665 m/s^2)
  mass   = {} kg
  height = {} m
  PE     = {} J",
        frmt(m),
        frmt(h),
        frmt(pe)
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
        "=== OHM'S LAW SOLUTION ===
  formula: I = V/R, P = V*I
  voltage   = {} V
  resistance = {} ohm
  current   = {} A
  power     = {} W",
        frmt(v),
        frmt(r),
        frmt(i),
        frmt(p)
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
        "=== COMPOUND INTEREST ===
  formula: A = P*(1 + r/n)^(n*t)
  principal = {}
  rate      = {}%/year
  compounds = {}/year
  years     = {}
  final     = {}
  interest  = {}",
        frmt(principal),
        frmt(rate * 100.0),
        frmt(periods),
        frmt(years),
        frmt(amount),
        frmt(interest)
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
        "=== EXPONENTIAL GROWTH/DECAY ===
  formula: N = N0 * e^(r*t)
  initial (N0) = {}
  rate (r)     = {} /time
  time (t)     = {}
  final (N)    = {}
  doubling time = {}",
        frmt(initial),
        frmt(rate),
        frmt(t),
        frmt(final_val),
        frmt(doubling_time)
    ))
}

// ==================== New Solvers (Phase 2) ====================

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
        "=== STATISTICS ===
  n            = {}
  sum          = {}
  mean         = {}
  median       = {}
  std_dev      = {}
  variance     = {}
  min          = {}
  max          = {}",
        frmt(n),
        frmt(sum),
        frmt(mean),
        frmt(median),
        frmt(std_dev),
        frmt(variance),
        frmt(min),
        frmt(max)
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
    let _v_check = (v0 * v0 + 2.0 * a * d).sqrt();
    Some(format!(
        "=== KINEMATICS SOLUTION ===
  formulas: a = (v-v0)/t, d = (v0+v)*t/2
  v0           = {} m/s
  v            = {} m/s
  t            = {} s
  a            = {} m/s^2
  d            = {} m
  d (alt)      = {} m",
        frmt(v0),
        frmt(v),
        frmt(t),
        frmt(a),
        frmt(d),
        frmt(d2)
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
    // Ideal gas law: PV = nRT
    let p_calc = if p == 0.0 { n * r_gas * t / v } else { p };
    let v_calc = if v == 0.0 { n * r_gas * t / p } else { v };
    let n_calc = if n == 0.0 { p * v / (r_gas * t) } else { n };
    let t_calc = if t == 0.0 { p * v / (n * r_gas) } else { t };
    // Internal energy (monatomic ideal gas): U = 3/2 nRT
    let u = 1.5 * n_calc * r_gas * t_calc;
    Some(format!(
        "=== THERMODYNAMICS (Ideal Gas) ===
  formula: PV = nRT, U = 3/2 nRT
  P            = {} Pa
  V            = {} m^3
  n            = {} mol
  T            = {} K
  R            = {} J/(mol*K)
  verify: PV   = {} J
  verify: nRT  = {} J
  U (internal) = {} J",
        frmt(p_calc),
        frmt(v_calc),
        frmt(n_calc),
        frmt(t_calc),
        frmt(r_gas),
        frmt(p_calc * v_calc),
        frmt(n_calc * r_gas * t_calc),
        frmt(u)
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
    let v_ref = 12.0; // reference voltage for power calc
    let i_series = v_ref / r_series;
    let i_parallel = v_ref / r_parallel;
    let p_series = v_ref * i_series;
    let p_parallel = v_ref * i_parallel;
    Some(format!(
        "=== CIRCUIT SOLUTION (2 resistors) ===
  formulas: R_series = R1+R2, R_parallel = 1/(1/R1+1/R2)
  R1           = {} ohm
  R2           = {} ohm
  R_series     = {} ohm
  R_parallel   = {} ohm
  (at V={}V)
  I_series     = {} A
  I_parallel   = {} A
  P_series     = {} W
  P_parallel   = {} W",
        frmt(r1),
        frmt(r2),
        frmt(r_series),
        frmt(r_parallel),
        frmt(v_ref),
        frmt(i_series),
        frmt(i_parallel),
        frmt(p_series),
        frmt(p_parallel)
    ))
}

fn solve_tvm(args: &str) -> Option<String> {
    let vals = parse_f64s(args)?;
    if vals.len() < 3 {
        return None;
    }
    let pv = vals[0];
    let r = vals[1]; // interest rate per period (decimal)
    let n = vals[2]; // number of periods
    let fv = pv * (1.0 + r).powf(n);
    let npv = fv - pv;
    Some(format!(
        "=== TIME VALUE OF MONEY ===
  formula: FV = PV * (1+r)^n
  PV (present) = {} USD
  r (periodic) = {}%
  n (periods)  = {}
  FV (future)  = {} USD
  NPV          = {} USD",
        frmt(pv),
        frmt(r * 100.0),
        frmt(n),
        frmt(fv),
        frmt(npv)
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
        "=== THIN LENS SOLUTION ===
  formula: 1/f = 1/do + 1/di, m = -di/do
  f (focal)    = {} m
  do (object)  = {} m
  di (image)   = {} m
  m (mag)      = {}x",
        frmt(f),
        frmt(do_),
        frmt(di),
        frmt(mag)
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
