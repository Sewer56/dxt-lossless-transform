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
    pub decorrelation_mode: u8, // Maps to YCoCgVariant
}

impl From<Dltbc1TransformSettings> for crate::Bc1TransformSettings {
    fn from(settings: Dltbc1TransformSettings) -> Self {
        Self {
            split_colour_endpoints: settings.split_colour_endpoints,
            decorrelation_mode: match settings.decorrelation_mode {
                0 => YCoCgVariant::None,
                1 => YCoCgVariant::Variant1,
                2 => YCoCgVariant::Variant2,
                3 => YCoCgVariant::Variant3,
                _ => YCoCgVariant::None, // Default fallback
            },
        }
    }
}

impl From<Dltbc1DetransformSettings> for crate::Bc1DetransformSettings {
    fn from(settings: Dltbc1DetransformSettings) -> Self {
        // Convert to Bc1TransformSettings first, then to Bc1DetransformSettings
        let transform_settings = crate::Bc1TransformSettings {
            split_colour_endpoints: settings.split_colour_endpoints,
            decorrelation_mode: match settings.decorrelation_mode {
                0 => YCoCgVariant::None,
                1 => YCoCgVariant::Variant1,
                2 => YCoCgVariant::Variant2,
                3 => YCoCgVariant::Variant3,
                _ => YCoCgVariant::None, // Default fallback
            },
        };
        transform_settings.into()
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
