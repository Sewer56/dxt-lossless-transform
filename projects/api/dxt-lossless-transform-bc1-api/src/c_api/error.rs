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
    /// Null pointer provided for Dltbc1ManualTransformBuilder parameter
    NullManualTransformBuilderPointer = 10,
    /// Null pointer provided for Dltbc1EstimateSettingsBuilder parameter
    NullBuilderPointer = 11,
    /// Null pointer provided for manual builder output parameter
    NullManualBuilderOutputPointer = 12,
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
        Dltbc1ErrorCode::NullManualTransformBuilderPointer => {
            c"Null pointer provided for Dltbc1ManualTransformBuilder parameter".as_ptr()
                as *const c_char
        }
        Dltbc1ErrorCode::NullBuilderPointer => {
            c"Null pointer provided for Dltbc1EstimateSettingsBuilder parameter".as_ptr()
                as *const c_char
        }
        Dltbc1ErrorCode::NullManualBuilderOutputPointer => {
            c"Null pointer provided for manual builder output parameter".as_ptr() as *const c_char
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::CStr;

    #[test]
    fn test_dltbc1_error_message_success() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::Success);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Success");
        }
    }

    #[test]
    fn test_dltbc1_error_message_invalid_length() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::InvalidLength);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Invalid input length: Length must be divisible by 8 (BC1 block size)"
            );
        }
    }

    #[test]
    fn test_dltbc1_error_message_output_buffer_too_small() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::OutputBufferTooSmall);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Output buffer too small for the operation");
        }
    }

    #[test]
    fn test_dltbc1_error_message_allocation_failed() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::AllocationFailed);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Memory allocation failed");
        }
    }

    #[test]
    fn test_dltbc1_error_message_size_estimation_failed() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::SizeEstimationFailed);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Size estimation failed during transform optimization"
            );
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_data_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullDataPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Null pointer provided for data parameter");
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_estimator_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullEstimatorPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Null pointer provided for DltSizeEstimator parameter"
            );
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_transform_settings_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullTransformSettingsPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Null pointer provided for Dltbc1TransformSettings parameter"
            );
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_input_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullInputPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Null pointer provided for input parameter");
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_output_buffer_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullOutputBufferPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(message, "Null pointer provided for output parameter");
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_manual_transform_builder_pointer() {
        unsafe {
            let message_ptr =
                dltbc1_error_message(Dltbc1ErrorCode::NullManualTransformBuilderPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Null pointer provided for Dltbc1ManualTransformBuilder parameter"
            );
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_builder_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullBuilderPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Null pointer provided for Dltbc1EstimateSettingsBuilder parameter"
            );
        }
    }

    #[test]
    fn test_dltbc1_error_message_null_manual_builder_output_pointer() {
        unsafe {
            let message_ptr = dltbc1_error_message(Dltbc1ErrorCode::NullManualBuilderOutputPointer);
            assert!(!message_ptr.is_null());

            let c_str = CStr::from_ptr(message_ptr);
            let message = c_str.to_str().unwrap();
            assert_eq!(
                message,
                "Null pointer provided for manual builder output parameter"
            );
        }
    }

    /// Test that all error message strings are null-terminated and valid UTF-8
    #[test]
    fn test_all_error_messages_are_valid() {
        let error_codes = [
            Dltbc1ErrorCode::Success,
            Dltbc1ErrorCode::InvalidLength,
            Dltbc1ErrorCode::OutputBufferTooSmall,
            Dltbc1ErrorCode::AllocationFailed,
            Dltbc1ErrorCode::SizeEstimationFailed,
            Dltbc1ErrorCode::NullDataPointer,
            Dltbc1ErrorCode::NullEstimatorPointer,
            Dltbc1ErrorCode::NullTransformSettingsPointer,
            Dltbc1ErrorCode::NullInputPointer,
            Dltbc1ErrorCode::NullOutputBufferPointer,
            Dltbc1ErrorCode::NullManualTransformBuilderPointer,
            Dltbc1ErrorCode::NullBuilderPointer,
            Dltbc1ErrorCode::NullManualBuilderOutputPointer,
        ];

        for &error_code in &error_codes {
            unsafe {
                let message_ptr = dltbc1_error_message(error_code);
                assert!(
                    !message_ptr.is_null(),
                    "Error message pointer is null for {error_code:?}"
                );

                let c_str = CStr::from_ptr(message_ptr);
                let message = c_str.to_str().unwrap_or_else(|_| {
                    panic!("Error message is not valid UTF-8 for {error_code:?}")
                });

                assert!(
                    !message.is_empty(),
                    "Error message is empty for {error_code:?}",
                );
            }
        }
    }

    #[test]
    fn test_dltbc1_result_success() {
        let result = Dltbc1Result::success();
        assert_eq!(result.error_code, Dltbc1ErrorCode::Success);
        assert!(result.is_success());
    }

    #[test]
    fn test_dltbc1_result_from_error_code() {
        let result = Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength);
        assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidLength);
        assert!(!result.is_success());
    }

    #[test]
    fn test_dltbc1_result_from_bc1_error() {
        let bc1_error: Bc1Error<&'static str> = Bc1Error::InvalidLength(24);
        let result: Dltbc1Result = bc1_error.into();
        assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidLength);
        assert!(!result.is_success());
    }

    #[test]
    fn test_dltbc1_result_from_validation_error() {
        let validation_error = Bc1ValidationError::InvalidLength(24);
        let result: Dltbc1Result = validation_error.into();
        assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidLength);
        assert!(!result.is_success());
    }
}
