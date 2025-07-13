use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-alphas untransform for BC3.
/// Combines separate arrays of alpha0, alpha1, alpha_indices, colors and color_indices back into standard interleaved BC3 blocks.
///
/// # Safety
///
/// - `alpha0_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha1_out` must be valid for reads of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `colors_out` must be valid for reads of `block_count * 4` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
#[inline]
pub(crate) unsafe fn untransform_with_split_alphas(
    mut alpha0_ptr: *const u8,
    mut alpha1_ptr: *const u8,
    mut alpha_indices_ptr: *const u16,
    mut colors_ptr: *const u32,
    mut color_indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Calculate end pointer for alpha0 (as it's used for loop termination)
    let alpha0_ptr_end = alpha0_ptr.add(block_count);

    while alpha0_ptr < alpha0_ptr_end {
        // Read the split values
        let alpha0 = alpha0_ptr.read();
        let alpha1 = alpha1_ptr.read();

        // Read alpha indices (6 bytes) as u16 + u32
        let alpha_indices_part1 = alpha_indices_ptr.read_u16_at(0);
        let alpha_indices_part2 = alpha_indices_ptr.read_u32_at(2);

        let colors = colors_ptr.read_u32_at(0);
        let color_indices = color_indices_ptr.read_u32_at(0);

        // Write BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]
        output_ptr.write(alpha0);
        output_ptr.add(1).write(alpha1);

        // Write alpha indices (6 bytes) as u16 + u32
        output_ptr.write_u16_at(2, alpha_indices_part1);
        output_ptr.write_u32_at(4, alpha_indices_part2);

        output_ptr.write_u32_at(8, colors);
        output_ptr.write_u32_at(12, color_indices);

        // Advance all pointers
        alpha0_ptr = alpha0_ptr.add(1);
        alpha1_ptr = alpha1_ptr.add(1);
        alpha_indices_ptr = alpha_indices_ptr.add(3); // 6 bytes = 3 u16s
        colors_ptr = colors_ptr.add(1);
        color_indices_ptr = color_indices_ptr.add(1);
        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::untransform_with_split_alphas;
    use crate::test_prelude::*;

    #[test]
    fn can_untransform_unaligned() {
        // 1 block processed per iteration (no SIMD) (* 2 == 2)
        run_with_split_alphas_untransform_unaligned_test(
            untransform_with_split_alphas,
            2,
            "untransform_with_split_alphas (generic, unaligned)",
        );
    }
}
