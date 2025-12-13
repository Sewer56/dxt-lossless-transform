//! # Split BC3 Blocks Module
//!
//! This module provides optimized functions for separating BC3 data into five distinct arrays
//! for better compression efficiency by grouping similar data together.
//!
//! Below is a description of the untransformation process.
//! For transformation, swap the `output` and `input`.
//!
//! ## Input Format
//!
//! The module expects BC3 blocks in standard interleaved format:
//!
//! ### BC3 Blocks (`input_ptr`)
//! - Type: `*const u8`
//! - Contains standard BC3/DXT5 compressed texture blocks
//! - Each block is 16 bytes in the following format:
//!   ```ignore
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 1    | alpha0 (first alpha endpoint for interpolation)
//!   1      | 1    | alpha1 (second alpha endpoint for interpolation)
//!   2      | 6    | alpha indices (16x 3-bit indices for alpha interpolation)
//!   8      | 2    | color0 (RGB565, little-endian)
//!   10     | 2    | color1 (RGB565, little-endian)
//!   12     | 4    | color indices (2 bits per pixel, little-endian)
//!   ```
//!
//! ## Output Format
//!
//! The module outputs five separate arrays:
//!
//! ### Alpha Endpoints Array (`alpha_endpoints_out`)
//! - Type: `*mut u16`
//! - Contains the alpha endpoints for each BC3 block (2 bytes per block)
//!
//! ### Alpha Indices Array (`alpha_indices_out`)
//! - Type: `*mut u16`
//! - Contains the alpha indices for each BC3 block (6 bytes per block)
//!
//! ### Color0 Array (`color0_out`)
//! - Type: `*mut u16`
//! - Contains the first color endpoint for each BC3 block
//!
//! ### Color1 Array (`color1_out`)
//! - Type: `*mut u16`
//! - Contains the second color endpoint for each BC3 block
//!
//! ### Color Indices Array (`color_indices_out`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC3 block

pub(crate) mod transform;
pub(crate) mod untransform;

/// Transform BC3 data from standard interleaved format to five separate arrays
/// (alpha_endpoints, alpha_indices, color0, color1, color_indices) using best known implementation for current CPU.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_endpoints_out` must be valid for writes of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `color0_out` must be valid for writes of `block_count * 2` bytes
/// - `color1_out` must be valid for writes of `block_count * 2` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
///
/// # Parameters
///
/// - `input_ptr`: Pointer to source BC3 block data
/// - `alpha_endpoints_out`: Pointer to destination alpha endpoints array
/// - `alpha_indices_out`: Pointer to destination alpha indices array
/// - `color0_out`: Pointer to destination color0 array  
/// - `color1_out`: Pointer to destination color1 array
/// - `color_indices_out`: Pointer to destination color indices array
/// - `block_count`: Number of BC3 blocks to process
///
/// # Returns
///
/// This function does not return a value. On successful completion, the input
/// BC3 blocks will have been split into five separate arrays.
///
/// # Remarks
///
/// This function automatically selects the best available SIMD implementation
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    color0_out: *mut u16,
    color1_out: *mut u16,
    color_indices_out: *mut u32,
    block_count: usize,
) {
    transform::transform_with_split_colour(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        color0_out,
        color1_out,
        color_indices_out,
        block_count,
    );
}

/// Transform BC3 data from five separate arrays (alpha_endpoints, alpha_indices, color0, color1, color_indices) back to
/// standard interleaved format using best known implementation for current CPU.
///
/// # Safety
///
/// - `alpha_endpoints_out` must be valid for reads of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `color0_out` must be valid for reads of `block_count * 2` bytes
/// - `color1_out` must be valid for reads of `block_count * 2` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
///
/// # Parameters
///
/// - `alpha_endpoints_out`: Pointer to source alpha endpoints array
/// - `alpha_indices_out`: Pointer to source alpha indices array
/// - `color0_out`: Pointer to source color0 array
/// - `color1_out`: Pointer to source color1 array
/// - `color_indices_out`: Pointer to source color indices array
/// - `output_ptr`: Pointer to destination BC3 block data
/// - `block_count`: Number of BC3 blocks to process
///
/// # Returns
///
/// This function does not return a value. On successful completion, the five
/// separate arrays will have been combined into standard BC3 block format.
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
    alpha_endpoints_out: *const u16,
    alpha_indices_out: *const u16,
    color0_out: *const u16,
    color1_out: *const u16,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform::untransform_with_split_colour(
        alpha_endpoints_out,
        alpha_indices_out,
        color0_out,
        color1_out,
        color_indices_out,
        output_ptr,
        block_count,
    );
}
