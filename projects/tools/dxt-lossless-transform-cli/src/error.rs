use std::io;
use std::path::StripPrefixError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransformError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    PathError(#[from] StripPrefixError),
    #[cfg(feature = "debug")]
    #[error("{0}")]
    MmapError(String),

    #[cfg(feature = "debug")]
    #[error("Unsupported DDS format, {0}")]
    UnsupportedFormat(String),
    #[cfg(feature = "debug")]
    #[error("File was skipped by filter")]
    IgnoredByFilter,
    #[cfg(feature = "debug")]
    #[error("Invalid DDS file")]
    InvalidDdsFile,
    #[error(transparent)]
    AllocateError(#[from] dxt_lossless_transform_common::allocate::AllocateError),
    #[error(transparent)]
    FileOperationError(
        #[from] dxt_lossless_transform_file_formats_api::file_io::FileOperationError,
    ),
    /// Reserved for arbitrary errors in debug/test functionality, not runtime/end user stuff.
    #[cfg(feature = "debug")]
    #[error("{0}")]
    Debug(String),
}
