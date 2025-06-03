use core::ptr::{read_unaligned, write_unaligned};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Get pointers to the color and index sections
    let colours_ptr = input_ptr as *const u32;
    let indices_ptr = input_ptr.add(len / 2) as *const u32;

    u32_detransform_with_separate_pointers(colours_ptr, indices_ptr, output_ptr, len);
}

/// # Safety
///
/// - colours_ptr must be valid for reads of len/2 bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub(crate) unsafe fn u32_detransform_with_separate_pointers(
    mut colours_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    // Calculate end pointer for the indices section
    let max_indices_ptr = indices_ptr.add(len / 8);

    while indices_ptr < max_indices_ptr {
        // Read color and index values
        let index_value = read_unaligned(indices_ptr);
        indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

        let color_value = read_unaligned(colours_ptr);
        colours_ptr = colours_ptr.add(1);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value);

        // Move output pointer by 8 bytes (one complete block)
        output_ptr = output_ptr.add(8);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub unsafe fn u32_detransform_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_aligned_input = input_ptr.add(len.saturating_sub(16 - 8)) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_aligned_input {
        // Load indices first and advance pointer immediately
        let index_value1 = read_unaligned(indices_ptr);
        let index_value2 = read_unaligned(indices_ptr.add(1));
        indices_ptr = indices_ptr.add(2);

        // Load colors after indices
        let color_value1 = read_unaligned(colours_ptr);
        let color_value2 = read_unaligned(colours_ptr.add(1));
        colours_ptr = colours_ptr.add(2);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value1);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value1);

        write_unaligned(output_ptr.add(8) as *mut u32, color_value2);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value2);

        output_ptr = output_ptr.add(16);
    }

    let max_input = input_ptr.add(len) as *const u32;
    while indices_ptr < max_input {
        // Read color and index values
        let index_value = read_unaligned(indices_ptr);
        indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

        let color_value = read_unaligned(colours_ptr);
        colours_ptr = colours_ptr.add(1);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value);

        // Move output pointer by 8 bytes (one complete block)
        output_ptr = output_ptr.add(8);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub unsafe fn u32_detransform_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_aligned_input = input_ptr.add(len.saturating_sub(32 - 8)) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_aligned_input {
        // Load all indices first and advance pointer immediately
        let index_value1 = read_unaligned(indices_ptr);
        let index_value2 = read_unaligned(indices_ptr.add(1));
        let index_value3 = read_unaligned(indices_ptr.add(2));
        let index_value4 = read_unaligned(indices_ptr.add(3));
        indices_ptr = indices_ptr.add(4);

        // Load all colors after indices
        let color_value1 = read_unaligned(colours_ptr);
        let color_value2 = read_unaligned(colours_ptr.add(1));
        let color_value3 = read_unaligned(colours_ptr.add(2));
        let color_value4 = read_unaligned(colours_ptr.add(3));
        colours_ptr = colours_ptr.add(4);

        // Write all values to output in order
        write_unaligned(output_ptr as *mut u32, color_value1);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value1);

        write_unaligned(output_ptr.add(8) as *mut u32, color_value2);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value2);

        write_unaligned(output_ptr.add(16) as *mut u32, color_value3);
        write_unaligned(output_ptr.add(20) as *mut u32, index_value3);

        write_unaligned(output_ptr.add(24) as *mut u32, color_value4);
        write_unaligned(output_ptr.add(28) as *mut u32, index_value4);

        output_ptr = output_ptr.add(32);
    }

    let max_input = input_ptr.add(len) as *const u32;
    while indices_ptr < max_input {
        // Read color and index values
        let index_value = read_unaligned(indices_ptr);
        indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

        let color_value = read_unaligned(colours_ptr);
        colours_ptr = colours_ptr.add(1);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value);

        // Move output pointer by 8 bytes (one complete block)
        output_ptr = output_ptr.add(8);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
