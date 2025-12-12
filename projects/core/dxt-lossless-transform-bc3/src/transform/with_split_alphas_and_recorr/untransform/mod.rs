use dxt_lossless_transform_common::color_565::YCoCgVariant;

mod generic;

/// Untransform BC3 data from separate alpha and decorrelated color arrays back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - `alpha0_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha1_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `decorrelated_colors_out` must be valid for reads of `block_count * 4` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn untransform_with_split_alphas_and_recorr(
    alpha0_out: *const u8,
    alpha1_out: *const u8,
    alpha_indices_out: *const u16,
    decorrelated_colors_out: *const u32,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    generic::untransform_with_split_alphas_and_recorr(
        alpha0_out,
        alpha1_out,
        alpha_indices_out,
        decorrelated_colors_out,
        color_indices_out,
        output_ptr,
        block_count,
        recorrelation_mode,
    );
}
