use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Algorithm used to compute delta-T (TT minus UT1) for a given epoch.
pub enum DeltaTModel {
    /// Stephenson, Morrison & Hohenkerk (2016), "Measurement of the Earth's
    /// Rotation: 720 BC to AD 2015" (Proc. R. Soc. A 472:20160404).
    ///
    /// This is the GENUINE published model, not an approximation:
    ///   * `[-720, 2016]` — the published 54-row cubic *regression* spline
    ///     (Table S15 of the supplement; coefficients read verbatim, evaluated
    ///     with the local normalized variable `t = (Y-K_i)/(K_{i+1}-K_i)` and
    ///     `ΔT = a0 + a1·t + a2·t² + a3·t³`).
    ///   * outside `[-720, 2016]` — the model's own long-term extrapolation:
    ///     the integral of the published length-of-day formula
    ///     `lod = 1.72·τ − 3.5·sin(2π·(τ+0.75)/14)` ms/day, `τ = (Y-1825)/100`,
    ///     with the integration constant chosen for continuity at the domain
    ///     boundary. (NOT the §4 discussion parabola, which does not match the
    ///     spline at the boundary and is only a background fit for plotting.)
    ///
    /// The spline's last fitted knot is AD 2016. Beyond it the value is an
    /// extrapolation and `delta_t_with_uncertainty` widens σ accordingly.
    /// The published model carries a scalar per-epoch error envelope only —
    /// there is NO published coefficient covariance matrix, so none is claimed.
    /// See CREDITS.md / docs/ACCURACY.md.
    StephensonMorrisonHohenkerk2016,
    /// Espenak & Meeus (2006) -- NASA polynomial expressions.
    EspenakMeeus2006,
    /// Morrison & Stephenson (2004) -- simple parabolic model.
    MorrisonStephenson2004,
    /// Always returns zero (no TT-UT1 correction).
    Zero,
}

/// Compute delta-T (TT minus UT1) in seconds for a given Julian Date.
pub fn delta_t(jd_ut: f64, model: &DeltaTModel) -> f64 {
    match model {
        DeltaTModel::StephensonMorrisonHohenkerk2016 => smh2016(jd_ut),
        DeltaTModel::EspenakMeeus2006 => espenak_meeus(jd_ut),
        DeltaTModel::MorrisonStephenson2004 => morrison_stephenson_2004(jd_ut),
        DeltaTModel::Zero => 0.0,
    }
}

/// First (−720.0) and last (2016.0) fitted knots of the SMH2016 cubic spline.
/// Inside this closed interval ΔT is the published regression spline; outside,
/// it is the long-term lod-integral extrapolation. The upper bound also marks
/// where the directly-fitted era ends and the uncertainty begins to grow.
const SMH_FIRST_KNOT_YEAR: f64 = -720.0;
const SMH_LAST_KNOT_YEAR: f64 = 2016.0;

