//! C API error handling for BC1 operations.

use crate::error::Bc1Error;
use core::ffi::c_char;
use dxt_lossless_transform_bc1::{
    Bc1AutoTransformError, Bc1ValidationError, DetermineBestTransformError,
};

/// C-compatible error codes for BC1 operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dltbc1ErrorCode {
    /// Operation succeeded
    Success = 0,
    /// Invalid input length: Length must be divisible by 8 (BC1 block size)
    InvalidLength = 1,
    /// Output buffer too small for the operation
    OutputBufferTooSmall = 2,
    /// Memory allocation failed
    AllocationFailed = 3,
    /// Size estimation failed during transform optimization
    SizeEstimationFailed = 4,
    /// Null pointer provided for data parameter
    NullDataPointer = 5,
    /// Null pointer provided for DltSizeEstimator parameter
    NullEstimatorPointer = 6,
    /// Null pointer provided for Dltbc1TransformSettings parameter
    NullTransformSettingsPointer = 7,
    /// Null pointer provided for input parameter
    NullInputPointer = 8,
    /// Null pointer provided for output buffer parameter
    NullOutputBufferPointer = 9,
    /// Null pointer provided for Dltbc1TransformSettingsBuilder parameter
    NullTransformSettingsBuilderPointer = 10,
    /// Null pointer provided for Dltbc1EstimateSettingsBuilder parameter
    NullBuilderPointer = 11,
}

/// C-compatible Result type for BC1 operations.
#[repr(C)]
pub struct Dltbc1Result {
    /// Error code (0 = success, non-zero = error)
    pub error_code: Dltbc1ErrorCode,
}

impl Dltbc1Result {
    /// Create a success result
    pub const fn success() -> Self {
        Self {
            error_code: Dltbc1ErrorCode::Success,
        }
    }

    /// Create an error result from an error code
    pub const fn from_error_code(error_code: Dltbc1ErrorCode) -> Self {
        Self { error_code }
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self.error_code, Dltbc1ErrorCode::Success)
    }
}

impl<T> From<Result<T, Bc1Error>> for Dltbc1Result {
    fn from(result: Result<T, Bc1Error>) -> Self {
        match result {
            Ok(_) => Self::success(),
            Err(e) => e.into(),
        }
    }
}

impl<E> From<Bc1Error<E>> for Dltbc1Result
where
    E: core::fmt::Debug,
{
    fn from(error: Bc1Error<E>) -> Self {
        let error_code = match error {
            Bc1Error::InvalidLength(_) => Dltbc1ErrorCode::InvalidLength,
            Bc1Error::OutputBufferTooSmall { .. } => Dltbc1ErrorCode::OutputBufferTooSmall,
            Bc1Error::AllocationFailed(_) => Dltbc1ErrorCode::AllocationFailed,
            Bc1Error::SizeEstimationFailed(_) => Dltbc1ErrorCode::SizeEstimationFailed,
        };
        Self::from_error_code(error_code)
    }
}

impl From<Bc1ValidationError> for Dltbc1Result {
    fn from(error: Bc1ValidationError) -> Self {
        let error_code = match error {
            Bc1ValidationError::InvalidLength(_) => Dltbc1ErrorCode::InvalidLength,
            Bc1ValidationError::OutputBufferTooSmall { .. } => {
                Dltbc1ErrorCode::OutputBufferTooSmall
            }
        };
        Self::from_error_code(error_code)
    }
}

impl<E> From<Bc1AutoTransformError<E>> for Dltbc1Result
where
    E: core::fmt::Debug,
{
    fn from(error: Bc1AutoTransformError<E>) -> Self {
        let error_code = match error {
            Bc1AutoTransformError::InvalidLength(_) => Dltbc1ErrorCode::InvalidLength,
            Bc1AutoTransformError::OutputBufferTooSmall { .. } => {
                Dltbc1ErrorCode::OutputBufferTooSmall
            }
            Bc1AutoTransformError::DetermineBestTransform(inner) => match inner {
                DetermineBestTransformError::SizeEstimationError(_) => {
                    Dltbc1ErrorCode::SizeEstimationFailed
                }
                DetermineBestTransformError::AllocateError(_) => Dltbc1ErrorCode::AllocationFailed,
            },
        };
        Self::from_error_code(error_code)
    }
}

/// Get a null-terminated string description of the error code.
///
/// The returned string is a static string literal that does not need to be freed.
///
/// # Safety
/// This function is safe to call with any error code value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_error_message(error_code: Dltbc1ErrorCode) -> *const c_char {
    match error_code {
        Dltbc1ErrorCode::Success => c"Success".as_ptr() as *const c_char,
        Dltbc1ErrorCode::InvalidLength => {
            c"Invalid input length: Length must be divisible by 8 (BC1 block size)".as_ptr()
                as *const c_char
        }
        Dltbc1ErrorCode::OutputBufferTooSmall => {
            c"Output buffer too small for the operation".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::AllocationFailed => c"Memory allocation failed".as_ptr() as *const c_char,
        Dltbc1ErrorCode::SizeEstimationFailed => {
            c"Size estimation failed during transform optimization".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::NullDataPointer => {
            c"Null pointer provided for data parameter".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::NullEstimatorPointer => {
            c"Null pointer provided for DltSizeEstimator parameter".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::NullTransformSettingsPointer => {
            c"Null pointer provided for Dltbc1TransformSettings parameter".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::NullInputPointer => {
            c"Null pointer provided for input parameter".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::NullOutputBufferPointer => {
            c"Null pointer provided for output parameter".as_ptr() as *const c_char
        }
        Dltbc1ErrorCode::NullTransformSettingsBuilderPointer => {
            c"Null pointer provided for Dltbc1TransformSettingsBuilder parameter".as_ptr()
                as *const c_char
        }
        Dltbc1ErrorCode::NullBuilderPointer => {
            c"Null pointer provided for Dltbc1EstimateSettingsBuilder parameter".as_ptr()
                as *const c_char
        }
    }
}
