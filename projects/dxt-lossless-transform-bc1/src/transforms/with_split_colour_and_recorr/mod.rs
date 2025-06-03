//! # Unsplit Split Colour-Split Blocks and Decorrelate Module
//!
//! This module provides optimized functions for combining split color data, applying decorrelation,
//! and unsplitting block indices back into standard BC1 (DXT1) compressed texture blocks in a single
//! optimized step. This eliminates the need for intermediate memory copies by performing both
//! decorrelation and unsplitting operations directly.
//!
//! Below is a description of the detransformation process.
//! For transformation, swap the `output` and `input`.
//!
//! ## Input Format
//!
//! The module expects three separate arrays as input:
//!
//! ### Color0 Array (`color0_ptr`)
//! - Type: `*const u16`
//! - Contains the first color value for each BC1 block (in transformed/correlated form)
//!
//! ### Color1 Array (`color1_ptr`)
//! - Type: `*const u16`
//! - Contains the second color value for each BC1 block (in transformed/correlated form)
//!
//! ### Indices Array (`indices_ptr`)
//! - Type: `*const u32`
//! - Contains the 2-bit per pixel color indices for each BC1 block
//!
//! ## Output Format
//!
//! ### BC1 Blocks (`output_ptr`)
//! - Type: `*mut u8`
//! - Contains standard BC1/DXT1 compressed texture blocks
//! - Each block is 8 bytes in the following format:
//!   ```ignore
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 2    | color0 (RGB565, little-endian, decorrelated)
//!   2      | 2    | color1 (RGB565, little-endian, decorrelated)  
//!   4      | 4    | indices (2 bits per pixel, little-endian)
//!   ```

pub mod transform;
pub mod untransform;

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Transform BC1 data from three separate arrays (color0, color1, indices) back to
/// standard interleaved format while applying YCoCg decorrelation using best known
/// implementation for current CPU.
///
/// # Safety
///
/// - color0_ptr must be valid for reads of block_count * 2 bytes
/// - color1_ptr must be valid for reads of block_count * 2 bytes  
/// - indices_ptr must be valid for reads of block_count * 4 bytes
/// - output_ptr must be valid for writes of block_count * 8 bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_with_split_colour_and_recorr(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    untransform::untransform_with_split_colour_and_recorr(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
        decorrelation_mode,
    );
}
