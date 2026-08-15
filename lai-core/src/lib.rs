pub mod domain;
pub mod error;
pub mod formula;
pub mod primitive;
pub mod strategy;

pub use domain::{
    CompositionAspect, CurriculumBand, Direction, Domain, MasteryLayer, Node, Position,
    Relationship, UnderstandingAxis, WheelError, WheelGraph, WheelResult, ALL_DOMAINS,
    LEVELS_PER_CYCLE, MAX_LEVEL,
};
pub use error::LaiError;
pub use formula::{Formula, FormulaError, FormulaType, Verifiability};
pub use primitive::{
    add4, and, bits_to_u8, full_adder, half_adder, implies, nand, nor, not, or, u8_to_bits, xnor,
    xor, ExprNode, NandDag, NandExprError, NandExpression, NandNode,
};
pub use strategy::{
    generate_challenge, simulate_sequence, Action, ChallengeReport, Claim, Contradiction,
    ContradictionStatus, Counterexample, DeterministicSimulator, EntityResolution, EpistemicStatus,
    Evidence, Observation, Simulator, StateDelta, Strategy, StrategyIR, StrategyVerdict,
    Transition, Unknown, WorldState,
};
