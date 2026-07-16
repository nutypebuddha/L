// Copyright 2026 nutypebuddha
// SPDX-License-Identifier: Apache-2.0

pub mod aspect;
pub mod astrology;
pub mod chart;
pub mod compute;
pub mod csp;
pub mod descent;
pub mod digest;
pub mod domain_graph;
pub mod economy;
pub mod entity;
pub mod ephemeris;
pub mod formula;
pub mod hungarian;
pub mod nlp;
pub mod primitive;
pub mod profile;
pub mod query;
pub mod router;
pub mod sandbox;
pub mod scoring;
pub mod strategy;
pub mod time;
pub mod validation;
pub mod verify;

pub mod build;
#[cfg(feature = "graph")]
pub mod graph;
#[cfg(feature = "mcp")]
pub mod mcp;
pub mod optimize;

#[cfg(feature = "llm")]
pub mod inference;

pub mod cli;
pub mod companion;

pub mod prelude {
    pub use crate::aspect::{compute_aspect, is_adjacent, is_conjunction, validate_graha_index};
    pub use crate::astrology::barnum::{analyze_barnum, passes_barnum_filter};
    pub use crate::astrology::{
        AtomClassification, ChangeSorter, Element, Graha, Guna, House, Modality, Nakshatra,
        PlanetaryRuler, Rashi, Sign, SignAspect, VedicClassification, VedicElement,
    };
    pub use crate::build::{
        build, compute_objective_weights, parse_domain_profile, validate_domain_profile,
        BuildResult, DomainProfile,
    };
    pub use crate::chart::{
        derive_personality, AspectModifier, AstroAspect, ChartSnapshot, PersonalityProfile, Pillar,
        SynastryAspect, WatchArchetype,
    };
    pub use crate::compute::{
        compute_formula, create_env, evaluate_expr, evaluate_nl, evaluate_pipeline,
        extract_identifiers, is_expression_valid, solve_problem,
    };
    pub use crate::descent::{
        lowercase_string, tokenize_descent, DescentEngine, DescentLayer, ProvenanceStep,
        SettledToken, SettlingMatrix,
    };
    pub use crate::digest::{sha256, sha256_hex, to_hex};
    pub use crate::domain_graph::{
        CompositionAspect, Domain, UnderstandingAxis, WheelError, WheelGraph, ALL_DOMAINS,
    };
    pub use crate::economy::budget::Budget;
    pub use crate::economy::conversation::ConversationTracker;
    pub use crate::economy::cost::CostTracker;
    pub use crate::economy::tray::BallEconomy;
    pub use crate::entity::{
        compute_entity_hash, validate_entity_id, DynamicEntity, Entity, EntityRegistry,
        EventRegistry, ShikaiFormRegistry,
    };
    pub use crate::ephemeris::{
        ayanamsa_deg, julian_day, julian_day_to_date, lahiri_ayanamsa, tropical_longitude,
        AyanamsaSystem, GrahaPosition,
    };
    pub use crate::formula::{
        extract_formula_domain, validate_formula_id, Formula, FormulaRegistry, FormulaType,
    };
    pub use crate::nlp::{extract_keywords, normalize_query_text, NlpContext};
    pub use crate::optimize::{explain, parse_schema, solve, Allocation, Schema};
    pub use crate::primitive::arithmetic::{add_unsigned_8, full_adder, half_adder};
    pub use crate::primitive::dag::NandDag;
    pub use crate::primitive::expr::{NandExprError, NandExpression};
    pub use crate::primitive::nand::{and_gate, nand_gate, not_gate, or_gate, xor_gate};
    pub use crate::profile::{Profile, TemperamentProfile, WuXingProfile};
    pub use crate::query::{determine_query_domain, extract_numerical_values, parse_query_intent};
    pub use crate::router::{GyroDynamics, GyroRouter, GyroState, RouteResult, Vec3};
    pub use crate::verify::diagnostics::{Diagnostic, DiagnosticGate, DiagnosticReport, Severity};
    pub use crate::verify::protocol::RefinementSession;
    pub use crate::verify::verifier::{
        verify_expression, verify_proposal, LlmProposal, ProposalKind,
    };
    pub use crate::verify::{aggregate_confidence_scores, compute_confidence_score};
}
