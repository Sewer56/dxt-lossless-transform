//! Error types for BC1 transform operations.

use dxt_lossless_transform_bc1::{
    Bc1AutoTransformError, Bc1ValidationError, DetermineBestTransformError,
};
use dxt_lossless_transform_common::allocate::AllocateError;
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
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(#[from] AllocateError),

    /// Size estimation failed during transform optimization.
    #[error("Size estimation failed: {0:?}")]
    SizeEstimationFailed(E),
}

// Implement From conversions to reduce boilerplate

impl<E> From<Bc1ValidationError> for Bc1Error<E>
where
    E: core::fmt::Debug,
{
    fn from(err: Bc1ValidationError) -> Self {
        match err {
            Bc1ValidationError::InvalidLength(len) => Bc1Error::InvalidLength(len),
            Bc1ValidationError::OutputBufferTooSmall { needed, actual } => {
                Bc1Error::OutputBufferTooSmall { needed, actual }
            }
        }
    }
}

impl<E> From<Bc1AutoTransformError<E>> for Bc1Error<E>
where
    E: core::fmt::Debug,
{
    fn from(err: Bc1AutoTransformError<E>) -> Self {
        match err {
            Bc1AutoTransformError::InvalidLength(len) => Bc1Error::InvalidLength(len),
            Bc1AutoTransformError::OutputBufferTooSmall { needed, actual } => {
                Bc1Error::OutputBufferTooSmall { needed, actual }
            }
            Bc1AutoTransformError::DetermineBestTransform(transform_err) => match transform_err {
                DetermineBestTransformError::AllocateError(alloc_err) => {
                    Bc1Error::AllocationFailed(alloc_err)
                }
                DetermineBestTransformError::SizeEstimationError(est_err) => {
                    Bc1Error::SizeEstimationFailed(est_err)
                }
            },
        }
    }
}
