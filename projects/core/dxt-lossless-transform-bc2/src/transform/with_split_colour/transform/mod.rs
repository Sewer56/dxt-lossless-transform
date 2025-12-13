//! Split and separate BC2 blocks into alpha, color0, color1, and indices arrays using the best known implementation for the current CPU.
//!
//! For the inverse of `untransform_with_split_colour`, see the corresponding untransform module.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512bw;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

mod generic;

/// Split standard interleaved BC2 blocks into separate alpha, color0, color1, and index buffers.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_ptr` must be valid for writes of `block_count * 8` bytes
/// - `color0_ptr` must be valid for writes of `block_count * 2` bytes
/// - `color1_ptr` must be valid for writes of `block_count * 2` bytes
/// - `indices_ptr` must be valid for writes of `block_count * 4` bytes
///
/// The buffers must not overlap.
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    alpha_ptr: *mut u64,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        transform_with_split_colour_x86(
            input_ptr,
            alpha_ptr,
            color0_ptr,
            color1_ptr,
            indices_ptr,
            block_count,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::transform_with_split_colour(
            input_ptr,
            alpha_ptr,
            color0_ptr,
            color1_ptr,
            indices_ptr,
            block_count,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn transform_with_split_colour_x86(
    input_ptr: *const u8,
    alpha_ptr: *mut u64,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if has_avx512bw() {
            avx512bw::transform_with_split_colour(
                input_ptr,
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
            );
            return;
        }

        if has_avx2() {
            avx2::transform_with_split_colour(
                input_ptr,
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
            );
            return;
        }

        if has_sse2() {
            sse2::transform_with_split_colour(
                input_ptr,
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx512bw") {
            avx512bw::transform_with_split_colour(
                input_ptr,
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::transform_with_split_colour(
                input_ptr,
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::transform_with_split_colour(
                input_ptr,
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                block_count,
            );
            return;
        }
    }

    generic::transform_with_split_colour(
        input_ptr,
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        block_count,
    );
}
