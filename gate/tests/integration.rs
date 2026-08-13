use cid::core::ball::{Ball, TokenCandidate};
use cid::core::pin::{Gate, PinField};
use cid::core::pocket::Pocket;
use cid::economy::budget::Budget;
use cid::economy::tray::BallEconomy;
use cid::gates::GateValidator;
use cid::gates::{
    confidence::ConfidenceGate, fact::FactGate, formal::FormalGate, logic::LogicGate,
    math::MathGate,
};
use cid::inference::{InferenceEngine, ValidationRequest};
use cid::kb::facts::Domain;
use cid::kb::facts::KnowledgeBase;
use cid::state::machine::StateMachine;

#[test]
fn test_math_gate() {
    let gate = MathGate::new();
    let candidate = TokenCandidate::new(0, "42", 0.5);
    let mut ball = Ball::new(candidate);
    let result = gate.validate(&mut ball, "math");
    assert!(result.passed);
}

#[test]
fn test_logic_gate() {
    let gate = LogicGate::new();
    let candidate = TokenCandidate::new(0, "therefore", 0.5);
    let mut ball = Ball::new(candidate);
    let result = gate.validate(&mut ball, "because all men are mortal");
    assert!(result.passed);
}

#[test]
fn test_fact_gate() {
    let kb = KnowledgeBase::new();
    let gate = FactGate::new(&kb);
    let candidate = TokenCandidate::new(0, "3.14159", 0.5);
    let mut ball = Ball::new(candidate);
    let result = gate.validate(&mut ball, "pi is approximately");
    assert!(result.passed);
}

#[test]
fn test_confidence_gate() {
    let gate = ConfidenceGate::new(0.4);
    let candidate = TokenCandidate::new(0, "42", 1.0);
    let mut ball = Ball::new(candidate);
    let result = gate.validate(&mut ball, "general");
    assert!(result.passed);
}

#[test]
fn test_ball_validation() {
    let candidate = TokenCandidate::new(0, "42", 0.5);
    let mut ball = Ball::new(candidate);

    let math_result = MathGate::new().validate(&mut ball, "math");
    ball.add_result(math_result);

    let logic_result = LogicGate::new().validate(&mut ball, "math");
    ball.add_result(logic_result);

    assert!(ball.all_passed());
    assert!(ball.total_score > 0.0);
}

#[test]
fn test_state_machine() {
    let mut sm = StateMachine::new();
    assert!(sm.current().is_normal());

    sm.transition(true);
    assert!(sm.current().is_kakuhen());

    sm.transition(false);
    assert!(sm.current().is_jitan());

    sm.transition(true);
    assert!(sm.current().is_kakuhen());
}

#[test]
fn test_ball_economy() {
    let mut economy = BallEconomy::new(100);
    assert_eq!(economy.balance(), 100);

    economy.spend(10);
    assert_eq!(economy.balance(), 90);

    economy.win(5);
    assert_eq!(economy.balance(), 95);
}

#[test]
fn test_budget() {
    let mut budget = Budget::new(1000, 10.0);
    assert!(!budget.is_exhausted());

    budget.spend_tokens(500);
    budget.spend_cost(5.0);
    assert!(!budget.is_exhausted());

    budget.spend_tokens(500);
    budget.spend_cost(5.0);
    assert!(budget.is_exhausted());
}

#[test]
fn test_knowledge_base() {
    let kb = KnowledgeBase::new();
    assert!(kb.lookup("pi").is_some());
    assert!(kb.lookup("nonexistent").is_none());

    let pi = kb.lookup("pi").unwrap();
    let expected_pi: f64 = std::f64::consts::PI;
    assert!((pi.value - expected_pi).abs() < 1e-10);
}

#[test]
fn test_pin_field() {
    let pin_field = PinField::new();
    assert_eq!(pin_field.pins.len(), 5);

    let math_only = PinField::math_only();
    assert_eq!(math_only.pins.len(), 1);
    assert_eq!(math_only.pins[0].gate, Gate::Math);
}

#[test]
fn test_pocket_selection() {
    let mut low_ball = Ball::new(TokenCandidate::new(0, "low", 0.5));
    low_ball.total_score = 0.3;
    low_ball.validated = true;

    let mut high_ball = Ball::new(TokenCandidate::new(1, "high", 0.5));
    high_ball.total_score = 0.8;
    high_ball.validated = true;

    let candidates = vec![low_ball, high_ball];

    let pocket = Pocket::select_best(candidates);
    assert!(pocket.is_some());
    assert_eq!(pocket.unwrap().ball.candidate.token, "high");
}

