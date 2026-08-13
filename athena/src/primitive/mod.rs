//! # Primitive — NAND as the Bedrock Primitive
//!
//! All computation bottoms out to NAND gates. Every formula, every expression,
//! every gate compiles to a Directed Acyclic Graph (DAG) of NAND operations.
//!
//! Core types and functions are defined in `lai-core` and re-exported here
//! for backward compatibility.

pub use lai_core::primitive::{
    add4, and, bits_to_u8, full_adder, half_adder, implies, nand, nor, not, or, u8_to_bits, xnor,
    xor, ExprNode, NandDag, NandExprError, NandExpression, NandNode,
};