/// Compute delta-T with an estimated 1-sigma uncertainty in seconds.
///
/// The σ returned is **model-specific** — it is NOT a single universal envelope:
///   * [`DeltaTModel::Zero`] — ΔT is identically 0 by construction, so σ = 0.
///     (Returning the SMH envelope here would falsely attribute uncertainty to a
///     value the caller asked to be exactly zero.)
///   * [`DeltaTModel::StephensonMorrisonHohenkerk2016`] — the published NAO/SMH
///     per-epoch scalar error envelope inside the fitted era, widened past the
///     last fitted knot (max of the smooth NAO envelope and the Huber random
///     walk). This is the only model with a published companion σ.
///   * [`DeltaTModel::EspenakMeeus2006`] / [`DeltaTModel::MorrisonStephenson2004`]
///     — these polynomial fits ship NO published per-epoch σ. We do NOT borrow
///     the SMH envelope (the fits differ from SMH by far more than the SMH σ at
///     many epochs, so the SMH envelope would understate the real disagreement).
///     σ is returned as `f64::NAN` to signal "uncertainty unavailable for this
///     model" rather than a fabricated number.
pub fn delta_t_with_uncertainty(jd_ut: f64, model: &DeltaTModel) -> (f64, f64) {
    let dt = delta_t(jd_ut, model);
    // σ is only published for the SMH2016 model. Zero has σ = 0 by construction;
    // the polynomial fits carry no published per-epoch envelope.
    match model {
        DeltaTModel::Zero => return (dt, 0.0),
        DeltaTModel::EspenakMeeus2006 | DeltaTModel::MorrisonStephenson2004 => {
            return (dt, f64::NAN);
        }
        DeltaTModel::StephensonMorrisonHohenkerk2016 => {}
    }
    let year = jd_to_year(jd_ut);
    let sigma = if year > SMH_LAST_KNOT_YEAR {
        // Past the last fitted spline knot (AD 2016) ΔT is an extrapolation.
        // Two published uncertainty estimates exist for this regime and they
        // disagree by ~5×: the NAO/SMH scalar envelope (smooth lod-trend, ≈6 s
        // at 2050 / ≈10 s at 2100) and Espenak's Huber Brownian-motion model
        // (random walk, ≈15.5 s at 2050 / ≈47.9 s at 2100). We report the
        // LARGER of the two so the stated σ never understates: the smooth
        // envelope assumes the long-term trend holds, the random walk does not,
        // and for a process driven by unpredictable core-mantle coupling the
        // more conservative bound is the honest one. Floored at the observed-era
        // precision (0.1 s). Refs: NAO "Earth Rotation" σ table; EclipseWise
        // "Uncertainty in ΔT" (Huber calibration year +2005).
        published_sigma(year)
            .max(huber_sigma(year, 2005.0))
            .max(0.1)
    } else {
        // Within the fitted era use the published NAO/SMH scalar error envelope
        // (eclipse/occultation fit scatter, per epoch). This replaces the old
        // ad-hoc Morrison–Stephenson parabola σ = 0.8·((Y−1820)/100)², which
        // overstated ancient σ by ~3× (≈516 s vs the published 180 s at −720).
        published_sigma(year).max(0.1)
    };
    (dt, sigma)
}

fn jd_to_year(jd: f64) -> f64 {
    2000.0 + (jd - 2_451_545.0) / 365.25
}

/// Espenak's Huber Brownian-motion 1-σ uncertainty model for ΔT outside the
/// directly-observed era (EclipseWise, "Uncertainty in ΔT"):
///
/// ```text
/// σ = 365.25 · N · sqrt[ (N·Q/3)·(1 + N/M) ] / 1000   seconds
/// ```
///
/// with `N = |target_year − calibration_year|`, `M = 2500` yr and
/// `Q = 0.058 ms²/yr`. The published calibration year is +2005 for future
/// targets (and −500 for targets before 500 BCE). Yields ≈47.9 s at 2100.
fn huber_sigma(year: f64, calibration_year: f64) -> f64 {
    const M: f64 = 2500.0;
    const Q: f64 = 0.058; // ms² / yr
    let n = (year - calibration_year).abs();
    365.25 * n * ((n * Q / 3.0) * (1.0 + n / M)).sqrt() / 1000.0
}

// ===========================================================================
// Stephenson–Morrison–Hohenkerk (2016) ΔT model
// ===========================================================================
//
// Spline coefficients are taken verbatim from the supplement of
//   Stephenson, Morrison & Hohenkerk (2016), Proc. R. Soc. A 472:20160404,
//   "Table S15: The Polynomial Coefficients for DT -720.0 to 2016.0"
// (figshare dataset 4290866, CC-BY-4.0). They form a least-squares cubic
// *regression* spline: continuity of value / 1st / 2nd derivative at interior
// knots is built into the coefficients, so they are read in mechanically — we
// do NOT re-solve a tridiagonal system. The 54 spline rows are transcribed and
// checked directly against the SMH2016 Table-S15 supplement; the lod-integral
// tail formula and the eps_tab σ-envelope convention were cross-checked against
// ytliu0/DeltaT (MIT) — note its spline table is the 2020/2025-extended variant
// with different knots and c1/c2, so only the tail/σ conventions transfer.

