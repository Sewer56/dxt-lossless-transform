//! # Unsplit Split Colour-Split Blocks and Decorrelate Module
//!
//! This module provides optimized functions for combining split color data, applying decorrelation,
//! and unsplitting block indices back into standard BC1 (DXT1) compressed texture blocks in a single
//! optimized step. This eliminates the need for intermediate memory copies by performing both
//! decorrelation and unsplitting operations directly.
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

use dxt_lossless_transform_common::color_565::YCoCgVariant;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

mod generic;

pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        untransform_with_split_colour_and_recorr_x86(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
            decorrelation_mode,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::untransform_with_split_colour_and_recorr_generic(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
            decorrelation_mode,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe fn untransform_with_split_colour_and_recorr_x86(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() {
            avx512::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if has_avx2() {
            avx2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if has_sse2() {
            sse2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::untransform_with_split_colour_and_recorr(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
    }

    generic::untransform_with_split_colour_and_recorr_generic(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
        decorrelation_mode,
    );
}
