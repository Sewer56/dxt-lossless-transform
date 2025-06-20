//! # Unsplit Split Colour-Split Blocks Module
//!
//! This module provides optimized functions for combining split color data and block indices
//! back into standard BC1 (DXT1) compressed texture blocks. This is part of the detransformation
//! process that reverses the lossless transformation applied to BC1 data.
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
//! - Contains the first color value for each BC1 block
//!
//! ### Color1 Array (`color1_ptr`)
//! - Type: `*const u16`
//! - Contains the second color value for each BC1 block
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
//!   0      | 2    | color0 (RGB565, little-endian)
//!   2      | 2    | color1 (RGB565, little-endian)  
//!   4      | 4    | indices (2 bits per pixel, little-endian)

pub(crate) mod transform;
pub(crate) mod untransform;

/// Transform BC1 data from standard interleaved format to three separate arrays
/// (color0, color1, indices) using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of block_count * 8 bytes
/// - color0_ptr must be valid for writes of block_count * 2 bytes
/// - color1_ptr must be valid for writes of block_count * 2 bytes
/// - indices_ptr must be valid for writes of block_count * 4 bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
) {
    transform::transform_with_split_colour(
        input_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        block_count,
    );
}

/// Transform BC1 data from three separate arrays (color0, color1, indices) back to
/// standard interleaved format using best known implementation for current CPU.
///
/// # Safety
///
/// - color0_ptr must be valid for reads of block_count * 2 bytes
/// - color1_ptr must be valid for reads of block_count * 2 bytes  
/// - indices_ptr must be valid for reads of block_count * 4 bytes
/// - output_ptr must be valid for writes of block_count * 8 bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform::untransform_with_split_colour(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    );
}
