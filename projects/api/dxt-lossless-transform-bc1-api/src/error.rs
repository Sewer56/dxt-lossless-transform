//! Error types for BC1 transform operations.

use alloc::string::String;
use dxt_lossless_transform_bc1::{
    Bc1AutoTransformError, Bc1ValidationError, DetermineBestTransformError,
};
use thiserror::Error;

/// Errors that can occur during BC1 transform operations.
#[derive(Debug, Error)]
pub enum Bc1Error<E = String>
where
    E: core::fmt::Debug,
{
    /// The input data length is invalid (must be divisible by 8).
    #[error("Invalid input length: {0} bytes. Length must be divisible by 8 (BC1 block size).")]
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
impl<E> Bc1Error<E>
where
    E: core::fmt::Debug,
{
    /// Convert from core validation error (internal use only)
    pub(crate) fn from_validation_error(err: Bc1ValidationError) -> Self {
        match err {
            Bc1ValidationError::InvalidLength(len) => Bc1Error::InvalidLength(len),
            Bc1ValidationError::OutputBufferTooSmall { needed, actual } => {
                Bc1Error::OutputBufferTooSmall { needed, actual }
            }
        }
    }

    /// Convert from core auto transform error (internal use only)
    pub(crate) fn from_auto_transform_error(err: Bc1AutoTransformError<E>) -> Self {
        match err {
            Bc1AutoTransformError::InvalidLength(len) => Bc1Error::InvalidLength(len),
            Bc1AutoTransformError::OutputBufferTooSmall { needed, actual } => {
                Bc1Error::OutputBufferTooSmall { needed, actual }
            }
            Bc1AutoTransformError::DetermineBestTransform(transform_err) => match transform_err {
                DetermineBestTransformError::AllocateError(_) => Bc1Error::AllocationFailed,
                DetermineBestTransformError::SizeEstimationError(est_err) => {
                    Bc1Error::SizeEstimationFailed(est_err)
                }
            },
        }
    }
}
