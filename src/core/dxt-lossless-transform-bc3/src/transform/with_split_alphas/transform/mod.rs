//! Split and separate BC3 blocks into alpha0, alpha1, alpha_indices, colors, and color_indices arrays using the best known implementation for the current CPU.
//!
//! For the inverse of `untransform_with_split_alphas`, see the corresponding untransform module.

pub(crate) mod generic;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub(crate) mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn transform_with_split_alphas_x86(
    input_ptr: *const u8,
    alpha0_out: *mut u8,
    alpha1_out: *mut u8,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    block_count: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::transform_with_split_alphas(
                input_ptr,
                alpha0_out,
                alpha1_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                block_count,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx2") {
            avx2::transform_with_split_alphas(
                input_ptr,
                alpha0_out,
                alpha1_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                block_count,
            );
            return;
        }
    }

    // Fallback to generic implementation
    generic::transform_with_split_alphas(
        input_ptr,
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        block_count,
    );
}

/// Split standard interleaved BC3 blocks into separate alpha0, alpha1, alpha_indices, colors, and color_indices buffers.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha0_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha1_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `colors_out` must be valid for writes of `block_count * 4` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
///
/// The buffers must not overlap.
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
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        transform_with_split_alphas_x86(
            input_ptr,
            alpha0_out,
            alpha1_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            block_count,
        );
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        generic::transform_with_split_alphas(
            input_ptr,
            alpha0_out,
            alpha1_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            block_count,
        );
    }
}
