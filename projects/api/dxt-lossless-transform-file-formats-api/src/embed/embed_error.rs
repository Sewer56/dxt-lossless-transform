//! Error types for embed operations.

use thiserror::Error;

/// Errors that can occur during embed/unembed operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EmbedError {
    /// The embedded data is corrupted or invalid
    #[error("Corrupted embedded data. Info about the transform stored is invalid.")]
    CorruptedEmbeddedData,

    /// Unknown transform format in header
    #[error("Unknown transform format: header contains unrecognized format value")]
    UnknownFormat,
}
