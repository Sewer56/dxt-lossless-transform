//! BC2 manual transform builder for C API.
//!
//! This module provides ABI-stable functions for managing and configuring BC2 manual transform builder
//! through a builder pattern. The builder stores BC2 transform configuration and must be
//! explicitly freed. These functions offer a stable interface that is guaranteed
//! to remain compatible across library versions.
//!
//! For users requiring maximum performance and willing to accept potential breaking
//! changes, see the core crate functions directly.

use crate::{
    c_api::error::{Dltbc2ErrorCode, Dltbc2Result},
    transform::Bc2ManualTransformBuilder,
};
use alloc::boxed::Box;
use core::{ptr, slice};
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

// =============================================================================
// Type Definitions
// =============================================================================

/// Opaque manual transform builder type for BC2 transform operations.
///
/// This builder stores the current transform configuration and must be:
///
/// - Created with [`dltbc2_new_ManualTransformBuilder()`]
/// - Modified using the builder functions
/// - Passed to transform operations
/// - Freed with [`dltbc2_free_ManualTransformBuilder()`] when no longer needed
///
/// The builder is NOT thread-safe and should not be shared between threads.
/// Each thread should create its own builder.
///
/// # cbindgen Opaque Type Rule
/// Per cbindgen documentation (<https://github.com/mozilla/cbindgen/blob/master/docs.md>):
/// "If a type is determined to have a guaranteed layout, a full definition will be emitted in the header.
/// If the type doesn't have a guaranteed layout, only a forward declaration will be emitted. This may be
/// fine if the type is intended to be passed around opaquely and by reference."
///
/// This struct intentionally lacks `#[repr(C)]` to ensure it generates as an opaque forward declaration.
pub struct Dltbc2ManualTransformBuilder {
    pub(crate) builder: Bc2ManualTransformBuilder,
}

/// Get mutable access to the manual transform builder.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
pub(crate) unsafe fn get_manual_builder_mut(
    builder: *mut Dltbc2ManualTransformBuilder,
) -> &'static mut Dltbc2ManualTransformBuilder {
    debug_assert!(!builder.is_null());
    unsafe { &mut *builder }
}

// =============================================================================
// Lifecycle Functions
// =============================================================================

/// Create a new BC2 manual transform builder with default settings.
///
/// The returned builder must be freed with [`dltbc2_free_ManualTransformBuilder()`] when no longer needed.
///
/// # Returns
/// A pointer to a newly allocated manual transform builder, or null if allocation fails.
///
/// # Remarks
/// This function corresponds to [`crate::Bc2ManualTransformBuilder::new`] in the Rust API.
#[unsafe(no_mangle)]
pub extern "C" fn dltbc2_new_ManualTransformBuilder() -> *mut Dltbc2ManualTransformBuilder {
    let inner = Box::new(Dltbc2ManualTransformBuilder {
        builder: crate::transform::Bc2ManualTransformBuilder::new(),
    });

    Box::into_raw(inner)
}

