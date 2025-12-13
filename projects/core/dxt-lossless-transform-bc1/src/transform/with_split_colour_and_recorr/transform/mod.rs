//! Split colour **and** decorrelate transform
//!
//! Takes standard interleaved BC1 blocks (`input_ptr`) and produces three separate
//! arrays: `color0_ptr`, `color1_ptr` (decorrelated [`Color565`] endpoints) and
//! `indices_ptr` (packed 2-bit indices).
//!
//! This module selects the best available implementation for the current CPU at
//! runtime (unless `no-runtime-cpu-detection` feature is enabled).
//!
//! [`Color565`]: dxt_lossless_transform_common::color_565::Color565

use dxt_lossless_transform_common::color_565::YCoCgVariant;
mod generic;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

/// Split-colour transform with YCoCg-R decorrelation using the best known
/// implementation for the current CPU.
///
/// # Safety
/// Refer to individual backend implementations for exact requirements. In
/// general:
/// - `input_ptr` must be valid for `block_count*8` bytes of reads.
/// - `color0_out` must be valid for `block_count*2` bytes of writes.
/// - `color1_out` must be valid for `block_count*2` bytes of writes.
/// - `indices_out` must be valid for `block_count*4` bytes of writes.
#[inline]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    debug_assert!(decorrelation_mode != YCoCgVariant::None);

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        transform_with_split_colour_and_recorr_x86(
            input_ptr,
            color0_out,
            color1_out,
            indices_out,
            block_count,
            decorrelation_mode,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::transform_with_split_colour_and_decorr_generic(
            input_ptr,
            color0_out,
            color1_out,
            indices_out,
            block_count,
            decorrelation_mode,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn transform_with_split_colour_and_recorr_x86(
    input_ptr: *const u8,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if has_avx512f() && has_avx512bw() {
            avx512::transform_with_split_colour_and_decorr(
                input_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if has_avx2() {
            avx2::transform_with_split_colour_and_decorr(
                input_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
        if has_sse2() {
            sse2::transform_with_split_colour_and_decorr(
                input_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::transform_with_split_colour_and_decorr(
                input_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::transform_with_split_colour_and_decorr(
                input_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
        if cfg!(target_feature = "sse2") {
            sse2::transform_with_split_colour_and_decorr(
                input_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
                decorrelation_mode,
            );
            return;
        }
    }

    // Fallback
    generic::transform_with_split_colour_and_decorr_generic(
        input_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        block_count,
        decorrelation_mode,
    );
}