/// Start year `K_i` of each spline interval `[K_i, K_{i+1}]` (row 1 starts at
/// −720; the implied end of the last interval is [`SMH_LAST_KNOT_YEAR`]).
const SMH_KNOTS: [f64; 54] = [
    -720.0, 400.0, 1000.0, 1500.0, 1600.0, 1650.0, 1720.0, 1800.0, 1810.0, 1820.0, 1830.0, 1840.0,
    1850.0, 1855.0, 1860.0, 1865.0, 1870.0, 1875.0, 1880.0, 1885.0, 1890.0, 1895.0, 1900.0, 1905.0,
    1910.0, 1915.0, 1920.0, 1925.0, 1930.0, 1935.0, 1940.0, 1945.0, 1950.0, 1953.0, 1956.0, 1959.0,
    1962.0, 1965.0, 1968.0, 1971.0, 1974.0, 1977.0, 1980.0, 1983.0, 1986.0, 1989.0, 1992.0, 1995.0,
    1998.0, 2001.0, 2004.0, 2007.0, 2010.0, 2013.0,
];

/// End year `K_{i+1}` of each spline interval (parallel to [`SMH_KNOTS`]).
const SMH_KNOTS_END: [f64; 54] = [
    400.0, 1000.0, 1500.0, 1600.0, 1650.0, 1720.0, 1800.0, 1810.0, 1820.0, 1830.0, 1840.0, 1850.0,
    1855.0, 1860.0, 1865.0, 1870.0, 1875.0, 1880.0, 1885.0, 1890.0, 1895.0, 1900.0, 1905.0, 1910.0,
    1915.0, 1920.0, 1925.0, 1930.0, 1935.0, 1940.0, 1945.0, 1950.0, 1953.0, 1956.0, 1959.0, 1962.0,
    1965.0, 1968.0, 1971.0, 1974.0, 1977.0, 1980.0, 1983.0, 1986.0, 1989.0, 1992.0, 1995.0, 1998.0,
    2001.0, 2004.0, 2007.0, 2010.0, 2013.0, 2016.0,
];

/// Cubic coefficients `[a0, a1, a2, a3]` for each interval (parallel to
/// [`SMH_KNOTS`]). `ΔT = a0 + a1·t + a2·t² + a3·t³`, `t = (Y−K_i)/(K_{i+1}−K_i)`.
#[rustfmt::skip]
const SMH_COEFFS: [[f64; 4]; 54] = [
    [20550.593, -21268.478, 11863.418, -4541.129],
    [ 6604.404,  -5981.266,  -505.093,  1349.609],
    [ 1467.654,  -2452.187,  2460.927, -1183.759],
    [  292.635,   -216.322,   -43.614,    56.681],
    [   89.380,    -66.754,    31.607,   -10.497],
    [   43.736,    -49.043,     0.227,    15.811],
    [   10.730,     -1.321,    62.250,   -52.946],
    [   18.714,     -4.457,    -1.509,     2.507],
    [   15.255,      0.046,     6.012,    -4.634],
    [   16.679,     -1.831,    -7.889,     3.799],
    [   10.758,     -6.211,     3.509,    -0.388],
    [    7.668,     -0.357,     2.345,    -0.338],
    [    9.317,      1.659,     0.332,    -0.932],
    [   10.376,     -0.472,    -2.463,     1.596],
    [    9.038,     -0.610,     2.325,    -2.497],
    [    8.256,     -3.450,    -5.166,     2.729],
    [    2.369,     -5.596,     3.020,    -0.919],
    [   -1.126,     -2.312,     0.264,    -0.037],
    [   -3.211,     -1.894,     0.154,     0.562],
    [   -4.388,      0.101,     1.841,    -1.438],
    [   -3.884,     -0.531,    -2.473,     1.870],
    [   -5.017,      0.134,     3.138,    -0.232],
    [   -1.977,      5.715,     2.443,    -1.257],
    [    4.923,      6.828,    -1.329,     0.720],
    [   11.142,      6.330,     0.831,    -0.825],
    [   17.479,      5.518,    -1.643,     0.262],
    [   21.617,      3.020,    -0.856,     0.008],
    [   23.789,      1.333,    -0.831,     0.127],
    [   24.418,      0.052,    -0.449,     0.142],
    [   24.164,     -0.419,    -0.022,     0.702],
    [   24.426,      1.645,     2.086,    -1.106],
    [   27.050,      2.499,    -1.232,     0.614],
    [   28.932,      1.127,     0.220,    -0.277],
    [   30.002,      0.737,    -0.610,     0.631],
    [   30.760,      1.409,     1.282,    -0.799],
    [   32.652,      1.577,    -1.115,     0.507],
    [   33.621,      0.868,     0.406,     0.199],
    [   35.093,      2.275,     1.002,    -0.414],
    [   37.956,      3.035,    -0.242,     0.202],
    [   40.951,      3.157,     0.364,    -0.229],
    [   44.244,      3.198,    -0.323,     0.172],
    [   47.291,      3.069,     0.193,    -0.192],
    [   50.361,      2.878,    -0.384,     0.081],
    [   52.936,      2.354,    -0.140,    -0.166],
    [   54.984,      1.577,    -0.637,     0.448],
    [   56.373,      1.649,     0.709,    -0.277],
    [   58.453,      2.235,    -0.122,     0.111],
    [   60.677,      2.324,     0.212,    -0.315],
    [   62.899,      1.804,    -0.732,     0.112],
    [   64.082,      0.675,    -0.396,     0.193],
    [   64.555,      0.463,     0.184,    -0.008],
    [   65.194,      0.809,     0.161,    -0.101],
    [   66.063,      0.828,    -0.142,     0.168],
    [   66.917,      1.046,     0.360,    -0.282],
];

