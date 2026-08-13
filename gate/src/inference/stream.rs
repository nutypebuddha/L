use crate::core::ball::{Ball, TokenCandidate};
use crate::core::pin::{Gate, PinField};
use crate::gates::{
    GateValidator, confidence::ConfidenceGate, fact::FactGate, logic::LogicGate, math::MathGate,
};
use crate::kb::facts::KnowledgeBase;

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Token {
        index: usize,
        text: String,
        confidence: f64,
        passed: bool,
    },
    Fix {
        original: String,
        fixed: String,
        reason: String,
    },
    Warning {
        message: String,
    },
    Complete {
        total_tokens: usize,
        passed_count: usize,
        avg_confidence: f64,
    },
}

impl StreamEvent {
    pub fn to_sse(&self) -> String {
        match self {
            StreamEvent::Token {
                index,
                text,
                confidence,
                passed,
            } => {
                format!(
                    "event: token\ndata: {{\"index\":{},\"text\":\"{}\",\"confidence\":{:.4},\"passed\":{}}}\n\n",
                    index,
                    escape_json(text),
                    confidence,
                    passed
                )
            }
            StreamEvent::Fix {
                original,
                fixed,
                reason,
            } => {
                format!(
                    "event: fix\ndata: {{\"original\":\"{}\",\"fixed\":\"{}\",\"reason\":\"{}\"}}\n\n",
                    escape_json(original),
                    escape_json(fixed),
                    escape_json(reason)
                )
            }
            StreamEvent::Warning { message } => {
                format!(
                    "event: warning\ndata: {{\"message\":\"{}\"}}\n\n",
                    escape_json(message)
                )
            }
            StreamEvent::Complete {
                total_tokens,
                passed_count,
                avg_confidence,
            } => {
                format!(
                    "event: complete\ndata: {{\"total_tokens\":{},\"passed_count\":{},\"avg_confidence\":{:.4}}}\n\n",
                    total_tokens, passed_count, avg_confidence
                )
            }
        }
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub struct StreamValidator {
    pin_field: PinField,
    kb: KnowledgeBase,
    token_count: usize,
    passed_count: usize,
    total_confidence: f64,
}

impl StreamValidator {
    pub fn new() -> Self {
        StreamValidator {
            pin_field: PinField::new(),
            kb: KnowledgeBase::new(),
            token_count: 0,
            passed_count: 0,
            total_confidence: 0.0,
        }
    }

    pub fn with_pins(mut self, pin_field: PinField) -> Self {
        self.pin_field = pin_field;
        self
    }

    pub fn with_kb(mut self, kb: KnowledgeBase) -> Self {
        self.kb = kb;
        self
    }

    pub fn validate_token(&mut self, token: &str, context: &str) -> StreamEvent {
        let candidate = TokenCandidate::new(self.token_count as u32, token, 0.5);
        let mut ball = Ball::new(candidate);

        for pin in self.pin_field.pins.iter() {
            if !pin.enabled {
                continue;
            }

            let result = match pin.gate {
                Gate::Math => MathGate::new().validate(&mut ball, context),
                Gate::Logic => LogicGate::new().validate(&mut ball, context),
                Gate::Fact => FactGate::new(&self.kb).validate(&mut ball, context),
                Gate::Confidence => {
                    ConfidenceGate::with_platt_for_domain(context).validate(&mut ball, context)
                }
                Gate::Formal => {
                    crate::gates::formal::FormalGate::new().validate(&mut ball, context)
                }
            };

            ball.add_result(result);
        }

        let confidence = ball.total_score;
        let passed = ball.validated;

        self.token_count += 1;
        if passed {
            self.passed_count += 1;
        }
        self.total_confidence += confidence;

        StreamEvent::Token {
            index: self.token_count - 1,
            text: token.to_string(),
            confidence,
            passed,
        }
    }

    pub fn validate_stream(&mut self, tokens: &[String], context: &str) -> Vec<StreamEvent> {
        let mut events = Vec::new();

        for token in tokens {
            let event = self.validate_token(token, context);
            events.push(event);
        }

        events.push(self.complete());
        events
    }

    pub fn complete(&self) -> StreamEvent {
        let avg = if self.token_count > 0 {
            self.total_confidence / self.token_count as f64
        } else {
            0.0
        };

        StreamEvent::Complete {
            total_tokens: self.token_count,
            passed_count: self.passed_count,
            avg_confidence: avg,
        }
    }

    pub fn reset(&mut self) {
        self.token_count = 0;
        self.passed_count = 0;
        self.total_confidence = 0.0;
    }

    pub fn stats(&self) -> (usize, usize, f64) {
        let avg = if self.token_count > 0 {
            self.total_confidence / self.token_count as f64
        } else {
            0.0
        };
        (self.token_count, self.passed_count, avg)
    }
}

impl Default for StreamValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_event_token() {
        let event = StreamEvent::Token {
            index: 0,
            text: "hello".to_string(),
            confidence: 0.95,
            passed: true,
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: token"));
        assert!(sse.contains("\"text\":\"hello\""));
        assert!(sse.contains("\"passed\":true"));
    }

    #[test]
    fn test_stream_event_fix() {
        let event = StreamEvent::Fix {
            original: "hte".to_string(),
            fixed: "the".to_string(),
            reason: "typo".to_string(),
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: fix"));
        assert!(sse.contains("\"original\":\"hte\""));
        assert!(sse.contains("\"fixed\":\"the\""));
    }

    #[test]
    fn test_stream_event_complete() {
        let event = StreamEvent::Complete {
            total_tokens: 10,
            passed_count: 8,
            avg_confidence: 0.85,
        };
        let sse = event.to_sse();
        assert!(sse.contains("event: complete"));
        assert!(sse.contains("\"total_tokens\":10"));
        assert!(sse.contains("\"passed_count\":8"));
    }

    #[test]
    fn test_stream_validator_token() {
        let mut validator = StreamValidator::new();
        let event = validator.validate_token("42", "math");
        match event {
            StreamEvent::Token {
                index,
                text,
                passed,
                ..
            } => {
                assert_eq!(index, 0);
                assert_eq!(text, "42");
                assert!(passed);
            }
            _ => panic!("Expected Token event"),
        }
    }

    #[test]
    fn test_stream_validator_batch() {
        let mut validator = StreamValidator::new();
        let tokens: Vec<String> = vec!["hello".into(), "world".into()];
        let events = validator.validate_stream(&tokens, "general");
        assert_eq!(events.len(), 3);
        assert!(matches!(events.last(), Some(StreamEvent::Complete { .. })));
    }

    #[test]
    fn test_stream_validator_stats() {
        let mut validator = StreamValidator::new();
        validator.validate_token("42", "math");
        validator.validate_token("100", "math");
        let (total, passed, _avg) = validator.stats();
        assert_eq!(total, 2);
        assert!(passed > 0);
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("hello"), "hello");
        assert_eq!(escape_json("he\"llo"), "he\\\"llo");
        assert_eq!(escape_json("line\none"), "line\\none");
    }
}
