//! Error types for embed/unembed operations.

use thiserror::Error;

/// Error types for embed/unembed operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EmbedError {
    /// The format specified is not supported for embedding.
    #[error("The format specified is not supported for embedding")]
    UnsupportedFormat,
    /// The embedded data is corrupted or invalid.
    #[error("The embedded data is corrupted or invalid")]
    CorruptedEmbeddedData,
    /// The header data doesn't match the expected format.
    #[error("The header data doesn't match the expected format")]
    InvalidHeaderFormat,
}
