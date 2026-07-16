pub mod confidence_gate;
pub mod fallacy_gate;
pub mod formal_gate;
pub mod logic_gate;
pub mod math_gate;
pub mod pipeline;

use crate::scoring::ball::{Ball, GateResult};
use crate::scoring::pin::{GateKind, ValidationPin};

pub trait GateValidator {
    fn validate(&self, ball: &mut Ball, context: &str) -> GateResult;
}

pub fn validate_ball(ball: &mut Ball, pins: &[ValidationPin], context: &str) {
    for pin in pins {
        if !pin.enabled {
            continue;
        }

        let result = match pin.gate {
            GateKind::Math => math_gate::validate(ball, context),
            GateKind::Logic => logic_gate::validate(ball, context),
            GateKind::Confidence => confidence_gate::validate(ball, context, pin.threshold),
            GateKind::Formal => formal_gate::validate(ball, context),
            GateKind::Fact => GateResult::passed(GateKind::Fact, 0.8),
        };

        ball.add_result(result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::ball::TokenCandidate;
    use crate::scoring::pin::{PinField, ValidationPin};

    #[test]
    fn test_validate_ball_all_gates() {
        let candidate = TokenCandidate::new(0, "2+3", 0.5);
        let mut ball = Ball::new(candidate);
        let field = PinField::new();
        let pins: Vec<ValidationPin> = field.active_pins().into_iter().cloned().collect();
        validate_ball(&mut ball, &pins, "math expression");
        assert_eq!(ball.gate_results.len(), 5);
    }
}
