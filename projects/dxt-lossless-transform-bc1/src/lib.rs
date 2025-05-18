#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(avx512_target_feature))]
#![cfg_attr(feature = "nightly", feature(stdarch_x86_avx512))]

use core::ptr::copy_nonoverlapping;

use dxt_lossless_transform_common::{
    color_565::{Color565, YCoCgVariant},
    transforms::split_565_color_endpoints::split_color_endpoints,
};
use normalize_blocks::{normalize_blocks, ColorNormalizationMode};
use split_blocks::{split_blocks, unsplit_blocks};
pub mod determine_optimal_transform;
pub mod normalize_blocks;
pub mod split_blocks;
pub mod util;

/// The information about the BC1 transform that was just performed.
/// Each item transformed via [`transform_bc1`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass the same instance to [`untransform_bc1`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Bc1TransformDetails {
    /// The color normalization mode that was used to normalize the data.
    pub color_normalization_mode: ColorNormalizationMode,

    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

impl Default for Bc1TransformDetails {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            color_normalization_mode: ColorNormalizationMode::Color0Only,
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `work_ptr`: A pointer to a work buffer (used by function)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr` and half size of `work_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`determine_optimal_transform::determine_best_transform_details`] or
///   [`Bc1TransformDetails::default`] for less optimal result(s).
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// `output_ptr` will be written to twice if normalization is used (it normally is).
/// This may have performance implications if `output_ptr` is a pointer to a memory mapped file
/// and amount of available memory is scarce. Outside of that, memory should be fairly unaffected.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - work_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    work_ptr: *mut u8,
    len: usize,
    transform_options: Bc1TransformDetails,
) {
    debug_assert!(len % 8 == 0);

    let has_normalization =
        transform_options.color_normalization_mode != ColorNormalizationMode::None;
    let has_split_colours = transform_options.split_colour_endpoints;

    // Both normalization and split colours. 11
    if has_normalization && has_split_colours {
        // This one kinda sucks because we're doing an unnecessary copy of half the data,
        // but not much we can do there for now.

        // We do a double write to output area here, using a two buffer strategy.


        // TODO: Either:
        // - Normalize pre-split blocks (easier)
        // or
        // - Split with secondary pointer where only the indices go, so we can write those directly to output.

        // Normalize the blocks into the output area directly
        normalize_blocks(
            input_ptr,
            output_ptr,
            len,
            transform_options.color_normalization_mode,
        );

        // Split the normalized blocks into the work area.
        split_blocks(output_ptr, work_ptr, len);

        // Decorrelate the colours in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            work_ptr as *const Color565,
            work_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(),
            transform_options.decorrelation_mode,
        );

        // Split the colour endpoints, writing them to the output buffer alongside the indices
        split_color_endpoints(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Copy the index data (this can be avoided if we normalize pre-split)
        copy_nonoverlapping(work_ptr.add(len / 2), output_ptr.add(len / 2), len / 2);
    }
    // Only normalization. 10
    else if has_normalization {
        // Normalize the blocks into the work area.
        normalize_blocks(
            input_ptr,
            work_ptr,
            len,
            transform_options.color_normalization_mode,
        );

        // Split the blocks into the output area.
        split_blocks(work_ptr, output_ptr, len);

        // Decorrelate the colours in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(),
            transform_options.decorrelation_mode,
        );
    }
    // Only split colours. 01
    else if has_split_colours {
        // Split the blocks into the work area.
        split_blocks(input_ptr, work_ptr, len);

        // Split the colour endpoints, writing them to the output buffer.
        split_color_endpoints(
            work_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Decorrelate the colours in output buffer in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(),
            transform_options.decorrelation_mode,
        );

        // Copy the remainder of the split block data (indices)
        copy_nonoverlapping(work_ptr.add(len / 2), output_ptr.add(len / 2), len / 2);
    }
    // None. 00
    else {
        // Split the blocks directly into expected output.
        split_blocks(input_ptr, output_ptr, len);

        // And if there's colour decorrelation, do it right now (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(),
            transform_options.decorrelation_mode,
        );
    }
}

/// Untransform BC1 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks).
///   Output from [`transform_bc1`].
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `_details`: A struct containing information about the transform that was performed.
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc1(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    _details: &Bc1TransformDetails,
) {
    debug_assert!(len % 8 == 0);
    unsplit_blocks(input_ptr, output_ptr, len);
}
