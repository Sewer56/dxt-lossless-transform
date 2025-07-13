mod generic;

/// Transform BC3 data from standard interleaved format to separate alpha and color arrays
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha0_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha1_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `color0_out` must be valid for writes of `block_count * 2` bytes
/// - `color1_out` must be valid for writes of `block_count * 2` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn transform_with_split_alphas_and_colour(
    input_ptr: *const u8,
    alpha0_out: *mut u8,
    alpha1_out: *mut u8,
    alpha_indices_out: *mut u16,
    color0_out: *mut u16,
    color1_out: *mut u16,
    color_indices_out: *mut u32,
    block_count: usize,
) {
    generic::transform_with_split_alphas_and_colour(
        input_ptr,
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        color0_out,
        color1_out,
        color_indices_out,
        block_count,
    );
}
