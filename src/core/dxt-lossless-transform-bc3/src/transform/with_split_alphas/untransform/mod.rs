//! Combine separate BC3 arrays (alpha0, alpha1, alpha_indices, colors, color_indices) back into standard interleaved format using the best known implementation for the current CPU.
//!
//! For the inverse of `transform_with_split_alphas`, see the corresponding transform module.

pub(crate) mod generic;

#[cfg(target_arch = "x86_64")]
pub(crate) mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn untransform_with_split_alphas_x86(
    alpha0_ptr: *const u8,
    alpha1_ptr: *const u8,
    alpha_indices_ptr: *const u16,
    colors_ptr: *const u32,
    color_indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    // SSE2 is required by x86-64, so no runtime check needed.
    // On i686, SSE2 is slower, so use generic fallback.
    #[cfg(target_arch = "x86_64")]
    {
        sse2::untransform_with_split_alphas_sse2(
            alpha0_ptr,
            alpha1_ptr,
            alpha_indices_ptr,
            colors_ptr,
            color_indices_ptr,
            output_ptr,
            block_count,
        );
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        generic::untransform_with_split_alphas(
            alpha0_ptr,
            alpha1_ptr,
            alpha_indices_ptr,
            colors_ptr,
            color_indices_ptr,
            output_ptr,
            block_count,
        );
    }
}

/// Combine separate arrays of alpha0, alpha1, alpha_indices, colors, and color_indices back into standard interleaved BC3 blocks.
///
/// # Safety
///
/// - `alpha0_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha1_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `colors_out` must be valid for reads of `block_count * 4` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
///
/// The buffers must not overlap.
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
    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        untransform_with_split_alphas_x86(
            alpha0_out,
            alpha1_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            output_ptr,
            block_count,
        );
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        generic::untransform_with_split_alphas(
            alpha0_out,
            alpha1_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            output_ptr,
            block_count,
        );
    }
}
