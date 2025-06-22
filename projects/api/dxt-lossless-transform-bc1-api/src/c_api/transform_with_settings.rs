//! BC1 transform operations with explicit settings for C API.
//!
//! This module provides both ABI-stable and ABI-unstable functions for transforming and
//! untransforming BC1 data using specific transform settings.

use crate::Bc1Error;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::{Dltbc1DetransformSettings, Dltbc1TransformSettings};
use crate::transform::{transform_bc1_with_settings, untransform_bc1_with_settings};
use core::slice;

// =============================================================================
// ABI-Unstable Functions
// =============================================================================

/// Transform BC1 data using specified transform settings (ABI-unstable).
///
/// ## ABI Instability Warning
/// This function accepts ABI-unstable structures which may change between versions.
/// Use [`dltbc1_TransformContext_Transform`] for ABI stability.
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
/// [`dltbc1_TransformContext_Transform`]: crate::c_api::transform_settings_builder::dltbc1_TransformContext_Transform
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

/// Untransform BC1 data using specified detransform settings (ABI-unstable).
///
/// ## ABI Instability Warning
/// This function accepts ABI-unstable structures which may change between versions.
/// Use [`dltbc1_TransformContext_Untransform`] for ABI stability.
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
/// [`dltbc1_TransformContext_Untransform`]: crate::c_api::transform_settings_builder::dltbc1_TransformContext_Untransform
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

    // Perform the untransformation
    match untransform_bc1_with_settings(input_slice, output_slice, settings) {
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
