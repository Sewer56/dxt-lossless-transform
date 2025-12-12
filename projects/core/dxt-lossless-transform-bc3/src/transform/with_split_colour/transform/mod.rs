//! Split and separate BC3 blocks into alpha_endpoints, alpha_indices, color0, color1, and color_indices arrays using the best known implementation for the current CPU.
//!
//! For the inverse of `untransform_with_split_colour`, see the corresponding untransform module.

mod generic;

/// Split standard interleaved BC3 blocks into separate alpha_endpoints, alpha_indices, color0, color1, and color_indices buffers.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_endpoints_out` must be valid for writes of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `color0_out` must be valid for writes of `block_count * 2` bytes
/// - `color1_out` must be valid for writes of `block_count * 2` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
///
/// The buffers must not overlap.
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    color0_out: *mut u16,
    color1_out: *mut u16,
    color_indices_out: *mut u32,
    block_count: usize,
) {
    generic::transform_with_split_colour(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        color0_out,
        color1_out,
        color_indices_out,
        block_count,
    );
}
