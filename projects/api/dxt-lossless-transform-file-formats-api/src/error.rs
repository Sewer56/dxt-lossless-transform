//! Error types for transform operations.

use crate::embed::{EmbedError, TransformFormat};
use alloc::{format, string::String};
use thiserror::Error;

/// Result type for transform operations
pub type TransformResult<T> = Result<T, TransformError>;

/// Errors that can occur during transform operations
#[derive(Debug, Error)]
pub enum TransformError {
    /// Error from embed/unembed operations
    #[error("Embed error: {0}")]
    Embed(#[from] EmbedError),

    /// Unknown or unsupported file format - the file format itself cannot be detected or parsed
    #[error("Unknown file format")]
    UnknownFileFormat,

    /// Unrecognized transform format in header - the transform header contains an unsupported format variant
    #[error("Unrecognized or unsupported transform format in header")]
    UnknownTransformFormat,

    /// No transform builder provided for the detected format
    #[error("No transform builder provided for format: {0:?}")]
    NoBuilderForFormat(TransformFormat),

    /// Output buffer is too small for the operation
    #[error("Output buffer too small: required {required} bytes, got {actual} bytes")]
    OutputBufferTooSmall { required: usize, actual: usize },

    /// Input buffer is too short for the operation
    #[error("Input buffer too short: required at least {required} bytes, got {actual} bytes")]
    InputTooShort { required: usize, actual: usize },

    /// Input and output buffer sizes must match
    #[error("Buffer size mismatch: input has {input_len} bytes, output has {output_len} bytes")]
    BufferSizeMismatch { input_len: usize, output_len: usize },

    /// Data is not properly aligned for the format requirements
    #[error("Invalid data alignment: size {size} is not divisible by {required_divisor}")]
    InvalidDataAlignment {
        size: usize,
        required_divisor: usize,
    },

    /// Could not parse input file header during transform operation
    #[error("Invalid input file header during transform")]
    InvalidInputFileHeader,

    /// Input texture format is not supported by the library during transform operation
    #[error("Unsupported texture format during transform - file format is valid but texture compression format is not supported by this library")]
    InvalidInputFileFormat,

    /// Could not validate restored file header during untransform operation
    #[error("Invalid restored file header during untransform - file may be corrupted or wrong handler used")]
    InvalidRestoredFileHeader,

    /// Transform format is not yet implemented
    #[error("{0:?} format not yet implemented")]
    FormatNotImplemented(TransformFormat),

    /// Transform operation failed
    #[error("Transform failed: {0}")]
    TransformFailed(String),
}

// Allow converting from BC1 API errors
impl From<dxt_lossless_transform_bc1_api::Bc1Error> for TransformError {
    fn from(e: dxt_lossless_transform_bc1_api::Bc1Error) -> Self {
        Self::TransformFailed(format!("BC1: {e:?}"))
    }
}
