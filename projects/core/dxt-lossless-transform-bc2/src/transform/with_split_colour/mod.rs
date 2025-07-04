//! # Split BC2 Blocks Module
//!
//! This module provides optimized functions for separating BC2 data into four distinct arrays
//! for better compression efficiency by grouping similar data together.
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
//! - Contains the alpha data for each BC2 block (8 bytes per block)
//!
//! ### Color0 Array (`color0_ptr`)
//! - Type: `*mut u16`
//! - Contains the first color value for each BC2 block
//!
//! ### Color1 Array (`color1_ptr`)
//! - Type: `*mut u16`
//! - Contains the second color value for each BC2 block
//!
//! ### Indices Array (`indices_ptr`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC2 block

pub(crate) mod transform;
pub(crate) mod untransform;

/// Transform BC2 data from standard interleaved format to four separate arrays
/// (alpha, color0, color1, indices) using best known implementation for current CPU.
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
///
/// # Parameters
///
/// - `input_ptr`: Pointer to source BC2 block data
/// - `alpha_ptr`: Pointer to destination alpha array
/// - `color0_ptr`: Pointer to destination color0 array  
/// - `color1_ptr`: Pointer to destination color1 array
/// - `indices_ptr`: Pointer to destination indices array
/// - `block_count`: Number of BC2 blocks to process
///
/// # Returns
///
/// This function does not return a value. On successful completion, the input
/// BC2 blocks will have been split into four separate arrays.
///
/// # Remarks
///
/// This function automatically selects the best available SIMD implementation
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    alpha_ptr: *mut u64,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
) {
    transform::transform_with_split_colour(
        input_ptr,
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        block_count,
    );
}

/// Transform BC2 data from four separate arrays (alpha, color0, color1, indices) back to
/// standard interleaved format using best known implementation for current CPU.
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
///
/// # Parameters
///
/// - `alpha_ptr`: Pointer to source alpha array
/// - `color0_ptr`: Pointer to source color0 array
/// - `color1_ptr`: Pointer to source color1 array
/// - `indices_ptr`: Pointer to source indices array
/// - `output_ptr`: Pointer to destination BC2 block data
/// - `block_count`: Number of BC2 blocks to process
///
/// # Returns
///
/// This function does not return a value. On successful completion, the four
/// separate arrays will have been combined into standard BC2 block format.
///
/// # Remarks
///
/// This function automatically selects the best available SIMD implementation
/// for the current CPU architecture. The performance characteristics are
/// identical to [`transform_with_split_colour`] but in reverse.
///
/// This function is the exact inverse of [`transform_with_split_colour`].
/// Applying transform followed by untransform (or vice versa) should
/// produce the original data.
#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform::untransform_with_split_colour(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    );
}
