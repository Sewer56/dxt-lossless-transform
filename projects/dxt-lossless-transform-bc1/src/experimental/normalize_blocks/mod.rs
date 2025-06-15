//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC1 blocks to improve compression ratio
//! by making solid color blocks and transparent blocks have consistent representations.
//!
//! ## BC1 Block Format
//!
//! First, let's recall the BC1 block format:
//!
//! ```text
//! Address: 0        2        4       8
//!          +--------+--------+--------+
//! Data:    | Color0 | Color1 | Indices|
//!          +--------+--------+--------+
//! ```
//!
//! Where:
//! - `Color0` and `Color1` are 16-bit RGB565 color values (2 bytes each)
//! - `Indices` are 4 bytes containing sixteen 2-bit indices (one for each pixel in the 4x4 block)
//!
//! ## Normalization Rules
//!
//! The normalization process applies the following rules to improve compression:
//!
//! ### 1. Solid Color Blocks
//!
//! When an entire block represents a single solid color with a clean conversion between RGBA8888
//! and RGB565, we standardize the representation:
//!
//! ```text
//! +--------+--------+--------+
//! | Color  | 0x0000 |  0x00  |
//! +--------+--------+--------+
//! ```
//!
//! We place the block's color in `Color0`, set `Color1` to zero, and set all indices to zero
//! (represented by all-zero bytes in the indices section). Can also be repeat of same byte.
//!
//! The implementation checks for this case by:
//! 1. Decoding the block to get all 16 pixels
//! 2. Checking that all pixels have the same color
//! 3. Verifying the color can be cleanly round-tripped through RGB565 encoding
//! 4. Constructing a new normalized block with the pattern above
//!
//! ### 2. Fully Transparent Blocks
//!
//! For blocks that are completely transparent (common in textures with alpha), we standardize
//! the representation to all 1's:
//!
//! ```text
//! +--------+--------+--------+
//! | 0xFFFF | 0xFFFF | 0xFFFF |
//! +--------+--------+--------+
//! ```
//!
//! The implementation detects transparent blocks by:
//! 1. Decoding all 16 pixels in the block
//! 2. Checking if all pixels have alpha=0 (check if first pixel is transparent, after checking if all are equal)
//! 3. Setting the entire block content to 0xFF bytes
//!
//! ### 3. Mixed Transparency Blocks
//!
//! In BC1, when `Color0 <= Color1`, the block is in "punch-through alpha" mode, where index `11`
//! represents a transparent pixel. Blocks containing both opaque and transparent pixels
//! (mixed alpha) use this mode.
//!
//! For these blocks, we can't apply significant normalization without changing the visual
//! appearance, so we preserve them unchanged.
//!
//! ## Implementation Details
//!
//! The normalization process uses the BC1 decoder to analyze the block content, then rebuilds
//! blocks according to the rules above.
//!
//! When normalizing blocks, we:
//!
//! 1. Look at the RGB565 color values to determine if we're in alpha mode (`Color0 <= Color1`)
//! 2. Decode the block to get the 16 pixels with their colors
//! 3. Apply one of the three normalization cases based on the block properties
//! 4. Write the normalized block to the output

pub mod normalize;
pub use normalize::*;

