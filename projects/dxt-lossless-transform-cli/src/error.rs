use std::io;
use std::path::StripPrefixError;

#[derive(Debug)]
pub enum TransformError {
    IoError(io::Error),
    PathError(StripPrefixError),
    MmapError(String),
    UnsupportedFormat(String),
    InvalidDdsFile,
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

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransformError::IoError(e) => write!(f, "{}", e),
            TransformError::PathError(e) => write!(f, "{}", e),
            TransformError::MmapError(e) => write!(f, "{}", e),
            TransformError::UnsupportedFormat(e) => write!(f, "Unsupported DDS format, {}", e),
            TransformError::InvalidDdsFile => write!(f, "Invalid DDS file"),
        }
    }
}

impl std::error::Error for TransformError {}