#[test]
fn test_anthropology_facts() {
    let kb = KnowledgeBase::new();

    // Test human evolution facts
    let homo_habilis = kb.lookup("homo_habilis").unwrap();
    assert!((homo_habilis.value - 2.8).abs() < 0.01);

    let homo_erectus = kb.lookup("homo_erectus").unwrap();
    assert!((homo_erectus.value - 1.8).abs() < 0.01);

    let homo_sapiens = kb.lookup("homo_sapiens").unwrap();
    assert!((homo_sapiens.value - 0.3).abs() < 0.01);

    let human_chimp = kb.lookup("human_chimp_divergence").unwrap();
    assert!((human_chimp.value - 7.0).abs() < 0.01);

    // Test tool use and fire
    let first_tool = kb.lookup("first_tool_use").unwrap();
    assert!((first_tool.value - 3.3).abs() < 0.01);

    let control_fire = kb.lookup("control_of_fire").unwrap();
    assert!((control_fire.value - 1.0).abs() < 0.01);

    // Test brain sizes
    let sapiens_brain = kb.lookup("homo_sapiens_brain").unwrap();
    assert!((sapiens_brain.value - 1350.0).abs() < 1.0);

    let neanderthal_brain = kb.lookup("neanderthal_brain").unwrap();
    assert!((neanderthal_brain.value - 1500.0).abs() < 1.0);
}

#[test]
fn test_historical_facts() {
    let kb = KnowledgeBase::new();

    // Test empire dates
    let roman_empire = kb.lookup("roman_empire").unwrap();
    assert!((roman_empire.value - (-27.0)).abs() < 1.0);

    let mongol_empire = kb.lookup("mongol_empire").unwrap();
    assert!((mongol_empire.value - 1206.0).abs() < 1.0);

    let british_empire = kb.lookup("british_empire").unwrap();
    assert!((british_empire.value - 1583.0).abs() < 1.0);

    // Test empire statistics
    let british_pop = kb.lookup("british_empire_population").unwrap();
    assert!((british_pop.value - 458.0).abs() < 1.0);

    let british_area = kb.lookup("british_empire_area").unwrap();
    assert!((british_area.value - 35.5).abs() < 1.0);

    let mongol_pop = kb.lookup("mongol_empire_population").unwrap();
    assert!((mongol_pop.value - 100.0).abs() < 1.0);

    let mongol_area = kb.lookup("mongol_empire_area").unwrap();
    assert!((mongol_area.value - 24.0).abs() < 1.0);
}

#[test]
fn test_cultural_universals() {
    let kb = KnowledgeBase::new();

    // Test cultural universals
    let language = kb.lookup("language_universal").unwrap();
    assert_eq!(language.value, 1.0);

    let music = kb.lookup("music_universal").unwrap();
    assert_eq!(music.value, 1.0);

    let dance = kb.lookup("dance_universal").unwrap();
    assert_eq!(dance.value, 1.0);

    let religion = kb.lookup("religion_universal").unwrap();
    assert_eq!(religion.value, 1.0);

    let kinship = kb.lookup("kinship_universal").unwrap();
    assert_eq!(kinship.value, 1.0);
}

#[test]
fn test_historical_population() {
    let kb = KnowledgeBase::new();

    // Test world population estimates
    let pop_1ad = kb.lookup("world_population_1ad").unwrap();
    assert!((pop_1ad.value - 200.0).abs() < 1.0);

    let pop_1000ad = kb.lookup("world_population_1000ad").unwrap();
    assert!((pop_1000ad.value - 310.0).abs() < 1.0);

    let pop_1500ad = kb.lookup("world_population_1500ad").unwrap();
    assert!((pop_1500ad.value - 460.0).abs() < 1.0);

    let pop_2024 = kb.lookup("world_population_2024ad").unwrap();
    assert!((pop_2024.value - 8100.0).abs() < 1.0);
}

#[test]
fn test_formal_gate() {
    let gate = FormalGate::new();
    let candidate = TokenCandidate::new(0, "proof", 0.5);
    let mut ball = Ball::new(candidate);
    let result = gate.validate(&mut ball, "mathematical proof");
    assert!(result.passed);
}

