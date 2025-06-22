//! BC1 transform settings builder for C API.
//!
//! This module provides ABI-stable functions for managing and configuring BC1 transform settings
//! through a builder pattern. The builder stores BC1 transform configuration and must be
//! explicitly freed. These functions offer a stable interface that is guaranteed
//! to remain compatible across library versions.
//!
//! For users requiring maximum performance and willing to accept potential breaking
//! changes, see the [`transform_with_settings`] module for ABI-unstable alternatives.
//!
//! [`transform_with_settings`]: super::unstable::transform_with_settings

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use core::{ptr, slice};
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

// =============================================================================
// Type Definitions
// =============================================================================

/// Opaque transform settings builder type for BC1 transform operations.
///
/// This builder stores the current transform configuration and must be:
///
/// - Created with [`dltbc1_new_TransformSettingsBuilder()`]
/// - Modified using the builder functions
/// - Passed to transform operations
/// - Freed with [`dltbc1_free_TransformSettingsBuilder()`] when no longer needed
///
/// The builder is NOT thread-safe and should not be shared between threads.
/// Each thread should create its own builder.
#[repr(C)]
pub struct Dltbc1TransformSettingsBuilder {
    // Private field to ensure it's opaque
    _private: [u8; 0],
}

/// Internal representation of the transform settings builder
pub(crate) struct Dltbc1TransformSettingsBuilderInner {
    pub(crate) builder: crate::transform::Bc1ManualTransformBuilder,
}

/// Get mutable access to the inner transform settings builder.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
pub(crate) unsafe fn get_settings_builder_mut(
    builder: *mut Dltbc1TransformSettingsBuilder,
) -> &'static mut Dltbc1TransformSettingsBuilderInner {
    debug_assert!(!builder.is_null());
    unsafe { &mut *(builder as *mut Dltbc1TransformSettingsBuilderInner) }
}

// =============================================================================
// Lifecycle Functions
// =============================================================================

/// Create a new BC1 transform settings builder with default settings.
///
/// The returned builder must be freed with [`dltbc1_free_TransformSettingsBuilder()`] when no longer needed.
///
/// # Returns
/// A pointer to a newly allocated transform settings builder, or null if allocation fails.
#[unsafe(no_mangle)]
pub extern "C" fn dltbc1_new_TransformSettingsBuilder() -> *mut Dltbc1TransformSettingsBuilder {
    let inner = Box::new(Dltbc1TransformSettingsBuilderInner {
        builder: crate::transform::Bc1ManualTransformBuilder::new(),
    });

    Box::into_raw(inner) as *mut Dltbc1TransformSettingsBuilder
}

/// Free a BC1 transform settings builder.
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc1_new_TransformSettingsBuilder()`]
/// - `builder` must not have been freed already
/// - After calling this function, `builder` becomes invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_free_TransformSettingsBuilder(
    builder: *mut Dltbc1TransformSettingsBuilder,
) {
    if !builder.is_null() {
        unsafe {
            drop(Box::from_raw(
                builder as *mut Dltbc1TransformSettingsBuilderInner,
            ));
        }
    }
}

/// Clone a BC1 transform settings builder.
///
/// Creates a new builder with the same settings as the source builder.
/// The returned builder must be freed independently.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
///
/// # Returns
/// A pointer to a newly allocated transform settings builder with the same settings, or null if allocation fails.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_clone_TransformSettingsBuilder(
    builder: *const Dltbc1TransformSettingsBuilder,
) -> *mut Dltbc1TransformSettingsBuilder {
    if builder.is_null() {
        return ptr::null_mut();
    }

    let inner = unsafe { &*(builder as *const Dltbc1TransformSettingsBuilderInner) };
    let cloned = Box::new(Dltbc1TransformSettingsBuilderInner {
        builder: inner.builder,
    });

    Box::into_raw(cloned) as *mut Dltbc1TransformSettingsBuilder
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
/// For automatic optimization, consider using [`super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform`] instead.
///
/// [`super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform`]: super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform
///
/// # Parameters
/// - `builder`: The BC1 settings builder to modify
/// - `mode`: The decorrelation mode to use
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformSettingsBuilder_SetDecorrelationMode(
    builder: *mut Dltbc1TransformSettingsBuilder,
    mode: YCoCgVariant,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_settings_builder_mut(builder) };
    inner.builder = inner.builder.decorrelation_mode(mode.to_internal_variant());
}

/// Set whether to split colour endpoints for the builder.
///
/// This setting controls whether BC1 texture color endpoints are separated during processing,
/// which can improve compression efficiency for many textures.
///
/// **File Size**: This setting reduces file size around 78% of the time.
///
/// For automatic optimization, consider using [`super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform`] instead.
///
/// [`super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform`]: super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform
///
/// # Parameters
/// - `builder`: The BC1 settings builder to modify
/// - `split`: Whether to split colour endpoints
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformSettingsBuilder_SetSplitColourEndpoints(
    builder: *mut Dltbc1TransformSettingsBuilder,
    split: bool,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_settings_builder_mut(builder) };
    inner.builder = inner.builder.split_colour_endpoints(split);
}

/// Reset the builder to default settings.
///
/// # Parameters
/// - `builder`: The BC1 settings builder to reset
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformSettingsBuilder_ResetToDefaults(
    builder: *mut Dltbc1TransformSettingsBuilder,
) {
    if builder.is_null() {
        return;
    }

    let inner = unsafe { get_settings_builder_mut(builder) };
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
/// - `builder`: The transform settings builder containing the settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
///
/// # Examples
///
/// ```c
/// // Create and configure transform settings builder
/// Dltbc1TransformSettingsBuilder* builder = dltbc1_new_TransformSettingsBuilder();
/// dltbc1_TransformSettingsBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
/// dltbc1_TransformSettingsBuilder_SetSplitColourEndpoints(builder, true);
///
/// // Transform the data
/// Dltbc1Result result = dltbc1_TransformSettingsBuilder_Transform(
///     bc1_data, sizeof(bc1_data),
///     transformed_data, sizeof(transformed_data),
///     builder);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformSettingsBuilder_Transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    builder: *mut Dltbc1TransformSettingsBuilder,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformSettingsBuilderPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the builder and perform transformation using its method
    let builder_inner = unsafe { get_settings_builder_mut(builder) };

    // Perform the transformation using the builder's method
    match builder_inner
        .builder
        .build_and_transform(input_slice, output_slice)
    {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => e.into(),
    }
}

/// Untransform BC1 data using the settings configured in the builder.
///
/// This function reverses the transformation applied by [`dltbc1_TransformSettingsBuilder_Transform`],
/// restoring the original BC1 data. The builder must contain the same settings that were
/// used for the original transformation.
///
/// # Parameters
/// - `input`: Pointer to the transformed BC1 data to untransform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to the output buffer where the original BC1 data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `builder`: The transform settings builder containing the detransform settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
/// - The builder must contain the same settings used for the original transformation
///
/// # Examples
///
/// ```c
/// // Use the same builder that was used for transform
/// Dltbc1Result result = dltbc1_TransformSettingsBuilder_Untransform(
///     transformed_data, sizeof(transformed_data),
///     restored_data, sizeof(restored_data),
///     builder);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformSettingsBuilder_Untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    builder: *mut Dltbc1TransformSettingsBuilder,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformSettingsBuilderPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the builder and perform untransformation using its method
    let builder_inner = unsafe { get_settings_builder_mut(builder) };

    // Perform the untransformation using the builder's method
    match builder_inner
        .builder
        .build_and_untransform(input_slice, output_slice)
    {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => e.into(),
    }
}
