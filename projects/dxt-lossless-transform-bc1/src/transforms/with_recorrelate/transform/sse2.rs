use crate::transforms::with_recorrelate::transform::generic;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// SSE2 stub for transform with YCoCg-R decorrelation.
/// Falls back to generic implementation.
#[inline(always)]
pub(crate) unsafe fn transform_with_recorrelate(
    input_ptr: *const u8,
    colours_ptr: *mut u32,
    indices_ptr: *mut u32,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    generic::transform_with_recorrelate_generic(
        input_ptr,
        colours_ptr,
        indices_ptr,
        num_blocks,
        decorrelation_mode,
    );
}
