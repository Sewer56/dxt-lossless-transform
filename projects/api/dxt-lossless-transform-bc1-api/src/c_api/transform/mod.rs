//! C API for BC1 transform and untransform operations.
//!
//! This module provides ABI-stable transform functions using opaque context objects.
//!
//! [`Dltbc1Result`]: crate::c_api::error::Dltbc1Result
//! [`Dltbc1TransformContext`]: crate::c_api::transform::transform_context::Dltbc1TransformContext

pub mod builder;
pub mod transform_context;
pub mod unstable;

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform::transform_context::{
    Dltbc1TransformContext, get_detransform_details, get_transform_details,
};
use crate::c_api::transform::unstable::{dltbc1_unstable_transform, dltbc1_unstable_untransform};

/// Transform BC1 data using a pre-allocated buffer (ABI-stable).
///
/// This function provides ABI stability by using an opaque context object.
/// Internally, it extracts transform details from the context and calls
/// the unstable transform function.
///
/// # Parameters
/// - `input`: Pointer to input BC1 data
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer
/// - `output_len`: Length of output buffer (must be at least `input_len`)
/// - `context`: The BC1 context containing transform options
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `context` must be a valid pointer to a Dltbc1TransformContext
/// - Pointers must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_Transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    context: *const Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate context pointer
    if context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformContextPointer);
    }

    // Get transform options from context
    let details = unsafe { get_transform_details(context) };

    // Call the unstable transform function
    unsafe { dltbc1_unstable_transform(input, input_len, output, output_len, details.into()) }
}

/// Untransform BC1 data using a pre-allocated buffer (ABI-stable).
///
/// This function provides ABI stability by using an opaque context object.
/// Internally, it extracts detransform details from the context and calls
/// the unstable untransform function.
///
/// # Parameters
/// - `input`: Pointer to transformed BC1 data
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer
/// - `output_len`: Length of output buffer (must be at least `input_len`)
/// - `context`: The BC1 context containing detransform options (must match original transform)
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
/// - Pointers must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_Untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    context: *const Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate context pointer
    if context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformContextPointer);
    }

    // Get detransform options from context
    let details = unsafe { get_detransform_details(context) };

    // Call the unstable untransform function
    unsafe { dltbc1_unstable_untransform(input, input_len, output, output_len, details.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c_api::transform::builder::*;
    use crate::c_api::transform::transform_context::*;
    use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

    #[test]
    fn test_stable_transform_roundtrip() {
        // Create test data (8 bytes = 1 BC1 block)
        let input = vec![0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut transformed = vec![0u8; 8];
        let mut restored = vec![0u8; 8];

        // Create context
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        // Configure context
        unsafe {
            dltbc1_TransformContext_SetDecorrelationMode(context, YCoCgVariant::Variant1);
            dltbc1_TransformContext_SetSplitColourEndpoints(context, true);
        }

        // Transform
        unsafe {
            let result = dltbc1_TransformContext_Transform(
                input.as_ptr(),
                input.len(),
                transformed.as_mut_ptr(),
                transformed.len(),
                context,
            );
            assert!(result.is_success());
        }

        // Untransform
        unsafe {
            let result = dltbc1_TransformContext_Untransform(
                transformed.as_ptr(),
                transformed.len(),
                restored.as_mut_ptr(),
                restored.len(),
                context,
            );
            assert!(result.is_success());
        }

        // Verify roundtrip
        assert_eq!(input, restored);

        // Clean up
        unsafe {
            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_stable_null_context_handling() {
        let data = [0u8; 8];
        let mut output = vec![0u8; 8];

        // Test null context for transform
        unsafe {
            let result = dltbc1_TransformContext_Transform(
                data.as_ptr(),
                8,
                output.as_mut_ptr(),
                8,
                core::ptr::null(),
            );
            assert!(!result.is_success());
        }

        // Test null context for untransform
        unsafe {
            let result = dltbc1_TransformContext_Untransform(
                data.as_ptr(),
                8,
                output.as_mut_ptr(),
                8,
                core::ptr::null(),
            );
            assert!(!result.is_success());
        }
    }

    /// Test that matches the "Basic Transform Operation" C example
    #[test]
    fn test_c_example_basic_transform() {
        // Your BC1 texture data (8 bytes per BC1 block)
        let bc1_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut transformed_data = [0u8; 8];

        // Create and configure transform context
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        unsafe {
            dltbc1_TransformContext_SetDecorrelationMode(context, YCoCgVariant::Variant1);
            dltbc1_TransformContext_SetSplitColourEndpoints(context, true);
        }

        // Transform the data
        unsafe {
            let result = dltbc1_TransformContext_Transform(
                bc1_data.as_ptr(),
                bc1_data.len(),
                transformed_data.as_mut_ptr(),
                transformed_data.len(),
                context,
            );

            if result.is_success() {
                println!("Transform successful!");
                // Now compress 'transformed_data' with your compressor...
            } else {
                panic!("Transform failed with error code: {:?}", result.error_code);
            }
        }

        // Clean up
        unsafe {
            dltbc1_free_TransformContext(context);
        }
    }

    /// Test that matches the "Untransform Operation" C example
    #[test]
    fn test_c_example_untransform() {
        // Your transformed BC1 data (after decompression)
        let transformed_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut restored_data = [0u8; 8];

        // Create context with SAME settings used for original transform
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        unsafe {
            dltbc1_TransformContext_SetDecorrelationMode(context, YCoCgVariant::Variant1);
            dltbc1_TransformContext_SetSplitColourEndpoints(context, true);
        }

        // Restore original BC1 data
        unsafe {
            let result = dltbc1_TransformContext_Untransform(
                transformed_data.as_ptr(),
                transformed_data.len(),
                restored_data.as_mut_ptr(),
                restored_data.len(),
                context,
            );

            if result.is_success() {
                println!("Untransform successful!");
                // 'restored_data' now contains original BC1 data
            } else {
                panic!(
                    "Untransform failed with error code: {:?}",
                    result.error_code
                );
            }
        }

        // Clean up
        unsafe {
            dltbc1_free_TransformContext(context);
        }
    }
}
