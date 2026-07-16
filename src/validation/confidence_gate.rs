use crate::scoring::ball::{Ball, GateResult};
use crate::scoring::pin::GateKind;

#[derive(Debug, Clone)]
pub struct Platt {
    pub a: f64,
    pub b: f64,
}

impl Platt {
    pub fn new(a: f64, b: f64) -> Self {
        Platt { a, b }
    }

    pub fn calibrate(&self, raw: f64) -> f64 {
        let z = self.a * raw + self.b;
        let clamped = z.clamp(-500.0, 500.0);
        1.0 / (1.0 + (-clamped).exp())
    }

    pub fn identity() -> Self {
        Platt { a: 0.0, b: 0.0 }
    }
}

#[derive(Debug, Clone)]
pub struct TemperatureScaling {
    pub temperature: f64,
}

impl TemperatureScaling {
    pub fn new(temperature: f64) -> Self {
        TemperatureScaling {
            temperature: temperature.max(0.01),
        }
    }

    pub fn probability(&self, logit: f64) -> f64 {
        let scaled = logit / self.temperature;
        let clamped = scaled.clamp(-500.0, 500.0);
        1.0 / (1.0 + (-clamped).exp())
    }

    pub fn fit(logits: &[f64], labels: &[bool], learning_rate: f64, iterations: usize) -> f64 {
        if logits.is_empty() || logits.len() != labels.len() {
            return 1.0;
        }

        let mut temp = 1.0;

        for _ in 0..iterations {
            let mut grad = 0.0;

            for (logit, &label) in logits.iter().zip(labels.iter()) {
                let z = logit / temp;
                let clamped = z.clamp(-500.0, 500.0);
                let p = 1.0 / (1.0 + (-clamped).exp());

                let d_nll = if label {
                    -logit / (temp * temp) * p * (1.0 - p)
                } else {
                    logit / (temp * temp) * p * (1.0 - p)
                };
                grad += d_nll;
            }

            grad /= logits.len() as f64;
            temp -= learning_rate * grad;
            temp = temp.max(0.01);
        }

        temp
    }
}

pub fn expected_calibration_error(confidences: &[f64], correct: &[bool], n_bins: usize) -> f64 {
    if confidences.is_empty() || confidences.len() != correct.len() {
        return 0.0;
    }

    let n = confidences.len() as f64;
    let bin_size = 1.0 / n_bins as f64;
    let mut ece = 0.0;

    for b in 0..n_bins {
        let lower = b as f64 * bin_size;
        let upper = (b + 1) as f64 * bin_size;

        let bin_samples: Vec<(f64, bool)> = confidences
            .iter()
            .zip(correct.iter())
            .filter(|(c, _)| **c >= lower && **c < upper)
            .map(|(&c, &r)| (c, r))
            .collect();

        if bin_samples.is_empty() {
            continue;
        }

        let bin_size_actual = bin_samples.len() as f64;
        let mean_conf = bin_samples.iter().map(|(c, _)| c).sum::<f64>() / bin_size_actual;
        let mean_acc = bin_samples
            .iter()
            .map(|(_, r)| if *r { 1.0 } else { 0.0 })
            .sum::<f64>()
            / bin_size_actual;

        ece += (bin_size_actual / n) * (mean_conf - mean_acc).abs();
    }

    ece
}

pub fn bayesian_update(prior: f64, sensitivity: f64, specificity: f64, positive_test: bool) -> f64 {
    let prior = prior.clamp(0.001, 0.999);
    let sensitivity = sensitivity.clamp(0.001, 0.999);
    let specificity = specificity.clamp(0.001, 0.999);

    if positive_test {
        let p_d_given_h = sensitivity;
        let p_d_given_not_h = 1.0 - specificity;
        let marginal = p_d_given_h * prior + p_d_given_not_h * (1.0 - prior);
        if marginal > 0.0 {
            p_d_given_h * prior / marginal
        } else {
            0.5
        }
    } else {
        let p_not_d_given_h = 1.0 - sensitivity;
        let p_not_d_given_not_h = specificity;
        let marginal = p_not_d_given_h * prior + p_not_d_given_not_h * (1.0 - prior);
        if marginal > 0.0 {
            p_not_d_given_h * prior / marginal
        } else {
            0.5
        }
    }
}

