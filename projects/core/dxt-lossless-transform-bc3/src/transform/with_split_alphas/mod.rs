//! # BC3 Block Splitting Module with Split Alpha Endpoints
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
//! ### Alpha0 Array (`alpha0_out`)
//! - Type: `*mut u8`
//! - Contains the first alpha endpoint for each BC3 block (1 byte per block)
//!
//! ### Alpha1 Array (`alpha1_out`)
//! - Type: `*mut u8`
//! - Contains the second alpha endpoint for each BC3 block (1 byte per block)
//!
//! ### Alpha Indices Array (`alpha_indices_out`)
//! - Type: `*mut u16`
//! - Contains the alpha indices for each BC3 block (6 bytes per block)
//!
//! ### Colors Array (`colors_out`)
//! - Type: `*mut u32`
//! - Contains the color endpoints for each BC3 block (4 bytes per block)
//!
//! ### Color Indices Array (`color_indices_out`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC3 block

pub(crate) mod transform;
pub(crate) mod untransform;

/// Transform BC3 data from standard interleaved format to five separate arrays
/// (alpha0, alpha1, alpha_indices, colors, color_indices) using best known implementation for current CPU.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha0_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha1_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `colors_out` must be valid for writes of `block_count * 4` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
///
/// # Parameters
///
/// - `input_ptr`: Pointer to source BC3 block data
/// - `alpha0_out`: Pointer to destination alpha0 array
/// - `alpha1_out`: Pointer to destination alpha1 array  
/// - `alpha_indices_out`: Pointer to destination alpha indices array
/// - `colors_out`: Pointer to destination colors array
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
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn transform_with_split_alphas(
    input_ptr: *const u8,
    alpha0_out: *mut u8,
    alpha1_out: *mut u8,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    block_count: usize,
) {
    transform::transform_with_split_alphas(
        input_ptr,
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        block_count,
    );
}

/// Transform BC3 data from five separate arrays (alpha0, alpha1, alpha_indices, colors, color_indices) back to
/// standard interleaved format using best known implementation for current CPU.
///
/// # Safety
///
/// - `alpha0_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha1_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `colors_out` must be valid for reads of `block_count * 4` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
///
/// # Parameters
///
/// - `alpha0_out`: Pointer to source alpha0 array
/// - `alpha1_out`: Pointer to source alpha1 array
/// - `alpha_indices_out`: Pointer to source alpha indices array
/// - `colors_out`: Pointer to source colors array
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
/// identical to [`transform_with_split_alphas`] but in reverse.
///
/// This function is the exact inverse of [`transform_with_split_alphas`].
/// Applying transform followed by untransform (or vice versa) should
/// produce the original data.
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn untransform_with_split_alphas(
    alpha0_out: *const u8,
    alpha1_out: *const u8,
    alpha_indices_out: *const u16,
    colors_out: *const u32,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform::untransform_with_split_alphas(
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        output_ptr,
        block_count,
    );
}
