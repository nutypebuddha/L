pub mod arithmetic;
pub mod dag;
pub mod expr;
pub mod nand;
pub mod nand_continuous;

/// Backward-compatible boolean gate access path for descent module.
pub mod digital {
    pub fn nand(left_input: bool, right_input: bool) -> bool {
        super::nand::nand_gate(left_input, right_input)
    }
}

pub use dag::NandDag;
pub use expr::{NandExprError, NandExpression};
