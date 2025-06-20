use std::io;
use std::path::StripPrefixError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransformError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    PathError(#[from] StripPrefixError),
    #[error("{0}")]
    MmapError(String),
    #[error("Unsupported DDS format, {0}")]
    UnsupportedFormat(String),
    #[error("File was skipped by filter")]
    IgnoredByFilter,
    #[error("Invalid DDS file")]
    InvalidDdsFile,
    #[error(transparent)]
    AllocateError(#[from] dxt_lossless_transform_common::allocate::AllocateError),
    /// Reserved for arbitrary errors in debug/test functionality, not runtime/end user stuff.
    #[cfg(feature = "debug")]
    #[error("{0}")]
    Debug(String),
}
