mod axis;
mod edges;
mod graph;
mod nodes;

pub use axis::{BleachLayer, CurriculumBand, UnderstandingAxis, LEVELS_PER_CYCLE, MAX_LEVEL};
pub use edges::{CompositionAspect, Relationship};
pub use graph::WheelGraph;
pub use nodes::{compute_all_nodes, Domain, Node, ALL_DOMAINS};

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
