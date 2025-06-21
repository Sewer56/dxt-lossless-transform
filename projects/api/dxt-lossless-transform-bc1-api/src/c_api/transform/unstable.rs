//! ABI-unstable C API functions for BC1 transform operations.
//!
//! ## Warning: ABI Instability
//!
//! Functions in this module are prefixed with `dltbc1_unstable_` and are **NOT ABI-stable**.
//! The parameter structures may change between versions, potentially breaking compatibility.
//!
//! These functions provide direct access to transform operations by accepting transform
//! details structures as parameters, avoiding the overhead of context objects.

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::{Dltbc1DetransformDetails, Dltbc1TransformDetails};
use crate::transform::{transform_bc1_slice, untransform_bc1_slice};
use core::slice;

/// Transform BC1 data using transform details directly (ABI-unstable).
///
/// ## ABI Instability Warning
/// This function accepts [`Dltbc1TransformDetails`] directly, which may change between versions.
/// Use [`dltbc1_TransformContext_Transform`] with a context for ABI stability.
///
/// # Parameters
/// - `input`: Pointer to input BC1 data
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer
/// - `output_len`: Length of output buffer (must be at least `input_len`)
/// - `details`: The transform details to apply
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - Pointers must remain valid for the duration of the call
///
/// [`dltbc1_TransformContext_Transform`]: crate::c_api::transform::dltbc1_TransformContext_Transform
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    details: Dltbc1TransformDetails,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullInputPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Perform transform
    let result = transform_bc1_slice(input_slice, output_slice, details.into());

    result.into()
}

/// Untransform BC1 data using detransform details directly (ABI-unstable).
///
/// ## ABI Instability Warning
/// This function accepts [`Dltbc1DetransformDetails`] directly, which may change between versions.
/// Use [`dltbc1_TransformContext_Untransform`] with a context for ABI stability.
///
/// # Parameters
/// - `input`: Pointer to transformed BC1 data
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer
/// - `output_len`: Length of output buffer (must be at least `input_len`)
/// - `details`: The detransform details (must match original transform)
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - Pointers must remain valid for the duration of the call
///
/// [`dltbc1_TransformContext_Untransform`]: crate::c_api::transform::dltbc1_TransformContext_Untransform
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    details: Dltbc1DetransformDetails,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullInputPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Perform untransform
    let result = untransform_bc1_slice(input_slice, output_slice, details.into());

    result.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

    #[test]
    fn test_unstable_transform_roundtrip() {
        // Create test data (8 bytes = 1 BC1 block)
        let input = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut transformed = vec![0u8; 8];
        let mut restored = vec![0u8; 8];

        // Test with default options
        let transform_details = Dltbc1TransformDetails {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        // Transform
        unsafe {
            let result = dltbc1_unstable_transform(
                input.as_ptr(),
                input.len(),
                transformed.as_mut_ptr(),
                transformed.len(),
                transform_details,
            );
            assert!(result.is_success());
        }

        // Untransform
        let detransform_details = Dltbc1DetransformDetails {
            decorrelation_mode: transform_details.decorrelation_mode,
            split_colour_endpoints: transform_details.split_colour_endpoints,
        };
        unsafe {
            let result = dltbc1_unstable_untransform(
                transformed.as_ptr(),
                transformed.len(),
                restored.as_mut_ptr(),
                restored.len(),
                detransform_details,
            );
            assert!(result.is_success());
        }

        // Verify roundtrip
        assert_eq!(input, restored);
    }

    #[test]
    fn test_unstable_null_pointer_handling() {
        let data = [0u8; 8];
        let mut output = vec![0u8; 8];
        let details = Dltbc1TransformDetails {
            decorrelation_mode: YCoCgVariant::None,
            split_colour_endpoints: false,
        };

        // Test null input
        unsafe {
            let result =
                dltbc1_unstable_transform(core::ptr::null(), 8, output.as_mut_ptr(), 8, details);
            assert!(!result.is_success());
        }

        // Test null output
        unsafe {
            let result =
                dltbc1_unstable_transform(data.as_ptr(), 8, core::ptr::null_mut(), 8, details);
            assert!(!result.is_success());
        }
    }
}
