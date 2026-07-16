#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateKind {
    Math,
    Logic,
    Fact,
    Confidence,
    Formal,
}

#[derive(Debug, Clone)]
pub struct ValidationPin {
    pub gate: GateKind,
    pub threshold: f64,
    pub enabled: bool,
    pub cost: f64,
}

impl ValidationPin {
    pub fn new(gate: GateKind, threshold: f64) -> Self {
        ValidationPin {
            gate,
            threshold,
            enabled: true,
            cost: match gate {
                GateKind::Math => 0.001,
                GateKind::Logic => 0.002,
                GateKind::Fact => 0.001,
                GateKind::Confidence => 0.0005,
                GateKind::Formal => 0.003,
            },
        }
    }

    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.threshold = threshold;
        self
    }

    pub fn disable(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PinField {
    pub pins: Vec<ValidationPin>,
}

impl PinField {
    pub fn new() -> Self {
        PinField {
            pins: vec![
                ValidationPin::new(GateKind::Math, 0.7),
                ValidationPin::new(GateKind::Logic, 0.7),
                ValidationPin::new(GateKind::Fact, 0.7),
                ValidationPin::new(GateKind::Confidence, 0.5),
                ValidationPin::new(GateKind::Formal, 0.7),
            ],
        }
    }

    pub fn math_only() -> Self {
        PinField {
            pins: vec![ValidationPin::new(GateKind::Math, 0.7)],
        }
    }

    pub fn strict() -> Self {
        PinField {
            pins: vec![
                ValidationPin::new(GateKind::Math, 0.9),
                ValidationPin::new(GateKind::Logic, 0.9),
                ValidationPin::new(GateKind::Fact, 0.9),
                ValidationPin::new(GateKind::Confidence, 0.8),
                ValidationPin::new(GateKind::Formal, 0.9),
            ],
        }
    }

    pub fn lenient() -> Self {
        PinField {
            pins: vec![
                ValidationPin::new(GateKind::Math, 0.5),
                ValidationPin::new(GateKind::Logic, 0.5),
                ValidationPin::new(GateKind::Fact, 0.5),
                ValidationPin::new(GateKind::Confidence, 0.3),
                ValidationPin::new(GateKind::Formal, 0.5),
            ],
        }
    }

    pub fn active_pins(&self) -> Vec<&ValidationPin> {
        self.pins.iter().filter(|p| p.enabled).collect()
    }

    pub fn total_cost(&self) -> f64 {
        self.active_pins().iter().map(|p| p.cost).sum()
    }
}

impl Default for PinField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_field_new() {
        let field = PinField::new();
        assert_eq!(field.pins.len(), 5);
    }

    #[test]
    fn test_pin_field_math_only() {
        let field = PinField::math_only();
        assert_eq!(field.pins.len(), 1);
        assert_eq!(field.pins[0].gate, GateKind::Math);
    }

    #[test]
    fn test_pin_field_strict() {
        let field = PinField::strict();
        for pin in &field.pins {
            assert!(pin.threshold >= 0.8);
        }
    }

    #[test]
    fn test_active_pins() {
        let mut field = PinField::new();
        field.pins[0].enabled = false;
        assert_eq!(field.active_pins().len(), 4);
    }
}
