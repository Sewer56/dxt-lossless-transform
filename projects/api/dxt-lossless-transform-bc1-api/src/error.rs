//! Error types for BC1 transform operations.

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