pub fn adjust_overconfidence(raw_confidence: f64) -> f64 {
    if raw_confidence > 0.8 {
        (raw_confidence * 0.85).clamp(0.0, 1.0)
    } else if raw_confidence > 0.6 {
        (raw_confidence * 0.92).clamp(0.0, 1.0)
    } else {
        (raw_confidence * 1.05).clamp(0.0, 1.0)
    }
}

fn domain_platt(context: &str) -> Platt {
    let lower = context.to_lowercase();
    if lower.contains("math") {
        Platt::new(10.0, -5.0)
    } else if lower.contains("science") {
        Platt::new(8.0, -4.0)
    } else if lower.contains("code") {
        Platt::new(12.0, -6.0)
    } else if lower.contains("fact") {
        Platt::new(7.0, -3.5)
    } else {
        Platt::identity()
    }
}

fn calculate_raw_confidence(ball: &Ball, context: &str) -> f64 {
    let base = ball.candidate.probability;

    let gate_scores: Vec<f64> = ball.gate_results.iter().map(|r| r.score).collect();
    let gate_avg = if gate_scores.is_empty() {
        base
    } else {
        gate_scores.iter().sum::<f64>() / gate_scores.len() as f64
    };

    let has_failures = ball.gate_results.iter().any(|r| !r.passed);
    let failure_penalty = if has_failures { 0.4 } else { 0.0 };

    let context_bonus = if context.len() > 10 { 0.05 } else { 0.0 };

    let raw = if gate_scores.is_empty() {
        base + context_bonus
    } else {
        (base * 0.2) + (gate_avg * 0.7) + context_bonus - failure_penalty
    };
    raw.clamp(0.0, 1.0)
}

fn apply_kakuhen(confidence: f64, consecutive_wins: u32) -> f64 {
    if consecutive_wins >= 3 {
        let boost = (consecutive_wins as f64).min(10.0) * 0.1;
        (confidence + boost).min(1.0)
    } else {
        confidence
    }
}

pub fn validate(ball: &mut Ball, context: &str, threshold: f64) -> GateResult {
    let raw = calculate_raw_confidence(ball, context);
    let platt = domain_platt(context);
    let calibrated = platt.calibrate(raw).clamp(0.0, 1.0);

    let kakuhen_eligible = calibrated > 0.9;
    let consecutive_wins = if kakuhen_eligible {
        ball.gate_results.iter().filter(|r| r.passed).count() as u32
    } else {
        0
    };
    let final_confidence = apply_kakuhen(calibrated, consecutive_wins);

    let passed = final_confidence >= threshold;

    let reason = if !passed {
        Some(format!(
            "Confidence {:.2} (raw {:.2}) below threshold {:.2}",
            final_confidence, raw, threshold
        ))
    } else {
        None
    };

    if passed {
        GateResult::passed(GateKind::Confidence, final_confidence)
    } else {
        GateResult::failed(
            GateKind::Confidence,
            final_confidence,
            &reason.unwrap_or_default(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platt_calibration() {
        let platt = Platt::new(10.0, -5.0);
        let low = platt.calibrate(0.2);
        let mid = platt.calibrate(0.5);
        let high = platt.calibrate(0.8);
        assert!(low < mid);
        assert!(mid < high);
        assert!(high > 0.5);
    }

    #[test]
    fn test_platt_identity() {
        let platt = Platt::identity();
        let cal = platt.calibrate(0.7);
        assert!((cal - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_bayesian_update() {
        let posterior = bayesian_update(0.01, 0.99, 0.95, true);
        assert!(posterior > 0.10 && posterior < 0.25);
    }

    #[test]
    fn test_overconfidence() {
        let high = adjust_overconfidence(0.9);
        assert!(high < 0.9);
        let low = adjust_overconfidence(0.5);
        assert!(low > 0.5);
    }
}
