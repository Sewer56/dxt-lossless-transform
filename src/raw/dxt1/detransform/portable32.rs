/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    unsafe {
        // Get pointers to the color, index sections, and end of data.
        let mut colours_ptr = input_ptr as *const u32;
        let mut indices_ptr = input_ptr.add(len / 2) as *const u32;
        let max_input = input_ptr.add(len) as *const u32;

        let mut output_ptr = output_ptr;

        while indices_ptr < max_input {
            // Read color and index values
            let index_value = *indices_ptr;
            indices_ptr = indices_ptr.add(1); // we compare this in loop condition, so eval as fast as possible.

            let color_value = *colours_ptr;
            colours_ptr = colours_ptr.add(1);

            // Write interleaved values to output
            *(output_ptr as *mut u32) = color_value;
            *(output_ptr.add(4) as *mut u32) = index_value;

            // Move output pointer by 8 bytes (one complete block)
            output_ptr = output_ptr.add(8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::dxt1::transform::tests::generate_dxt1_test_data;
    use crate::raw::transform;
    use rstest::rstest;

    #[rstest]
    #[case::min_size(16)] // 128 bytes - minimum size for unroll-8
    #[case::one_unroll(32)] // 256 bytes - tests double minimum size
    #[case::many_unrolls(256)] // 2KB - tests multiple unroll iterations
    #[case::large(1024)] // 8KB - large dataset
    fn test_detransform(#[case] num_blocks: usize) {
        // Create test data using the existing generator
        let original = generate_dxt1_test_data(num_blocks);
        let mut transformed = vec![0u8; original.len()];
        let mut reconstructed = vec![0u8; original.len()];

        // First transform the data
        unsafe {
            transform::u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
        }

        // Then detransform it
        unsafe {
            u32_detransform(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
        }

        // Verify the reconstruction matches the original
        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "Detransform failed to reconstruct original data for {} blocks.\n\
             Original data has predictable pattern from generate_dxt1_test_data",
            num_blocks
        );
    }
}
