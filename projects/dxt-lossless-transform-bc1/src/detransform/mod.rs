//! This module contains accelerated routines for performing certain combined detransformation steps.

use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use split_and_decorrelate::*;

pub(crate) mod split_and_decorrelate;
pub(crate) mod unsplit_split_colour_split_blocks;
pub(crate) use unsplit_split_colour_split_blocks::unsplit_split_colour_split_blocks;

/// Combines unsplitting of BC1 blocks with YCoCg recorrelation in a single optimized pass.
///
/// This function transforms BC1 data from split format (colors separated from indices) back to
/// standard interleaved format while simultaneously applying YCoCg recorrelation to the color
/// endpoints for improved decompression.
///
/// # Input Format
///
/// The input data is expected to be in split format:
/// - First half: color endpoints (4 bytes per block: 2x RGB565 values)  
/// - Second half: color indices (4 bytes per block: 16x 2-bit indices)
///
/// # Output Format
///
/// Standard BC1 block format (8 bytes per block):
/// - Bytes 0-3: color endpoints after recorrelation  
/// - Bytes 4-7: color indices (unchanged)
///
/// # Parameters
///
/// - `input_ptr`: Pointer to split BC1 data
/// - `output_ptr`: Pointer to output buffer for standard BC1 blocks
/// - `len`: Total length in bytes (must be divisible by 8)
/// - `decorrelation_mode`: [`YCoCgVariant`] specifying the recorrelation variant to apply
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `output_ptr` must be valid for writes of `len` bytes  
/// - `len` must be divisible by 8 (BC1 block size)
/// - `decorrelation_mode` must not be [`YCoCgVariant::None`]
#[inline(always)]
pub(crate) unsafe fn untransform_split_and_decorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    decorrelation_mode: YCoCgVariant,
) {
    debug_assert!(len % 8 == 0);

    unsafe {
        // Setup pointers
        let colors_ptr = input_ptr as *const u32;
        let indices_ptr = input_ptr.add(len / 2) as *const u32;
        let num_blocks = len / 8; // Each BC1 block is 8 bytes

        // Combined recorrelation + unsplit operation
        match decorrelation_mode {
            YCoCgVariant::Variant1 => {
                untransform_split_and_decorrelate_variant1(
                    colors_ptr,
                    indices_ptr,
                    output_ptr,
                    num_blocks,
                );
            }
            YCoCgVariant::Variant2 => {
                untransform_split_and_decorrelate_variant2(
                    colors_ptr,
                    indices_ptr,
                    output_ptr,
                    num_blocks,
                );
            }
            YCoCgVariant::Variant3 => {
                untransform_split_and_decorrelate_variant3(
                    colors_ptr,
                    indices_ptr,
                    output_ptr,
                    num_blocks,
                );
            }
            YCoCgVariant::None => {
                // This should be unreachable based on the calling context
                unreachable_unchecked()
            }
        }
    }
}
