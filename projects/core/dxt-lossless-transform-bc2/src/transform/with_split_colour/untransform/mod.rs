//! Combine split BC2 data back into standard interleaved format using the best known implementation for the current CPU.

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx2;
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse2;

mod generic;

/// Combine separate alpha, color0, color1, and index buffers back into standard interleaved BC2 blocks.
///
/// # Safety
///
/// - `alpha_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for reads of `block_count * 2` bytes
/// - `color1_ptr` must be valid for reads of `block_count * 2` bytes
/// - `indices_ptr` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
///
/// The buffers must not overlap.
#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        untransform_with_split_colour_x86(
            alpha_ptr,
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        generic::untransform_with_split_colour(
            alpha_ptr,
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        );
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn untransform_with_split_colour_x86(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    use dxt_lossless_transform_common::cpu_detect::*;

    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() && has_avx512bw() {
            avx512::untransform_with_split_colour(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if has_avx2() {
            avx2::untransform_with_split_colour(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if has_sse2() {
            sse2::untransform_with_split_colour(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") && cfg!(target_feature = "avx512bw") {
            avx512::untransform_with_split_colour(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::untransform_with_split_colour(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::untransform_with_split_colour(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }
    }

    generic::untransform_with_split_colour(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    );
}
