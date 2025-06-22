//! BC1 transform settings builder for C API.
//!
//! This module provides functions to configure BC1 transform settings through a builder pattern
//! that operates on the transform context.

use crate::Bc1Error;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform_context::{Dltbc1TransformContext, get_context_mut};
use crate::transform::{transform_bc1_with_settings, untransform_bc1_with_settings};
use core::slice;
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

/// Set the decorrelation mode for the context.
///
/// Controls the YCoCg-R color space decorrelation variant used for transformation.
/// Different variants can provide varying compression ratios depending on the texture content.
///
/// **Note**: When manually testing decorrelation modes, the typical improvement from
/// using different variants is <0.1% in practice. For better compression gains,
/// it's recommended to use a compression level on the estimator (e.g., ZStandard estimator)
/// closer to your final compression level instead.
///
/// For automatic optimization, consider using [`dltbc1_EstimateSettingsBuilder_BuildAndTransform`] instead.
///
/// # Parameters
/// - `context`: The BC1 context to modify
/// - `mode`: The decorrelation mode to use
///
/// # Safety
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_SetDecorrelationMode(
    context: *mut Dltbc1TransformContext,
    mode: YCoCgVariant,
) {
    if context.is_null() {
        return;
    }

    let inner = unsafe { get_context_mut(context) };
    inner.builder = inner.builder.decorrelation_mode(mode.to_internal_variant());
}

/// Set whether to split colour endpoints for the context.
///
/// This setting controls whether BC1 texture color endpoints are separated during processing,
/// which can improve compression efficiency for many textures.
///
/// **File Size**: This setting reduces file size around 78% of the time.
///
/// For automatic optimization, consider using [`dltbc1_EstimateSettingsBuilder_BuildAndTransform`] instead.
///
/// # Parameters
/// - `context`: The BC1 context to modify
/// - `split`: Whether to split colour endpoints
///
/// # Safety
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_SetSplitColourEndpoints(
    context: *mut Dltbc1TransformContext,
    split: bool,
) {
    if context.is_null() {
        return;
    }

    let inner = unsafe { get_context_mut(context) };
    inner.builder = inner.builder.split_colour_endpoints(split);
}

/// Reset the context to default settings.
///
/// # Parameters
/// - `context`: The BC1 context to reset
///
/// # Safety
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_ResetToDefaults(
    context: *mut Dltbc1TransformContext,
) {
    if context.is_null() {
        return;
    }

    let inner = unsafe { get_context_mut(context) };
    inner.builder = crate::transform::Bc1TransformSettingsBuilder::new();
}

/// Transform BC1 data using the settings configured in the context.
///
/// This function applies the transformation directly using the settings stored in the
/// provided context without any optimization or testing of different configurations.
///
/// # Parameters
/// - `input`: Pointer to the BC1 data to transform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to the output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `context`: The transform context containing the settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
///
/// # Examples
///
/// ```c
/// // Create and configure transform context
/// Dltbc1TransformContext* context = dltbc1_new_TransformContext();
/// dltbc1_TransformContext_SetDecorrelationMode(context, YCOCG_VARIANT_1);
/// dltbc1_TransformContext_SetSplitColourEndpoints(context, true);
///
/// // Transform the data
/// Dltbc1Result result = dltbc1_TransformContext_Transform(
///     bc1_data, sizeof(bc1_data),
///     transformed_data, sizeof(transformed_data),
///     context);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_Transform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    context: *mut Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformContextPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the settings from the context
    let context_inner = unsafe { get_context_mut(context) };
    let settings = context_inner.builder.build();

    // Perform the transformation
    match transform_bc1_with_settings(input_slice, output_slice, settings) {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => {
            // Map the error to error codes
            match e {
                Bc1Error::InvalidLength(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength)
                }
                Bc1Error::OutputBufferTooSmall { .. } => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::OutputBufferTooSmall)
                }
                Bc1Error::AllocationFailed(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::AllocationFailed)
                }
                Bc1Error::SizeEstimationFailed(_) => {
                    // This shouldn't happen in transform_with_settings, but handle it anyway
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::SizeEstimationFailed)
                }
            }
        }
    }
}

/// Untransform BC1 data using the settings configured in the context.
///
/// This function reverses the transformation applied by [`dltbc1_TransformContext_Transform`],
/// restoring the original BC1 data. The context must contain the same settings that were
/// used for the original transformation.
///
/// # Parameters
/// - `input`: Pointer to the transformed BC1 data to untransform
/// - `input_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to the output buffer where the original BC1 data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `input_len`)
/// - `context`: The transform context containing the detransform settings to use
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `input` must be valid for reads of `input_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
/// - The context must contain the same settings used for the original transformation
///
/// # Examples
///
/// ```c
/// // Use the same context that was used for transform
/// Dltbc1Result result = dltbc1_TransformContext_Untransform(
///     transformed_data, sizeof(transformed_data),
///     restored_data, sizeof(restored_data),
///     context);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_TransformContext_Untransform(
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_len: usize,
    context: *mut Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate pointers
    if input.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformContextPointer);
    }

    // Create slices from raw pointers
    let input_slice = unsafe { slice::from_raw_parts(input, input_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Get the settings from the context and convert to detransform settings
    let context_inner = unsafe { get_context_mut(context) };
    let transform_settings = context_inner.builder.build();
    let detransform_settings = transform_settings.into();

    // Perform the untransformation
    match untransform_bc1_with_settings(input_slice, output_slice, detransform_settings) {
        Ok(()) => Dltbc1Result::success(),
        Err(e) => {
            // Map the error to error codes
            match e {
                Bc1Error::InvalidLength(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength)
                }
                Bc1Error::OutputBufferTooSmall { .. } => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::OutputBufferTooSmall)
                }
                Bc1Error::AllocationFailed(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::AllocationFailed)
                }
                Bc1Error::SizeEstimationFailed(_) => {
                    // This shouldn't happen in untransform_with_settings, but handle it anyway
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::SizeEstimationFailed)
                }
            }
        }
    }
}
