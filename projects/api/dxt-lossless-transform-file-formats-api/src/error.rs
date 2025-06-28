//! Error types for file format operations.

use crate::embed::{EmbedError, TransformFormat};
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
