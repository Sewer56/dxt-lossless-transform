//! BC1 transform operations with explicit settings for C API (ABI-unstable).
//!
//! **⚠️ ABI Instability Warning**: All functions in this module accept ABI-unstable
//! structures which may change between versions without major version bumps.
//! For production use, prefer the ABI-stable builder patterns in
//! [`super::super::manual_transform_builder`].
//!
//! This module provides ABI-unstable functions for transforming and
//! untransforming BC1 data using specific transform settings.

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::{Dltbc1DetransformSettings, Dltbc1TransformSettings};
use core::slice;
use dxt_lossless_transform_bc1::{
    transform_bc1_with_settings_safe, untransform_bc1_with_settings_safe,
};

// =============================================================================
// ABI-Unstable Functions
// =============================================================================

/// Transform BC1 data using specified transform settings (ABI-unstable).
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
///
/// **⚠️ ABI Instability Warning**: This function accepts ABI-unstable structures
/// which may change between library versions. For production use, prefer
/// [`super::super::manual_transform_builder::dltbc1_TransformSettingsBuilder_Transform`]
/// which provides ABI stability and builder pattern convenience.
///
/// **Prefer the ABI-stable builder:** This function provides direct access but may have
/// breaking changes. Consider using
/// [`super::super::manual_transform_builder::dltbc1_TransformSettingsBuilder_Transform`]
/// which provides ABI stability and convenient configuration methods.
///
/// # Recommended Alternative
///
/// For production use:
/// ```c
/// // Create and configure builder (ABI-stable)
/// Dltbc1TransformSettingsBuilder* builder = dltbc1_new_TransformSettingsBuilder();
/// dltbc1_TransformSettingsBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
/// dltbc1_TransformSettingsBuilder_SetSplitColourEndpoints(builder, true);
///
/// // Transform the data
/// Dltbc1Result result = dltbc1_TransformSettingsBuilder_Transform(
///     input, input_len, output, output_len, builder);
///
/// // Clean up
/// dltbc1_free_TransformSettingsBuilder(builder);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_transform(
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

/// Untransform BC1 data using specified detransform settings (ABI-unstable).
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
///
/// **⚠️ ABI Instability Warning**: This function accepts ABI-unstable structures
/// which may change between library versions. For production use, prefer
/// [`super::super::manual_transform_builder::dltbc1_TransformSettingsBuilder_Untransform`]
/// which provides ABI stability.
///
/// **Prefer the ABI-stable builder:** This function provides direct access but may have
/// breaking changes. Consider using
/// [`super::super::manual_transform_builder::dltbc1_TransformSettingsBuilder_Untransform`]
/// which provides ABI stability.
///
/// # Recommended Alternative
///
/// For production use:
/// ```c
/// // Use the same builder that was used for transform (ABI-stable)
/// Dltbc1Result result = dltbc1_TransformSettingsBuilder_Untransform(
///     transformed_data, transformed_len, restored_data, restored_len, builder);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_untransform(
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
