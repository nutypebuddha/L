//! Chart-matrix regression sweep (T-NEW-5).
//!
//! Runs `chart` and `build` against a fixed matrix of diverse
//! timestamp/location pairs (`tests/fixtures/chart_matrix.tsv`) and asserts
//! invariants that must hold regardless of chart content:
//!
//! * chart: for every graha `rashi == RASHIS[floor(sidereal / 30) % 12]`
//!   (the exact check that would have caught the original
//!   `sign_from_domain` corruption), `sidereal` in `[0, 360)`, `lagna` a valid
//!   rashi, output byte-identical across 3 runs, and the full-precision
//!   personality pillar weights sum to `1.0 ± 1e-6` with no negative/NaN.
//! * build: the 7 pillar weights and the objective weights sum to
//!   `1.0 ± 1e-5` (canon_f64 6-decimal display floor), none negative or NaN,
//!   and the proof-out JSON is byte-identical across 3 runs.
//!
//! Rows are `strict` (every invariant must hold) or `loose` (known-hard
//! boundary case: invariants must hold, OR the command fails loud with a
//! clear error — never silently emit corrupted-but-well-formed-looking data).
//! A near-single-pillar-dominant chart is logged for review, not swallowed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

/// Canonical 12-rashi order starting at mesha (vedic.rs `ALL_RASHIS`),
/// matching the `#[serde(rename_all = "lowercase")]` used in JSON output.
const RASHIS: [&str; 12] = [
    "mesha",
    "vrishabha",
    "mithuna",
    "karka",
    "simha",
    "kanya",
    "tula",
    "vrishchika",
    "dhanu",
    "makara",
    "kumbha",
    "meena",
];

/// Build weights are serialized through `canon_f64` (6 decimals), so the
/// displayed pillar/objective sums can drift from 1.0 by at most
/// 7 * 0.5e-6 ≈ 3.5e-6. 1e-5 is the tightest tolerance observable through the
/// JSON surface while still catching real drift; the raw weights themselves
/// are asserted at full precision (1e-6) via the chart personality block.
const BUILD_SUM_TOLERANCE: f64 = 1e-5;
/// A chart whose single dominant pillar carries ~all weight is degenerate;
/// flag it for manual review rather than silently passing (T-NEW-5).
const DOMINANCE_REVIEW_THRESHOLD: f64 = 0.999;

#[derive(Debug)]
struct Case {
    datetime: String,
    tz: String,
    latitude: String,
    longitude: String,
    label: String,
    strict: bool,
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn parse_fixture() -> Vec<Case> {
    let raw = std::fs::read_to_string(fixtures().join("chart_matrix.tsv"))
        .expect("read chart_matrix.tsv");
    let mut cases = Vec::new();
    for (lineno, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("datetime|") {
            continue;
        }
        let cols: Vec<&str> = line.split('|').map(str::trim).collect();
        assert_eq!(cols.len(), 6, "line {}: expected 6 columns", lineno + 1);
        let strict = match cols[5] {
            "strict" => true,
            "loose" => false,
            other => panic!("line {}: unknown mode '{other}'", lineno + 1),
        };
        cases.push(Case {
            datetime: cols[0].to_string(),
            tz: cols[1].to_string(),
            latitude: cols[2].to_string(),
            longitude: cols[3].to_string(),
            label: cols[4].to_string(),
            strict,
        });
    }
    assert!(!cases.is_empty(), "chart_matrix.tsv has no cases");
    cases
}

fn chart_args(c: &Case) -> Vec<String> {
    vec![
        "chart".to_string(),
        "--datetime".to_string(),
        c.datetime.clone(),
        "--tz".to_string(),
        c.tz.clone(),
        "--latitude".to_string(),
        c.latitude.clone(),
        "--longitude".to_string(),
        c.longitude.clone(),
        "--format".to_string(),
        "json".to_string(),
    ]
}

fn run_chart(c: &Case) -> Output {
    Command::new(env!("CARGO_BIN_EXE_lai"))
        .args(chart_args(c))
        .output()
        .expect("spawn lai chart")
}

fn check_chart(c: &Case) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    let mut runs: Vec<Output> = (0..3).map(|_| run_chart(c)).collect();
    let first = runs.remove(0);

