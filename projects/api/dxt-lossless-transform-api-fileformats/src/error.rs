//! Error types for file format operations.

use dxt_lossless_transform_api_common::embed::EmbedError;
use thiserror::Error;

/// Errors that can occur during file format operations.
#[derive(Debug, Error)]
pub enum FileFormatError {
    /// Core embedding error from the API layer
    #[error(transparent)]
    Embed(#[from] EmbedError),

    /// Transform operation failed
    #[error("Transform operation failed: {0}")]
    Transform(String),

    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// File format not supported
    #[error("Unsupported file format for file: {0}")]
    UnsupportedFormat(String),

    /// File format not detected  
    #[error("Could not detect file format for: {0}")]
    FormatNotDetected(String),

    /// Invalid file data
    #[error("Invalid file data: {0}")]
    InvalidFileData(String),

    /// Memory mapping error
    #[error("Memory mapping error: {0}")]
    MemoryMapping(String),

    /// Transform format mismatch
    #[error("Transform format mismatch: expected {expected:?}, found {found:?}")]
    FormatMismatch { expected: String, found: String },

    /// File is filtered out by format filter
    #[error("File filtered out by format filter")]
    FilteredOut,

    /// API functionality not yet implemented
    #[error("API functionality not yet implemented")]
    ApiNotImplemented,
}

/// Result type for file format operations.
pub type FileFormatResult<T> = Result<T, FileFormatError>;

impl From<FileFormatError> for EmbedError {
    fn from(err: FileFormatError) -> Self {
        match err {
            FileFormatError::Embed(embed_err) => embed_err,
            _other => EmbedError::CorruptedEmbeddedData, // Fallback for other errors
        }
    }
}

// Re-export with old names for backwards compatibility
#[deprecated(note = "Use FileFormatError instead")]
pub type EmbeddingError = FileFormatError;

#[deprecated(note = "Use FileFormatResult instead")]
pub type EmbeddingResult<T> = FileFormatResult<T>;
