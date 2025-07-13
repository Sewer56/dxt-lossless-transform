//! Combine separate BC3 arrays (alpha0, alpha1, alpha_indices, colors, color_indices) back into standard interleaved format using the best known implementation for the current CPU.
//!
//! For the inverse of `transform_with_split_alphas`, see the corresponding transform module.

mod generic;

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
#[allow(dead_code)]
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
    // For now, we only have the generic implementation
    // In the future, SIMD implementations can be added following the same pattern as BC2
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