    if !first.status.success() {
        if c.strict {
            return Err(format!(
                "chart exited {status} (stderr: {stderr})",
                status = first
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
                stderr = String::from_utf8_lossy(&first.stderr).trim()
            ));
        }
        return Ok(warnings); // loose: fail-loud via exit code is acceptable
    }

    let stdout = String::from_utf8(first.stdout.clone()).expect("chart stdout is utf-8");
    let v: Value =
        serde_json::from_str(&stdout).map_err(|e| format!("chart stdout is not JSON: {e}"))?;
    if v.get("chart").is_none() {
        let msg = if v.get("kind").is_some() {
            "chart failed loud (structured error)"
        } else {
            "chart returned non-chart JSON with no structured error"
        };
        if c.strict {
            return Err(format!("{msg}: {stdout:.160}"));
        }
        return Ok(warnings); // loose: structured fail-loud is acceptable
    }
    let chart = v.get("chart").expect("chart key present");

    // 1. rashi == RASHIS[floor(sidereal / 30) % 12] for every graha.
    let positions = chart
        .get("graha_positions")
        .and_then(Value::as_array)
        .ok_or("chart JSON has no graha_positions array")?;
    for gp in positions {
        let graha = gp.get("graha").and_then(Value::as_str).unwrap_or("?");
        let sidereal = gp
            .get("sidereal")
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("{graha}: no numeric sidereal"))?;
        if !(0.0..360.0).contains(&sidereal) {
            return Err(format!("{graha}: sidereal {sidereal} outside [0, 360)"));
        }
        let expected = RASHIS[(sidereal / 30.0).floor() as usize % 12];
        let rashi = gp.get("rashi").and_then(Value::as_str).unwrap_or("");
        if rashi != expected {
            return Err(format!(
                "{graha}: rashi {rashi} != RASHIS[floor({sidereal}/30)%12] = {expected}"
            ));
        }
    }

    // 2. lagna is one of the 12 valid rashis.
    let lagna = chart
        .get("lagna")
        .and_then(Value::as_str)
        .ok_or("chart lagna is null/missing (latitude/longitude not applied?)")?;
    if !RASHIS.contains(&lagna) {
        return Err(format!("lagna {lagna} is not a valid rashi"));
    }

    // 3. Byte-identical across repeated runs.
    for (i, run) in runs.iter().enumerate() {
        if run.stdout != first.stdout {
            return Err(format!("chart output differs on repeat run {}", i + 2));
        }
    }

    // 4. Full-precision personality pillar weights sum to 1.0, none negative/NaN.
    let weights = v
        .get("personality")
        .and_then(|p| p.get("pillar_weights"))
        .and_then(Value::as_array)
        .ok_or("chart JSON has no personality.pillar_weights array")?;
    let weights: Vec<f64> = weights
        .iter()
        .map(|w| w.as_f64().ok_or("pillar weight is not a number"))
        .collect::<Result<_, _>>()?;
    check_weight_sum(&weights, 1e-6, "personality pillar")?;
    check_dominance(&weights, c, &mut warnings);

    Ok(warnings)
}

fn check_weight_sum(weights: &[f64], tolerance: f64, what: &str) -> Result<(), String> {
    for w in weights {
        if *w < 0.0 || w.is_nan() {
            return Err(format!("{what} weight {w} is negative or NaN"));
        }
    }
    let sum: f64 = weights.iter().sum();
    if (sum - 1.0).abs() > tolerance {
        return Err(format!(
            "{what} weights sum to {sum}, expected 1.0 ± {tolerance}"
        ));
    }
    Ok(())
}

fn check_dominance(weights: &[f64], c: &Case, warnings: &mut Vec<String>) {
    let max = weights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if max > DOMINANCE_REVIEW_THRESHOLD {
        warnings.push(format!(
            "review: '{}' is near-single-pillar-dominant (max weight {max:.6}) — chart may be degenerate; manual sign-off required",
            c.label
        ));
    }
}

