use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-colour untransform for BC3.
/// Combines separate arrays of alpha_endpoints, alpha_indices, color0, color1 and color_indices back into standard interleaved BC3 blocks.
///
/// # Safety
///
/// - `alpha_endpoints_out` must be valid for reads of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for reads of `block_count * 6` bytes
/// - `color0_out` must be valid for reads of `block_count * 2` bytes
/// - `color1_out` must be valid for reads of `block_count * 2` bytes
/// - `color_indices_out` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    mut alpha_endpoints_ptr: *const u16,
    mut alpha_indices_ptr: *const u16,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut color_indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Calculate end pointer for alpha_endpoints (as it's a consistent element)
    let alpha_endpoints_ptr_end = alpha_endpoints_ptr.add(block_count);

    while alpha_endpoints_ptr < alpha_endpoints_ptr_end {
        // Read the split values
        let alpha_endpoints = alpha_endpoints_ptr.read_u16_at(0);

        // Read alpha indices (6 bytes) as u16 + u32
        let alpha_indices_part1 = alpha_indices_ptr.read_u16_at(0);
        let alpha_indices_part2 = alpha_indices_ptr.read_u32_at(2);

        let color0 = color0_ptr.read_u16_at(0);
        let color1 = color1_ptr.read_u16_at(0);
        let color_indices = color_indices_ptr.read_u32_at(0);

        // Write BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]

        // Write alpha endpoints (2 bytes)
        output_ptr.write_u16_at(0, alpha_endpoints);

        // Write alpha indices (6 bytes) as u16 + u32
        output_ptr.write_u16_at(2, alpha_indices_part1);
        output_ptr.write_u32_at(4, alpha_indices_part2);

        // Write color endpoints
        output_ptr.write_u16_at(8, color0);
        output_ptr.write_u16_at(10, color1);

        // Write color indices
        output_ptr.write_u32_at(12, color_indices);

        // Advance all pointers
        alpha_endpoints_ptr = alpha_endpoints_ptr.add(1);
        alpha_indices_ptr = alpha_indices_ptr.add(3); // 6 bytes = 3 u16s
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        color_indices_ptr = color_indices_ptr.add(1);
        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::untransform_with_split_colour;
    use crate::test_prelude::*;

    #[test]
    fn can_untransform_unaligned() {
        // 1 block processed per iteration (no SIMD) (* 2 == 2)
        run_with_split_colour_untransform_unaligned_test(
            untransform_with_split_colour,
            2,
            "untransform_with_split_colour (generic, unaligned)",
        );
    }
}
