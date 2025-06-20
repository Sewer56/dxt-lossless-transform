//! C API error handling for BC1 operations.

use crate::error::Bc1Error;
use core::ffi::c_char;

/// C-compatible error codes for BC1 operations.
#[repr(u8)]
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
            Err(e) => {
                let error_code = match e {
                    Bc1Error::InvalidLength(_) => Dltbc1ErrorCode::InvalidLength,
                    Bc1Error::OutputBufferTooSmall { .. } => Dltbc1ErrorCode::OutputBufferTooSmall,
                    Bc1Error::AllocationFailed(_) => Dltbc1ErrorCode::AllocationFailed,
                    Bc1Error::SizeEstimationFailed(_) => Dltbc1ErrorCode::SizeEstimationFailed,
                };
                Self::from_error_code(error_code)
            }
        }
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
    }
}
