use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransformError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Allocate(#[from] dxt_lossless_transform_common::allocate::AllocateError),
    #[error(transparent)]
    FileOperation(#[from] dxt_lossless_transform_file_formats_api::file_io::FileOperationError),
    /// Reserved for arbitrary errors in debug/test functionality, not runtime/end user stuff.
    #[cfg(feature = "debug-format")]
    #[error("{0}")]
    Debug(String),
}
