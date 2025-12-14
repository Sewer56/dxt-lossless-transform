use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// AVX512 implementation of BC3 untransform with YCoCg-R recorrelation.
///
/// # Safety
///
/// - alpha_endpoints_in must be valid for reads of num_blocks * 2 bytes
/// - alpha_indices_in must be valid for reads of num_blocks * 6 bytes
/// - colors_in must be valid for reads of num_blocks * 4 bytes
/// - color_indices_in must be valid for reads of num_blocks * 4 bytes
/// - output_ptr must be valid for writes of num_blocks * 16 bytes
/// - recorrelation_mode must be a valid [`YCoCgVariant`]
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    // TODO: Implement AVX512 optimized version
    // For now, fall back to generic implementation
    super::generic::untransform_with_recorrelate_generic(
        alpha_endpoints_in,
        alpha_indices_in,
        colors_in,
        color_indices_in,
        output_ptr,
        num_blocks,
        recorrelation_mode,
    );
}