fn build_args(c: &Case, proof: &Path) -> Vec<String> {
    vec![
        "build".to_string(),
        "--domain".to_string(),
        fixtures()
            .join("generic_test_domain.toml")
            .to_string_lossy()
            .into_owned(),
        "--datetime".to_string(),
        c.datetime.clone(),
        "--tz".to_string(),
        c.tz.clone(),
        "--latitude".to_string(),
        c.latitude.clone(),
        "--longitude".to_string(),
        c.longitude.clone(),
        "--proof-out".to_string(),
        proof.to_string_lossy().into_owned(),
    ]
}

fn check_build(c: &Case) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    let proof_dir = std::env::temp_dir().join("lai-chart-matrix");
    std::fs::create_dir_all(&proof_dir).expect("create proof temp dir");

    let mut outputs: Vec<(Output, Vec<u8>)> = Vec::new();
    for i in 0..3 {
        let proof = proof_dir.join(format!("proof_{}_{}.json", sanitize(&c.label), i));
        let out = Command::new(env!("CARGO_BIN_EXE_lai"))
            .args(build_args(c, &proof))
            .output()
            .expect("spawn lai build");
        let bytes = if out.status.success() {
            std::fs::read(&proof).unwrap_or_default()
        } else {
            Vec::new()
        };
        outputs.push((out, bytes));
    }

    let (first, first_bytes) = &outputs[0];
    if !first.status.success() {
        if c.strict {
            return Err(format!(
                "build exited {status} (stderr: {stderr})",
                status = first
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
                stderr = String::from_utf8_lossy(&first.stderr).trim()
            ));
        }
        return Ok(warnings); // loose: fail-loud via exit code is acceptable
    }
    if first_bytes.is_empty() {
        return Err("build succeeded but --proof-out produced no file".to_string());
    }

    // Byte-identical proof-out across repeated runs.
    for (i, (_, bytes)) in outputs.iter().enumerate().skip(1) {
        if bytes != first_bytes {
            return Err(format!("build proof-out differs on repeat run {}", i + 1));
        }
    }

    let v: Value = serde_json::from_slice(first_bytes)
        .map_err(|e| format!("build proof-out is not JSON: {e}"))?;
    let outs = v.get("outputs").ok_or("proof-out has no outputs")?;

    let pillars = outs
        .get("pillar_weights")
        .and_then(Value::as_array)
        .ok_or("proof-out outputs.pillar_weights missing")?;
    let pillars: Vec<f64> = pillars
        .iter()
        .map(|w| w.as_f64().ok_or("pillar weight is not a number"))
        .collect::<Result<_, _>>()?;
    check_weight_sum(&pillars, BUILD_SUM_TOLERANCE, "pillar")?;
    check_dominance(&pillars, c, &mut warnings);

    let objectives = outs
        .get("objective_weights")
        .and_then(Value::as_object)
        .ok_or("proof-out outputs.objective_weights missing")?;
    let objectives: Vec<f64> = objectives
        .values()
        .map(|w| w.as_f64().ok_or("objective weight is not a number"))
        .collect::<Result<_, _>>()?;
    check_weight_sum(&objectives, BUILD_SUM_TOLERANCE, "objective")?;

    Ok(warnings)
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect()
}

#[test]
fn chart_and_build_invariants_hold_across_the_matrix() {
    let cases = parse_fixture();
    let mut failures: Vec<String> = Vec::new();
    for c in &cases {
        match check_chart(c) {
            Ok(warnings) => {
                for w in warnings {
                    eprintln!("chart-matrix note: {w}");
                }
            }
            Err(e) => failures.push(format!("{}: chart: {e}", c.label)),
        }
        match check_build(c) {
            Ok(warnings) => {
                for w in warnings {
                    eprintln!("chart-matrix note: {w}");
                }
            }
            Err(e) => failures.push(format!("{}: build: {e}", c.label)),
        }
    }
    assert!(
        failures.is_empty(),
        "chart-matrix failures:\n{}",
        failures.join("\n")
    );
}