pub unsafe fn u32_detransform_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let mut colours_ptr = input_ptr as *const u32;
    let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
    let max_aligned_input = input_ptr.add(len.saturating_sub(64 - 8)) as *const u32;

    let mut output_ptr = output_ptr;

    while indices_ptr < max_aligned_input {
        // Load all indices first and advance pointer immediately
        let index_value1 = read_unaligned(indices_ptr);
        let index_value2 = read_unaligned(indices_ptr.add(1));
        let index_value3 = read_unaligned(indices_ptr.add(2));
        let index_value4 = read_unaligned(indices_ptr.add(3));
        let index_value5 = read_unaligned(indices_ptr.add(4));
        let index_value6 = read_unaligned(indices_ptr.add(5));
        let index_value7 = read_unaligned(indices_ptr.add(6));
        let index_value8 = read_unaligned(indices_ptr.add(7));
        indices_ptr = indices_ptr.add(8);

        // Load all colors after indices
        let color_value1 = read_unaligned(colours_ptr);
        let color_value2 = read_unaligned(colours_ptr.add(1));
        let color_value3 = read_unaligned(colours_ptr.add(2));
        let color_value4 = read_unaligned(colours_ptr.add(3));
        let color_value5 = read_unaligned(colours_ptr.add(4));
        let color_value6 = read_unaligned(colours_ptr.add(5));
        let color_value7 = read_unaligned(colours_ptr.add(6));
        let color_value8 = read_unaligned(colours_ptr.add(7));
        colours_ptr = colours_ptr.add(8);

        // Write all values to output in order
        write_unaligned(output_ptr as *mut u32, color_value1);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value1);

        write_unaligned(output_ptr.add(8) as *mut u32, color_value2);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value2);

        write_unaligned(output_ptr.add(16) as *mut u32, color_value3);
        write_unaligned(output_ptr.add(20) as *mut u32, index_value3);

        write_unaligned(output_ptr.add(24) as *mut u32, color_value4);
        write_unaligned(output_ptr.add(28) as *mut u32, index_value4);

        write_unaligned(output_ptr.add(32) as *mut u32, color_value5);
        write_unaligned(output_ptr.add(36) as *mut u32, index_value5);

        write_unaligned(output_ptr.add(40) as *mut u32, color_value6);
        write_unaligned(output_ptr.add(44) as *mut u32, index_value6);

        write_unaligned(output_ptr.add(48) as *mut u32, color_value7);
        write_unaligned(output_ptr.add(52) as *mut u32, index_value7);

        write_unaligned(output_ptr.add(56) as *mut u32, color_value8);
        write_unaligned(output_ptr.add(60) as *mut u32, index_value8);

        output_ptr = output_ptr.add(64);
    }

    let max_input = input_ptr.add(len) as *const u32;
    while indices_ptr < max_input {
        // Read color and index values
        let index_value = read_unaligned(indices_ptr);
        indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

        let color_value = read_unaligned(colours_ptr);
        colours_ptr = colours_ptr.add(1);

        // Write interleaved values to output
        write_unaligned(output_ptr as *mut u32, color_value);
        write_unaligned(output_ptr.add(4) as *mut u32, index_value);

        // Move output pointer by 8 bytes (one complete block)
        output_ptr = output_ptr.add(8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::standard::transform::tests::assert_implementation_matches_reference;
    use crate::transforms::standard::transform::tests::generate_bc1_test_data;
    use crate::transforms::standard::transform::u32;
    use dxt_lossless_transform_common::allocate::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32_detransform, "u32")]
    #[case(u32_detransform_unroll_2, "unroll_2")]
    #[case(u32_detransform_unroll_4, "unroll_4")]
    #[case(u32_detransform_unroll_8, "unroll_8")]
    fn test_portable32_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);
            let mut transformed = allocate_align_64(original.len()).unwrap();
            let mut reconstructed = allocate_align_64(original.len()).unwrap();

            unsafe {
                // Transform using standard implementation
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Reconstruct using the implementation being tested
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                &format!("{impl_name} (aligned)"),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(u32_detransform, "u32")]
    #[case(u32_detransform_unroll_2, "unroll_2")]
    #[case(u32_detransform_unroll_4, "unroll_4")]
    #[case(u32_detransform_unroll_8, "unroll_8")]
    fn test_portable32_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            unsafe {
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);

            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed_unaligned.as_ptr().add(1),
                    reconstructed.as_mut_ptr().add(1),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}
