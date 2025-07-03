//! BC2 Transform Operations
//!
//! This module provides the core transformation functions for BC2 data.

use crate::transform::{
    standard, with_recorrelate, with_split_colour, with_split_colour_and_recorr,
};
use dxt_lossless_transform_common::color_565::YCoCgVariant;

use super::settings::{Bc2TransformSettings, Bc2UntransformSettings};

/// Transform BC2 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC2 blocks)
/// - `output_ptr`: A pointer to the output data (output BC2 blocks)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`crate::transform_bc2_auto`] or
///   [`Bc2TransformSettings::default`] for less optimal result(s).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc2_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: Bc2TransformSettings,
) {
    debug_assert!(len % 16 == 0);

    let has_split_colours = transform_options.split_colour_endpoints;

    if has_split_colours {
        if transform_options.decorrelation_mode == YCoCgVariant::None {
            with_split_colour::transform_with_split_colour(
                input_ptr,
                output_ptr as *mut u64, // alpha values (8 bytes per block)
                output_ptr.add(len / 2) as *mut u16, // color0 values
                output_ptr.add(len / 2 + len / 8) as *mut u16, // color1 values
                output_ptr.add(len / 2 + len / 4) as *mut u32, // indices
                len / 16,               // number of blocks (16 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::transform_with_split_colour_and_recorr(
                input_ptr,
                output_ptr as *mut u64, // alpha values (8 bytes per block)
                output_ptr.add(len / 2) as *mut u16, // color0 values
                output_ptr.add(len / 2 + len / 8) as *mut u16, // color1 values
                output_ptr.add(len / 2 + len / 4) as *mut u32, // indices
                len / 16,               // number of blocks (16 bytes per block)
                transform_options.decorrelation_mode,
            );
        }
    } else if transform_options.decorrelation_mode == YCoCgVariant::None {
        // Standard transform – no split-colour and no decorrelation.
        standard::transform(input_ptr, output_ptr, len);
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

/// Untransform BC2 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC2 blocks).
///   Output from [`transform_bc2_with_settings`].
/// - `output_ptr`: A pointer to the output data (output BC2 blocks)
/// - `len`: The length of the input data in bytes
/// - `untransform_options`: A struct containing information about the transform that was originally performed.
///   Must match the settings used in [`transform_bc2_with_settings`] function (excluding color normalization).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc2_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    untransform_options: Bc2UntransformSettings,
) {
    debug_assert!(len % 16 == 0);

    let has_split_colours = untransform_options.split_colour_endpoints;

    if has_split_colours {
        if untransform_options.decorrelation_mode == YCoCgVariant::None {
            // Optimized single-pass operation: unsplit split colors and combine with indices
            // directly into BC2 blocks, avoiding intermediate memory copies
            with_split_colour::untransform_with_split_colour(
                input_ptr as *const u64,              // alpha values (8 bytes per block)
                input_ptr.add(len / 2) as *const u16, // color0 values
                input_ptr.add(len / 2 + len / 8) as *const u16, // color1 values
                input_ptr.add(len / 2 + len / 4) as *const u32, // indices
                output_ptr,                           // output BC2 blocks
                len / 16,                             // number of blocks (16 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::untransform_with_split_colour_and_recorr(
                input_ptr as *const u64,              // alpha values (8 bytes per block)
                input_ptr.add(len / 2) as *const u16, // color0 values
                input_ptr.add(len / 2 + len / 8) as *const u16, // color1 values
                input_ptr.add(len / 2 + len / 4) as *const u32, // indices
                output_ptr,                           // output BC2 blocks
                len / 16,                             // number of blocks (16 bytes per block)
                untransform_options.decorrelation_mode,
            );
        }
    } else if untransform_options.decorrelation_mode == YCoCgVariant::None {
        // Standard transform – no split-colour and no decorrelation.
        standard::untransform(input_ptr, output_ptr, len);
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
