use crate::utils::combine_u16_pair_to_u32;
use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-alphas and split-colour untransform for BC3.
/// Combines separate arrays of alpha0, alpha1, alpha_indices, color0, color1 and
/// color_indices back into standard interleaved BC3 blocks.
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
    mut alpha0_out: *const u8,
    mut alpha1_out: *const u8,
    mut alpha_indices_out: *const u16,
    mut color0_out: *const u16,
    mut color1_out: *const u16,
    mut color_indices_out: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Process each block
    let alpha0_end = alpha0_out.add(block_count);
    while alpha0_out < alpha0_end {
        // Read from separate arrays
        let alpha0 = alpha0_out.read();
        let alpha1 = alpha1_out.read();

        // Read alpha indices (6 bytes) optimized with combined reads
        let alpha_indices_part1 = alpha_indices_out.read_u16_at(0);
        let alpha_indices_part2 = alpha_indices_out.read_u32_at(2);

        let color0 = color0_out.read_u16_at(0);
        let color1 = color1_out.read_u16_at(0);
        let color_indices = color_indices_out.read_u32_at(0);

        // Write BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]
        output_ptr.write(alpha0);
        output_ptr.add(1).write(alpha1);

        // Write alpha indices (6 bytes) optimized with combined writes
        output_ptr.write_u16_at(2, alpha_indices_part1);
        output_ptr.write_u32_at(4, alpha_indices_part2);

        let colors_combined = combine_u16_pair_to_u32(color0, color1);
        output_ptr.write_u32_at(8, colors_combined);
        output_ptr.write_u32_at(12, color_indices);

        // Advance all pointers
        alpha0_out = alpha0_out.add(1);
        alpha1_out = alpha1_out.add(1);
        alpha_indices_out = alpha_indices_out.add(3); // 6 bytes = 3 u16
        color0_out = color0_out.add(1);
        color1_out = color1_out.add(1);
        color_indices_out = color_indices_out.add(1);
        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[test]
    fn can_untransform_unaligned() {
        // 1 block processed per iteration (no SIMD) (* 2 == 2)
        run_with_split_alphas_and_colour_untransform_unaligned_test(
            untransform_with_split_alphas_and_colour,
            2,
            "untransform_with_split_alphas_and_colour (generic, unaligned)",
        );
    }
}
