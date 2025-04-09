use core::ptr::{read_unaligned, write_unaligned};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Get pointers to the alpha, color, index sections.
    let alphas_ptr = input_ptr as *const u64;
    let colours_ptr = input_ptr.add(len / 2) as *const u32;
    let indices_ptr = (colours_ptr as *const u8).add(len / 4) as *const u32;

    u32_detransform_with_separate_pointers(alphas_ptr, colours_ptr, indices_ptr, output_ptr, len);
}

/// # Safety
///
/// - `alphas_ptr` must point to valid `u64` data for `len / 2` bytes.
/// - `colours_ptr` must point to valid `u32` data for `len / 4` bytes.
/// - `indices_ptr` must point to valid `u32` data for `len / 4` bytes.
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub(crate) unsafe fn u32_detransform_with_separate_pointers(
    mut alphas_ptr: *const u64,
    mut colours_ptr: *const u32,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    // Calculate the end pointer for the alpha section
    let max_input = (alphas_ptr as *const u8).add(len / 2) as *const u64;

    while alphas_ptr < max_input {
        // Read Alpha, Color and Index values using unaligned reads
        let alpha_value = read_unaligned(alphas_ptr);
        alphas_ptr = alphas_ptr.add(1);
        let color_value = read_unaligned(colours_ptr);
        colours_ptr = colours_ptr.add(1);
        let index_value = read_unaligned(indices_ptr);
        indices_ptr = indices_ptr.add(1);

        // Write interleaved values to output using unaligned writes
        write_unaligned(output_ptr as *mut u64, alpha_value);
        write_unaligned(output_ptr.add(8) as *mut u32, color_value);
        write_unaligned(output_ptr.add(12) as *mut u32, index_value);

        // Move output pointer by 16 bytes (one complete block)
        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc2_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32_detransform, "no_unroll")]
    fn test_portable32_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        // Test with different block counts to ensure they all work correctly
        for block_count in 1..=512 {
            // Generate test data
            let original = generate_bc2_test_data(block_count);
            let mut transformed = allocate_align_64(original.len());
            let mut reconstructed = allocate_align_64(original.len());

            unsafe {
                // Transform the original test data
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Re-transform it back using the implementation under test
                (detransform_fn)(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            // Verify the results match
            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                impl_name,
                block_count,
            );
        }
    }

    #[rstest]
    #[case(u32_detransform, "no_unroll")]
    fn test_portable32_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        // Test with different block counts to ensure they all work correctly
        for block_count in 1..=512 {
            // Generate test data
            let original = generate_bc2_test_data(block_count);

            // Create unaligned buffers (allocate an extra byte and offset by 1)
            let mut unaligned_transformed = vec![0u8; original.len() + 1];
            let mut unaligned_reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Transform the original test data
                u32(
                    original.as_ptr(),
                    unaligned_transformed.as_mut_ptr().add(1),
                    original.len(),
                );

                // Re-transform it back using the implementation under test
                (detransform_fn)(
                    unaligned_transformed.as_mut_ptr().add(1),
                    unaligned_reconstructed.as_mut_ptr().add(1),
                    unaligned_transformed.len() - 1,
                );
            }

            // Verify the results match
            assert_implementation_matches_reference(
                original.as_slice(),
                &unaligned_reconstructed[1..],
                impl_name,
                block_count,
            );
        }
    }
}
