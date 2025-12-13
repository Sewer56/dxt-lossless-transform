//! C API error handling for BC2 operations.

use crate::error::Bc2Error;
use core::ffi::c_char;
use dxt_lossless_transform_bc2::{
    Bc2AutoTransformError, Bc2ValidationError, DetermineBestTransformError,
};

/// C-compatible error codes for BC2 operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dltbc2ErrorCode {
    /// Operation succeeded
    Success = 0,
    /// Invalid input length: Length must be divisible by 16 (BC2 block size)
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
    /// Null pointer provided for Dltbc2TransformSettings parameter
    NullTransformSettingsPointer = 7,
    /// Null pointer provided for input parameter
    NullInputPointer = 8,
    /// Null pointer provided for output buffer parameter
    NullOutputBufferPointer = 9,
    /// Null pointer provided for Dltbc2ManualTransformBuilder parameter
    NullManualTransformBuilderPointer = 10,
    /// Null pointer provided for Dltbc2EstimateSettingsBuilder parameter
    NullBuilderPointer = 11,
    /// Null pointer provided for manual builder output parameter
    NullManualBuilderOutputPointer = 12,
}

/// C-compatible Result type for BC2 operations.
#[repr(C)]
pub struct Dltbc2Result {
    /// Error code (0 = success, non-zero = error)
    pub error_code: Dltbc2ErrorCode,
}

impl Dltbc2Result {
    /// Create a success result
    pub const fn success() -> Self {
        Self {
            error_code: Dltbc2ErrorCode::Success,
        }
    }

    /// Create an error result from an error code
    pub const fn from_error_code(error_code: Dltbc2ErrorCode) -> Self {
        Self { error_code }
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self.error_code, Dltbc2ErrorCode::Success)
    }
}

impl<T> From<Result<T, Bc2Error>> for Dltbc2Result {
    fn from(result: Result<T, Bc2Error>) -> Self {
        match result {
            Ok(_) => Self::success(),
            Err(e) => e.into(),
        }
    }
}

impl<E> From<Bc2Error<E>> for Dltbc2Result
where
    E: core::fmt::Debug,
{
    fn from(error: Bc2Error<E>) -> Self {
        let error_code = match error {
            Bc2Error::InvalidLength(_) => Dltbc2ErrorCode::InvalidLength,
            Bc2Error::OutputBufferTooSmall { .. } => Dltbc2ErrorCode::OutputBufferTooSmall,
            Bc2Error::AllocationFailed => Dltbc2ErrorCode::AllocationFailed,
            Bc2Error::SizeEstimationFailed(_) => Dltbc2ErrorCode::SizeEstimationFailed,
        };
        Self::from_error_code(error_code)
    }
}

impl From<Bc2ValidationError> for Dltbc2Result {
    fn from(error: Bc2ValidationError) -> Self {
        let error_code = match error {
            Bc2ValidationError::InvalidLength(_) => Dltbc2ErrorCode::InvalidLength,
            Bc2ValidationError::OutputBufferTooSmall { .. } => {
                Dltbc2ErrorCode::OutputBufferTooSmall
            }
        };
        Self::from_error_code(error_code)
    }
}

impl<E> From<Bc2AutoTransformError<E>> for Dltbc2Result
where
    E: core::fmt::Debug,
{
    fn from(error: Bc2AutoTransformError<E>) -> Self {
        let error_code = match error {
            Bc2AutoTransformError::InvalidLength(_) => Dltbc2ErrorCode::InvalidLength,
            Bc2AutoTransformError::OutputBufferTooSmall { .. } => {
                Dltbc2ErrorCode::OutputBufferTooSmall
            }
            Bc2AutoTransformError::DetermineBestTransform(inner) => match inner {
                DetermineBestTransformError::SizeEstimationError(_) => {
                    Dltbc2ErrorCode::SizeEstimationFailed
                }
                DetermineBestTransformError::AllocateError(_) => Dltbc2ErrorCode::AllocationFailed,
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
pub unsafe extern "C" fn dltbc2_error_message(error_code: Dltbc2ErrorCode) -> *const c_char {
    match error_code {
        Dltbc2ErrorCode::Success => c"Success".as_ptr() as *const c_char,
        Dltbc2ErrorCode::InvalidLength => {
            c"Invalid input length: Length must be divisible by 16 (BC2 block size)".as_ptr()
                as *const c_char
        }
        Dltbc2ErrorCode::OutputBufferTooSmall => {
            c"Output buffer too small for the operation".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::AllocationFailed => c"Memory allocation failed".as_ptr() as *const c_char,
        Dltbc2ErrorCode::SizeEstimationFailed => {
            c"Size estimation failed during transform optimization".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::NullDataPointer => {
            c"Null pointer provided for data parameter".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::NullEstimatorPointer => {
            c"Null pointer provided for DltSizeEstimator parameter".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::NullTransformSettingsPointer => {
            c"Null pointer provided for Dltbc2TransformSettings parameter".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::NullInputPointer => {
            c"Null pointer provided for input parameter".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::NullOutputBufferPointer => {
            c"Null pointer provided for output parameter".as_ptr() as *const c_char
        }
        Dltbc2ErrorCode::NullManualTransformBuilderPointer => {
            c"Null pointer provided for Dltbc2ManualTransformBuilder parameter".as_ptr()
                as *const c_char
        }
        Dltbc2ErrorCode::NullBuilderPointer => {
            c"Null pointer provided for Dltbc2EstimateSettingsBuilder parameter".as_ptr()
                as *const c_char
        }
        Dltbc2ErrorCode::NullManualBuilderOutputPointer => {
            c"Null pointer provided for manual builder output parameter".as_ptr() as *const c_char
        }
    }
}
