//! # Unified Error Boundary
//!
//! Re-exports `LaiError` from `lai-core` and provides `From` conversions for
//! every crate-local error type, so that `Result<T, LaiError>` can be used as
//! the uniform return type at public API boundaries.

pub use lai_core::error::{InferenceError, LaiError, ReasoningError, ValidationError};

/// Unified result type for public API boundaries.
pub type Result<T> = std::result::Result<T, LaiError>;

use crate::chart::HouseSystemError;
use crate::time::TimezoneError;

impl From<TimezoneError> for LaiError {
    fn from(e: TimezoneError) -> Self {
        LaiError::Validation(ValidationError::Malformed(format!("{e}")))
    }
}

impl From<HouseSystemError> for LaiError {
    fn from(e: HouseSystemError) -> Self {
        let HouseSystemError::PlacidusUnsupportedAtLatitude(lat) = e;
        LaiError::Validation(ValidationError::OutOfRange {
            field: "latitude".to_string(),
            value: lat,
            min: f64::NEG_INFINITY,
            max: 66.0,
        })
    }
}

#[cfg(feature = "llm")]
impl From<crate::inference::CopilotError> for LaiError {
    fn from(e: crate::inference::CopilotError) -> Self {
        LaiError::Inference(match e {
            crate::inference::CopilotError::BinaryMissing => {
                InferenceError::ModelNotLoaded("llama binary missing".into())
            }
            crate::inference::CopilotError::ModelMissing => {
                InferenceError::ModelNotLoaded("model missing".into())
            }
            crate::inference::CopilotError::SpawnFailed(msg) => {
                InferenceError::Failed(format!("spawn failed: {msg}"))
            }
            crate::inference::CopilotError::NoOutput => {
                InferenceError::Failed("no output from model".into())
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timezone_error_converts() {
        let e: LaiError = TimezoneError::MissingTimezone.into();
        assert!(e.to_string().contains("timezone"));
    }

    #[test]
    fn test_house_system_error_converts() {
        let e: LaiError = HouseSystemError::PlacidusUnsupportedAtLatitude(70.0).into();
        assert!(e.to_string().contains("latitude"));
    }

    #[test]
    fn test_string_converts_to_internal() {
        let e: LaiError = "boom".to_string().into();
        assert!(e.to_string().contains("boom"));
    }

    #[cfg(feature = "llm")]
    #[test]
    fn test_copilot_error_converts() {
        let e: LaiError = crate::inference::CopilotError::NoOutput.into();
        assert!(e.to_string().contains("output"));
    }
}
