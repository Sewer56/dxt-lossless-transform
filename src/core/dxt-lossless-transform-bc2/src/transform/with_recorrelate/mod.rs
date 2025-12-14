//! # BC2 Block Splitting and Decorrelation Module
//!
//! This module provides optimized functions for separating BC2 data into two distinct arrays
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
//!
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
//! The module outputs two separate arrays:
//!
//! ### Alpha Array (`alpha_ptr`)
//! - Type: `*mut u64`
//! - Contains the alpha data for each BC2 block (8 bytes per block)
//!
//! ### Colors Array (`color_ptr`)
//! - Type: `*mut u32`
//! - Contains the color endpoints for each BC2 block (4 bytes per block)
//!
//! ### Indices Array (`indices_ptr`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC2 block
//!
//! ## YCoCg Decorrelation Variants
//!
//! The module supports three YCoCg decorrelation variants specified by [`YCoCgVariant`]:
//! - [`YCoCgVariant::Variant1`]: Standard YCoCg decorrelation
//! - [`YCoCgVariant::Variant2`]: Alternative YCoCg decorrelation formula
//! - [`YCoCgVariant::Variant3`]: Third YCoCg decorrelation variant
//!
//! Each variant applies a different mathematical transformation to improve compression ratios
//! by decorrelating the color channels in the YCoCg color space. The alpha channel (bytes 0-7)
//! is always preserved unchanged as it's already optimally stored.
//!
//! [`YCoCgVariant::Variant1`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1
//! [`YCoCgVariant::Variant2`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2
//! [`YCoCgVariant::Variant3`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3
//! [`YCoCgVariant`]: dxt_lossless_transform_common::color_565::YCoCgVariant
//! [`Color565`]: dxt_lossless_transform_common::color_565::Color565

pub mod transform;
pub mod untransform;

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Transform BC2 data from standard interleaved format to separated alpha/color/index format
/// while applying YCoCg decorrelation using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    decorrelation_mode: YCoCgVariant,
) {
    transform::transform_with_decorrelate(input_ptr, output_ptr, len, decorrelation_mode);
}

/// Transform BC2 data from separated alpha/color/index format back to standard interleaved format
/// while applying YCoCg recorrelation using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    recorrelation_mode: YCoCgVariant,
) {
    untransform::untransform_with_recorrelate(input_ptr, output_ptr, len, recorrelation_mode);
}
