/// Generic fallback implementation of split-colour transform.
/// Splits standard interleaved BC1 blocks into separate arrays of colour0, colour1 and indices.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for writes of `block_count * 2` bytes
/// - `color1_ptr` must be valid for writes of `block_count * 2` bytes
/// - `indices_ptr` must be valid for writes of `block_count * 4` bytes
#[inline]
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    // Initialize pointers
    let mut input_ptr = input_ptr;
    let mut color0_ptr = color0_out;
    let mut color1_ptr = color1_out;
    let mut indices_ptr = indices_out;

    // Process each block
    let input_end = input_ptr.add(block_count * 8);
    while input_ptr < input_end {
        // Read BC1 block format: [color0: u16, color1: u16, indices: u32]
        let color0 = (input_ptr as *const u16).read_unaligned();
        let color1 = (input_ptr.add(2) as *const u16).read_unaligned();
        let indices = (input_ptr.add(4) as *const u32).read_unaligned();

        // Write to separate arrays
        color0_ptr.write_unaligned(color0);
        color1_ptr.write_unaligned(color1);
        indices_ptr.write_unaligned(indices);

        // Advance all pointers
        input_ptr = input_ptr.add(8);
        color0_ptr = color0_ptr.add(1);
        color1_ptr = color1_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn generic_transform_roundtrip() {
        // Generic processes 8 bytes per iteration (* 2 / 8 == 2)
        run_split_colour_transform_roundtrip_test(transform_with_split_colour, 2, "Generic");
    }
}
