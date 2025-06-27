//! Error types for file format operations.

use crate::embed::EmbedError;
use alloc::{format, string::String};
use thiserror::Error;

/// Result type for file format operations
pub type FileFormatResult<T> = Result<T, FileFormatError>;

/// Specific lightweight-mmap errors
#[cfg(feature = "lightweight-mmap")]
#[derive(Debug, Error)]
pub enum LightweightMmapError {
    /// File handle operation error
    #[error("File handle error: {0}")]
    FileHandle(#[from] lightweight_mmap::handles::HandleOpenError),

    /// Memory mapping operation error  
    #[error("Memory mapping error: {0}")]
    MemoryMapping(#[from] lightweight_mmap::mmap::MmapError),
}

/// File I/O errors that can occur with different backends
#[derive(Debug, Error)]
pub enum FileIoError {
    /// Error from lightweight-mmap operations
    #[cfg(feature = "lightweight-mmap")]
    #[error("Lightweight mmap error: {0}")]
    LightweightMmap(#[from] LightweightMmapError),

    /// Error from std I/O operations
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    Std(#[from] std::io::Error),
}

/// Errors that can occur during file format operations
#[derive(Debug, Error)]
pub enum FileFormatError {
    /// Error from embed/unembed operations
    #[error("Embed error: {0}")]
    Embed(#[from] EmbedError),

    /// Unknown or unsupported file format
    #[error("Unknown file format")]
    UnknownFormat,

    /// File format was detected but is not supported
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(&'static str),

    /// No transform builder provided for the detected format
    #[error("No transform builder provided for format: {0}")]
    NoBuilderForFormat(&'static str),

    /// Invalid or corrupted file data
    #[error("Invalid file data: {0}")]
    InvalidFileData(String),

    /// Transform operation failed
    #[error("Transform failed: {0}")]
    TransformFailed(String),

    /// Untransform operation failed
    #[error("Untransform failed: {0}")]
    UntransformFailed(String),

    /// File I/O error
    #[cfg(feature = "file-io")]
    #[error("File I/O error: {0}")]
    FileIo(#[from] FileIoError),
}

// Direct From implementations for specific error types used with ? operator
#[cfg(all(feature = "lightweight-mmap", feature = "file-io"))]
impl From<lightweight_mmap::handles::HandleOpenError> for FileFormatError {
    fn from(e: lightweight_mmap::handles::HandleOpenError) -> Self {
        Self::FileIo(FileIoError::LightweightMmap(
            LightweightMmapError::FileHandle(e),
        ))
    }
}

#[cfg(all(feature = "lightweight-mmap", feature = "file-io"))]
impl From<lightweight_mmap::mmap::MmapError> for FileFormatError {
    fn from(e: lightweight_mmap::mmap::MmapError) -> Self {
        Self::FileIo(FileIoError::LightweightMmap(
            LightweightMmapError::MemoryMapping(e),
        ))
    }
}

// Allow converting from BC1 API errors
impl From<dxt_lossless_transform_bc1_api::Bc1Error> for FileFormatError {
    fn from(e: dxt_lossless_transform_bc1_api::Bc1Error) -> Self {
        Self::TransformFailed(format!("BC1: {e:?}"))
    }
}
