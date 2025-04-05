/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    // Split output into color and index sections
    let colours_ptr = output_ptr as *mut u32;
    let indices_ptr = output_ptr.add(len / 2) as *mut u32;

    u32_with_separate_pointers(input_ptr, colours_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colours_ptr must be valid for writes of len/2 bytes
/// - indices_ptr must be valid for writes of len/2 bytes
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub(crate) unsafe fn u32_with_separate_pointers(
    input_ptr: *const u8,
    mut colours_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

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
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 16 * 16) as *mut u8; // Aligned to 16 bytes
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 16-byte chunks (2x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
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

    // Handle remaining 8-byte chunk if any
    while input_ptr < max_ptr {
        // Process remaining 8 bytes
        let color_value = *(input_ptr as *const u32);
        let index_value = *(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store to respective sections
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
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 32 * 32) as *mut u8; // Aligned to 32 bytes
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 32-byte aligned chunks (4x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
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

    // Handle remaining 8-byte chunks if any
    while input_ptr < max_ptr {
        // Process 8 bytes
        let color_value = *(input_ptr as *const u32);
        let index_value = *(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store values
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
/// - len must be divisible by 8
/// - pointers must be properly aligned for u64/u32 access
pub unsafe fn u32_unroll_8(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    let max_aligned_ptr = input_ptr.add(len / 64 * 64) as *mut u8; // Aligned to 64 bytes
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    let mut colours_ptr = output_ptr as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Process 64-byte aligned chunks (8x 8-byte blocks)
    while input_ptr < max_aligned_ptr {
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

    // Handle remaining 8-byte chunks if any
    while input_ptr < max_ptr {
        // Process 8 bytes
        let color_value = *(input_ptr as *const u32);
        let index_value = *(input_ptr.add(4) as *const u32);
        input_ptr = input_ptr.add(8);

        // Store values
        *colours_ptr = color_value;
        *indices_ptr = index_value;

        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc1_test_data;
    use crate::split_blocks::split::tests::transform_with_reference_implementation;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32, "u32 no-unroll")]
    #[case(u32_unroll_2, "u32 unroll_2")]
    #[case(u32_unroll_4, "u32 unroll_4")]
    #[case(u32_unroll_8, "u32 unroll_8")]
    fn test_portable32_implementation(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len()];

            transform_with_reference_implementation(
                input.as_slice(),
                output_expected.as_mut_slice(),
            );

            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            assert_eq!(
                output_expected.as_slice(),
                output_test.as_slice(),
                "{} implementation produced different results than reference for {} blocks.\n\
                First differing block will have predictable values:\n\
                Colors: Sequential 1-4 + (block_num * 4)\n\
                Indices: Sequential 128-131 + (block_num * 4)",
                impl_name,
                num_blocks
            );
        }
    }
}
