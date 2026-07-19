//! # Primitive — NAND as the Bedrock Primitive
//!
//! All computation bottoms out to NAND gates. Every formula, every expression,
//! every gate compiles to a Directed Acyclic Graph (DAG) of NAND operations.
//!
//! Core types and functions are defined in `lai-core` and re-exported here
//! for backward compatibility.

pub use lai_core::primitive::{
    nand, not, and, or, nor, xor, xnor, implies,
    half_adder, full_adder, add4, bits_to_u8, u8_to_bits,
    ExprNode, NandDag, NandExprError, NandExpression, NandNode,
};
