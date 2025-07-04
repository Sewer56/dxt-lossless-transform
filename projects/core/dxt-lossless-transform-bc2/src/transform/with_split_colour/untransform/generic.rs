/// Generic fallback implementation of split-colour untransform for BC2.
/// Combines separate arrays of alpha, colour0, colour1 and indices back into standard interleaved BC2 blocks.
///
/// # Safety
///
/// - `alpha_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for reads of `block_count * 2` bytes
/// - `color1_ptr` must be valid for reads of `block_count * 2` bytes
/// - `indices_ptr` must be valid for reads of `block_count * 4` bytes
/// - `output_ptr` must be valid for writes of `block_count * 16` bytes
#[inline]
pub(crate) unsafe fn untransform_with_split_colour(
    mut alpha_ptr: *const u64,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Calculate end pointer for alpha (as it's the largest element)
    let alpha_ptr_end = alpha_ptr.add(block_count);

    while alpha_ptr < alpha_ptr_end {
        // Read the split values
        let alpha = alpha_ptr.read_unaligned();
        let color0 = color0_ptr.read_unaligned();
        let color1 = color1_ptr.read_unaligned();
        let indices = indices_ptr.read_unaligned();

        // Write BC2 block format: [alpha: u64, color0: u16, color1: u16, indices: u32]
        (output_ptr as *mut u64).write_unaligned(alpha);
        (output_ptr.add(8) as *mut u16).write_unaligned(color0);
        (output_ptr.add(10) as *mut u16).write_unaligned(color1);
        (output_ptr.add(12) as *mut u32).write_unaligned(indices);

        // Advance all pointers
        alpha_ptr = alpha_ptr.add(1);
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
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
