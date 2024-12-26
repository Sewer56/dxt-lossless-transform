/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    // Split output into color and index sections
    // Note(sewer): Compiler will split u64 into 2 u32 registers, so from our perspective
    // whether we go for u32 or u64 is irrelevant
    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let alpha_value = *(input_ptr as *const u64);
        let color_value = *(input_ptr.add(8) as *const u32);
        let index_value = *(input_ptr.add(12) as *const u32);
        input_ptr = input_ptr.add(16);

        // Store colours and indices to their respective halves
        *alphas_ptr = alpha_value;
        *colours_ptr = color_value;
        *indices_ptr = index_value;

        alphas_ptr = alphas_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 32
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0); // Must be divisible by 32 for unroll-2

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    while input_ptr < max_ptr {
        // First block
        let alpha_value1 = *(input_ptr as *const u64);
        let color_value1 = *(input_ptr.add(8) as *const u32);
        let index_value1 = *(input_ptr.add(12) as *const u32);

        // Second block
        let alpha_value2 = *(input_ptr.add(16) as *const u64);
        let color_value2 = *(input_ptr.add(24) as *const u32);
        let index_value2 = *(input_ptr.add(28) as *const u32);
        // compiler may reorder this to later (unfortauntely)
        // I tried memory barriers (fence / compiler_fence), but it didn't seem to help
        input_ptr = input_ptr.add(32);

        // Store first block
        *alphas_ptr = alpha_value1;
        *colours_ptr = color_value1;
        *indices_ptr = index_value1;

        // Store second block
        *alphas_ptr.add(1) = alpha_value2;
        *colours_ptr.add(1) = color_value2;
        *indices_ptr.add(1) = index_value2;

        // Advance pointers
        alphas_ptr = alphas_ptr.add(2);
        colours_ptr = colours_ptr.add(2);
        indices_ptr = indices_ptr.add(2);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0); // Must be divisible by 64 for unroll-4

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    while input_ptr < max_ptr {
        // First block
        let alpha_value1 = *(input_ptr as *const u64);
        let color_value1 = *(input_ptr.add(8) as *const u32);
        let index_value1 = *(input_ptr.add(12) as *const u32);

        // Second block
        let alpha_value2 = *(input_ptr.add(16) as *const u64);
        let color_value2 = *(input_ptr.add(24) as *const u32);
        let index_value2 = *(input_ptr.add(28) as *const u32);

        // Third block
        let alpha_value3 = *(input_ptr.add(32) as *const u64);
        let color_value3 = *(input_ptr.add(40) as *const u32);
        let index_value3 = *(input_ptr.add(44) as *const u32);

        // Fourth block
        let alpha_value4 = *(input_ptr.add(48) as *const u64);
        let color_value4 = *(input_ptr.add(56) as *const u32);
        let index_value4 = *(input_ptr.add(60) as *const u32);
        // compiler may reorder this to later (unfortauntely)
        // I tried memory barriers (fence / compiler_fence), but it didn't seem to help
        input_ptr = input_ptr.add(64);

        // Store all blocks
        *alphas_ptr = alpha_value1;
        *colours_ptr = color_value1;
        *indices_ptr = index_value1;

        *alphas_ptr.add(1) = alpha_value2;
        *colours_ptr.add(1) = color_value2;
        *indices_ptr.add(1) = index_value2;

        *alphas_ptr.add(2) = alpha_value3;
        *colours_ptr.add(2) = color_value3;
        *indices_ptr.add(2) = index_value3;

        *alphas_ptr.add(3) = alpha_value4;
        *colours_ptr.add(3) = color_value4;
        *indices_ptr.add(3) = index_value4;

        // Advance pointers
        alphas_ptr = alphas_ptr.add(4);
        colours_ptr = colours_ptr.add(4);
        indices_ptr = indices_ptr.add(4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc2::transform::tests::generate_bc2_test_data;
    use crate::raw::bc2::transform::tests::transform_with_reference_implementation;
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::min_size(16)] // 128 bytes - minimum size for unroll-8
    #[case::one_unroll(32)] // 256 bytes - tests double minimum size
    #[case::many_unrolls(256)] // 2KB - tests multiple unroll iterations
    #[case::large(1024)] // 8KB - large dataset
    fn test_implementations(#[case] num_blocks: usize) {
        let input = generate_bc2_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test each SSE2 implementation variant
        let implementations = [
            ("u32 no-unroll", u32 as TransformFn),
            ("u32 unroll-2", u32_unroll_2 as TransformFn),
            ("u32 unroll-4", u32_unroll_4 as TransformFn),
        ];

        for (impl_name, implementation) in implementations {
            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
            assert_eq!(
                output_expected, output_test,
                "{} implementation produced different results than reference for {} blocks.\n\
                First differing block will have predictable values:\n\
                Colors: Sequential 1-4 + (block_num * 4)\n\
                Indices: Sequential 128-131 + (block_num * 4)",
                impl_name, num_blocks
            );
        }
    }
}