use crate::determine_optimal_transform::*;
use crate::YCoCgVariant;
use crate::{
    transforms::standard::{transform, transform_with_separate_pointers},
    Bc1TransformDetails,
};
use dxt_lossless_transform_common::allocate::FixedRawAllocArray;
use dxt_lossless_transform_common::{
    color_565::Color565, transforms::split_565_color_endpoints::split_color_endpoints,
};

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
/// - work_ptr must be valid for writes of len/2 bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
///
/// [`determine_optimal_transform::determine_best_transform_details`]: crate::determine_optimal_transform::determine_best_transform_details
#[inline]
pub unsafe fn transform_bc1_with_normalize_blocks(
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
        // Split the blocks, colours to work area, indices to final destination.
        transform_with_separate_pointers(
            input_ptr,                           // from our input
            work_ptr as *mut u32,                // colours to go our work area
            output_ptr.add(len / 2) as *mut u32, // but the indices go to their final destination
            len,
        );

        // Now normalize the blocks in place. In place is faster because it avoids copying the data unnecessarily.
        normalize_split_blocks_in_place(
            work_ptr,                // colours are in first half of the work buffer
            output_ptr.add(len / 2), // indices are in second half of the output buffer
            len / 8,                 // 8 bytes per block, so len / 8 blocks
            transform_options.color_normalization_mode,
        );

        // Split the colour endpoints, writing them to the output buffer alongside the indices for final result
        split_color_endpoints(
            work_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Decorrelate the colours in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
    // Only normalization. 10
    else if has_normalization {
        // Split the blocks into the output area.
        transform(input_ptr, output_ptr, len);

        // Now normalize them in place. In place is faster because it avoids copying the data unnecessarily.
        normalize_split_blocks_in_place(
            output_ptr,              // colours are in first half of the output buffer
            output_ptr.add(len / 2), // indices are in second half of the output buffer
            len / 8,                 // 8 bytes per block, so len / 8 blocks
            transform_options.color_normalization_mode,
        );

        // Decorrelate the colours in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
    // Only split colours. 01
    else if has_split_colours {
        // Split the blocks, colours to work area, indices to final destination.
        transform_with_separate_pointers(
            input_ptr,                           // from our input
            work_ptr as *mut u32,                // colours to go our work area
            output_ptr.add(len / 2) as *mut u32, // but the indices go to their final destination
            len,
        );

        // Split the colour endpoints, writing them to the final output buffer.
        split_color_endpoints(
            work_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Decorrelate the colours in output buffer in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
    // None. 00
    else {
        // Split the blocks directly into expected output.
        transform(input_ptr, output_ptr, len);

        // And if there's colour decorrelation, do it right now (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
}

/// Determine the best transform details with full normalization testing.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `len`: The length of the input data in bytes
///
/// # Returns
///
/// The best (smallest size) format for the given data.
///
/// # Remarks
///
/// This function tests all normalization options, the characteristics of this function are:
///
/// - 1/24th of the compression speed ([`ColorNormalizationMode`] * [`YCoCgVariant`] * 2 (split_colours))
/// - Uses 6x the memory of input size
///
/// # Safety
///
/// Function is unsafe because it deals with raw pointers which must be correct.
pub(crate) unsafe fn determine_best_transform_details_with_normalization<F>(
    input_ptr: *const u8,
    len: usize,
    transform_options: Bc1EstimateOptions<F>,
) -> Result<Bc1TransformDetails, DetermineBestTransformError>
where
    F: Fn(*const u8, usize) -> usize,
{
    const NUM_NORMALIZE: usize = ColorNormalizationMode::all_values().len();
    let mut normalize_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let mut split_blocks_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let normalize_buffers_ptrs = normalize_buffers.get_pointer_slice();
    let split_blocks_buffers_ptrs = split_blocks_buffers.get_pointer_slice();

    // Normalize blocks into all possible modes.
    let any_normalized = normalize_blocks_all_modes(input_ptr, &normalize_buffers_ptrs, len);

    // Now we got all blocks normalized and split, and have to test all the different possibilities.
    // We can repurpose the normalize_buffers
    let mut best_transform_details = Bc1TransformDetails::default();
    let mut best_size = usize::MAX;

    // split_blocks_buffers_ptrs: buffer_a
    // result_pointers: buffer_b (output)
    let result_pointers = normalize_buffers.get_pointer_slice();
    if any_normalized {
        // At least 1 block was normalized, so we have to test all options.
        // Now split all blocks.
        for x in 0..NUM_NORMALIZE {
            transform(normalize_buffers_ptrs[x], split_blocks_buffers_ptrs[x], len);
        }

        for norm_idx in 0..NUM_NORMALIZE {
            for decorrelation_mode in YCoCgVariant::all_values() {
                for split_colours in [true, false] {
                    // Get the current mode we're testing.
                    let current_mode = Bc1TransformDetails {
                        color_normalization_mode: ColorNormalizationMode::all_values()[norm_idx],
                        decorrelation_mode: *decorrelation_mode,
                        split_colour_endpoints: split_colours,
                    };

                    // Get input/output buffers.
                    let input = split_blocks_buffers_ptrs[norm_idx];
                    let output = result_pointers[norm_idx];

                    test_normalize_variant(
                        input,
                        output,
                        len,
                        &transform_options,
                        &mut best_transform_details,
                        &mut best_size,
                        current_mode,
                    );
                }
            }
        }

        Ok(best_transform_details)
    } else {
        // No blocks were normalized, we can skip testing normalize steps
        // Since no normalization occurred, we can use the original input directly after splitting
        transform(input_ptr, split_blocks_buffers_ptrs[0], len);

        for decorrelation_mode in YCoCgVariant::all_values() {
            for split_colours in [true, false] {
                // Get the current mode we're testing.
                let current_mode = Bc1TransformDetails {
                    color_normalization_mode: ColorNormalizationMode::None, // Skip normalization step
                    decorrelation_mode: *decorrelation_mode,
                    split_colour_endpoints: split_colours,
                };

                // Get input/output buffers.
                let input = split_blocks_buffers_ptrs[0]; // Use first buffer since no normalization variants
                let output = result_pointers[0];

                test_normalize_variant(
                    input,
                    output,
                    len,
                    &transform_options,
                    &mut best_transform_details,
                    &mut best_size,
                    current_mode,
                );
            }
        }

        Ok(best_transform_details)
    }
}
