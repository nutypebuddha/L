/// Simple energy-based Voice Activity Detector.
/// No external model needed — just checks audio RMS energy.
#[derive(Default)]
pub struct VadEngine {
    threshold: f64,
    silence_count: usize,
    silence_limit: usize,
    in_speech: bool,
}

impl VadEngine {
    pub fn new() -> Self {
        Self {
            threshold: 500.0, // RMS energy threshold (tune for mic)
            silence_count: 0,
            silence_limit: 5, // ~5 chunks of silence = end of speech
            in_speech: false,
        }
    }

    /// Check if a raw PCM chunk (i16 LE, 16kHz mono) contains speech.
    pub fn is_speech(&mut self, data: &[u8]) -> bool {
        let energy = compute_rms(data);
        let is_speech = energy > self.threshold;

        if is_speech {
            self.silence_count = 0;
            self.in_speech = true;
            true
        } else if self.in_speech {
            self.silence_count += 1;
            if self.silence_count >= self.silence_limit {
                self.in_speech = false;
                self.silence_count = 0;
            }
            self.in_speech
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.silence_count = 0;
        self.in_speech = false;
    }
}

fn compute_rms(data: &[u8]) -> f64 {
    let samples: Vec<i16> = data
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| (s as f64).powi(2)).sum();
    (sum / samples.len() as f64).sqrt()
}
