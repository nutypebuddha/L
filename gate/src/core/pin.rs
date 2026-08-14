#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Math,
    Logic,
    Fact,
    Confidence,
    Formal,
    /// A token is *domain-bound* iff it is grounded in the verified context that
    /// the answer is allowed to draw from. Unbound tokens have no provenance and
    /// are rejected (they are either fabricated or off-domain).
    DomainBinding,
    /// A token that asserts a verifiable numeric claim must *recompute* to match.
    /// This is the "proof recomputation match" gate — a claim with a wrong or
    /// non-recomputable proof is rejected.
    ProofRecompute,
}

#[derive(Debug, Clone)]
pub struct Pin {
    pub gate: Gate,
    pub threshold: f64,
    pub enabled: bool,
    pub cost: f64,
}

impl Pin {
    pub fn new(gate: Gate, threshold: f64) -> Self {
        Pin {
            gate,
            threshold,
            enabled: true,
            cost: match gate {
                Gate::Math => 0.001,
                Gate::Logic => 0.002,
                Gate::Fact => 0.001,
                Gate::Confidence => 0.0005,
                Gate::Formal => 0.003,
                Gate::DomainBinding => 0.002,
                Gate::ProofRecompute => 0.002,
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
    pub pins: Vec<Pin>,
}

impl PinField {
    pub fn new() -> Self {
        let pins = vec![
            Pin::new(Gate::Math, 0.7),
            Pin::new(Gate::Logic, 0.5),
            Pin::new(Gate::Fact, 0.5),
            Pin::new(Gate::Confidence, 0.5),
            Pin::new(Gate::Formal, 0.5),
        ];
        PinField { pins }
    }

    pub fn math_only() -> Self {
        PinField {
            pins: vec![Pin::new(Gate::Math, 0.7)],
        }
    }

    pub fn logic_only() -> Self {
        PinField {
            pins: vec![Pin::new(Gate::Logic, 0.7)],
        }
    }

    pub fn strict() -> Self {
        let pins = vec![
            Pin::new(Gate::Math, 0.9),
            Pin::new(Gate::Logic, 0.9),
            Pin::new(Gate::Fact, 0.9),
            Pin::new(Gate::Confidence, 0.8),
            Pin::new(Gate::Formal, 0.9),
        ];
        PinField { pins }
    }

    pub fn lenient() -> Self {
        let pins = vec![
            Pin::new(Gate::Math, 0.5),
            Pin::new(Gate::Logic, 0.5),
            Pin::new(Gate::Fact, 0.5),
            Pin::new(Gate::Confidence, 0.3),
            Pin::new(Gate::Formal, 0.5),
        ];
        PinField { pins }
    }

    /// Enforce the two hallucination-hardening gates on top of the default field:
    /// `DomainBinding` (reject tokens with no provenance in the verified context)
    /// and `ProofRecompute` (reject claims whose proof does not recompute). Use
    /// this for the LLM completion path where un-grounded or un-verifiable tokens
    /// must be refused rather than emitted.
    pub fn with_domain_proof_enforcement() -> Self {
        let mut base = PinField::new();
        base.pins.push(Pin::new(Gate::DomainBinding, 0.8));
        base.pins.push(Pin::new(Gate::ProofRecompute, 0.9));
        base
    }

    pub fn adjust_pin(&mut self, gate: Gate, threshold: f64) {
        for pin in &mut self.pins {
            if pin.gate == gate {
                pin.threshold = threshold;
            }
        }
    }

    pub fn enable_pin(&mut self, gate: Gate) {
        for pin in &mut self.pins {
            if pin.gate == gate {
                pin.enabled = true;
            }
        }
    }

    pub fn disable_pin(&mut self, gate: Gate) {
        for pin in &mut self.pins {
            if pin.gate == gate {
                pin.enabled = false;
            }
        }
    }

    pub fn active_pins(&self) -> Vec<&Pin> {
        self.pins.iter().filter(|p| p.enabled).collect()
    }

    pub fn total_cost(&self) -> f64 {
        self.active_pins().iter().map(|p| p.cost).sum()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pin> {
        self.pins.iter()
    }
}

impl Default for PinField {
    fn default() -> Self {
        Self::new()
    }
}
