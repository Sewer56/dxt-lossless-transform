/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Get pointers to the alpha, color, index sections, and end of data.
    let mut alphas_ptr = input_ptr as *const u64;
    let mut colours_ptr = input_ptr.add(len / 2) as *const u32;
    let mut indices_ptr = (colours_ptr as *const u8).add(len / 4) as *const u32;
    let max_input = colours_ptr as *const u64;

    let mut output_ptr = output_ptr;

    while alphas_ptr < max_input {
        // Read Alpha, Color and Index values
        let alpha_value = *alphas_ptr;
        alphas_ptr = alphas_ptr.add(1);
        let color_value = *colours_ptr;
        colours_ptr = colours_ptr.add(1);
        let index_value = *indices_ptr;
        indices_ptr = indices_ptr.add(1);

        // Write interleaved values to output
        *(output_ptr as *mut u64) = alpha_value;
        *(output_ptr.add(8) as *mut u32) = color_value;
        *(output_ptr.add(12) as *mut u32) = index_value;

        // Move output pointer by 16 bytes (one complete block)
        output_ptr = output_ptr.add(16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc2::transform::tests::generate_bc2_test_data;
    use crate::raw::bc2::transform::u32;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: DetransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::u32(
        TestCase {
            name: "no_unroll",
            func: u32_detransform,
            min_blocks: 1,
            many_blocks: 1024,
        }
    )]
    fn test_detransform(#[case] test_case: TestCase) {
        // Test with minimum blocks
        test_blocks(&test_case, test_case.min_blocks);

        // Test with many blocks
        test_blocks(&test_case, test_case.many_blocks);
    }

    fn test_blocks(test_case: &TestCase, num_blocks: usize) {
        let original = generate_bc2_test_data(num_blocks);
        let mut transformed = vec![0u8; original.len()];
        let mut reconstructed = vec![0u8; original.len()];

        unsafe {
            u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            (test_case.func)(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
        }

        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "{} detransform failed to reconstruct original data for {} blocks",
            test_case.name,
            num_blocks
        );
    }
}
