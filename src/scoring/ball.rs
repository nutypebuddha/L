use super::pin::GateKind;

#[derive(Debug, Clone)]
pub struct TokenCandidate {
    pub id: u32,
    pub token: String,
    pub logit: f64,
    pub probability: f64,
}

impl TokenCandidate {
    pub fn new(id: u32, token: &str, logit: f64) -> Self {
        let probability = 1.0 / (1.0 + (-logit).exp());
        TokenCandidate {
            id,
            token: token.to_string(),
            logit,
            probability,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GateResult {
    pub gate: GateKind,
    pub passed: bool,
    pub score: f64,
    pub reason: Option<String>,
}

impl GateResult {
    pub fn passed(gate: GateKind, score: f64) -> Self {
        GateResult {
            gate,
            passed: true,
            score,
            reason: None,
        }
    }

    pub fn failed(gate: GateKind, score: f64, reason: &str) -> Self {
        GateResult {
            gate,
            passed: false,
            score,
            reason: Some(reason.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ball {
    pub candidate: TokenCandidate,
    pub gate_results: Vec<GateResult>,
    pub total_score: f64,
    pub validated: bool,
}

impl Ball {
    pub fn new(candidate: TokenCandidate) -> Self {
        Ball {
            candidate,
            gate_results: Vec::new(),
            total_score: 0.0,
            validated: false,
        }
    }

    pub fn add_result(&mut self, result: GateResult) {
        self.gate_results.push(result);
        self.recalculate_score();
    }

    pub fn recalculate_score(&mut self) {
        if self.gate_results.is_empty() {
            self.total_score = 0.0;
            self.validated = false;
            return;
        }

        let total: f64 = self.gate_results.iter().map(|r| r.score).sum();
        let count = self.gate_results.len() as f64;
        self.total_score = total / count;
        self.validated = self.gate_results.iter().all(|r| r.passed);
    }

    pub fn passed_gate(&self, gate: GateKind) -> bool {
        self.gate_results.iter().any(|r| r.gate == gate && r.passed)
    }

    pub fn failed_gate(&self, gate: GateKind) -> bool {
        self.gate_results
            .iter()
            .any(|r| r.gate == gate && !r.passed)
    }

    pub fn all_passed(&self) -> bool {
        self.validated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ball_new() {
        let candidate = TokenCandidate::new(0, "test", 0.5);
        let ball = Ball::new(candidate);
        assert_eq!(ball.gate_results.len(), 0);
        assert!(!ball.validated);
    }

    #[test]
    fn test_ball_add_result() {
        let candidate = TokenCandidate::new(0, "test", 0.5);
        let mut ball = Ball::new(candidate);
        ball.add_result(GateResult::passed(GateKind::Math, 0.9));
        assert_eq!(ball.gate_results.len(), 1);
        assert!(ball.validated);
    }

    #[test]
    fn test_ball_all_passed() {
        let candidate = TokenCandidate::new(0, "test", 0.5);
        let mut ball = Ball::new(candidate);
        ball.add_result(GateResult::passed(GateKind::Math, 0.9));
        ball.add_result(GateResult::passed(GateKind::Logic, 0.8));
        assert!(ball.all_passed());

        ball.add_result(GateResult::failed(GateKind::Fact, 0.3, "not found"));
        assert!(!ball.all_passed());
    }
}
