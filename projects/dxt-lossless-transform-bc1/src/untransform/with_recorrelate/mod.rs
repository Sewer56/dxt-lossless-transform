//! # Unsplit and Recorrelate BC1 Blocks Module
//!
//! This module provides optimized functions for transforming BC1 data from split format
//! (colors separated from indices) back to standard interleaved format while simultaneously
//! applying YCoCg recorrelation to the color endpoints. This combines two operations into
//! a single optimized pass for improved performance.
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

use dxt_lossless_transform_common::color_565::YCoCgVariant;

#[cfg(not(feature = "no-runtime-cpu-detection"))]
use dxt_lossless_transform_common::cpu_detect::*;

pub mod generic;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[inline(always)]
pub(crate) unsafe fn untransform_with_recorrelate(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    recorrelation_mode: YCoCgVariant,
) {
    debug_assert!(len % 8 == 0);

    unsafe {
        // Setup pointers
        let colors_ptr = input_ptr as *const u32;
        let indices_ptr = input_ptr.add(len / 2) as *const u32;
        let num_blocks = len / 8; // Each BC1 block is 8 bytes

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            untransform_with_recorrelate_x86(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                recorrelation_mode,
            );
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            generic::untransform_with_recorrelate_generic(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                recorrelation_mode,
            );
        }
    }
}

pub unsafe fn untransform_with_recorrelate_x86(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::untransform_with_recorrelate(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if has_avx2() {
            avx2::untransform_with_recorrelate(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if has_sse2() {
            sse2::untransform_with_recorrelate(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_with_recorrelate(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if cfg!(target_feature = "avx2") {
            avx2::untransform_with_recorrelate(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }

        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        if cfg!(target_feature = "sse2") {
            sse2::untransform_with_recorrelate(
                colors_ptr,
                indices_ptr,
                output_ptr,
                num_blocks,
                decorrelation_mode,
            );
            return;
        }
    }

    // Fallback to portable implementation
    generic::untransform_with_recorrelate_generic(
        colors_ptr,
        indices_ptr,
        output_ptr,
        num_blocks,
        decorrelation_mode,
    );
}
