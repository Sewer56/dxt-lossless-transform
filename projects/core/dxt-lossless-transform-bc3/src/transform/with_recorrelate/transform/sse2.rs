use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// SSE2 implementation of BC3 transform with YCoCg-R decorrelation.
///
/// # Safety
///
/// - input_ptr must be valid for reads of num_blocks * 16 bytes
/// - alpha_endpoints_out must be valid for writes of num_blocks * 2 bytes
/// - alpha_indices_out must be valid for writes of num_blocks * 6 bytes
/// - colors_out must be valid for writes of num_blocks * 4 bytes
/// - color_indices_out must be valid for writes of num_blocks * 4 bytes
/// - decorrelation_mode must be a valid [`YCoCgVariant`]
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    // TODO: Implement SSE2 optimized version
    // For now, fall back to generic implementation
    super::generic::transform_with_decorrelate_generic(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
        decorrelation_mode,
    );
}