/// Integration constant making the lod-integral tail continuous with the spline
/// at the lower domain boundary (Y = −720). Equals `spline(−720) − lod(−720)`.
/// Re-derived for the 2016 spline (`spline(−720) = 20550.593`); guarded by
/// `lod_tail_is_continuous_with_spline`.
const SMH_TAIL_C1: f64 = 179.752_739_546_147_5;

/// Integration constant making the lod-integral tail continuous with the spline
/// at the upper domain boundary (Y = 2016). Equals `spline(2016) − lod(2016)`.
const SMH_TAIL_C2: f64 = -151.409_208_815_920_07;

fn smh2016(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);
    if year < SMH_FIRST_KNOT_YEAR {
        smh_lod_tail(year, SMH_TAIL_C1)
    } else if year > SMH_LAST_KNOT_YEAR {
        smh_lod_tail(year, SMH_TAIL_C2)
    } else {
        smh_spline(year)
    }
}

/// Evaluate the SMH2016 cubic regression spline at `year ∈ [−720, 2016]`.
fn smh_spline(year: f64) -> f64 {
    // Largest interval start `K_i <= year`. Linear scan over 54 short intervals
    // is cheaper than a branchy binary search and keeps the data contiguous.
    let mut i = 0usize;
    while i + 1 < SMH_KNOTS.len() && SMH_KNOTS[i + 1] <= year {
        i += 1;
    }
    let t = (year - SMH_KNOTS[i]) / (SMH_KNOTS_END[i] - SMH_KNOTS[i]);
    let [a0, a1, a2, a3] = SMH_COEFFS[i];
    a0 + t * (a1 + t * (a2 + t * a3)) // Horner
}

/// The SMH long-term extrapolation tail: the analytic integral of the published
/// length-of-day formula `lod = 1.72·τ − 3.5·sin(2π·(τ+0.75)/14)` ms/day, with
/// `τ = (Y − 1825)/100`. Integrating (1 ms = 1e-3 s, 1 Julian yr = 365.25 d):
///
/// ```text
/// ΔT(Y) = C + 31.4115·τ² + (894.8625/π)·cos(2π·(τ+0.75)/14)   seconds
/// ```
///
/// `C` is the integration constant fixing continuity at the relevant domain
/// boundary (`SMH_TAIL_C1` below −720, `SMH_TAIL_C2` above 2016).
fn smh_lod_tail(year: f64, c: f64) -> f64 {
    let tau = 0.01 * (year - 1825.0);
    c + 31.4115 * tau * tau + 284.843_580_525_142_4 * (0.448_798_950_512_827_6 * (tau + 0.75)).cos()
}

