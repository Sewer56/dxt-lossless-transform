use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-colour transform for BC3.
/// Splits standard interleaved BC3 blocks into separate arrays of alpha_endpoints, alpha_indices, color0, color1 and color_indices.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha_endpoints_out` must be valid for writes of `block_count * 2` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `color0_out` must be valid for writes of `block_count * 2` bytes
/// - `color1_out` must be valid for writes of `block_count * 2` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    mut input_ptr: *const u8,
    mut alpha_endpoints_out: *mut u16,
    mut alpha_indices_out: *mut u16,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut color_indices_out: *mut u32,
    block_count: usize,
) {
    let input_end = input_ptr.add(block_count * 16);
    while input_ptr < input_end {
        // Read BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]

        // Read alpha endpoints (2 bytes: alpha0 + alpha1)
        let alpha_endpoints = input_ptr.read_u16_at(0);

        // Read alpha indices (6 bytes) - read as u16 + u32
        let alpha_indices_part1 = input_ptr.read_u16_at(2);
        let alpha_indices_part2 = input_ptr.read_u32_at(4);

        // Read color endpoints
        let color0 = input_ptr.read_u16_at(8);
        let color1 = input_ptr.read_u16_at(10);

        // Read color indices (4 bytes)
        let color_indices = input_ptr.read_u32_at(12);

        // Write to separate arrays
        alpha_endpoints_out.write_u16_at(0, alpha_endpoints);

        // Write alpha indices (6 bytes) as u16 + u32
        alpha_indices_out.write_u16_at(0, alpha_indices_part1);
        alpha_indices_out.write_u32_at(2, alpha_indices_part2);

        color0_out.write_u16_at(0, color0);
        color1_out.write_u16_at(0, color1);
        color_indices_out.write_u32_at(0, color_indices);

        // Advance all pointers
        input_ptr = input_ptr.add(16);
        alpha_endpoints_out = alpha_endpoints_out.add(1);
        alpha_indices_out = alpha_indices_out.add(3); // 6 bytes = 3 u16s
        color0_out = color0_out.add(1);
        color1_out = color1_out.add(1);
        color_indices_out = color_indices_out.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn generic_transform_roundtrip() {
        // Generic processes 16 bytes per iteration (* 2 / 16 == 2)
        run_split_colour_transform_roundtrip_test(transform_with_split_colour, 2, "Generic");
    }
}
