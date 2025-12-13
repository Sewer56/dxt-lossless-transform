//! BC3 Transform Operations
//!
//! This module provides the core transformation functions for BC3 data.

use crate::transform::{
    standard, with_recorrelate, with_split_alphas, with_split_alphas_and_colour,
    with_split_alphas_and_recorr, with_split_alphas_colour_and_recorr, with_split_colour,
    with_split_colour_and_recorr,
};
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

    let has_split_alphas = transform_options.split_alpha_endpoints;
    let has_split_colours = transform_options.split_colour_endpoints;
    let has_decorrelation = transform_options.decorrelation_mode != YCoCgVariant::None;

    match (has_split_alphas, has_split_colours, has_decorrelation) {
        // Standard transform – no split-alpha, no split-colour and no decorrelation.
        (false, false, false) => {
            standard::transform(input_ptr, output_ptr, len);
        }
        // Split alphas only
        (true, false, false) => {
            let block_count = len / 16;
            with_split_alphas::transform_with_split_alphas(
                input_ptr,
                output_ptr,                                   // alpha0 (1 byte per block)
                output_ptr.add(block_count),                  // alpha1 (1 byte per block)
                output_ptr.add(block_count * 2) as *mut u16,  // alpha_indices (6 bytes per block)
                output_ptr.add(block_count * 8) as *mut u32,  // colors (4 bytes per block)
                output_ptr.add(block_count * 12) as *mut u32, // color_indices (4 bytes per block)
                block_count,
            );
        }
        // Decorrelation only
        (false, false, true) => {
            with_recorrelate::transform_with_decorrelate(
                input_ptr,
                output_ptr,
                len,
                transform_options.decorrelation_mode,
            );
        }
        // Split colours only
        (false, true, false) => {
            let block_count = len / 16;
            with_split_colour::transform_with_split_colour(
                input_ptr,
                output_ptr as *mut u16, // alpha_endpoints (2 bytes per block)
                output_ptr.add(block_count * 2) as *mut u16, // alpha_indices (6 bytes per block)
                output_ptr.add(block_count * 8) as *mut u16, // color0 (2 bytes per block)
                output_ptr.add(block_count * 10) as *mut u16, // color1 (2 bytes per block)
                output_ptr.add(block_count * 12) as *mut u32, // color_indices (4 bytes per block)
                block_count,
            );
        }
        // Split alphas + decorrelation
        (true, false, true) => {
            let block_count = len / 16;
            with_split_alphas_and_recorr::transform_with_split_alphas_and_recorr(
                input_ptr,
                output_ptr,                                   // alpha0 (1 byte per block)
                output_ptr.add(block_count),                  // alpha1 (1 byte per block)
                output_ptr.add(block_count * 2) as *mut u16,  // alpha_indices (6 bytes per block)
                output_ptr.add(block_count * 8) as *mut u32,  // colors (4 bytes per block)
                output_ptr.add(block_count * 12) as *mut u32, // color_indices (4 bytes per block)
                block_count,
                transform_options.decorrelation_mode,
            );
        }
        // Split alphas + split colours
        (true, true, false) => {
            let block_count = len / 16;
            with_split_alphas_and_colour::transform_with_split_alphas_and_colour(
                input_ptr,
                output_ptr,                                   // alpha0 (1 byte per block)
                output_ptr.add(block_count),                  // alpha1 (1 byte per block)
                output_ptr.add(block_count * 2) as *mut u16,  // alpha_indices (6 bytes per block)
                output_ptr.add(block_count * 8) as *mut u16,  // color0 (2 bytes per block)
                output_ptr.add(block_count * 10) as *mut u16, // color1 (2 bytes per block)
                output_ptr.add(block_count * 12) as *mut u32, // color_indices (4 bytes per block)
                block_count,
            );
        }
        // Split colours + decorrelation
        (false, true, true) => {
            let block_count = len / 16;
            with_split_colour_and_recorr::transform_with_split_colour_and_recorr(
                input_ptr,
                output_ptr as *mut u16, // alpha_endpoints (2 bytes per block)
                output_ptr.add(block_count * 2) as *mut u16, // alpha_indices (6 bytes per block)
                output_ptr.add(block_count * 8) as *mut u16, // color0 (2 bytes per block)
                output_ptr.add(block_count * 10) as *mut u16, // color1 (2 bytes per block)
                output_ptr.add(block_count * 12) as *mut u32, // color_indices (4 bytes per block)
                block_count,
                transform_options.decorrelation_mode,
            );
        }
        // Split alphas + split colours + decorrelation
        (true, true, true) => {
            let block_count = len / 16;
            with_split_alphas_colour_and_recorr::transform_with_split_alphas_colour_and_recorr(
                input_ptr,
                output_ptr,                                   // alpha0 (1 byte per block)
                output_ptr.add(block_count),                  // alpha1 (1 byte per block)
                output_ptr.add(block_count * 2) as *mut u16,  // alpha_indices (6 bytes per block)
                output_ptr.add(block_count * 8) as *mut u16,  // color0 (2 bytes per block)
                output_ptr.add(block_count * 10) as *mut u16, // color1 (2 bytes per block)
                output_ptr.add(block_count * 12) as *mut u32, // color_indices (4 bytes per block)
                block_count,
                transform_options.decorrelation_mode,
            );
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

    let has_split_alphas = untransform_options.split_alpha_endpoints;
    let has_split_colours = untransform_options.split_colour_endpoints;
    let has_decorrelation = untransform_options.decorrelation_mode != YCoCgVariant::None;

    match (has_split_alphas, has_split_colours, has_decorrelation) {
        // Standard transform – no split-alpha, no split-colour and no decorrelation.
        (false, false, false) => {
            standard::untransform(input_ptr, output_ptr, len);
        }
        // Split alphas only
        (true, false, false) => {
            let block_count = len / 16;
            with_split_alphas::untransform_with_split_alphas(
                input_ptr,                                     // alpha0 (1 byte per block)
                input_ptr.add(block_count),                    // alpha1 (1 byte per block)
                input_ptr.add(block_count * 2) as *const u16,  // alpha_indices (6 bytes per block)
                input_ptr.add(block_count * 8) as *const u32,  // colors (4 bytes per block)
                input_ptr.add(block_count * 12) as *const u32, // color_indices (4 bytes per block)
                output_ptr,
                block_count,
            );
        }
        // Decorrelation only
        (false, false, true) => {
            with_recorrelate::untransform_with_recorrelate(
                input_ptr,
                output_ptr,
                len,
                untransform_options.decorrelation_mode,
            );
        }
        // Split colours only
        (false, true, false) => {
            let block_count = len / 16;
            with_split_colour::untransform_with_split_colour(
                input_ptr as *const u16, // alpha_endpoints (2 bytes per block)
                input_ptr.add(block_count * 2) as *const u16, // alpha_indices (6 bytes per block)
                input_ptr.add(block_count * 8) as *const u16, // color0 (2 bytes per block)
                input_ptr.add(block_count * 10) as *const u16, // color1 (2 bytes per block)
                input_ptr.add(block_count * 12) as *const u32, // color_indices (4 bytes per block)
                output_ptr,
                block_count,
            );
        }
        // Split alphas + decorrelation
        (true, false, true) => {
            let block_count = len / 16;
            with_split_alphas_and_recorr::untransform_with_split_alphas_and_recorr(
                input_ptr,                                     // alpha0 (1 byte per block)
                input_ptr.add(block_count),                    // alpha1 (1 byte per block)
                input_ptr.add(block_count * 2) as *const u16,  // alpha_indices (6 bytes per block)
                input_ptr.add(block_count * 8) as *const u32,  // colors (4 bytes per block)
                input_ptr.add(block_count * 12) as *const u32, // color_indices (4 bytes per block)
                output_ptr,
                block_count,
                untransform_options.decorrelation_mode,
            );
        }
        // Split alphas + split colours
        (true, true, false) => {
            let block_count = len / 16;
            with_split_alphas_and_colour::untransform_with_split_alphas_and_colour(
                input_ptr,                                     // alpha0 (1 byte per block)
                input_ptr.add(block_count),                    // alpha1 (1 byte per block)
                input_ptr.add(block_count * 2) as *const u16,  // alpha_indices (6 bytes per block)
                input_ptr.add(block_count * 8) as *const u16,  // color0 (2 bytes per block)
                input_ptr.add(block_count * 10) as *const u16, // color1 (2 bytes per block)
                input_ptr.add(block_count * 12) as *const u32, // color_indices (4 bytes per block)
                output_ptr,
                block_count,
            );
        }
        // Split colours + decorrelation
        (false, true, true) => {
            let block_count = len / 16;
            with_split_colour_and_recorr::untransform_with_split_colour_and_recorr(
                input_ptr as *const u16, // alpha_endpoints (2 bytes per block)
                input_ptr.add(block_count * 2) as *const u16, // alpha_indices (6 bytes per block)
                input_ptr.add(block_count * 8) as *const u16, // color0 (2 bytes per block)
                input_ptr.add(block_count * 10) as *const u16, // color1 (2 bytes per block)
                input_ptr.add(block_count * 12) as *const u32, // color_indices (4 bytes per block)
                output_ptr,
                block_count,
                untransform_options.decorrelation_mode,
            );
        }
        // Split alphas + split colours + decorrelation
        (true, true, true) => {
            let block_count = len / 16;
            with_split_alphas_colour_and_recorr::untransform_with_split_alphas_colour_and_recorr(
                input_ptr,                                     // alpha0 (1 byte per block)
                input_ptr.add(block_count),                    // alpha1 (1 byte per block)
                input_ptr.add(block_count * 2) as *const u16,  // alpha_indices (6 bytes per block)
                input_ptr.add(block_count * 8) as *const u16,  // color0 (2 bytes per block)
                input_ptr.add(block_count * 10) as *const u16, // color1 (2 bytes per block)
                input_ptr.add(block_count * 12) as *const u32, // color_indices (4 bytes per block)
                output_ptr,
                block_count,
                untransform_options.decorrelation_mode,
            );
        }
    }
}
