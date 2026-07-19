//! # Athena
//!
//! Relational intelligence engine — formulas, not facts.
//!
//! ## 4-Layer Security Architecture
//!
//! ```text
//! Asauchi  →  Zanpakuto  →  Shikai  →  Bankai
//! (public)    (access)     (queries)   (solved atomically)
//! ```
//!
//! - **Asauchi**: The public face. What everyone knows about. Basic validation, system info.
//! - **Zanpakuto**: The access layer. Name it, it becomes yours. Authentication and authorization.
//! - **Shikai**: The query layer. Release command. Intent recognition and query processing.
//! - **Bankai**: The solved form. Atomic computation — final, verified, complete.
//!
//! ## Core Components
//!
//! - **Wheel**: 12-node symbolic graph with 5 aspect types (conjunction, sextile, trine, square, opposition)
//! - **Formula**: Typed formulas in 3 tiers (atomic, bridging, vortex) + non-math (grammar, code, logic)
//! - **Gates**: 5 validation gates (math, logic, fact, confidence, formal)

pub mod asauchi;
pub mod astrology;
pub mod bankai;
#[cfg(feature = "budget")]
pub mod budget;
pub mod descent;
pub mod entity;
pub mod ephemeris;
pub mod error;
pub mod formula;
pub mod gates;
pub mod gyro;
pub mod inference;
pub mod influence;
pub mod ledger;
pub mod primitive;
pub mod shikai;
pub mod tanto;
pub mod wheel;
pub mod zanpakuto;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(feature = "llm")]
pub mod llm {
    pub use crate::bankai::llm_router::{
        LlmRouteResult, LlmRouter, QueryCategory, RouterIntent, RouterToken,
    };
    pub use crate::inference::{
        config::InferenceConfig, BackendKind, Capability, HealthStatus, InferenceBackend,
        InferenceError, InferenceRequest, InferenceResponse,
    };
}

/// Prelude: re-exports the most commonly used types.
pub mod prelude {
    pub use crate::asauchi::Asauchi;
    pub use crate::bankai::{Bankai, BankaiSolve, ChainResult};
    #[cfg(feature = "budget")]
    pub use crate::budget::{BudgetCheck, TokenBudget, TokenSpend};
    pub use crate::entity::{Entity, EntityRegistry};
    pub use crate::formula::{Formula, FormulaType};
    pub use crate::gates::{GateOutput, GateResult};
    pub use crate::influence::{DomainInfluence, InfluenceEngine, InfluenceMap};
    pub use crate::ledger::{
        record_detailed, record_event, session_ledger, Event, EventKind, Ledger,
    };
    pub use crate::primitive::{
        and, implies, nand, nor, not, or, xnor, xor, NandDag, NandExpression,
    };
    pub use crate::shikai::{Ambiguity, Intent, Shikai, ShikaiQuery};
    pub use crate::tanto::{evaluate as tanto_evaluate, TantoEnv};
    pub use crate::wheel::{Aspect, Domain, Node, WheelGraph};
    pub use crate::zanpakuto::{
        guest_identity, AccessTier, Capability, Identity, Stage, Zanpakuto,
    };

    pub use crate::error::{LaiError, ReasoningError, ValidationError};
}

/// Current version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build timestamp (set during compilation).
pub const BUILD_TIME: &str = env!("ATHENA_BUILD_TIME");
