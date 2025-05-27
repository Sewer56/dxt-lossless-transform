use std::io;
use std::path::StripPrefixError;

// TODO: Use thiserror here.

#[derive(Debug)]
pub enum TransformError {
    IoError(io::Error),
    PathError(StripPrefixError),
    MmapError(String),
    UnsupportedFormat(String),
    IgnoredByFilter,
    InvalidDdsFile,
    AllocateError(dxt_lossless_transform_common::allocate::AllocateError),
    /// Reserved for arbitrary errors in debug/test functionality, not runtime/end user stuff.
    Debug(String),
}

impl From<io::Error> for TransformError {
    fn from(error: io::Error) -> Self {
        TransformError::IoError(error)
    }
}

impl From<StripPrefixError> for TransformError {
    fn from(error: StripPrefixError) -> Self {
        TransformError::PathError(error)
    }
}

impl From<dxt_lossless_transform_common::allocate::AllocateError> for TransformError {
    fn from(error: dxt_lossless_transform_common::allocate::AllocateError) -> Self {
        TransformError::AllocateError(error)
    }
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::IoError(e) => write!(f, "{e}"),
            TransformError::PathError(e) => write!(f, "{e}"),
            TransformError::MmapError(e) => write!(f, "{e}"),
            TransformError::UnsupportedFormat(e) => write!(f, "Unsupported DDS format, {e}"),
            TransformError::InvalidDdsFile => write!(f, "Invalid DDS file"),
            TransformError::IgnoredByFilter => write!(f, "File was skipped by filter"),
            TransformError::Debug(e) => write!(f, "{e}"),
            TransformError::AllocateError(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for TransformError {}
