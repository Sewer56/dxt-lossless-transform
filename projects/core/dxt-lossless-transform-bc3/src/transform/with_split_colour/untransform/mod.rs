//! Combine separate BC3 arrays into standard interleaved blocks using the best known implementation for the current CPU.
//!
//! For the inverse of `transform_with_split_colour`, see the corresponding transform module.

mod generic;

/// Combine separate alpha_endpoints, alpha_indices, color0, color1, and color_indices buffers into standard interleaved BC3 blocks.
///
/// # Safety
///
/// - `alpha_endpoints_out` must be valid for reads of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `color0_out` must be valid for reads of `block_count * 2` bytes
/// - `color1_out` must be valid for reads of `block_count * 2` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
///
/// The buffers must not overlap.
#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    alpha_endpoints_out: *const u16,
    alpha_indices_out: *const u16,
    color0_out: *const u16,
    color1_out: *const u16,
    color_indices_out: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    generic::untransform_with_split_colour(
        alpha_endpoints_out,
        alpha_indices_out,
        color0_out,
        color1_out,
        color_indices_out,
        output_ptr,
        block_count,
    );
}