/// Published NAO/SMH scalar 1-σ error envelope for ΔT (seconds). These are the
/// per-epoch uncertainties the model authors publish alongside ΔT (derived from
/// eclipse/occultation fit scatter); there is no coefficient covariance matrix.
///
/// Reproduced faithfully from the NAO "Earth Rotation — ΔT" table as a
/// **left-continuous step lookup** (the largest tabulated year ≤ `year` gives
/// the value) over `[−2000, 2500]` — exactly the convention of the reference
/// implementation `ytliu0/DeltaT::DeltaT_error_estimate`. In the historical era
/// σ decreases toward the present, so the step always takes the older, larger
/// value and never understates. Outside the table, σ follows the NAO quadratic
/// tails `0.74e-4·(Y−1825)²` (before −2000) and `2.2e-4·(Y−1825)²` (from 2500),
/// each calibrated to join the table endpoints (−2000→≈1080 s, 2500→≈100 s);
/// the NAO note flags these tails as "probably not reliable" extrapolations.
fn published_sigma(year: f64) -> f64 {
    // (tabulated year, 1-σ seconds) — all 44 NAO rows, verbatim.
    const TAB: [(f64, f64); 44] = [
        (-2000.0, 1080.0),
        (-1600.0, 720.0),
        (-900.0, 360.0),
        (-720.0, 180.0),
        (-700.0, 170.0),
        (-600.0, 160.0),
        (-500.0, 150.0),
        (-400.0, 130.0),
        (-300.0, 120.0),
        (-200.0, 110.0),
        (-100.0, 100.0),
        (0.0, 90.0),
        (100.0, 80.0),
        (200.0, 70.0),
        (300.0, 60.0),
        (400.0, 50.0),
        (500.0, 40.0),
        (700.0, 30.0),
        (800.0, 25.0),
        (900.0, 20.0),
        (1000.0, 15.0),
        (1620.0, 20.0),
        (1660.0, 15.0),
        (1670.0, 10.0),
        (1680.0, 5.0),
        (1730.0, 2.0),
        (1770.0, 1.0),
        (1800.0, 0.5),
        (1802.0, 0.4),
        (1805.0, 0.3),
        (1809.0, 0.2),
        (1831.0, 0.1),
        (1870.0, 0.05),
        (2025.0, 0.1),
        (2025.5, 0.2),
        (2026.0, 1.0),
        (2030.0, 2.0),
        (2040.0, 4.0),
        (2050.0, 6.0),
        (2100.0, 10.0),
        (2200.0, 20.0),
        (2300.0, 30.0),
        (2400.0, 50.0),
        (2500.0, 100.0),
    ];
    let lo = TAB[0].0;
    let hi = TAB[TAB.len() - 1].0;
    if year < lo {
        let d = year - 1825.0;
        return 0.74e-4 * d * d;
    }
    if year >= hi {
        let d = year - 1825.0;
        return 2.2e-4 * d * d;
    }
    // Left-continuous step: largest tabulated year ≤ `year`.
    let mut sigma = TAB[0].1;
    for &(y, s) in &TAB {
        if y <= year {
            sigma = s;
        } else {
            break;
        }
    }
    sigma
}

fn espenak_meeus(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);
    espenak_meeus_segment(year)
}

