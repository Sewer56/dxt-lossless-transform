//! BC1 manual transform builder for C API.
//!
//! This module provides ABI-stable functions for managing and configuring BC1 manual transform builder
//! through a builder pattern. The builder stores BC1 transform configuration and must be
//! explicitly freed. These functions offer a stable interface that is guaranteed
//! to remain compatible across library versions.
//!
//! For users requiring maximum performance and willing to accept potential breaking
//! changes, see the core crate functions directly.

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use core::{ptr, slice};
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

// =============================================================================
// Type Definitions
// =============================================================================

/// Opaque manual transform builder type for BC1 transform operations.
///
/// This builder stores the current transform configuration and must be:
///
/// - Created with [`dltbc1_new_ManualTransformBuilder()`]
/// - Modified using the builder functions
/// - Passed to transform operations
/// - Freed with [`dltbc1_free_ManualTransformBuilder()`] when no longer needed
///
/// The builder is NOT thread-safe and should not be shared between threads.
/// Each thread should create its own builder.
#[repr(C)]
pub struct Dltbc1ManualTransformBuilder {
    // Private field to ensure it's opaque
    _private: [u8; 0],
}

/// Internal representation of the manual transform builder
pub(crate) struct Dltbc1ManualTransformBuilderInner {
    pub(crate) builder: crate::transform::Bc1ManualTransformBuilder,
}

/// Get mutable access to the inner manual transform builder.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
pub(crate) unsafe fn get_manual_builder_mut(
    builder: *mut Dltbc1ManualTransformBuilder,
) -> &'static mut Dltbc1ManualTransformBuilderInner {
    debug_assert!(!builder.is_null());
    unsafe { &mut *(builder as *mut Dltbc1ManualTransformBuilderInner) }
}

// =============================================================================
// Lifecycle Functions
// =============================================================================

/// Create a new BC1 manual transform builder with default settings.
///
/// The returned builder must be freed with [`dltbc1_free_ManualTransformBuilder()`] when no longer needed.
///
/// # Returns
/// A pointer to a newly allocated manual transform builder, or null if allocation fails.
///
/// # Remarks
/// This function corresponds to [`crate::Bc1ManualTransformBuilder::new`] in the Rust API.
#[unsafe(no_mangle)]
pub extern "C" fn dltbc1_new_ManualTransformBuilder() -> *mut Dltbc1ManualTransformBuilder {
    let inner = Box::new(Dltbc1ManualTransformBuilderInner {
        builder: crate::transform::Bc1ManualTransformBuilder::new(),
    });

    Box::into_raw(inner) as *mut Dltbc1ManualTransformBuilder
}

