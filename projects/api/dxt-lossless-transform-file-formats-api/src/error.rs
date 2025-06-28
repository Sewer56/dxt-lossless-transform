//! Error types for transform operations.

use crate::embed::{EmbedError, TransformFormat};
use thiserror::Error;

/// Result type for transform operations
pub type TransformResult<T> = Result<T, TransformError>;

/// Result type for format handler operations
pub type FormatHandlerResult<T> = Result<T, FormatHandlerError>;

/// Errors specific to file format handlers (DDS, etc.)
///
/// These errors occur when format handlers attempt to parse, validate,
/// or process file format-specific data.
#[derive(Debug, Error)]
pub enum FormatHandlerError {
    /// Unknown or unsupported file format - the file format itself cannot be detected or parsed
    #[error("Unknown file format")]
    UnknownFileFormat,

    /// Could not parse input file header during transform operation
    #[error("Invalid input file header during transform")]
    InvalidInputFileHeader,

    /// Could not validate restored file header during untransform operation
    #[error("Invalid restored file header during untransform - file may be corrupted or wrong handler used")]
    InvalidRestoredFileHeader,

    /// Transform format is not yet implemented by this handler
    #[error("{0:?} format not yet implemented")]
    FormatNotImplemented(TransformFormat),

    /// No transform builder provided for the detected format
    #[error("No transform builder provided for format: {0:?}")]
    NoBuilderForFormat(TransformFormat),

    /// Output buffer is too small for the operation
    #[error("Output buffer too small: required {required} bytes, got {actual} bytes")]
    OutputBufferTooSmall { required: usize, actual: usize },

    /// Input buffer is too short for the operation
    #[error("Input buffer too short: required at least {required} bytes, got {actual} bytes")]
    InputTooShort { required: usize, actual: usize },
}

/// Errors that can occur during core transform operations
///
/// These are pure library errors related to memory management, data validation,
/// and core transform functionality.
#[derive(Debug, Error)]
pub enum TransformError {
    /// Error from embed/unembed operations
    #[error("Embed error: {0}")]
    Embed(#[from] EmbedError),

    /// Format handler error
    #[error("Format handler error: {0}")]
    FormatHandler(#[from] FormatHandlerError),

    /// BC1 transform error
    #[error("BC1 transform error: {0}")]
    Bc1(#[from] dxt_lossless_transform_bc1_api::Bc1Error),

    /// Unrecognized transform format in header - the transform header contains an unsupported format variant
    #[error("Unrecognized or unsupported transform format in header")]
    UnknownTransformFormat,

    /// Data is not properly aligned for the format requirements
    #[error("Invalid data alignment: size {size} is not divisible by {required_divisor}")]
    InvalidDataAlignment {
        size: usize,
        required_divisor: usize,
    },
}
