use super::ball::Ball;

#[derive(Debug, Clone)]
pub struct Pocket {
    pub ball: Ball,
    pub kakuhen_triggered: bool,
    pub confidence_boost: f64,
}

impl Pocket {
    pub fn new(ball: Ball) -> Self {
        Pocket {
            ball,
            kakuhen_triggered: false,
            confidence_boost: 1.0,
        }
    }

    pub fn with_kakuhen(mut self, triggered: bool, boost: f64) -> Self {
        self.kakuhen_triggered = triggered;
        self.confidence_boost = boost;
        self
    }

    pub fn should_trigger_kakuhen(&self) -> bool {
        self.ball.all_passed() && self.ball.total_score > 0.8
    }

    pub fn select_best(candidates: Vec<Ball>) -> Option<Pocket> {
        let mut validated: Vec<Ball> = candidates.into_iter().filter(|b| b.all_passed()).collect();

        if validated.is_empty() {
            return None;
        }

        validated.sort_by(|a, b| {
            b.total_score
                .partial_cmp(&a.total_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let best = validated.into_iter().next()?;
        let kakuhen = best.total_score > 0.9;
        let boost = if kakuhen {
            1.0 + (best.total_score - 0.9) * 10.0
        } else {
            1.0
        };

        Some(Pocket::new(best).with_kakuhen(kakuhen, boost))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::ball::{GateResult, TokenCandidate};
    use crate::scoring::pin::GateKind;

    #[test]
    fn test_select_best() {
        let c1 = TokenCandidate::new(0, "a", 0.5);
        let mut b1 = Ball::new(c1);
        b1.add_result(GateResult::passed(GateKind::Math, 0.9));

        let c2 = TokenCandidate::new(1, "b", 0.8);
        let mut b2 = Ball::new(c2);
        b2.add_result(GateResult::passed(GateKind::Math, 0.95));

        let pocket = Pocket::select_best(vec![b1, b2]).unwrap();
        assert_eq!(pocket.ball.candidate.id, 1);
    }

    #[test]
    fn test_select_best_none_valid() {
        let c1 = TokenCandidate::new(0, "a", 0.5);
        let mut b1 = Ball::new(c1);
        b1.add_result(GateResult::failed(GateKind::Math, 0.3, "bad"));

        assert!(Pocket::select_best(vec![b1]).is_none());
    }
}
