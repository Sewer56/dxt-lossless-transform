//! ABI-unstable C API functions for transforming BC1 data with automatic optimization.
//!
//! ## Warning: ABI Instability
//!
//! Functions in this module are prefixed with `dltbc1_unstable_` and are **NOT ABI-stable**.
//! The parameter structures may change between versions, potentially breaking compatibility.
//!

use crate::Bc1Error;
use crate::c_api::Dltbc1TransformDetails;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::transform::transform_bc1_auto;
use core::slice;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;

/// Settings for automatic BC1 transform configuration (ABI-unstable).
///
/// This struct contains all settings that affect how the optimal transform
/// is determined and applied. Using a struct allows adding new fields without breaking
/// the function signature, though the struct layout itself is still ABI-unstable.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc1AutoTransformSettings {
    /// If true, tests all decorrelation modes; if false, only tests Variant1 and None
    pub use_all_modes: bool,
}

/// Transform BC1 data using automatically determined optimal settings (ABI-unstable).
///
/// ## ABI Instability Warning
/// This function accepts and returns ABI-unstable structures which may change between versions.
/// Use [`dltbc1_EstimateOptionsBuilder_BuildAndTransform`] for ABI stability.
///
/// # Parameters
/// - `data`: Pointer to BC1 data to transform
/// - `data_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `data_len`)
/// - `estimator`: The size estimator to use for compression estimation
/// - `settings`: Settings controlling the optimization process
/// - `out_details`: Pointer where transform details will be written on success
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `out_details` must be a valid pointer for writing [`Dltbc1TransformDetails`]
/// - The estimator's context and functions must remain valid for the duration of the call
///
/// [`dltbc1_EstimateOptionsBuilder_BuildAndTransform`]: crate::c_api::auto_transform::dltbc1_EstimateOptionsBuilder_BuildAndTransform
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_transform_auto(
    data: *const u8,
    data_len: usize,
    output: *mut u8,
    output_len: usize,
    estimator: *const DltSizeEstimator,
    settings: Dltbc1AutoTransformSettings,
    out_details: *mut Dltbc1TransformDetails,
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
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformDetailsPointer);
    }

    // Create slices from raw pointers
    let data_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Use the provided estimator
    let estimator_ref = unsafe { &*estimator };

    // Create options struct
    let options = dxt_lossless_transform_bc1::determine_optimal_transform::Bc1EstimateOptions {
        size_estimator: estimator_ref,
        use_all_decorrelation_modes: settings.use_all_modes,
    };

    // Transform with automatic optimization
    match transform_bc1_auto(data_slice, output_slice, options) {
        Ok(transform_details) => {
            // Write the transform details to the output pointer
            unsafe {
                *out_details = transform_details.into();
            }
            Dltbc1Result::success()
        }
        Err(e) => {
            // Map the error to error codes
            match e {
                Bc1Error::SizeEstimationFailed(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::SizeEstimationFailed)
                }
                Bc1Error::InvalidLength(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength)
                }
                Bc1Error::OutputBufferTooSmall { .. } => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::OutputBufferTooSmall)
                }
                Bc1Error::AllocationFailed(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::AllocationFailed)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::c_void;
    use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;

    /// Create a test size estimator that returns predictable results
    fn create_test_estimator() -> DltSizeEstimator {
        // Simple estimator functions that return predictable results
        unsafe extern "C" fn test_max_compressed_size(
            _context: *mut c_void,
            _len_bytes: usize,
            out_size: *mut usize,
        ) -> u32 {
            unsafe {
                *out_size = 0;
            } // No buffer needed
            0 // Success
        }

        unsafe extern "C" fn test_estimate_compressed_size(
            _context: *mut c_void,
            _input_ptr: *const u8,
            len_bytes: usize,
            _data_type: u8,
            _output_ptr: *mut u8,
            _output_len: usize,
            out_size: *mut usize,
        ) -> u32 {
            unsafe {
                *out_size = len_bytes / 2;
            } // Return half the input size as estimate
            0 // Success
        }

        DltSizeEstimator {
            context: core::ptr::null_mut(),
            max_compressed_size: test_max_compressed_size,
            estimate_compressed_size: test_estimate_compressed_size,
            supports_data_type_differentiation: false,
        }
    }

    /// Create a failing size estimator for error testing
    fn create_failing_estimator() -> DltSizeEstimator {
        unsafe extern "C" fn failing_max_compressed_size(
            _context: *mut c_void,
            _len_bytes: usize,
            _out_size: *mut usize,
        ) -> u32 {
            1 // Return error code
        }

        unsafe extern "C" fn failing_estimate_compressed_size(
            _context: *mut c_void,
            _input_ptr: *const u8,
            _len_bytes: usize,
            _data_type: u8,
            _output_ptr: *mut u8,
            _output_len: usize,
            _out_size: *mut usize,
        ) -> u32 {
            1 // Return error code
        }

        DltSizeEstimator {
            context: core::ptr::null_mut(),
            max_compressed_size: failing_max_compressed_size,
            estimate_compressed_size: failing_estimate_compressed_size,
            supports_data_type_differentiation: false,
        }
    }

    #[test]
    fn test_settings_default() {
        let settings = Dltbc1AutoTransformSettings::default();
        assert!(!settings.use_all_modes);
    }

    #[test]
    fn test_settings_construction() {
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: true,
        };
        assert!(settings.use_all_modes);

        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };
        assert!(!settings.use_all_modes);
    }

    #[test]
    fn test_unstable_transform_auto_success() {
        // Create test data (8 bytes = 1 BC1 block)
        let test_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut output = [0u8; 8];
        let mut details = Dltbc1TransformDetails::default();
        let estimator = create_test_estimator();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: false,
        };

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut details,
            );
            assert!(result.is_success());
        }
    }

    #[test]
    fn test_unstable_null_data_pointer() {
        let mut output = [0u8; 8];
        let mut details = Dltbc1TransformDetails::default();
        let estimator = create_test_estimator();
        let settings = Dltbc1AutoTransformSettings::default();

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                core::ptr::null(),
                8,
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut details,
            );
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::NullDataPointer);
        }
    }

    #[test]
    fn test_unstable_null_output_pointer() {
        let test_data = [0u8; 8];
        let mut details = Dltbc1TransformDetails::default();
        let estimator = create_test_estimator();
        let settings = Dltbc1AutoTransformSettings::default();

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                core::ptr::null_mut(),
                8,
                &estimator,
                settings,
                &mut details,
            );
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::NullOutputBufferPointer);
        }
    }

    #[test]
    fn test_unstable_null_estimator_pointer() {
        let test_data = [0u8; 8];
        let mut output = [0u8; 8];
        let mut details = Dltbc1TransformDetails::default();
        let settings = Dltbc1AutoTransformSettings::default();

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                core::ptr::null(),
                settings,
                &mut details,
            );
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::NullEstimatorPointer);
        }
    }

    #[test]
    fn test_unstable_null_details_pointer() {
        let test_data = [0u8; 8];
        let mut output = [0u8; 8];
        let estimator = create_test_estimator();
        let settings = Dltbc1AutoTransformSettings::default();

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                core::ptr::null_mut(),
            );
            assert!(!result.is_success());
            assert_eq!(
                result.error_code,
                Dltbc1ErrorCode::NullTransformDetailsPointer
            );
        }
    }

    #[test]
    fn test_unstable_invalid_data_length() {
        // Use 7 bytes (not divisible by 8)
        let test_data = [0u8; 7];
        let mut output = [0u8; 8];
        let mut details = Dltbc1TransformDetails::default();
        let estimator = create_test_estimator();
        let settings = Dltbc1AutoTransformSettings::default();

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut details,
            );
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidLength);
        }
    }

    #[test]
    fn test_unstable_size_estimation_failure() {
        let test_data = [0u8; 8];
        let mut output = [0u8; 8];
        let mut details = Dltbc1TransformDetails::default();
        let estimator = create_failing_estimator();
        let settings = Dltbc1AutoTransformSettings::default();

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut details,
            );
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::SizeEstimationFailed);
        }
    }

    #[test]
    fn test_unstable_multiple_blocks() {
        // Test with 2 BC1 blocks (16 bytes)
        let test_data = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66,
            0x77, 0x88,
        ];
        let mut output = [0u8; 16];
        let mut details = Dltbc1TransformDetails::default();
        let estimator = create_test_estimator();
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: true,
        };

        unsafe {
            let result = dltbc1_unstable_transform_auto(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut details,
            );
            assert!(result.is_success());
        }
    }

    /// Test that matches a typical C usage pattern
    #[test]
    fn test_c_example_unstable_transform_auto() {
        // Your BC1 texture data (8 bytes per BC1 block)
        let bc1_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut output = [0u8; 8];
        let mut optimal_details = Dltbc1TransformDetails::default();

        // Configure settings
        let settings = Dltbc1AutoTransformSettings {
            use_all_modes: true, // Test all decorrelation modes for best results
        };

        // Create estimator
        let estimator = create_test_estimator();

        // Transform with automatic optimization
        unsafe {
            let result = dltbc1_unstable_transform_auto(
                bc1_data.as_ptr(),
                bc1_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                settings,
                &mut optimal_details,
            );

            if result.is_success() {
                println!("Transform completed successfully!");
                // optimal_details now contains the transform details
            } else {
                panic!("Failed to transform data: {:?}", result.error_code);
            }
        }
    }
}