/// Free a BC2 manual transform builder.
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc2_new_ManualTransformBuilder()`]
/// - `builder` must not have been freed already
/// - After calling this function, `builder` becomes invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_free_ManualTransformBuilder(
    builder: *mut Dltbc2ManualTransformBuilder,
) {
    if !builder.is_null() {
        unsafe {
            drop(Box::from_raw(builder));
        }
    }
}

/// Clone a BC2 manual transform builder.
///
/// Creates a new builder with the same settings as the source builder.
/// The returned builder must be freed independently.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
///
/// # Returns
/// A pointer to a newly allocated manual transform builder with the same settings, or null if allocation fails.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_clone_ManualTransformBuilder(
    builder: *const Dltbc2ManualTransformBuilder,
) -> *mut Dltbc2ManualTransformBuilder {
    if builder.is_null() {
        return ptr::null_mut();
    }

    let inner = unsafe { &*builder };
    let cloned = Box::new(Dltbc2ManualTransformBuilder {
        builder: inner.builder,
    });

    Box::into_raw(cloned)
}

// =============================================================================
// Configuration Functions
// =============================================================================

/// Set the decorrelation mode for the builder.
///
/// Controls the YCoCg-R color space decorrelation variant used for transformation.
/// Different variants can provide varying compression ratios depending on the texture content.
///
/// **Note**: When manually testing decorrelation modes, the typical improvement from
/// using different variants is <0.1% in practice. For better compression gains,
/// it's recommended to use a compression level on the estimator (e.g., ZStandard estimator)
/// closer to your final compression level instead.
///
/// For automatic optimization, consider using [`dltbc2_AutoTransformBuilder_Transform`] instead.
///
/// [`dltbc2_AutoTransformBuilder_Transform`]: crate::c_api::transform::auto_transform_builder::dltbc2_AutoTransformBuilder_Transform
///
/// # Parameters
/// - `builder`: The BC2 manual builder to modify
/// - `mode`: The decorrelation mode to use
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
///
/// # Remarks
/// This function corresponds to [`crate::Bc2ManualTransformBuilder::decorrelation_mode`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_ManualTransformBuilder_SetDecorrelationMode(
    builder: *mut Dltbc2ManualTransformBuilder,
    mode: YCoCgVariant,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_manual_builder_mut(builder) };
    inner.builder = inner.builder.decorrelation_mode(mode);
}

/// Set whether to split colour endpoints for the builder.
///
/// This setting controls whether BC2 texture color endpoints are separated during processing,
/// which can improve compression efficiency for many textures.
///
/// **File Size**: This setting reduces file size around 78% of the time.
///
/// For automatic optimization, consider using [`dltbc2_AutoTransformBuilder_Transform`] instead.
///
/// [`dltbc2_AutoTransformBuilder_Transform`]: crate::c_api::transform::auto_transform_builder::dltbc2_AutoTransformBuilder_Transform
///
/// # Parameters
/// - `builder`: The BC2 manual builder to modify
/// - `split`: Whether to split colour endpoints
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
///
/// # Remarks
/// This function corresponds to [`crate::Bc2ManualTransformBuilder::split_colour_endpoints`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(
    builder: *mut Dltbc2ManualTransformBuilder,
    split: bool,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_manual_builder_mut(builder) };
    inner.builder = inner.builder.split_colour_endpoints(split);
}

/// Reset the builder to default settings.
///
/// # Parameters
/// - `builder`: The BC2 manual builder to reset
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_ManualTransformBuilder_ResetToDefaults(
    builder: *mut Dltbc2ManualTransformBuilder,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_manual_builder_mut(builder) };
    inner.builder = crate::transform::Bc2ManualTransformBuilder::new();
}

// =============================================================================
// Transform Operations
// =============================================================================

/// Transform BC2 data using the settings configured in the builder.
///
/// This function applies the transformation directly using the settings stored in the
/// provided builder without any optimization or testing of different configurations.
///
/// # Parameters
/// - `input`: Pointer to the BC2 data to transform
/// - `input_len`: Length of input data in bytes (must be divisible by 16)
/// - `output`: Pointer to the output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `builder`: The manual transform builder containing the settings to use
///
/// # Returns
/// A [`Dltbc2Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
///
/// # Examples
///
/// ```c
/// // Create and configure manual transform builder
/// Dltbc2ManualTransformBuilder* builder = dltbc2_new_ManualTransformBuilder();
/// dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
/// dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(builder, true);
///
/// // Transform the data
/// Dltbc2Result result = dltbc2_ManualTransformBuilder_Transform(
///     bc2_data, sizeof(bc2_data),
///     transformed_data, sizeof(transformed_data),
///     builder);
/// ```
///
/// # Remarks
/// This function corresponds to [`crate::Bc2ManualTransformBuilder::transform`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_ManualTransformBuilder_Transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    builder: *mut Dltbc2ManualTransformBuilder,
) -> Dltbc2Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullOutputBufferPointer);
    }
    if builder.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullManualTransformBuilderPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the builder and perform transformation using its method
    let builder_inner = unsafe { get_manual_builder_mut(builder) };

    // Perform the transformation using the builder's method
    match builder_inner.builder.transform(input_slice, output_slice) {
        Ok(()) => Dltbc2Result::success(),
        Err(e) => e.into(),
    }
}

