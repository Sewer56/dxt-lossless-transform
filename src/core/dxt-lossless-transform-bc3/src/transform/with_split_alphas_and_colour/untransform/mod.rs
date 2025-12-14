mod generic;

/// Untransform BC3 data from separate alpha and color arrays back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - `alpha0_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha1_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `color0_out` must be valid for reads of `block_count * 2` bytes
/// - `color1_out` must be valid for reads of `block_count * 2` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn untransform_with_split_alphas_and_colour(
    alpha0_out: *const u8,
    alpha1_out: *const u8,
    alpha_indices_out: *const u16,
    color0_out: *const u16,
    color1_out: *const u16,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    generic::untransform_with_split_alphas_and_colour(
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        color0_out,
        color1_out,
        color_indices_out,
        output_ptr,
        block_count,
    );
}
