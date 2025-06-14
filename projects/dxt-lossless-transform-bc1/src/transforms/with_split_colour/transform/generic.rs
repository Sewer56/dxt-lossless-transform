/// Generic fallback implementation of split-colour transform.
/// Splits standard interleaved BC1 blocks into separate arrays of colour0, colour1 and indices.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 8` bytes
/// - `color0_ptr` must be valid for writes of `block_count * 2` bytes
/// - `color1_ptr` must be valid for writes of `block_count * 2` bytes
/// - `indices_ptr` must be valid for writes of `block_count * 4` bytes
pub(crate) unsafe fn transform_with_split_colour(
    input_ptr: *const u8,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
) {
    // Initialize pointers
    let mut input_ptr = input_ptr;
    let mut color0_ptr = color0_ptr;
    let mut color1_ptr = color1_ptr;
    let mut indices_ptr = indices_ptr;

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
    use crate::transforms::with_split_colour::untransform::untransform_with_split_colour;

    #[rstest]
    fn generic_transform_roundtrip() {
        for num_blocks in 1..=128 {
            let original = generate_bc1_test_data(num_blocks);
            let mut colour0 = vec![0u16; num_blocks];
            let mut colour1 = vec![0u16; num_blocks];
            let mut indices = vec![0u32; num_blocks];
            let mut reconstructed = vec![0u8; original.len()];

            unsafe {
                transform_with_split_colour(
                    original.as_ptr(),
                    colour0.as_mut_ptr(),
                    colour1.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    num_blocks,
                );
                untransform_with_split_colour(
                    colour0.as_ptr(),
                    colour1.as_ptr(),
                    indices.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    num_blocks,
                );
            }

            assert_eq!(
                reconstructed.as_slice(),
                original.as_slice(),
                "generic transform split-colour roundtrip"
            );
        }
    }
}
