use ptr_utils::{UnalignedRead, UnalignedWrite};

/// Generic fallback implementation of split-alphas transform for BC3.
/// Splits standard interleaved BC3 blocks into separate arrays of alpha0, alpha1, alpha_indices, colors and color_indices.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha0_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha1_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `colors_out` must be valid for writes of `block_count * 4` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
#[allow(dead_code)]
#[inline]
pub(crate) unsafe fn transform_with_split_alphas(
    mut input_ptr: *const u8,
    mut alpha0_out: *mut u8,
    mut alpha1_out: *mut u8,
    mut alpha_indices_out: *mut u16,
    mut colors_out: *mut u32,
    mut color_indices_out: *mut u32,
    block_count: usize,
) {
    // Process each block
    let input_end = input_ptr.add(block_count * 16);
    while input_ptr < input_end {
        // Read BC3 block format: [alpha0: u8, alpha1: u8, alpha_indices: 6 bytes, color0: u16, color1: u16, color_indices: u32]
        let alpha0 = input_ptr.read();
        let alpha1 = input_ptr.add(1).read();

        // Read alpha indices (6 bytes) as u16 + u32
        let alpha_indices_part1 = input_ptr.read_u16_at(2);
        let alpha_indices_part2 = input_ptr.read_u32_at(4);

        // Read colors (4 bytes total: color0 + color1)
        let colors = input_ptr.read_u32_at(8);

        // Read color indices (4 bytes)
        let color_indices = input_ptr.read_u32_at(12);

        // Write to separate arrays
        alpha0_out.write(alpha0);
        alpha1_out.write(alpha1);

        // Write alpha indices (6 bytes) as u16 + u32
        alpha_indices_out.write_u16_at(0, alpha_indices_part1);
        alpha_indices_out.write_u32_at(2, alpha_indices_part2);

        colors_out.write_u32_at(0, colors);
        color_indices_out.write_u32_at(0, color_indices);

        // Advance all pointers
        input_ptr = input_ptr.add(16);
        alpha0_out = alpha0_out.add(1);
        alpha1_out = alpha1_out.add(1);
        alpha_indices_out = alpha_indices_out.add(3); // 6 bytes = 3 u16s
        colors_out = colors_out.add(1);
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
        run_split_alphas_transform_roundtrip_test(transform_with_split_alphas, 2, "Generic");
    }
}
