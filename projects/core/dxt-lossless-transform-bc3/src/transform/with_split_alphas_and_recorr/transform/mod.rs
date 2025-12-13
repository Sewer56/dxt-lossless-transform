use dxt_lossless_transform_common::color_565::YCoCgVariant;

mod generic;

/// Transform BC3 data from standard interleaved format to separate alpha and decorrelated color arrays
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha0_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha1_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `decorrelated_colors_out` must be valid for writes of `block_count * 4` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
/// - `decorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn transform_with_split_alphas_and_recorr(
    input_ptr: *const u8,
    alpha0_out: *mut u8,
    alpha1_out: *mut u8,
    alpha_indices_out: *mut u16,
    decorrelated_colors_out: *mut u32,
    color_indices_out: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    generic::transform_with_split_alphas_and_recorr(
        input_ptr,
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        decorrelated_colors_out,
        color_indices_out,
        block_count,
        decorrelation_mode,
    );
}
