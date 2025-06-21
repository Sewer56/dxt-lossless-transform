//! Error types for embed/unembed operations.

/// Error types for embed/unembed operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedError {
    /// The format specified is not supported for embedding.
    UnsupportedFormat,
    /// The embedded data is corrupted or invalid.
    CorruptedEmbeddedData,
    /// The header data doesn't match the expected format.
    InvalidHeaderFormat,
}
