//! Error types for embed operations.

use thiserror::Error;

/// Errors that can occur during embed/unembed operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EmbedError {
    /// The embedded data is corrupted or invalid
    #[error("Corrupted embedded data. Info about the transform stored is invalid.")]
    CorruptedEmbeddedData,

    /// Transform format mismatch between header and expected format
    #[error("Transform format mismatch: header contains different format than expected")]
    FormatMismatch,
}
