//! Error types for embed operations.

use thiserror::Error;

/// Errors that can occur during embed/unembed operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EmbedError {
    /// The embedded data is corrupted or invalid
    #[error("Corrupted embedded data")]
    CorruptedEmbeddedData,

    /// Insufficient data for the operation
    #[error("Insufficient data")]
    InsufficientData,

    /// Invalid transform format
    #[error("Invalid transform format")]
    InvalidFormat,
}
