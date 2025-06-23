//! BC1 automatic transform operations for C API.
//!
//! This module provides C-compatible FFI functions for transforming BC1 data
//! using automatically determined optimal settings.

use crate::{transform_bc1_auto_safe, Bc1AutoTransformError, Bc1EstimateSettings, YCoCgVariant};
use core::slice;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;

/// Settings for automatic BC1 transform configuration.
///
/// This struct contains all settings that affect how the optimal transform
/// is determined and applied. Using a struct allows adding new fields without breaking
/// the function signature.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc1AutoTransformSettings {
    /// If true, tests all decorrelation modes; if false, only tests Variant1 and None
    ///
    /// Note: The typical improvement from testing all decorrelation modes is <0.1% in practice.
    /// For better compression gains, consider using a compression level on the estimator
    /// (e.g., ZStandard estimator) closer to your final compression level instead.
    pub use_all_modes: bool,
}

/// Transform settings returned by automatic optimization.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc1TransformSettings {
    /// Whether to split colour endpoints
    pub split_colour_endpoints: bool,
    /// Decorrelation mode to use
    pub decorrelation_mode: u8, // Maps to YCoCgVariant
}

/// Error codes for BC1 operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dltbc1ErrorCode {
    /// Operation completed successfully
    Success = 0,
    /// Data pointer is null
    NullDataPointer = 1,
    /// Output buffer pointer is null
    NullOutputBufferPointer = 2,
    /// Size estimator pointer is null
    NullEstimatorPointer = 3,
    /// Transform settings pointer is null
    NullTransformSettingsPointer = 4,
    /// Data length is not divisible by block size
    InvalidDataLength = 5,
    /// Output buffer is too small
    OutputBufferTooSmall = 6,
    /// Size estimation failed
    SizeEstimationError = 7,
    /// Internal transformation error
    TransformationError = 8,
}

/// Result type for BC1 C API operations.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc1Result {
    /// Error code (0 = success)
    pub error_code: Dltbc1ErrorCode,
}

impl Dltbc1Result {
    /// Create a success result
    pub fn success() -> Self {
        Self {
            error_code: Dltbc1ErrorCode::Success,
        }
    }

    /// Create an error result from an error code
    pub fn from_error_code(error_code: Dltbc1ErrorCode) -> Self {
        Self { error_code }
    }

    /// Check if the result is successful
    pub fn is_success(&self) -> bool {
        matches!(self.error_code, Dltbc1ErrorCode::Success)
    }
}

impl<T: core::fmt::Debug> From<Bc1AutoTransformError<T>> for Dltbc1Result {
    fn from(error: Bc1AutoTransformError<T>) -> Self {
        let error_code = match error {
            Bc1AutoTransformError::InvalidLength(_) => Dltbc1ErrorCode::InvalidDataLength,
            Bc1AutoTransformError::OutputBufferTooSmall { .. } => {
                Dltbc1ErrorCode::OutputBufferTooSmall
            }
            Bc1AutoTransformError::DetermineBestTransform(_) => {
                Dltbc1ErrorCode::SizeEstimationError
            }
        };
        Self::from_error_code(error_code)
    }
}

impl From<crate::Bc1TransformSettings> for Dltbc1TransformSettings {
    fn from(settings: crate::Bc1TransformSettings) -> Self {
        Self {
            split_colour_endpoints: settings.split_colour_endpoints,
            decorrelation_mode: match settings.decorrelation_mode {
                YCoCgVariant::None => 0,
                YCoCgVariant::Variant1 => 1,
                YCoCgVariant::Variant2 => 2,
                YCoCgVariant::Variant3 => 3,
            },
        }
    }
}

// =============================================================================
// C API Functions
// =============================================================================

