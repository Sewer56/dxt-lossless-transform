use dxt_lossless_transform_common::color_565::YCoCgVariant;

mod generic;

/// Untransform BC3 data from separate alpha endpoints and decorrelated color arrays back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - `alpha_endpoints_out` must be valid for reads of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `decorrelated_color0_out` must be valid for reads of `block_count * 2` bytes
/// - `decorrelated_color1_out` must be valid for reads of `block_count * 2` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
/// - `recorrelation_mode` must be a valid [`YCoCgVariant`] (not [`YCoCgVariant::None`])
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_endpoints_out: *const u16,
    alpha_indices_out: *const u16,
    decorrelated_color0_out: *const u16,
    decorrelated_color1_out: *const u16,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    generic::untransform_with_split_colour_and_recorr(
        alpha_endpoints_out,
        alpha_indices_out,
        decorrelated_color0_out,
        decorrelated_color1_out,
        color_indices_out,
        output_ptr,
        block_count,
        recorrelation_mode,
    );
}