fn espenak_meeus_segment(year: f64) -> f64 {
    if year >= 2015.0 {
        let t = year - 2015.0;
        67.62 + 0.3645 * t + 0.0039755 * t * t
    } else if year >= 2005.0 {
        let t = year - 2005.0;
        64.69 + 0.2930 * t
    } else if year >= 1986.0 {
        let t = year - 2000.0;
        63.86 + 0.3345 * t - 0.060374 * t * t
            + 0.0017275 * t.powi(3)
            + 0.000651814 * t.powi(4)
            + 0.00002373599 * t.powi(5)
    } else if year >= 1961.0 {
        let t = year - 1975.0;
        45.45 + 1.067 * t - t * t / 260.0 - t.powi(3) / 718.0
    } else if year >= 1941.0 {
        let t = year - 1950.0;
        29.07 + 0.407 * t - t * t / 233.0 + t.powi(3) / 2547.0
    } else if year >= 1920.0 {
        let t = year - 1920.0;
        21.20 + 0.84493 * t - 0.076100 * t * t + 0.0020936 * t.powi(3)
    } else if year >= 1900.0 {
        let t = year - 1900.0;
        -2.79 + 1.494119 * t - 0.0598939 * t * t + 0.0061966 * t.powi(3) - 0.000197 * t.powi(4)
    } else if year >= 1860.0 {
        let t = year - 1860.0;
        7.62 + 0.5737 * t - 0.251754 * t * t + 0.01680668 * t.powi(3) - 0.0004473624 * t.powi(4)
            + t.powi(5) / 233174.0
    } else if year >= 1800.0 {
        let t = year - 1800.0;
        13.72 - 0.332447 * t + 0.0068612 * t * t + 0.0041116 * t.powi(3) - 0.00037436 * t.powi(4)
            + 0.0000121272 * t.powi(5)
            - 0.0000001699 * t.powi(6)
            + 0.000000000875 * t.powi(7)
    } else if year >= 1700.0 {
        let t = year - 1700.0;
        8.83 + 0.1603 * t - 0.0059285 * t * t + 0.00013336 * t.powi(3) - t.powi(4) / 1174000.0
    } else if year >= 1600.0 {
        let t = year - 1600.0;
        120.0 - 0.9808 * t - 0.01532 * t * t + t.powi(3) / 7129.0
    } else if year >= 500.0 {
        let t = (year - 1000.0) / 100.0;
        1574.2 - 556.01 * t + 71.23472 * t * t + 0.319781 * t.powi(3)
            - 0.8503463 * t.powi(4)
            - 0.005050998 * t.powi(5)
            + 0.0083572073 * t.powi(6)
    } else if year >= -500.0 {
        let t = year / 100.0;
        10583.6 - 1014.41 * t + 33.78311 * t * t - 5.952053 * t.powi(3) - 0.1798452 * t.powi(4)
            + 0.022174192 * t.powi(5)
            + 0.0090316521 * t.powi(6)
    } else {
        let u = (year - 1820.0) / 100.0;
        -20.0 + 32.0 * u * u
    }
}

