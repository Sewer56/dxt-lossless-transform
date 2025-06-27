//! BC1 transform operations with explicit settings for C API.
//!
//! This module provides C-compatible FFI functions for transforming and
//! untransforming BC1 data using specific transform settings.

use super::transform_auto::{Dltbc1ErrorCode, Dltbc1Result, Dltbc1TransformSettings};
use crate::{
    transform_bc1_with_settings_safe, untransform_bc1_with_settings_safe, Bc1ValidationError,
    YCoCgVariant,
};
use core::slice;

/// Detransform settings for BC1 data.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc1DetransformSettings {
    /// Whether colour endpoints were split during transform
    pub split_colour_endpoints: bool,
    /// Decorrelation mode used during transform
    pub decorrelation_mode: YCoCgVariant,
}

impl From<Dltbc1TransformSettings> for crate::Bc1TransformSettings {
    fn from(settings: Dltbc1TransformSettings) -> Self {
        Self {
            split_colour_endpoints: settings.split_colour_endpoints,
            decorrelation_mode: settings.decorrelation_mode,
        }
    }
}

impl From<Dltbc1DetransformSettings> for crate::Bc1DetransformSettings {
    fn from(settings: Dltbc1DetransformSettings) -> Self {
        crate::Bc1TransformSettings {
            split_colour_endpoints: settings.split_colour_endpoints,
            decorrelation_mode: settings.decorrelation_mode,
        }
    }
}

impl From<Bc1ValidationError> for Dltbc1Result {
    fn from(error: Bc1ValidationError) -> Self {
        let error_code = match error {
            Bc1ValidationError::InvalidLength(_) => Dltbc1ErrorCode::InvalidDataLength,
            Bc1ValidationError::OutputBufferTooSmall { .. } => {
                Dltbc1ErrorCode::OutputBufferTooSmall
            }
        };
        Self::from_error_code(error_code)
    }
}

// =============================================================================
// C API Functions
// =============================================================================

/// Transform BC1 data using specified transform settings.
///
/// # Parameters
/// - `input`: Pointer to BC1 data to transform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `details`: The transform settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1core_transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    details: Dltbc1TransformSettings,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Convert FFI details to internal settings
    let settings = details.into();

    // Perform the transformation using core crate's safe function
    match transform_bc1_with_settings_safe(input_slice, output_slice, settings) {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => e.into(),
    }
}