/// Untransform BC2 data using the settings configured in the builder.
///
/// This function reverses the transformation applied by [`dltbc2_ManualTransformBuilder_Transform`],
/// restoring the original BC2 data. The builder must contain the same settings that were
/// used for the original transformation.
///
/// # Parameters
/// - `input`: Pointer to the transformed BC2 data to untransform
/// - `input_len`: Length of input data in bytes (must be divisible by 16)
/// - `output`: Pointer to the output buffer where the original BC2 data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `builder`: The manual transform builder containing the untransform settings to use
///
/// # Returns
/// A [`Dltbc2Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `builder` must be a valid pointer to a [`Dltbc2ManualTransformBuilder`]
/// - The builder must contain the same settings used for the original transformation
///
/// # Examples
///
/// ```c
/// // Use the same builder that was used for transform
/// Dltbc2Result result = dltbc2_ManualTransformBuilder_Untransform(
///     transformed_data, sizeof(transformed_data),
///     restored_data, sizeof(restored_data),
///     builder);
/// ```
///
/// # Remarks
/// This function corresponds to [`crate::Bc2ManualTransformBuilder::untransform`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc2_ManualTransformBuilder_Untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    builder: *mut Dltbc2ManualTransformBuilder,
) -> Dltbc2Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullOutputBufferPointer);
    }
    if builder.is_null() {
        return Dltbc2Result::from_error_code(Dltbc2ErrorCode::NullManualTransformBuilderPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the builder and perform untransformation using its method
    let builder_inner = unsafe { get_manual_builder_mut(builder) };

    // Perform the untransformation using the builder's method
    match builder_inner.builder.untransform(input_slice, output_slice) {
        Ok(()) => Dltbc2Result::success(),
        Err(e) => e.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    /// Helper function to create sample BC2 test data (1 block = 16 bytes)
    fn create_test_bc2_data() -> Vec<u8> {
        vec![
            // Alpha data (8 bytes - 4-bit per pixel)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            // Color data (8 bytes - BC1-like)
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ]
    }

    #[test]
    fn test_dltbc2_new_manual_transform_builder() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        // Clean up
        unsafe {
            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_free_manual_transform_builder_null_pointer() {
        // Should not crash when freeing null pointer
        unsafe {
            dltbc2_free_ManualTransformBuilder(ptr::null_mut());
        }
    }

    #[test]
    fn test_dltbc2_clone_manual_transform_builder() {
        let original = dltbc2_new_ManualTransformBuilder();
        assert!(!original.is_null());

        unsafe {
            // Configure the original builder
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(original, YCoCgVariant::Variant1);
            dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(original, true);

            // Clone the builder
            let cloned = dltbc2_clone_ManualTransformBuilder(original);
            assert!(!cloned.is_null());

            // Both builders should work independently
            let test_data = create_test_bc2_data();
            let mut output1 = vec![0u8; test_data.len()];
            let mut output2 = vec![0u8; test_data.len()];

            let result1 = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output1.as_mut_ptr(),
                output1.len(),
                original,
            );
            assert_eq!(result1.error_code, Dltbc2ErrorCode::Success);

            let result2 = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output2.as_mut_ptr(),
                output2.len(),
                cloned,
            );
            assert_eq!(result2.error_code, Dltbc2ErrorCode::Success);

            // Outputs should be identical since both builders have same settings
            assert_eq!(output1, output2);

            // Clean up
            dltbc2_free_ManualTransformBuilder(original);
            dltbc2_free_ManualTransformBuilder(cloned);
        }
    }

    #[test]
    fn test_dltbc2_clone_manual_transform_builder_null_pointer() {
        unsafe {
            let cloned = dltbc2_clone_ManualTransformBuilder(ptr::null());
            assert!(cloned.is_null());
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_set_decorrelation_mode() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        unsafe {
            // Should not crash with valid builder
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCoCgVariant::Variant1);
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCoCgVariant::Variant2);
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCoCgVariant::Variant3);
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCoCgVariant::None);

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_set_decorrelation_mode_null_pointer() {
        unsafe {
            // Should not crash with null pointer
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(
                ptr::null_mut(),
                YCoCgVariant::Variant1,
            );
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_set_split_colour_endpoints() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        unsafe {
            // Should not crash with valid builder
            dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(builder, true);
            dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(builder, false);

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_set_split_colour_endpoints_null_pointer() {
        unsafe {
            // Should not crash with null pointer
            dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(ptr::null_mut(), true);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_reset_to_defaults() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        unsafe {
            // Configure with non-default settings
            dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, YCoCgVariant::Variant1);
            dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(builder, true);

            // Reset to defaults
            dltbc2_ManualTransformBuilder_ResetToDefaults(builder);

            // Should still be usable after reset
            let test_data = create_test_bc2_data();
            let mut output = vec![0u8; test_data.len()];

            let result = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                builder,
            );
            assert_eq!(result.error_code, Dltbc2ErrorCode::Success);

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_reset_to_defaults_null_pointer() {
        unsafe {
            // Should not crash with null pointer
            dltbc2_ManualTransformBuilder_ResetToDefaults(ptr::null_mut());
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_transform_basic() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = create_test_bc2_data();
        let mut output = vec![0u8; test_data.len()];

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::Success);
            assert!(result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_transform_null_input() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let mut output = vec![0u8; 16];

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Transform(
                ptr::null(),
                16,
                output.as_mut_ptr(),
                output.len(),
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullDataPointer);
            assert!(!result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_transform_null_output() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = create_test_bc2_data();

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                16,
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_transform_null_builder() {
        let test_data = create_test_bc2_data();
        let mut output = vec![0u8; test_data.len()];

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                ptr::null_mut(),
            );

            assert_eq!(
                result.error_code,
                Dltbc2ErrorCode::NullManualTransformBuilderPointer
            );
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_transform_invalid_length() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = [0u8; 15]; // Not divisible by 16
        let mut output = vec![0u8; 15];

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::InvalidLength);
            assert!(!result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_transform_output_too_small() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = create_test_bc2_data();
        let mut output = vec![0u8; test_data.len() - 1]; // Too small

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::OutputBufferTooSmall);
            assert!(!result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_untransform_basic() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = create_test_bc2_data();
        let mut transformed = vec![0u8; test_data.len()];
        let mut restored = vec![0u8; test_data.len()];

        unsafe {
            // Transform
            let result1 = dltbc2_ManualTransformBuilder_Transform(
                test_data.as_ptr(),
                test_data.len(),
                transformed.as_mut_ptr(),
                transformed.len(),
                builder,
            );
            assert_eq!(result1.error_code, Dltbc2ErrorCode::Success);

            // Untransform
            let result2 = dltbc2_ManualTransformBuilder_Untransform(
                transformed.as_ptr(),
                transformed.len(),
                restored.as_mut_ptr(),
                restored.len(),
                builder,
            );
            assert_eq!(result2.error_code, Dltbc2ErrorCode::Success);
            assert!(result2.is_success());

            // Should restore original data
            assert_eq!(restored, test_data);

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_untransform_null_input() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let mut output = vec![0u8; 16];

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Untransform(
                ptr::null(),
                16,
                output.as_mut_ptr(),
                output.len(),
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullDataPointer);
            assert!(!result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_untransform_null_output() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = create_test_bc2_data();

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Untransform(
                test_data.as_ptr(),
                test_data.len(),
                ptr::null_mut(),
                16,
                builder,
            );

            assert_eq!(result.error_code, Dltbc2ErrorCode::NullOutputBufferPointer);
            assert!(!result.is_success());

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_untransform_null_builder() {
        let test_data = create_test_bc2_data();
        let mut output = vec![0u8; test_data.len()];

        unsafe {
            let result = dltbc2_ManualTransformBuilder_Untransform(
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                ptr::null_mut(),
            );

            assert_eq!(
                result.error_code,
                Dltbc2ErrorCode::NullManualTransformBuilderPointer
            );
            assert!(!result.is_success());
        }
    }

    #[test]
    fn test_dltbc2_manual_transform_builder_round_trip_with_settings() {
        let builder = dltbc2_new_ManualTransformBuilder();
        assert!(!builder.is_null());

        let test_data = create_test_bc2_data();

        unsafe {
            // Test different decorrelation modes
            for variant in [
                YCoCgVariant::None,
                YCoCgVariant::Variant1,
                YCoCgVariant::Variant2,
                YCoCgVariant::Variant3,
            ] {
                for split_colours in [false, true] {
                    // Configure builder
                    dltbc2_ManualTransformBuilder_SetDecorrelationMode(builder, variant);
                    dltbc2_ManualTransformBuilder_SetSplitColourEndpoints(builder, split_colours);

                    let mut transformed = vec![0u8; test_data.len()];
                    let mut restored = vec![0u8; test_data.len()];

                    // Transform
                    let result1 = dltbc2_ManualTransformBuilder_Transform(
                        test_data.as_ptr(),
                        test_data.len(),
                        transformed.as_mut_ptr(),
                        transformed.len(),
                        builder,
                    );
                    assert_eq!(
                        result1.error_code,
                        Dltbc2ErrorCode::Success,
                        "Transform failed for variant {variant:?}, split_colours {split_colours}",
                    );

                    // Untransform
                    let result2 = dltbc2_ManualTransformBuilder_Untransform(
                        transformed.as_ptr(),
                        transformed.len(),
                        restored.as_mut_ptr(),
                        restored.len(),
                        builder,
                    );
                    assert_eq!(
                        result2.error_code,
                        Dltbc2ErrorCode::Success,
                        "Untransform failed for variant {variant:?}, split_colours {split_colours}",
                    );

                    // Should restore original data
                    assert_eq!(
                        restored, test_data,
                        "Round-trip failed for variant {variant:?}, split_colours {split_colours}",
                    );
                }
            }

            dltbc2_free_ManualTransformBuilder(builder);
        }
    }
}
