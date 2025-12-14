//! BC2 transform operations with explicit settings for C API.
//!
//! This module provides C-compatible FFI functions for transforming and
//! untransforming BC2 data using specific transform settings.

use super::transform_auto::{Dltbc2ErrorCode, Dltbc2Result, Dltbc2TransformSettings};
use crate::{
    transform_bc2_with_settings_safe, untransform_bc2_with_settings_safe, Bc2ValidationError,
};
use core::slice;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Untransform settings for BC2 data.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc2UntransformSettings {
    /// Whether colour endpoints were split during transform
    pub split_colour_endpoints: bool,
    /// Decorrelation mode used during transform
    pub decorrelation_mode: YCoCgVariant,
}

impl From<Dltbc2UntransformSettings> for crate::Bc2UntransformSettings {
    fn from(settings: Dltbc2UntransformSettings) -> Self {
        crate::Bc2TransformSettings {
            split_colour_endpoints: settings.split_colour_endpoints,
            decorrelation_mode: settings.decorrelation_mode,
        }
    }
}

impl From<Bc2ValidationError> for Dltbc2Result {
    fn from(error: Bc2ValidationError) -> Self {
        let error_code = match error {
            Bc2ValidationError::InvalidLength(_) => Dltbc2ErrorCode::InvalidDataLength,
            Bc2ValidationError::OutputBufferTooSmall { .. } => {
                Dltbc2ErrorCode::OutputBufferTooSmall
            }
        };
        Self::from_error_code(error_code)
    }
}

// =============================================================================
// C API Functions
// =============================================================================

/// Transform BC2 data using specified transform settings.
///
/// # Parameters
/// - `input`: Pointer to BC2 data to transform
/// - `input_len`: Length of input data in bytes (must be divisible by 16)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `details`: The transform settings to use
///
/// # Returns
/// A [`Dltbc2Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2core_transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    details: Dltbc2TransformSettings,
) -> Dltbc2Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullOutputBufferPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Convert FFI details to internal settings
    let settings = details.into();

    // Perform the transformation using core crate's safe function
    match transform_bc2_with_settings_safe(input_slice, output_slice, settings) {
        Ok(()) => Dltbc2Result::success(),
        Err(e) => e.into(),
    }
}

