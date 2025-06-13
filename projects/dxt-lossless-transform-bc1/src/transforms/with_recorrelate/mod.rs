//! # Unsplit and Recorrelate BC1 Blocks Module
//!
//! This module provides optimized functions for transforming BC1 data from split format
//! (colors separated from indices) back to standard interleaved format while simultaneously
//! applying YCoCg recorrelation to the color endpoints. This combines two operations into
//! a single optimized pass for improved performance.
//!
//! Below is a description of the detransformation process.
//! For transformation, swap the `output` and `input`.
//!
//! ## Input Format
//!
//! The module expects BC1 data in split format:
//!
//! ### Colors Section (`input_ptr`)
//! - Type: `*const u8` (interpreted as `*const u32`)
//! - First half of input data
//! - Contains color endpoints: 4 bytes per block (2× RGB565 values)
//! - Each u32 contains two [`Color565`] values packed as: `color1 << 16 | color0`
//!
//! ### Indices Section (`input_ptr + len/2`)
//! - Type: `*const u8` (interpreted as `*const u32`)
//! - Second half of input data
//! - Contains color indices: 4 bytes per block (16× 2-bit indices)
//!
//! ## Output Format
//!
//! ### BC1 Blocks (`output_ptr`)
//! - Type: `*mut u8`
//! - Contains standard BC1/DXT1 compressed texture blocks with recorrelated colors
//! - Each block is 8 bytes in the following format:
//!   ```ignore
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 2    | color0 (RGB565, after YCoCg recorrelation, little-endian)
//!   2      | 2    | color1 (RGB565, after YCoCg recorrelation, little-endian)  
//!   4      | 4    | indices (2 bits per pixel, unchanged, little-endian)
//!   ```
//!
//! ## YCoCg Recorrelation Variants
//!
//! The module supports three YCoCg recorrelation variants specified by [`YCoCgVariant`]:
//! - [`YCoCgVariant::Variant1`]: Standard YCoCg recorrelation
//! - [`YCoCgVariant::Variant2`]: Alternative YCoCg recorrelation formula
//! - [`YCoCgVariant::Variant3`]: Third YCoCg recorrelation variant
//!
//! Each variant applies a different mathematical transformation to improve compression ratios
//! by decorrelating the color channels in the YCoCg color space.
//!
//! [`YCoCgVariant::Variant1`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1
//! [`YCoCgVariant::Variant2`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2
//! [`YCoCgVariant::Variant3`]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3
//! [`YCoCgVariant`]: dxt_lossless_transform_common::color_565::YCoCgVariant
//! [`Color565`]: dxt_lossless_transform_common::color_565::Color565

pub mod transform;
pub mod untransform;

use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Transform BC1 data from standard interleaved format to separated color/index format
/// while applying YCoCg decorrelation using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    decorrelation_mode: YCoCgVariant,
) {
    transform::transform_with_decorrelate(input_ptr, output_ptr, len, decorrelation_mode);
}

/// Transform BC1 data from separated color/index format back to standard interleaved format
/// while applying YCoCg recorrelation using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_with_recorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    recorrelation_mode: YCoCgVariant,
) {
    untransform::untransform_with_recorrelate(input_ptr, output_ptr, len, recorrelation_mode);
}
