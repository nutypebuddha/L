//! # Wheel — the symbolic zodiac graph
//!
//! The wheel is a 12-node directed graph with typed edges (aspects).
//! Nodes represent knowledge domains; edges represent relationships between them.
//! Formulas live at edges — traversing the graph follows cross-domain reasoning chains.

mod axis;
mod edges;
mod graph;
mod nodes;

pub use axis::{BleachLayer, CurriculumBand, UnderstandingAxis, LEVELS_PER_CYCLE, MAX_LEVEL};
pub use edges::{Aspect, Relationship};
pub use graph::WheelGraph;
pub use nodes::{Domain, Node, ALL_DOMAINS, ALL_NODES};

use thiserror::Error;

/// Errors from wheel operations.
#[derive(Error, Debug)]
pub enum WheelError {
    #[error("unknown domain: {0}")]
    UnknownDomain(String),

    #[error("no path between {0} and {1}")]
    NoPath(Domain, Domain),

    #[error("cycle detected in traversal")]
    CycleDetected,

    #[error("max depth {0} exceeded")]
    MaxDepthExceeded(usize),
}

/// Result type for wheel operations.
pub type WheelResult<T> = Result<T, WheelError>;

/// A position on the wheel: a domain and an optional formula reference.
#[derive(Debug, Clone)]
pub struct Position {
    pub domain: Domain,
    pub formula_id: Option<String>,
}
