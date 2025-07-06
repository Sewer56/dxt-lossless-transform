//! # Split BC2 Blocks and Decorrelate Module
//!
//! This module provides optimized functions for separating BC2 data into four distinct arrays
//! while applying YCoCg decorrelation to color endpoints in a single optimized step.
//! This eliminates the need for intermediate memory copies by performing both
//! decorrelation and splitting operations directly.
//!
//! Below is a description of the untransformation process.
//! For transformation, swap the `output` and `input`.
//!
//! ## Input Format
//!
//! The module expects BC2 blocks in standard interleaved format:
//!
//! ### BC2 Blocks (`input_ptr`)
//! - Type: `*const u8`
//! - Contains standard BC2/DXT3 compressed texture blocks
//! - Each block is 16 bytes in the following format:
//!   ```ignore
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 8    | alpha data (4-bit per pixel, not interpolated)
//!   8      | 2    | color0 (RGB565, little-endian)
//!   10     | 2    | color1 (RGB565, little-endian)  
//!   12     | 4    | indices (2 bits per pixel, little-endian)
//!   ```
//!
//! ## Output Format
//!
//! The module outputs four separate arrays:
//!
//! ### Alpha Array (`alpha_ptr`)
//! - Type: `*mut u64`
//! - Contains the alpha data for each BC2 block (8 bytes per block, unchanged)
//!
//! ### Color0 Array (`color0_ptr`)
//! - Type: `*mut u16`
//! - Contains the first color value for each BC2 block (in transformed/decorrelated form)
//!
//! ### Color1 Array (`color1_ptr`)
//! - Type: `*mut u16`
//! - Contains the second color value for each BC2 block (in transformed/decorrelated form)
//!
//! ### Indices Array (`indices_ptr`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC2 block

use dxt_lossless_transform_common::color_565::YCoCgVariant;

pub(crate) mod transform;
pub(crate) mod untransform;

/// Transform BC2 data from standard interleaved format to four separate arrays
/// (alpha, color0, color1, indices) while applying YCoCg decorrelation using best known
/// implementation for current CPU.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_ptr` must be valid for writes of `block_count * 8` bytes
/// - `color0_ptr` must be valid for writes of `block_count * 2` bytes
/// - `color1_ptr` must be valid for writes of `block_count * 2` bytes
/// - `indices_ptr` must be valid for writes of `block_count * 4` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
/// - `decorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
///
/// # Parameters
///
/// - `input_ptr`: Pointer to source BC2 block data
/// - `alpha_ptr`: Pointer to destination alpha array
/// - `color0_ptr`: Pointer to destination color0 array  
/// - `color1_ptr`: Pointer to destination color1 array
/// - `indices_ptr`: Pointer to destination indices array
/// - `block_count`: Number of BC2 blocks to process
/// - `decorrelation_mode`: YCoCg decorrelation variant to apply to color endpoints
///
/// # Returns
///
/// This function does not return a value. On successful completion, the input
/// BC2 blocks will have been split into four separate arrays with decorrelated colors.
///
/// # Remarks
///
/// This function automatically selects the best available SIMD implementation
/// for the current CPU architecture. The color0 and color1 values are decorrelated
/// using the specified YCoCg variant, while alpha and indices remain unchanged.
/// This provides better compression ratios by reducing correlation between color channels.
#[inline]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    alpha_ptr: *mut u64,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    transform::transform_with_split_colour_and_recorr(
        input_ptr,
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        block_count,
        decorrelation_mode,
    );
}

/// Transform BC2 data from four separate arrays (alpha, color0, color1, indices) back to
/// standard interleaved format while applying YCoCg recorrelation using best known
/// implementation for current CPU.
///
/// # Safety
///
/// - `alpha_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for reads of `block_count * 2` bytes
/// - `color1_ptr` must be valid for reads of `block_count * 2` bytes  
/// - `indices_ptr` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
///
/// # Parameters
///
/// - `alpha_ptr`: Pointer to source alpha array
/// - `color0_ptr`: Pointer to source color0 array
/// - `color1_ptr`: Pointer to source color1 array
/// - `indices_ptr`: Pointer to source indices array
/// - `output_ptr`: Pointer to destination BC2 block data
/// - `block_count`: Number of BC2 blocks to process
/// - `recorrelation_mode`: YCoCg recorrelation variant to apply to color endpoints
///
/// # Returns
///
/// This function does not return a value. On successful completion, the four
/// separate arrays will have been combined into standard BC2 block format with
/// recorrelated colors.
///
/// # Remarks
///
/// This function automatically selects the best available SIMD implementation
/// for the current CPU architecture. The performance characteristics are
/// identical to [`transform_with_split_colour_and_recorr`] but in reverse.
///
/// This function is the exact inverse of [`transform_with_split_colour_and_recorr`].
/// Applying transform followed by untransform (or vice versa) with the same
/// decorrelation/recorrelation mode should produce the original data.
#[inline]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    untransform::untransform_with_split_colour_and_recorr(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
        recorrelation_mode,
    );
}