/// Untransform BC1 data using specified detransform settings.
///
/// # Parameters
/// - `input`: Pointer to transformed BC1 data to untransform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where original BC1 data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `details`: The detransform settings to use (must match original transform settings)
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - The detransform settings must match the settings used for the original transformation
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1core_untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    details: Dltbc1DetransformSettings,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Convert FFI details to internal settings
    let settings = details.into();

    // Perform the untransformation using core crate's safe function
    match untransform_bc1_with_settings_safe(input_slice, output_slice, settings) {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use std::ptr;

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
    fn test_dltbc1core_transform_basic() {
        let test_data = create_test_bc1_data();
        let mut output = vec![0u8; test_data.len()];
        let details = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc1core_transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::Success);
            assert!(result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_null_input() {
        let mut output = vec![0u8; 16];
        let details = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result =
                dltbc1core_transform(ptr::null(), 16, output.as_mut_ptr(), output.len(), details);

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullDataPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_null_output() {
        let test_data = create_test_bc1_data();
        let details = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc1core_transform(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                16,
                details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_invalid_length() {
        let test_data = [0u8; 15]; // Not divisible by 8
        let mut output = vec![0u8; 15];
        let details = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc1core_transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidDataLength);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_transform_output_too_small() {
        let test_data = create_test_bc1_data();
        let mut output = vec![0u8; test_data.len() - 1]; // Too small
        let details = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc1core_transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::OutputBufferTooSmall);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_untransform_basic() {
        let test_data = create_test_bc1_data();
        let mut transformed = vec![0u8; test_data.len()];
        let mut restored = vec![0u8; test_data.len()];
        let details = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };
        let detransform_details = Dltbc1DetransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            // Transform first
            let transform_result = dltbc1core_transform(
                test_data.as_ptr(),
                test_data.len(),
                transformed.as_mut_ptr(),
                transformed.len(),
                details,
            );
            assert_eq!(transform_result.error_code, Dltbc1ErrorCode::Success);

            // Then untransform
            let untransform_result = dltbc1core_untransform(
                transformed.as_ptr(),
                transformed.len(),
                restored.as_mut_ptr(),
                restored.len(),
                detransform_details,
            );

            assert_eq!(untransform_result.error_code, Dltbc1ErrorCode::Success);
            assert!(untransform_result.is_success());

            // Should restore original data
            assert_eq!(restored, test_data);
        }
    }

    #[test]
    fn test_dltbc1core_untransform_null_input() {
        let mut output = vec![0u8; 16];
        let details = Dltbc1DetransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result =
                dltbc1core_untransform(ptr::null(), 16, output.as_mut_ptr(), output.len(), details);

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullDataPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_untransform_null_output() {
        let test_data = create_test_bc1_data();
        let details = Dltbc1DetransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::None,
        };

        unsafe {
            let result = dltbc1core_untransform(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                16,
                details,
            );

            assert_eq!(result.error_code, Dltbc1ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc1core_round_trip_with_different_settings() {
        let test_data = create_test_bc1_data();

        // Test different decorrelation modes
        for decorr_mode in [
            YCoCgVariant::None,
            YCoCgVariant::Variant1,
            YCoCgVariant::Variant2,
            YCoCgVariant::Variant3,
        ] {
            for split_colours in [false, true] {
                let transform_settings = Dltbc1TransformSettings {
                    split_colour_endpoints: split_colours,
                    decorrelation_mode: decorr_mode,
                };
                let detransform_settings = Dltbc1DetransformSettings {
                    split_colour_endpoints: split_colours,
                    decorrelation_mode: decorr_mode,
                };

                let mut transformed = vec![0u8; test_data.len()];
                let mut restored = vec![0u8; test_data.len()];

                unsafe {
                    // Transform
                    let transform_result = dltbc1core_transform(
                        test_data.as_ptr(),
                        test_data.len(),
                        transformed.as_mut_ptr(),
                        transformed.len(),
                        transform_settings,
                    );
                    assert_eq!(
                        transform_result.error_code,
                        Dltbc1ErrorCode::Success,
                        "Transform failed for decorr_mode {decorr_mode:?}, split_colours {split_colours}"
                    );

                    // Untransform
                    let untransform_result = dltbc1core_untransform(
                        transformed.as_ptr(),
                        transformed.len(),
                        restored.as_mut_ptr(),
                        restored.len(),
                        detransform_settings,
                    );
                    assert_eq!(
                        untransform_result.error_code,
                        Dltbc1ErrorCode::Success,
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
    fn test_dltbc1_detransform_settings_conversion() {
        let settings = Dltbc1DetransformSettings {
            split_colour_endpoints: true,
            decorrelation_mode: YCoCgVariant::Variant2,
        };

        let rust_settings: crate::Bc1DetransformSettings = settings.into();
        assert!(rust_settings.split_colour_endpoints);
        assert_eq!(rust_settings.decorrelation_mode, YCoCgVariant::Variant2);
    }

    #[test]
    fn test_transform_settings_conversion() {
        let settings = Dltbc1TransformSettings {
            split_colour_endpoints: false,
            decorrelation_mode: YCoCgVariant::Variant3,
        };

        let rust_settings: crate::Bc1TransformSettings = settings.into();
        assert_eq!(rust_settings.decorrelation_mode, YCoCgVariant::Variant3);
    }

    #[test]
    fn test_validation_error_conversion() {
        let validation_error = Bc1ValidationError::InvalidLength(24);
        let result: Dltbc1Result = validation_error.into();
        assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidDataLength);
        assert!(!result.is_success());
    }
}