#[test]
fn test_domain_system() {
    let kb = KnowledgeBase::new();

    // Test domain enum
    let alpha = Domain::from_name("alpha");
    assert!(alpha.is_some());
    assert_eq!(alpha.unwrap(), Domain::Alpha);

    let math_domain = Domain::from_name("math");
    assert!(math_domain.is_some());
    assert_eq!(math_domain.unwrap(), Domain::Alpha);

    // Test domain symbols
    assert_eq!(Domain::Alpha.symbol(), "Α");
    assert_eq!(Domain::Alpha.symbol_lower(), "α");

    // Test domain descriptions
    assert_eq!(Domain::Alpha.description(), "Mathematics & Logic");
    assert_eq!(Domain::Iota.description(), "History & Anthropology");

    // Test all domains
    let all_domains = Domain::all();
    assert_eq!(all_domains.len(), 13);

    // Test domain counting
    let alpha_count = kb.count_domain(Domain::Alpha);
    assert!(alpha_count > 0);

    // Test facts by domain
    let alpha_facts = kb.facts_by_domain(Domain::Alpha);
    assert!(!alpha_facts.is_empty());

    // Test facts by domain name
    let math_facts = kb.facts_by_domain_name("math");
    assert!(!math_facts.is_empty());

    // Test fact domain lookup
    let pi_domain = kb.fact_domain("pi");
    assert!(pi_domain.is_some());
    assert_eq!(pi_domain.unwrap(), Domain::Alpha);

    // Test domain stats
    let stats = kb.domain_stats();
    assert_eq!(stats.len(), 13);
}

#[test]
fn test_rust_domain_registered() {
    assert_eq!(Domain::from_name("rust_code"), Some(Domain::Nu));
    assert_eq!(Domain::from_name("rust"), Some(Domain::Nu));
    let kb = KnowledgeBase::new();
    assert!(
        kb.facts_by_domain(Domain::Nu).len() >= 15,
        "rust_code domain should carry the curated Rust ABI facts"
    );
}

#[test]
fn test_gate_validates_rust_size_claims() {
    let mut engine = InferenceEngine::new();

    let ok = engine
        .validate(ValidationRequest::new(
            "A Rust Vec occupies 24 bytes on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(ok.passed, "true Rust Vec claim should validate");

    let ok2 = engine
        .validate(ValidationRequest::new(
            "An i32 occupies 4 bytes on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(ok2.passed, "true i32 claim should validate");

    let bad = engine
        .validate(ValidationRequest::new(
            "An i32 occupies 8 bytes on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(!bad.passed, "false i32 claim should be rejected");

    let flat = engine
        .validate(ValidationRequest::new("The earth is flat", "general"))
        .unwrap();
    assert!(!flat.passed, "known false claim should be rejected");
}

#[test]
fn test_gate_validates_rust_size_edge_cases() {
    // These exercise the phrasing variants that previously slipped through:
    // entity/number several tokens apart ("dyn trait object is 8 bytes") and a
    // non-"is" binding verb ("Arc adds 8 bytes").
    let mut engine = InferenceEngine::new();

    let ok_dyn = engine
        .validate(ValidationRequest::new(
            "A dyn trait object is 16 bytes on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(ok_dyn.passed, "true dyn/trait-object claim should validate");

    let bad_dyn = engine
        .validate(ValidationRequest::new(
            "A dyn trait object is 8 bytes on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(
        !bad_dyn.passed,
        "false dyn/trait-object claim should be rejected"
    );

    let ok_arc = engine
        .validate(ValidationRequest::new(
            "An Arc adds 16 bytes of overhead on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(ok_arc.passed, "true Arc overhead claim should validate");

    let bad_arc = engine
        .validate(ValidationRequest::new(
            "An Arc adds 8 bytes of overhead on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(
        !bad_arc.passed,
        "false Arc overhead claim should be rejected"
    );

    // Architecture qualifier ("64-bit") must not pair with an entity ("vec").
    let ok_vec = engine
        .validate(ValidationRequest::new(
            "A Rust Vec occupies 24 bytes on 64-bit",
            "rust_code",
        ))
        .unwrap();
    assert!(
        ok_vec.passed,
        "entity + 64-bit qualifier must not false-reject"
    );
}