/// Untransform BC2 data using specified untransform settings.
///
/// # Parameters
/// - `input`: Pointer to transformed BC2 data to untransform
/// - `input_len`: Length of input data in bytes (must be divisible by 16)
/// - `output`: Pointer to output buffer where original BC2 data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `details`: The untransform settings to use (must match original transform settings)
///
/// # Returns
/// A [`Dltbc2Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - The untransform settings must match the settings used for the original transformation
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2core_untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    details: Dltbc2UntransformSettings,
) -> Dltbc2Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullOutputBufferPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Convert FFI details to internal settings
    let settings = details.into();

    // Perform the untransformation using core crate's safe function
    match untransform_bc2_with_settings_safe(input_slice, output_slice, settings) {
        Ok(()) => Dltbc2Result::success(),
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use std::ptr;

    /// Helper function to create sample BC2 test data (2 blocks = 32 bytes)
    fn create_test_bc2_data() -> Vec<u8> {
        vec![
            // Block 1: 16 bytes (8 bytes alpha + 8 bytes color)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, // alpha
            0x00, 0x01, 0x02, 0x03, // colors
            0x80, 0x81, 0x82, 0x83, // indices
            // Block 2: 16 bytes (8 bytes alpha + 8 bytes color)
            0x88, 0x99, 0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, // alpha
            0x04, 0x05, 0x06, 0x07, // colors
            0x84, 0x85, 0x86, 0x87, // indices
        ]
    }

    #[test]
    fn test_dltbc2core_transform_basic() {
        let test_data = create_test_bc2_data();
        let mut output = vec![0u8; test_data.len()];
        let details = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc2core_transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                details,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::Success);
            assert!(result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_transform_null_input() {
        let mut output = vec![0u8; 32];
        let details = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result =
                dltbc2core_transform(ptr::null(), 32, output.as_mut_ptr(), output.len(), details);

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullDataPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_transform_null_output() {
        let test_data = create_test_bc2_data();
        let details = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc2core_transform(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                32,
                details,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_transform_invalid_length() {
        let test_data = [0u8; 15]; // Not divisible by 16
        let mut output = vec![0u8; 15];
        let details = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc2core_transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                details,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::InvalidDataLength);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_transform_output_too_small() {
        let test_data = create_test_bc2_data();
        let mut output = vec![0u8; test_data.len() - 1]; // Too small
        let details = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc2core_transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                details,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::OutputBufferTooSmall);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_untransform_basic() {
        let test_data = create_test_bc2_data();
        let mut transformed = vec![0u8; test_data.len()];
        let mut restored = vec![0u8; test_data.len()];
        let details = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };
        let untransform_details = Dltbc2UntransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            // Transform first
            let transform_result = dltbc2core_transform(
                test_data.as_ptr(),
                test_data.len(),
                transformed.as_mut_ptr(),
                transformed.len(),
                details,
            );
            assert_eq!(transform_result.error_code, Dltbc2ErrorCode::Success);

            // Then untransform
            let untransform_result = dltbc2core_untransform(
                transformed.as_ptr(),
                transformed.len(),
                restored.as_mut_ptr(),
                restored.len(),
                untransform_details,
            );

            assert_eq!(untransform_result.error_code, Dltbc2ErrorCode::Success);
            assert!(untransform_result.is_success());

            // Should restore original data
            assert_eq!(restored, test_data);
        }
    }

    #[test]
    fn test_dltbc2core_untransform_null_input() {
        let mut output = vec![0u8; 32];
        let details = Dltbc2UntransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result =
                dltbc2core_untransform(ptr::null(), 32, output.as_mut_ptr(), output.len(), details);

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullDataPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_untransform_null_output() {
        let test_data = create_test_bc2_data();
        let details = Dltbc2UntransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc2core_untransform(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                32,
                details,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2core_round_trip_with_different_settings() {
        let test_data = create_test_bc2_data();

        // Test different decorrelation modes
        for decorr_mode in [
            YCoCgVariant::None,
            YCoCgVariant::Variant1,
            YCoCgVariant::Variant2,
            YCoCgVariant::Variant3,
        ] {
            for split_colours in [false, true] {
                let transform_settings = Dltbc2TransformSettings {
                    split_colour_endpoints: split_colours,
                    decorrelation_mode: decorr_mode,
                };
                let untransform_settings = Dltbc2UntransformSettings {
                    split_colour_endpoints: split_colours,
                    decorrelation_mode: decorr_mode,
                };

                let mut transformed = vec![0u8; test_data.len()];
                let mut restored = vec![0u8; test_data.len()];

                unsafe {
                    // Transform
                    let transform_result = dltbc2core_transform(
                        test_data.as_ptr(),
                        test_data.len(),
                        transformed.as_mut_ptr(),
                        transformed.len(),
                        transform_settings,
                    );
                    assert_eq!(
                        transform_result.error_code,
                        Dltbc2ErrorCode::Success,
                        "Transform failed for decorr_mode {decorr_mode:?}, split_colours {split_colours}"
                    );

                    // Untransform
                    let untransform_result = dltbc2core_untransform(
                        transformed.as_ptr(),
                        transformed.len(),
                        restored.as_mut_ptr(),
                        restored.len(),
                        untransform_settings,
                    );
                    assert_eq!(
                        untransform_result.error_code,
                        Dltbc2ErrorCode::Success,
                        "Untransform failed for decorr_mode {decorr_mode:?}, split_colours {split_colours}"
                    );

                    // Should restore original data
                    assert_eq!(
                        restored, test_data,
                        "Round-trip failed for decorr_mode {decorr_mode:?}, split_colours {split_colours}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_dltbc2_untransform_settings_conversion() {
        let settings = Dltbc2UntransformSettings {
            split_colour_endpoints: true,
            decorrelation_mode: YCoCgVariant::Variant2,
        };

        let rust_settings: crate::Bc2UntransformSettings = settings.into();
        assert!(rust_settings.split_colour_endpoints);
        assert_eq!(rust_settings.decorrelation_mode, YCoCgVariant::Variant2);
    }

    #[test]
    fn test_transform_settings_conversion() {
        let settings = Dltbc2TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::Variant3,
        };

        let rust_settings: crate::Bc2TransformSettings = settings.into();
        assert_eq!(rust_settings.decorrelation_mode, YCoCgVariant::Variant3);
    }

    #[test]
    fn test_validation_error_conversion() {
        let validation_error = Bc2ValidationError::InvalidLength(24);
        let result: Dltbc2Result = validation_error.into();
        assert_eq!(result.error_code, Dltbc2ErrorCode::InvalidDataLength);
        assert!(!result.is_success());
    }
}
