#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Calculate end pointer for color0
    let color0_ptr_end = color0_ptr.add(block_count);

    while color0_ptr < color0_ptr_end {
        // Read the split color values
        let color0 = color0_ptr.read_unaligned();
        let color1 = color1_ptr.read_unaligned();
        let indices = indices_ptr.read_unaligned();

        // Write BC1 block format: [color0: u16, color1: u16, indices: u32]
        // Convert to bytes and write directly
        (output_ptr as *mut u16).write_unaligned(color0);
        (output_ptr.add(2) as *mut u16).write_unaligned(color1);
        (output_ptr.add(4) as *mut u32).write_unaligned(indices);

        // Advance all pointers
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
        output_ptr = output_ptr.add(8);
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
