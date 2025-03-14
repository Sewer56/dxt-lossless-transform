/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    // Split output into color and index sections
    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let color_value = *(input_ptr as *const u32);
        let index_value = *(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store colours and indices to their respective halves
        *colours_ptr = color_value;
        *indices_ptr = index_value;

        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (for 2x unroll)
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    while input_ptr < max_ptr {
        // Process first 8 bytes
        let color_value1 = *(input_ptr as *const u32);
        let index_value1 = *(input_ptr.add(4) as *const u32);

        // Process next 8 bytes
        let color_value2 = *(input_ptr.add(8) as *const u32);
        let index_value2 = *(input_ptr.add(12) as *const u32);
        input_ptr = input_ptr.add(16);

        // Store to respective sections
        *colours_ptr = color_value1;
        *indices_ptr = index_value1;

        *(colours_ptr.add(1)) = color_value2;
        *(indices_ptr.add(1)) = index_value2;

        colours_ptr = colours_ptr.add(2);
        indices_ptr = indices_ptr.add(2);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 32 (for 4x unroll)
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    while input_ptr < max_ptr {
        // Process 4 sets of 8 bytes each
        let color_value1 = *(input_ptr as *const u32);
        let index_value1 = *(input_ptr.add(4) as *const u32);

        let color_value2 = *(input_ptr.add(8) as *const u32);
        let index_value2 = *(input_ptr.add(12) as *const u32);

        let color_value3 = *(input_ptr.add(16) as *const u32);
        let index_value3 = *(input_ptr.add(20) as *const u32);

        let color_value4 = *(input_ptr.add(24) as *const u32);
        let index_value4 = *(input_ptr.add(28) as *const u32);
        input_ptr = input_ptr.add(32);

        // Store all values
        *colours_ptr = color_value1;
        *indices_ptr = index_value1;

        *(colours_ptr.add(1)) = color_value2;
        *(indices_ptr.add(1)) = index_value2;

        *(colours_ptr.add(2)) = color_value3;
        *(indices_ptr.add(2)) = index_value3;

        *(colours_ptr.add(3)) = color_value4;
        *(indices_ptr.add(3)) = index_value4;

        colours_ptr = colours_ptr.add(4);
        indices_ptr = indices_ptr.add(4);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (for 8x unroll)
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    while input_ptr < max_ptr {
        // Process 8 sets of 8 bytes each
        let color_value1 = *(input_ptr as *const u32);
        let index_value1 = *(input_ptr.add(4) as *const u32);

        let color_value2 = *(input_ptr.add(8) as *const u32);
        let index_value2 = *(input_ptr.add(12) as *const u32);

        let color_value3 = *(input_ptr.add(16) as *const u32);
        let index_value3 = *(input_ptr.add(20) as *const u32);

        let color_value4 = *(input_ptr.add(24) as *const u32);
        let index_value4 = *(input_ptr.add(28) as *const u32);

        let color_value5 = *(input_ptr.add(32) as *const u32);
        let index_value5 = *(input_ptr.add(36) as *const u32);

        let color_value6 = *(input_ptr.add(40) as *const u32);
        let index_value6 = *(input_ptr.add(44) as *const u32);

        let color_value7 = *(input_ptr.add(48) as *const u32);
        let index_value7 = *(input_ptr.add(52) as *const u32);

        let color_value8 = *(input_ptr.add(56) as *const u32);
        let index_value8 = *(input_ptr.add(60) as *const u32);
        input_ptr = input_ptr.add(64);

        // Store all values
        *colours_ptr = color_value1;
        *indices_ptr = index_value1;

        *(colours_ptr.add(1)) = color_value2;
        *(indices_ptr.add(1)) = index_value2;

        *(colours_ptr.add(2)) = color_value3;
        *(indices_ptr.add(2)) = index_value3;

        *(colours_ptr.add(3)) = color_value4;
        *(indices_ptr.add(3)) = index_value4;

        *(colours_ptr.add(4)) = color_value5;
        *(indices_ptr.add(4)) = index_value5;

        *(colours_ptr.add(5)) = color_value6;
        *(indices_ptr.add(5)) = index_value6;

        *(colours_ptr.add(6)) = color_value7;
        *(indices_ptr.add(6)) = index_value7;

        *(colours_ptr.add(7)) = color_value8;
        *(indices_ptr.add(7)) = index_value8;

        colours_ptr = colours_ptr.add(8);
        indices_ptr = indices_ptr.add(8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc1_test_data;
    use crate::split_blocks::split::tests::transform_with_reference_implementation;
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::min_size(16)] // 128 bytes - minimum size for unroll-8
    #[case::one_unroll(32)] // 256 bytes - tests double minimum size
    #[case::many_unrolls(256)] // 2KB - tests multiple unroll iterations
    #[case::large(1024)] // 8KB - large dataset
    fn test_implementations(#[case] num_blocks: usize) {
        let input = generate_bc1_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test each SSE2 implementation variant
        let implementations = [
            ("u32 no-unroll", u32 as TransformFn),
            ("u32 unroll-2", u32_unroll_2),
            ("u32 unroll-4", u32_unroll_4),
            ("u32 unroll-8", u32_unroll_8),
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
