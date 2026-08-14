use super::request::ValidationRequest;
use super::result::{CidError, CidResult, GateScore, ValidationResult};
use crate::core::ball::{Ball, TokenCandidate};
use crate::core::pin::PinField;
use crate::core::pocket::Pocket;
use crate::economy::budget::Budget;
use crate::economy::tray::BallEconomy;
use crate::gates::validate_candidate;
use crate::kb::facts::KnowledgeBase;
use crate::state::machine::{State, StateMachine};

pub struct Pipeline {
    pub pin_field: PinField,
    pub state_machine: StateMachine,
    pub economy: BallEconomy,
    pub budget: Budget,
    pub kb: KnowledgeBase,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            pin_field: PinField::new(),
            state_machine: StateMachine::new(),
            economy: BallEconomy::new(1000),
            budget: Budget::new(1_000_000, 10.0),
            kb: KnowledgeBase::new(),
        }
    }

    pub fn with_pins(mut self, pin_field: PinField) -> Self {
        self.pin_field = pin_field;
        self
    }

    pub fn with_budget(mut self, budget: Budget) -> Self {
        self.budget = budget;
        self
    }

    pub fn with_kb(mut self, kb: KnowledgeBase) -> Self {
        self.kb = kb;
        self
    }

    pub fn validate(
        &mut self,
        request: &ValidationRequest,
        domain: Option<&str>,
    ) -> CidResult<ValidationResult> {
        if self.budget.is_exhausted() {
            return Err(CidError::BudgetExhausted {
                remaining_tokens: self.budget.remaining_tokens(),
                remaining_cost: self.budget.remaining_cost(),
            });
        }

        // Per-pin cost accounting: charge the cumulative cost of every enabled
        // pin for this validation batch. The spend is the enforcement point — if
        // the operation cannot afford its configured pin cost, refuse to validate
        // rather than proceeding and billing after the fact. This closes the
        // partial-exhaustion gap where `is_exhausted()` was false (some cost
        // remained) yet the pin cost did not fit.
        let pin_cost = self.pin_field.total_cost();
        if !self.budget.spend_cost(pin_cost) {
            return Err(CidError::BudgetExhausted {
                remaining_tokens: self.budget.remaining_tokens(),
                remaining_cost: self.budget.remaining_cost(),
            });
        }

        let _effective_domain = domain.or(request.domain.as_deref()).unwrap_or("general");

        let mut result = ValidationResult::new(&request.text);

        if request.has_candidates() {
            let candidates =
                self.validate_candidates(&request.candidates, &request.context, &request.text)?;
            let validated_text = candidates
                .iter()
                .find(|b| b.validated)
                .map(|b| b.candidate.token.clone())
                .unwrap_or_else(|| request.text.clone());

            let gate_scores = candidates
                .first()
                .map(|b| {
                    b.gate_results
                        .iter()
                        .map(GateScore::from_gate_result)
                        .collect()
                })
                .unwrap_or_default();

            let confidence = candidates.first().map(|b| b.total_score).unwrap_or(0.0);

            let passed = candidates.iter().any(|b| b.validated);

            result = result
                .with_gate_scores(gate_scores)
                .with_confidence(confidence)
                .with_passed(passed);

            if passed {
                result.validated_text = validated_text;
            }
        } else {
            let ball =
                self.validate_single_token(&request.text, &request.context, &request.text)?;

            let gate_scores = ball
                .gate_results
                .iter()
                .map(GateScore::from_gate_result)
                .collect();
            let confidence = ball.total_score;
            let passed = ball.validated;

            result = result
                .with_gate_scores(gate_scores)
                .with_confidence(confidence)
                .with_passed(passed);

            if passed {
                result.validated_text = request.text.clone();
            }
        }

        result.state = self.state_machine.current().clone();
        result.cost_usd = self.economy.total_cost_usd();

        self.budget.spend_tokens(1);
        self.economy.spend(1);

        if result.passed {
            self.state_machine.transition(true);
            self.economy.win(1);
        } else {
            self.state_machine.transition(false);
        }

        Ok(result)
    }

    pub fn validate_single_token(
        &self,
        token: &str,
        context: &str,
        claim: &str,
    ) -> CidResult<Ball> {
        let candidate = TokenCandidate::new(0, token, 0.5);
        let mut ball = Ball::new(candidate);
        validate_candidate(&mut ball, &self.pin_field.pins, context, claim, &self.kb);
        Ok(ball)
    }

    pub fn validate_candidates(
        &self,
        candidates: &[String],
        context: &str,
        claim: &str,
    ) -> CidResult<Vec<Ball>> {
        let mut balls = Vec::new();

        for (i, token) in candidates.iter().enumerate() {
            let candidate = TokenCandidate::new(i as u32, token, 0.5);
            let mut ball = Ball::new(candidate);
            validate_candidate(&mut ball, &self.pin_field.pins, context, claim, &self.kb);
            balls.push(ball);
        }

        Ok(balls)
    }

    pub fn select_best(&self, candidates: Vec<Ball>) -> Option<Pocket> {
        Pocket::select_best(candidates)
    }

    pub fn state(&self) -> &State {
        self.state_machine.current()
    }

    pub fn economy(&self) -> &BallEconomy {
        &self.economy
    }

    pub fn budget(&self) -> &Budget {
        &self.budget
    }

    pub fn reset(&mut self) {
        self.state_machine.reset();
        self.economy.reset(1000);
        self.budget.reset();
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Pipeline;
    use crate::economy::budget::Budget;
    use crate::inference::request::ValidationRequest;
    use crate::inference::result::CidError;

    fn request(text: &str) -> ValidationRequest {
        ValidationRequest::new(text, "")
    }

    #[test]
    fn validation_proceeds_when_budget_affords_pin_cost() {
        let mut pipeline = Pipeline::new();
        // Default budget (max_cost_usd = 10.0) comfortably covers the default
        // PinField total cost (~0.0075).
        let res = pipeline
            .validate(&request("2 + 3 = 5"), None)
            .expect("affordable validation must succeed");
        assert!(res.passed);
    }

    #[test]
    fn validation_refuses_when_budget_cannot_afford_pin_cost() {
        let mut pipeline = Pipeline::new().with_budget(Budget::new(1_000_000, 0.001));
        // The default PinField total cost (~0.0075) exceeds the 0.001 cost
        // ceiling. `is_exhausted()` is still false (some cost remains), so only
        // the `spend_cost()` return value guards this path — proving the budget
        // cannot be bypassed.
        let err = pipeline
            .validate(&request("2 + 3 = 5"), None)
            .expect_err("unaffordable validation must be refused");
        assert!(
            matches!(err, CidError::BudgetExhausted { .. }),
            "expected BudgetExhausted, got {err}"
        );
    }
}