/// Transform BC1 data using automatically determined optimal settings.
///
/// This function provides maximum performance by accepting structs directly
/// for scenarios where the caller can work with the core API.
///
/// # Parameters
/// - `data`: Pointer to BC1 data to transform
/// - `data_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `data_len`)
/// - `estimator`: The size estimator to use for finding the best possible transform.
///   This will test different transform configurations and choose the one that results
///   in the smallest estimated compressed size according to this estimator.
/// - `settings`: Settings controlling the optimization process
/// - `out_details`: Pointer where transform details will be written.
///   On success, this will be set to the transform settings used.
///   On error, the value is undefined.
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `out_details` must be a valid pointer for writing [`Dltbc1TransformSettings`]
/// - The estimator's context and functions must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1core_transform_auto(
    data: *const u8,
    data_len: usize,
    output: *mut u8,
    output_len: usize,
    estimator: *const DltSizeEstimator,
    settings: Dltbc1AutoTransformSettings,
    out_details: *mut Dltbc1TransformSettings,
) -> Dltbc1Result {
    // Validate pointers
    if data.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if estimator.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullEstimatorPointer);
    }
    if out_details.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformSettingsPointer);
    }

    // Create slices from raw pointers
    let data_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Use the provided estimator
    let estimator_ref = unsafe { &*estimator };

    // Create options struct
    let options = Bc1EstimateSettings {
        size_estimator: estimator_ref,
        use_all_decorrelation_modes: settings.use_all_modes,
    };

    // Transform with automatic optimization using core crate's safe function
    match transform_bc1_auto_safe(data_slice, output_slice, options) {
        Ok(transform_details) => {
            // Write the transform details to the output pointer
            unsafe {
                *out_details = transform_details.into();
            }
            Dltbc1Result::success()
        }
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
    use std::{ffi::c_void, ptr};

    /// Test helper: Create a dummy size estimator for testing
    fn create_dummy_estimator() -> DltSizeEstimator {
        unsafe extern "C" fn dummy_max_compressed_size(
            _context: *mut c_void,
            len_bytes: usize,
            out_size: *mut usize,
        ) -> u32 {
            unsafe {
                *out_size = len_bytes; // Just return input size
            }
            0 // Success
        }

        unsafe extern "C" fn dummy_estimate_compressed_size(
            _context: *mut c_void,
            _input_ptr: *const u8,
            len_bytes: usize,
            _data_type: u8,
            _output_ptr: *mut u8,
            _output_len: usize,
            out_size: *mut usize,
        ) -> u32 {
            unsafe {
                *out_size = len_bytes; // Just return input size
            }
            0 // Success
        }

        DltSizeEstimator {
            context: ptr::null_mut(),
            max_compressed_size: dummy_max_compressed_size,
            estimate_compressed_size: dummy_estimate_compressed_size,
            supports_data_type_differentiation: false,
        }
    }

    /// Helper function to create sample BC1 test data (2 blocks = 16 bytes)
    fn create_test_bc1_data() -> Vec<u8> {
        vec![
            // Block 1: 8 bytes
            0x00, 0x01, 0x02, 0x03, // colors
            0x80, 0x81, 0x82, 0x83, // indices
            // Block 2: 8 bytes
            0x04, 0x05, 0x06, 0x07, // colors
            0x84, 0x85, 0x86, 0x87, // indices
        ]
    }

    #[test]
    fn test_dltbc1core_transform_auto_basic() {
        let estimator = create_dummy_estimator();
        let test_data = create_test_bc1_data();
        let mut output = vec![0u8; test_data.len()];
        let mut out_details = Dltbc1TransformSettings::default();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut out_details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::Success);
            assert!(result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_null_data() {
        let estimator = create_dummy_estimator();
        let mut output = vec![0u8; 16];
        let mut out_details = Dltbc1TransformSettings::default();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                ptr::null(),
                16,
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut out_details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullDataPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_null_output() {
        let estimator = create_dummy_estimator();
        let test_data = create_test_bc1_data();
        let mut out_details = Dltbc1TransformSettings::default();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                16,
                &estimator,
                settings,
                &mut out_details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_null_estimator() {
        let test_data = create_test_bc1_data();
        let mut output = vec![0u8; test_data.len()];
        let mut out_details = Dltbc1TransformSettings::default();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                ptr::null(),
                settings,
                &mut out_details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullEstimatorPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_null_out_details() {
        let estimator = create_dummy_estimator();
        let test_data = create_test_bc1_data();
        let mut output = vec![0u8; test_data.len()];
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                ptr::null_mut(),
            );

            assert_eq!(
                result.error_code,
                Dltbc1ErrorCode::NullTransformSettingsPointer
            );
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_invalid_length() {
        let estimator = create_dummy_estimator();
        let test_data = [0u8; 15]; // Not divisible by 8
        let mut output = vec![0u8; 15];
        let mut out_details = Dltbc1TransformSettings::default();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut out_details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidDataLength);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_output_too_small() {
        let estimator = create_dummy_estimator();
        let test_data = create_test_bc1_data();
        let mut output = vec![0u8; test_data.len() - 1]; // Too small
        let mut out_details = Dltbc1TransformSettings::default();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1core_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut out_details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::OutputBufferTooSmall);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_auto_different_modes() {
        let estimator = create_dummy_estimator();
        let test_data = create_test_bc1_data();

        // Test with use_all_modes = false
        {
            let mut output = vec![0u8; test_data.len()];
            let mut out_details = Dltbc1TransformSettings::default();
            let settings = Dltbc1AutoTransformSettings {
                use_all_modes: false,
            };

            unsafe {
                let result = dltbc1core_transform_auto(
                    test_data.as_ptr(),
                    test_data.len(),
                    output.as_mut_ptr(),
                    output.len(),
                    &estimator,
                    settings,
                    &mut out_details,
                );

                assert_eq!(result.error_code, Dltbc1ErrorCode::Success);
                assert!(result.is_success());
            }
        }

        // Test with use_all_modes = true
        {
            let mut output = vec![0u8; test_data.len()];
            let mut out_details = Dltbc1TransformSettings::default();
            let settings = Dltbc1AutoTransformSettings {
                use_all_modes: true,
            };

            unsafe {
                let result = dltbc1core_transform_auto(
                    test_data.as_ptr(),
                    test_data.len(),
                    output.as_mut_ptr(),
                    output.len(),
                    &estimator,
                    settings,
                    &mut out_details,
                );

                assert_eq!(result.error_code, Dltbc1ErrorCode::Success);
                assert!(result.is_success());
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
        let result = Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidDataLength);
        assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidDataLength);
        assert!(!result.is_success());
    }

    #[test]
    fn test_dltbc1_transform_settings_conversion() {
        let settings = Dltbc1TransformSettings {
            split_colour_endpoints: true,
            decorrelation_mode: 1, // Variant1
        };

        let rust_settings: crate::Bc1TransformSettings = settings.into();
        assert!(rust_settings.split_colour_endpoints);
        assert_eq!(rust_settings.decorrelation_mode, YCoCgVariant::Variant1);
    }

    #[test]
    fn test_dltbc1_transform_settings_from_rust() {
        let rust_settings = crate::Bc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::Variant2,
        };

        let c_settings: Dltbc1TransformSettings = rust_settings.into();
        assert!(!c_settings.split_colour_endpoints);
        assert_eq!(c_settings.decorrelation_mode, 2); // Variant2
    }
}
