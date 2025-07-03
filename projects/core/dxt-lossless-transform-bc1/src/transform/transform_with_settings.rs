//! BC1 Transform Operations
//!
//! This module provides the core transformation functions for BC1 data.

use crate::transform::{
    standard::{transform, untransform},
    with_recorrelate, with_split_colour, with_split_colour_and_recorr,
};
use dxt_lossless_transform_common::color_565::YCoCgVariant;

use super::settings::{Bc1TransformSettings, Bc1UntransformSettings};

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`crate::transform_bc1_auto`] or
///   [`Bc1TransformSettings::default`] for less optimal result(s).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: Bc1TransformSettings,
) {
    debug_assert!(len.is_multiple_of(8));

    let has_split_colours = transform_options.split_colour_endpoints;

    if has_split_colours {
        if transform_options.decorrelation_mode == YCoCgVariant::None {
            with_split_colour::transform_with_split_colour(
                input_ptr,
                output_ptr as *mut u16,              // color0 values
                output_ptr.add(len / 4) as *mut u16, // color1 values
                output_ptr.add(len / 2) as *mut u32, // indices in last half
                len / 8,                             // number of blocks (8 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::transform_with_split_colour_and_recorr(
                input_ptr,
                output_ptr as *mut u16,              // color0 values
                output_ptr.add(len / 4) as *mut u16, // color1 values
                output_ptr.add(len / 2) as *mut u32, // indices in last half
                len / 8,                             // number of blocks (8 bytes per block)
                transform_options.decorrelation_mode,
            );
        }
    } else if transform_options.decorrelation_mode == YCoCgVariant::None {
        // Standard transform – no split-colour and no decorrelation.
        transform(input_ptr, output_ptr, len);
    } else {
        // Standard transform + decorrelate.
        with_recorrelate::transform_with_decorrelate(
            input_ptr,
            output_ptr,
            len,
            transform_options.decorrelation_mode,
        );
    }
}

/// Untransform BC1 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks).
///   Output from [`transform_bc1_with_settings`].
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `untransform_options`: A struct containing information about the transform that was originally performed.
///   Must match the settings used in [`transform_bc1_with_settings`] function (excluding color normalization).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc1_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    untransform_options: Bc1UntransformSettings,
) {
    debug_assert!(len.is_multiple_of(8));

    let has_split_colours = untransform_options.split_colour_endpoints;

    if has_split_colours {
        if untransform_options.decorrelation_mode == YCoCgVariant::None {
            // Optimized single-pass operation: unsplit split colors and combine with indices
            // directly into BC1 blocks, avoiding intermediate memory copies
            with_split_colour::untransform_with_split_colour(
                input_ptr as *const u16,              // color0 values
                input_ptr.add(len / 4) as *const u16, // color1 values
                input_ptr.add(len / 2) as *const u32, // indices
                output_ptr,                           // output BC1 blocks
                len / 8,                              // number of blocks (8 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::untransform_with_split_colour_and_recorr(
                input_ptr as *const u16,              // color0 values
                input_ptr.add(len / 4) as *const u16, // color1 values
                input_ptr.add(len / 2) as *const u32, // indices
                output_ptr,                           // output BC1 blocks
                len / 8,                              // number of blocks (8 bytes per block)
                untransform_options.decorrelation_mode,
            );
        }
    } else if untransform_options.decorrelation_mode == YCoCgVariant::None {
        // Standard transform – no split-colour and no decorrelation.
        untransform(input_ptr, output_ptr, len);
    } else {
        // Standard transform + recorrelate.
        with_recorrelate::untransform_with_recorrelate(
            input_ptr,
            output_ptr,
            len,
            untransform_options.decorrelation_mode,
        );
    }
}
