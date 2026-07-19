pub mod domain;
pub mod formula;
pub mod primitive;
pub mod error;

pub use domain::{
    CompositionAspect, CurriculumBand, Direction, Domain, MasteryLayer, Node, Position,
    Relationship, UnderstandingAxis, WheelError, WheelGraph, WheelResult, ALL_DOMAINS, LEVELS_PER_CYCLE, MAX_LEVEL,
};
pub use error::LaiError;
pub use formula::{Formula, FormulaError, FormulaType};
pub use primitive::{
    nand, not, and, or, nor, xor, xnor, implies,
    half_adder, full_adder, add4, bits_to_u8, u8_to_bits,
    ExprNode, NandDag, NandExprError, NandExpression, NandNode,
};