fn morrison_stephenson_2004(jd_ut: f64) -> f64 {
    let year = jd_to_year(jd_ut);
    let u = (year - 1820.0) / 100.0;
    -20.0 + 32.0 * u * u
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inverse of `jd_to_year` for constructing test epochs from a calendar year.
    fn year_to_jd(year: f64) -> f64 {
        2_451_545.0 + (year - 2000.0) * 365.25
    }

    #[test]
    fn delta_t_j2000_is_about_64_seconds() {
        let jd = 2_451_545.0;
        let dt = delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        assert!(
            (dt - 63.83).abs() < 2.0,
            "delta-T at J2000 should be ~63.83s, got {dt}"
        );
    }

    #[test]
    fn delta_t_ancient_is_large() {
        let jd = 2_451_545.0 - 365.25 * 3000.0; // ~1000 BCE
        let dt = delta_t(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        assert!(
            dt > 20000.0,
            "delta-T at 1000 BCE should be > 20000s, got {dt}"
        );
    }

    #[test]
    fn delta_t_with_uncertainty_grows_for_ancient() {
        let modern = 2_451_545.0;
        let ancient = 2_451_545.0 - 365.25 * 3000.0;
        let (_, sigma_modern) =
            delta_t_with_uncertainty(modern, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        let (_, sigma_ancient) =
            delta_t_with_uncertainty(ancient, &DeltaTModel::StephensonMorrisonHohenkerk2016);
        assert!(
            sigma_ancient > sigma_modern * 100.0,
            "Ancient uncertainty should be >> modern"
        );
    }

    #[test]
    fn zero_model_returns_zero() {
        let dt = delta_t(2_451_545.0, &DeltaTModel::Zero);
        assert_eq!(dt, 0.0);
    }

    /// C10: σ must follow the SELECTED model, not silently borrow the SMH
    /// envelope. `Zero` is exactly zero (σ = 0); the polynomial fits publish no
    /// per-epoch envelope (σ = NaN = "unavailable"); only SMH2016 has a real σ.
    #[test]
    fn uncertainty_is_model_specific_not_borrowed_from_smh() {
        // Probe an ancient epoch and a modern epoch — both must obey the rule.
        for jd in [2_451_545.0, 2_451_545.0 - 365.25 * 3000.0] {
            // Zero: ΔT == 0 AND σ == 0 (NOT the SMH envelope of ~180 s at −720).
            let (dt0, s0) = delta_t_with_uncertainty(jd, &DeltaTModel::Zero);
            assert_eq!(dt0, 0.0, "Zero ΔT must be 0 at JD {jd}");
            assert_eq!(
                s0, 0.0,
                "Zero σ must be 0, not the SMH envelope, at JD {jd}"
            );

            // Espenak–Meeus and Morrison–Stephenson: no published σ → NaN.
            for model in [
                DeltaTModel::EspenakMeeus2006,
                DeltaTModel::MorrisonStephenson2004,
            ] {
                let (_, s) = delta_t_with_uncertainty(jd, &model);
                assert!(
                    s.is_nan(),
                    "σ for {model:?} must be NaN (unavailable), got {s} at JD {jd}"
                );
            }

            // SMH2016 keeps a real, finite, positive published σ.
            let (_, s_smh) =
                delta_t_with_uncertainty(jd, &DeltaTModel::StephensonMorrisonHohenkerk2016);
            assert!(
                s_smh.is_finite() && s_smh > 0.0,
                "SMH σ must be finite and positive, got {s_smh} at JD {jd}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // SMH2016 spline: validation against the paper's own published ΔT values.
    // -----------------------------------------------------------------------

    /// The genuine SMH2016 spline must reproduce the published Table-S15 ΔT
    /// values (the spline's own evaluation; transcription + Horner check). These
    /// are the values the 2016 paper reports for the historical fitted range.
    #[test]
    fn smh_spline_matches_published_values() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        // (year, published ΔT seconds) — from the SMH2016 Table-S15 spline.
        let cases = [
            (-720.0, 20550.593),
            (-500.0, 16796.179),
            (0.0, 10574.295),
            (500.0, 5599.744),
            (1000.0, 1467.654),
            (1500.0, 292.635),
            (1600.0, 89.380),
            (1800.0, 18.714),
            (1900.0, -1.977),
            (2000.0, 63.810),
            (2016.0, 68.041),
        ];
        for (year, expected) in cases {
            let got = delta_t(year_to_jd(year), &model);
            assert!(
                (got - expected).abs() < 0.05,
                "ΔT(SMH spline) at {year}: expected {expected:.3} s, got {got:.3} s"
            );
        }
    }

    /// The SMH spline is fit to the same IERS data Swiss/JPL use, so in the
    /// telescopic era it must track observed ΔT to within the fit residual.
    #[test]
    fn smh_spline_tracks_iers_modern_era() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        // (year, IERS observed ΔT seconds, tolerance = fit residual).
        let cases = [
            (1950.0, 29.15, 0.4),
            (1980.0, 50.54, 0.4),
            (2000.0, 63.83, 0.1),
            (2010.0, 66.07, 0.1),
        ];
        for (year, observed, tol) in cases {
            let got = delta_t(year_to_jd(year), &model);
            assert!(
                (got - observed).abs() < tol,
                "ΔT at {year}: observed {observed:.2} s, spline {got:.3} s (tol {tol})"
            );
        }
    }

    /// The lod-integral extrapolation tail must join the spline continuously at
    /// both domain boundaries — this is what `SMH_TAIL_C1`/`SMH_TAIL_C2` encode,
    /// so the test guards those constants against silent drift.
    #[test]
    fn lod_tail_is_continuous_with_spline() {
        // Lower boundary, Y = −720.
        let spline_lo = smh_spline(SMH_FIRST_KNOT_YEAR);
        let tail_lo = smh_lod_tail(SMH_FIRST_KNOT_YEAR, SMH_TAIL_C1);
        assert!(
            (spline_lo - tail_lo).abs() < 1e-6,
            "lod tail discontinuous at −720: spline={spline_lo}, tail={tail_lo}"
        );
        // Upper boundary, Y = 2016.
        let spline_hi = smh_spline(SMH_LAST_KNOT_YEAR);
        let tail_hi = smh_lod_tail(SMH_LAST_KNOT_YEAR, SMH_TAIL_C2);
        assert!(
            (spline_hi - tail_hi).abs() < 1e-6,
            "lod tail discontinuous at 2016: spline={spline_hi}, tail={tail_hi}"
        );
    }

    /// The whole ΔT function must be continuous across every interior point,
    /// including the 500 CE region (interior to spline row [400, 1000]) which a
    /// previous polynomial-hybrid implementation discontinuously approximated.
    #[test]
    fn delta_t_continuous_across_domain() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        for &year in &[-720.0, -500.0, 500.0, 1000.0, 1500.0, 1900.0, 2016.0] {
            let below = delta_t(year_to_jd(year - 0.001), &model);
            let above = delta_t(year_to_jd(year + 0.001), &model);
            let jump = (above - below).abs();
            assert!(
                jump < 0.5,
                "ΔT must be continuous across {year}: jump of {jump} s \
                 (below={below}, above={above})"
            );
        }
    }

    // P1-17: past the last fitted knot (2016) ΔT uncertainty must grow rather
    // than stay pinned at the observed-era 0.1 s.
    #[test]
    fn future_uncertainty_grows_beyond_observed() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        let (_, sigma_2010) = delta_t_with_uncertainty(year_to_jd(2010.0), &model);
        let (_, sigma_2100) = delta_t_with_uncertainty(year_to_jd(2100.0), &model);
        assert_eq!(
            sigma_2010, 0.1,
            "2010 is within the fitted era; σ floors at 0.1 s, got {sigma_2010}"
        );
        // Beyond 2016 we report max(published envelope, Huber random walk). The
        // Huber term (calibration year 2005) dominates the future: ≈47.9 s at
        // 2100 and ≈15.5 s at 2050. A flat 0.1 s, the NAO smooth envelope alone
        // (≈10 s at 2100), or the Morrison-Stephenson parabola (≈6.3 s) would
        // all understate this.
        assert!(
            (sigma_2100 - 47.9).abs() < 1.0,
            "2100 σ should match the Huber bound ≈47.9 s, got {sigma_2100}"
        );
        let (_, sigma_2050) = delta_t_with_uncertainty(year_to_jd(2050.0), &model);
        assert!(
            (sigma_2050 - 15.5).abs() < 1.0,
            "2050 σ should match the Huber bound ≈15.5 s, got {sigma_2050}"
        );
    }

    /// σ must FAITHFULLY reproduce the published NAO envelope as a left-continuous
    /// step lookup (not interpolation, which understates), with quadratic tails
    /// outside [−2000, 2500]. Reference: ytliu0/DeltaT::DeltaT_error_estimate.
    #[test]
    fn sigma_reproduces_published_step_envelope() {
        let model = DeltaTModel::StephensonMorrisonHohenkerk2016;
        let sig = |y: f64| delta_t_with_uncertainty(year_to_jd(y), &model).1;
        // Exact tabulated knots.
        assert!(
            (sig(-720.0) - 180.0).abs() < 1e-9,
            "−720 → 180 s, got {}",
            sig(-720.0)
        );
        assert!(
            (sig(1000.0) - 15.0).abs() < 1e-9,
            "1000 → 15 s, got {}",
            sig(1000.0)
        );
        // Between knots the step takes the OLDER, larger value (never understates):
        // −1000 ∈ [−1600, −900) → 720 s (a linear interp would give ≈411 s).
        assert!(
            (sig(-1000.0) - 720.0).abs() < 1e-9,
            "−1000 → 720 s (step), got {}",
            sig(-1000.0)
        );
        // Deep ancient uses the NAO quadratic tail, NOT a clamp at 1080 s:
        // −3000 → 0.74e-4·(−3000−1825)² ≈ 1722.8 s.
        assert!(
            (sig(-3000.0) - 1722.8).abs() < 1.0,
            "−3000 → ≈1723 s (quad tail), got {}",
            sig(-3000.0)
        );
    }
}
