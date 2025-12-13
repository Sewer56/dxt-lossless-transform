//! # BC3 Block Splitting with Split Color Endpoints and YCoCg Decorrelation Module
//!
//! This module provides optimized functions for separating BC3 data into five distinct arrays
//! while applying both color endpoint splitting and YCoCg decorrelation to color endpoints
//! in a single optimized step. This combines the benefits of [`with_split_colour`] and
//! [`with_recorrelate`] optimizations.
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
//! ### Decorrelated Color0 Array (`decorrelated_color0_out`)
//! - Type: `*mut u16`
//! - Contains the first YCoCg decorrelated color endpoint for each BC3 block (2 bytes per block)
//!
//! ### Decorrelated Color1 Array (`decorrelated_color1_out`)
//! - Type: `*mut u16`
//! - Contains the second YCoCg decorrelated color endpoint for each BC3 block (2 bytes per block)
//!
//! ### Color Indices Array (`color_indices_out`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC3 block (4 bytes per block)
//!
//! ## YCoCg Decorrelation Variants
//!
//! The module supports three YCoCg decorrelation variants specified by [`YCoCgVariant`]:
//! - [`YCoCgVariant::Variant1`]: Standard YCoCg decorrelation
//! - [`YCoCgVariant::Variant2`]: Alternative YCoCg decorrelation formula
//! - [`YCoCgVariant::Variant3`]: Third YCoCg decorrelation variant
//!
//! Each variant applies a different mathematical transformation to improve compression ratios
//! by decorrelating the color channels in the YCoCg color space. The alpha endpoints
//! are kept together as they are already efficiently packed, while the color endpoints
//! are split and decorrelated for optimal compression.
//!
//! [`with_split_colour`]: crate::transform::with_split_colour
//! [`with_recorrelate`]: crate::transform::with_recorrelate
//! [`YCoCgVariant::Variant1`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1
//! [`YCoCgVariant::Variant2`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2
//! [`YCoCgVariant::Variant3`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3
//! [`YCoCgVariant`]: dxt_lossless_transform_common::color_565::YCoCgVariant

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Transform operations that combine color endpoint splitting with YCoCg decorrelation
pub mod transform;
/// Untransform operations that combine color endpoint merging with YCoCg recorrelation
pub mod untransform;

/// Transform BC3 data from standard interleaved format to five separate arrays
/// (alpha_endpoints, alpha_indices, decorrelated_color0, decorrelated_color1, color_indices)
/// while applying YCoCg decorrelation using best known implementation for current CPU.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_endpoints_out` must be valid for writes of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `decorrelated_color0_out` must be valid for writes of `block_count * 2` bytes
/// - `decorrelated_color1_out` must be valid for writes of `block_count * 2` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
/// - `decorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
///
/// # Parameters
///
/// - `input_ptr`: Pointer to source BC3 block data
/// - `alpha_endpoints_out`: Pointer to destination alpha endpoints array
/// - `alpha_indices_out`: Pointer to destination alpha indices array
/// - `decorrelated_color0_out`: Pointer to destination decorrelated color0 array
/// - `decorrelated_color1_out`: Pointer to destination decorrelated color1 array
/// - `color_indices_out`: Pointer to destination color indices array
/// - `block_count`: Number of BC3 blocks to process
/// - `decorrelation_mode`: YCoCg decorrelation variant to apply to color endpoints
///
/// # Returns
///
/// This function does not return a value. On successful completion, the input
/// BC3 blocks will have been split into five separate arrays with split and
/// decorrelated color endpoints.
///
/// # Remarks
///
/// This function automatically selects the best available SIMD implementation
/// for the current CPU architecture. The color endpoints are split into separate
/// arrays and decorrelated using the specified YCoCg variant, while alpha data
/// remains efficiently packed, providing optimal compression efficiency.
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    decorrelated_color0_out: *mut u16,
    decorrelated_color1_out: *mut u16,
    color_indices_out: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    transform::transform_with_split_colour_and_recorr(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        decorrelated_color0_out,
        decorrelated_color1_out,
        color_indices_out,
        block_count,
        decorrelation_mode,
    );
}

/// Transform BC3 data from five separate arrays (alpha_endpoints, alpha_indices,
/// decorrelated_color0, decorrelated_color1, color_indices) back to standard
/// interleaved format while applying YCoCg recorrelation using best known
/// implementation for current CPU.
///
/// # Safety
///
/// - `alpha_endpoints_out` must be valid for reads of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `decorrelated_color0_out` must be valid for reads of `block_count * 2` bytes
/// - `decorrelated_color1_out` must be valid for reads of `block_count * 2` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - All buffers must not overlap
/// - `block_count` must not cause integer overflow when calculating buffer sizes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
///
/// # Parameters
///
/// - `alpha_endpoints_out`: Pointer to source alpha endpoints array
/// - `alpha_indices_out`: Pointer to source alpha indices array
/// - `decorrelated_color0_out`: Pointer to source decorrelated color0 array
/// - `decorrelated_color1_out`: Pointer to source decorrelated color1 array
/// - `color_indices_out`: Pointer to source color indices array
/// - `output_ptr`: Pointer to destination BC3 block data
/// - `block_count`: Number of BC3 blocks to process
/// - `recorrelation_mode`: YCoCg recorrelation variant to apply to color endpoints
///
/// # Returns
///
/// This function does not return a value. On successful completion, the five
/// separate arrays will have been combined into standard BC3 block format with
/// merged and recorrelated color endpoints.
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
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_endpoints_out: *const u16,
    alpha_indices_out: *const u16,
    decorrelated_color0_out: *const u16,
    decorrelated_color1_out: *const u16,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    untransform::untransform_with_split_colour_and_recorr(
        alpha_endpoints_out,
        alpha_indices_out,
        decorrelated_color0_out,
        decorrelated_color1_out,
        color_indices_out,
        output_ptr,
        block_count,
        recorrelation_mode,
    );
}
