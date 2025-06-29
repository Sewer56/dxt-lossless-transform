//! Error types for file I/O operations.

use crate::error::TransformError;
use thiserror::Error;

/// Result type for file operations  
pub type FileOperationResult<T> = Result<T, FileOperationError>;

/// Errors that can occur during file operations.
///
/// File operations can fail due to either I/O errors (file not found, permission denied, etc.)
/// or transform-related errors (invalid data, unsupported format, etc.).
#[derive(Debug, Error)]
pub enum FileOperationError {
    /// I/O operation failed
    #[error("I/O operation failed: {0}")]
    Io(#[from] FileIoError),

    /// Transform operation failed
    #[error("Transform operation failed: {0}")]
    Transform(#[from] TransformError),
}

/// Specific backend-related errors that can occur during file I/O operations.
/// This enum contains low-level I/O errors from different backends.
#[cfg(feature = "lightweight-mmap")]
#[derive(Debug, Error)]
pub enum LightweightMmapError {
    /// Error opening file handle
    #[error("Failed to open file handle: {0}")]
    FileHandle(#[from] lightweight_mmap::handles::HandleOpenError),

    /// Error creating memory mapping
    #[error("Failed to create memory mapping: {0}")]
    MemoryMapping(#[from] lightweight_mmap::mmap::MmapError),
}

/// File I/O errors that can occur with different backends
#[derive(Debug, Error)]
pub enum FileIoError {
    /// Error from lightweight-mmap backend
    #[cfg(feature = "lightweight-mmap")]
    #[error("lightweight-mmap error: {0}")]
    LightweightMmap(#[from] LightweightMmapError),

    /// Error from std I/O operations
    #[cfg(feature = "std")]
    #[error("I/O error: {0}")]
    Std(#[from] std::io::Error),
}

// Direct From implementations for specific error types used with ? operator in file operations
#[cfg(feature = "lightweight-mmap")]
impl From<lightweight_mmap::handles::HandleOpenError> for FileOperationError {
    fn from(e: lightweight_mmap::handles::HandleOpenError) -> Self {
        Self::Io(FileIoError::LightweightMmap(
            LightweightMmapError::FileHandle(e),
        ))
    }
}

#[cfg(feature = "lightweight-mmap")]
impl From<lightweight_mmap::mmap::MmapError> for FileOperationError {
    fn from(e: lightweight_mmap::mmap::MmapError) -> Self {
        Self::Io(FileIoError::LightweightMmap(
            LightweightMmapError::MemoryMapping(e),
        ))
    }
}

#[cfg(feature = "lightweight-mmap")]
impl From<lightweight_mmap::handles::HandleOpenError> for FileIoError {
    fn from(e: lightweight_mmap::handles::HandleOpenError) -> Self {
        Self::LightweightMmap(LightweightMmapError::FileHandle(e))
    }
}

#[cfg(feature = "lightweight-mmap")]
impl From<lightweight_mmap::mmap::MmapError> for FileIoError {
    fn from(e: lightweight_mmap::mmap::MmapError) -> Self {
        Self::LightweightMmap(LightweightMmapError::MemoryMapping(e))
    }
}
