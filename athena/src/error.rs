//! # Unified Error Boundary
//!
//! Re-exports `LaiError` from `lai-core` and provides `From` conversions for
//! every crate-local error type, so that `Result<T, LaiError>` can be used as
//! the uniform return type at public API boundaries.

pub use lai_core::error::{InferenceError, LaiError, ParserError, ReasoningError, ValidationError};

/// Unified result type for public API boundaries.
pub type Result<T> = std::result::Result<T, LaiError>;

use crate::bankai::BankaiError;
use crate::inference::InferenceError as AthenaInferenceError;
use crate::shikai::ShikaiError;
use crate::wheel::WheelError;
use crate::zanpakuto::ZanpakutoError;

impl From<WheelError> for LaiError {
    fn from(e: WheelError) -> Self {
        LaiError::Reasoning(ReasoningError::CircularDependency(format!("{e}")))
    }
}

impl From<BankaiError> for LaiError {
    fn from(e: BankaiError) -> Self {
        LaiError::Internal(format!("{e}"))
    }
}

impl From<ShikaiError> for LaiError {
    fn from(e: ShikaiError) -> Self {
        LaiError::Parser(ParserError::Syntax(format!("{e}")))
    }
}

impl From<ZanpakutoError> for LaiError {
    fn from(e: ZanpakutoError) -> Self {
        LaiError::Internal(format!("{e}"))
    }
}

impl From<AthenaInferenceError> for LaiError {
    fn from(e: AthenaInferenceError) -> Self {
        LaiError::Inference(match e {
            AthenaInferenceError::ModelNotLoaded(msg) => InferenceError::ModelNotLoaded(msg),
            AthenaInferenceError::InferenceFailed(msg) => InferenceError::Failed(msg),
            AthenaInferenceError::BackendUnavailable(msg) => InferenceError::Failed(msg),
            AthenaInferenceError::ConfigError(msg) => InferenceError::Failed(msg),
            AthenaInferenceError::HealthCheckFailed(msg) => InferenceError::Failed(msg),
            AthenaInferenceError::NotSupported(msg) => InferenceError::Failed(msg),
            AthenaInferenceError::Other(_) => {
                InferenceError::Failed("inference backend error".into())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wheel_error_converts() {
        let e: LaiError = WheelError::CycleDetected.into();
        assert!(e.to_string().contains("cycle"));
    }

    #[test]
    fn test_bankai_error_converts() {
        let e: LaiError = BankaiError::EvalError("bad".into()).into();
        assert!(e.to_string().contains("bad"));
    }

    #[test]
    fn test_shikai_error_converts() {
        let e: LaiError = ShikaiError::UnrecognizedIntent("?".into()).into();
        assert!(e.to_string().contains("intent"));
    }

    #[test]
    fn test_zanpakuto_error_converts() {
        let e: LaiError = ZanpakutoError::Unauthenticated.into();
        assert!(e.to_string().contains("unauthenticated"));
    }

    #[test]
    fn test_inference_error_converts() {
        let e: LaiError = AthenaInferenceError::ModelNotLoaded("x".into()).into();
        assert!(e.to_string().contains("model not loaded"));
    }

    #[test]
    fn test_string_converts_to_internal() {
        let e: LaiError = "boom".to_string().into();
        assert!(e.to_string().contains("boom"));
    }
}
