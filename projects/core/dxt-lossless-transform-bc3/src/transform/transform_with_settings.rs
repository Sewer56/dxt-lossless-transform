//! BC3 Transform Operations
//!
//! This module provides the core transformation functions for BC3 data.

use crate::transform::standard;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

use super::settings::{Bc3TransformSettings, Bc3UntransformSettings};

/// Transform BC3 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks)
/// - `output_ptr`: A pointer to the output data (output BC3 blocks)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`crate::transform_bc3_auto`] or
///   [`Bc3TransformSettings::default`] for less optimal result(s).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc3_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: Bc3TransformSettings,
) {
    debug_assert!(len.is_multiple_of(16));

    // For now, we only have the standard transform implementation
    // TODO: Implement the various transform variants based on settings
    match (
        transform_options.split_alpha_endpoints,
        transform_options.split_colour_endpoints,
        transform_options.decorrelation_mode,
    ) {
        // Standard transform – no split-alpha, no split-colour and no decorrelation.
        (false, false, YCoCgVariant::None) => {
            standard::split_blocks(input_ptr, output_ptr, len);
        }
        // All other combinations will need dedicated transform implementations
        // For now, fall back to standard transform
        _ => {
            // TODO: Implement remaining transform variants:
            // - with_split_alphas
            // - with_recorrelate
            // - with_split_colour
            // - with_split_alphas_and_recorr
            // - with_split_alphas_and_colour
            // - with_split_colour_and_recorr
            // - with_split_alphas_colour_and_recorr
            standard::split_blocks(input_ptr, output_ptr, len);
        }
    }
}

/// Untransform BC3 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks).
///   Output from [`transform_bc3_with_settings`].
/// - `output_ptr`: A pointer to the output data (output BC3 blocks)
/// - `len`: The length of the input data in bytes
/// - `untransform_options`: A struct containing information about the transform that was originally performed.
///   Must match the settings used in [`transform_bc3_with_settings`] function (excluding color normalization).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc3_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    untransform_options: Bc3UntransformSettings,
) {
    debug_assert!(len.is_multiple_of(16));

    // For now, we only have the standard transform implementation
    // TODO: Implement the various untransform variants based on settings
    match (
        untransform_options.split_alpha_endpoints,
        untransform_options.split_colour_endpoints,
        untransform_options.decorrelation_mode,
    ) {
        // Standard transform – no split-alpha, no split-colour and no decorrelation.
        (false, false, YCoCgVariant::None) => {
            standard::unsplit_blocks(input_ptr, output_ptr, len);
        }
        // All other combinations will need dedicated untransform implementations
        // For now, fall back to standard untransform
        _ => {
            // TODO: Implement remaining untransform variants:
            // - with_split_alphas
            // - with_recorrelate
            // - with_split_colour
            // - with_split_alphas_and_recorr
            // - with_split_alphas_and_colour
            // - with_split_colour_and_recorr
            // - with_split_alphas_colour_and_recorr
            standard::unsplit_blocks(input_ptr, output_ptr, len);
        }
    }
}