/// Free a BC1 manual transform builder.
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc1_new_ManualTransformBuilder()`]
/// - `builder` must not have been freed already
/// - After calling this function, `builder` becomes invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_free_ManualTransformBuilder(
    builder: *mut Dltbc1ManualTransformBuilder,
) {
    if !builder.is_null() {
        unsafe {
            drop(Box::from_raw(
                builder as *mut Dltbc1ManualTransformBuilderInner,
            ));
        }
    }
}

/// Clone a BC1 manual transform builder.
///
/// Creates a new builder with the same settings as the source builder.
/// The returned builder must be freed independently.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
///
/// # Returns
/// A pointer to a newly allocated manual transform builder with the same settings, or null if allocation fails.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_clone_ManualTransformBuilder(
    builder: *const Dltbc1ManualTransformBuilder,
) -> *mut Dltbc1ManualTransformBuilder {
    if builder.is_null() {
        return ptr::null_mut();
    }

    let inner = unsafe { &*(builder as *const Dltbc1ManualTransformBuilderInner) };
    let cloned = Box::new(Dltbc1ManualTransformBuilderInner {
        builder: inner.builder,
    });

    Box::into_raw(cloned) as *mut Dltbc1ManualTransformBuilder
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
/// For automatic optimization, consider using `dltbc1_AutoTransformBuilder_Transform` instead.
///
/// # Parameters
/// - `builder`: The BC1 manual builder to modify
/// - `mode`: The decorrelation mode to use
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
///
/// # Remarks
/// This function corresponds to [`crate::Bc1ManualTransformBuilder::decorrelation_mode`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_ManualTransformBuilder_SetDecorrelationMode(
    builder: *mut Dltbc1ManualTransformBuilder,
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
/// This setting controls whether BC1 texture color endpoints are separated during processing,
/// which can improve compression efficiency for many textures.
///
/// **File Size**: This setting reduces file size around 78% of the time.
///
/// For automatic optimization, consider using `dltbc1_AutoTransformBuilder_Transform` instead.
///
/// # Parameters
/// - `builder`: The BC1 manual builder to modify
/// - `split`: Whether to split colour endpoints
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
///
/// # Remarks
/// This function corresponds to [`crate::Bc1ManualTransformBuilder::split_colour_endpoints`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_ManualTransformBuilder_SetSplitColourEndpoints(
    builder: *mut Dltbc1ManualTransformBuilder,
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
/// - `builder`: The BC1 manual builder to reset
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_ManualTransformBuilder_ResetToDefaults(
    builder: *mut Dltbc1ManualTransformBuilder,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_manual_builder_mut(builder) };
    inner.builder = crate::transform::Bc1ManualTransformBuilder::new();
}

// =============================================================================
// Transform Operations
// =============================================================================

/// Transform BC1 data using the settings configured in the builder.
///
/// This function applies the transformation directly using the settings stored in the
/// provided builder without any optimization or testing of different configurations.
///
/// # Parameters
/// - `input`: Pointer to the BC1 data to transform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to the output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `builder`: The manual transform builder containing the settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
///
/// # Examples
///
/// ```c
/// // Create and configure manual transform builder
/// Dltbc1ManualTransformBuilder* builder = dltbc1_new_ManualTransformBuilder();
/// dltbc1_ManualTransformBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
/// dltbc1_ManualTransformBuilder_SetSplitColourEndpoints(builder, true);
///
/// // Transform the data
/// Dltbc1Result result = dltbc1_ManualTransformBuilder_Transform(
///     bc1_data, sizeof(bc1_data),
///     transformed_data, sizeof(transformed_data),
///     builder);
/// ```
///
/// # Remarks
/// This function corresponds to [`crate::Bc1ManualTransformBuilder::transform`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_ManualTransformBuilder_Transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    builder: *mut Dltbc1ManualTransformBuilder,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullManualTransformBuilderPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the builder and perform transformation using its method
    let builder_inner = unsafe { get_manual_builder_mut(builder) };

    // Perform the transformation using the builder's method
    match builder_inner.builder.transform(input_slice, output_slice) {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => e.into(),
    }
}

/// Untransform BC1 data using the settings configured in the builder.
///
/// This function reverses the transformation applied by [`dltbc1_ManualTransformBuilder_Transform`],
/// restoring the original BC1 data. The builder must contain the same settings that were
/// used for the original transformation.
///
/// # Parameters
/// - `input`: Pointer to the transformed BC1 data to untransform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to the output buffer where the original BC1 data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `builder`: The manual transform builder containing the untransform settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `builder` must be a valid pointer to a [`Dltbc1ManualTransformBuilder`]
/// - The builder must contain the same settings used for the original transformation
///
/// # Examples
///
/// ```c
/// // Use the same builder that was used for transform
/// Dltbc1Result result = dltbc1_ManualTransformBuilder_Untransform(
///     transformed_data, sizeof(transformed_data),
///     restored_data, sizeof(restored_data),
///     builder);
/// ```
///
/// # Remarks
/// This function corresponds to [`crate::Bc1ManualTransformBuilder::untransform`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_ManualTransformBuilder_Untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    builder: *mut Dltbc1ManualTransformBuilder,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullManualTransformBuilderPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the builder and perform untransformation using its method
    let builder_inner = unsafe { get_manual_builder_mut(builder) };

    // Perform the untransformation using the builder's method
    match builder_inner.builder.untransform(input_slice, output_slice) {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => e.into(),
    }
}
