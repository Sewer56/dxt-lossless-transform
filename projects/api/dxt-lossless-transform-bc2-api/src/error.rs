//! Error types for BC2 transform operations.

use alloc::string::String;
use dxt_lossless_transform_bc2::{
    Bc2AutoTransformError, Bc2ValidationError, DetermineBestTransformError,
};
use thiserror::Error;

/// Errors that can occur during BC2 transform operations.
#[derive(Debug, Error)]
pub enum Bc2Error<E = String>
where
    E: core::fmt::Debug,
{
    /// The input data length is invalid (must be divisible by 16).
    #[error("Invalid input length: {0} bytes. Length must be divisible by 16 (BC2 block size).")]
    InvalidLength(usize),

    /// The output buffer is too small for the operation.
    #[error("Output buffer too small: need {needed} bytes, but only {actual} bytes available.")]
    OutputBufferTooSmall {
        /// The required size in bytes
        needed: usize,
        /// The actual size in bytes  
        actual: usize,
    },

    /// Memory allocation failed.
    #[error("Memory allocation failed")]
    AllocationFailed,

    /// Size estimation failed during transform optimization.
    #[error("Size estimation failed: {0:?}")]
    SizeEstimationFailed(E),
}

// Internal conversion functions to avoid exposing core types in public From traits
// The types below are unstable, but ours have to be stable.
impl<E> Bc2Error<E>
where
    E: core::fmt::Debug,
{
    /// Convert from core validation error (internal use only)
    pub(crate) fn from_validation_error(err: Bc2ValidationError) -> Self {
        match err {
            Bc2ValidationError::InvalidLength(len) => Bc2Error::InvalidLength(len),
            Bc2ValidationError::OutputBufferTooSmall { needed, actual } => {
                Bc2Error::OutputBufferTooSmall { needed, actual }
            }
        }
    }

    /// Convert from core auto transform error (internal use only)
    pub(crate) fn from_auto_transform_error(err: Bc2AutoTransformError<E>) -> Self {
        match err {
            Bc2AutoTransformError::InvalidLength(len) => Bc2Error::InvalidLength(len),
            Bc2AutoTransformError::OutputBufferTooSmall { needed, actual } => {
                Bc2Error::OutputBufferTooSmall { needed, actual }
            }
            Bc2AutoTransformError::DetermineBestTransform(transform_err) => match transform_err {
                DetermineBestTransformError::AllocateError(_) => Bc2Error::AllocationFailed,
                DetermineBestTransformError::SizeEstimationError(est_err) => {
                    Bc2Error::SizeEstimationFailed(est_err)
                }
            },
        }
    }
}
