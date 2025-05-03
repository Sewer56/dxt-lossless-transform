/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Split output into color and index sections
    // Note(sewer): Compiler will split u64 into 2 u32 registers, so from our perspective
    // whether we go for u32 or u64 is irrelevant
    let alphas_ptr = output_ptr as *mut u64;
    let colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    u32_with_separate_pointers(input_ptr, alphas_ptr, colours_ptr, indices_ptr, len);
}

/// Inner function that processes the data with separate pointers for each component
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - alphas_ptr must be valid for writes of len/2 bytes
/// - colours_ptr must be valid for writes of len/4 bytes
/// - indices_ptr must be valid for writes of len/4 bytes
/// - len must be divisible by 16
pub unsafe fn u32_with_separate_pointers(
    input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colours_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    let max_ptr = input_ptr.add(len) as *mut u8;
    let mut input_ptr = input_ptr as *mut u8;

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let alpha_value = (input_ptr as *const u64).read_unaligned();
        let color_value = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value = (input_ptr.add(12) as *const u32).read_unaligned();
        input_ptr = input_ptr.add(16);

        // Store colours and indices to their respective halves
        alphas_ptr.write_unaligned(alpha_value);
        colours_ptr.write_unaligned(color_value);
        indices_ptr.write_unaligned(index_value);

        alphas_ptr = alphas_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    let mut input_ptr = input_ptr as *mut u8;
    let max_ptr = input_ptr.add(len);
    let max_aligned_input = input_ptr.add(len.saturating_sub(32));

    while input_ptr < max_aligned_input {
        // First block
        let alpha_value1 = (input_ptr as *const u64).read_unaligned();
        let color_value1 = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value1 = (input_ptr.add(12) as *const u32).read_unaligned();

        // Second block
        let alpha_value2 = (input_ptr.add(16) as *const u64).read_unaligned();
        let color_value2 = (input_ptr.add(24) as *const u32).read_unaligned();
        let index_value2 = (input_ptr.add(28) as *const u32).read_unaligned();
        // compiler may reorder this to later (unfortauntely)
        // I tried memory barriers (fence / compiler_fence), but it didn't seem to help
        input_ptr = input_ptr.add(32);

        // Store first block
        alphas_ptr.write_unaligned(alpha_value1);
        colours_ptr.write_unaligned(color_value1);
        indices_ptr.write_unaligned(index_value1);

        // Store second block
        alphas_ptr.add(1).write_unaligned(alpha_value2);
        colours_ptr.add(1).write_unaligned(color_value2);
        indices_ptr.add(1).write_unaligned(index_value2);

        // Advance pointers
        alphas_ptr = alphas_ptr.add(2);
        colours_ptr = colours_ptr.add(2);
        indices_ptr = indices_ptr.add(2);
    }

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let alpha_value = (input_ptr as *const u64).read_unaligned();
        let color_value = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value = (input_ptr.add(12) as *const u32).read_unaligned();
        input_ptr = input_ptr.add(16);

        // Store colours and indices to their respective halves
        alphas_ptr.write_unaligned(alpha_value);
        colours_ptr.write_unaligned(color_value);
        indices_ptr.write_unaligned(index_value);

        alphas_ptr = alphas_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
pub unsafe fn u32_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    let mut alphas_ptr = output_ptr as *mut u64;
    let mut colours_ptr = output_ptr.add(len / 2) as *mut u32;
    let mut indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    let mut input_ptr = input_ptr as *mut u8;
    let max_ptr = input_ptr.add(len);
    let max_aligned_input = input_ptr.add(len.saturating_sub(64));

    while input_ptr < max_aligned_input {
        // First block
        let alpha_value1 = (input_ptr as *const u64).read_unaligned();
        let color_value1 = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value1 = (input_ptr.add(12) as *const u32).read_unaligned();

        // Second block
        let alpha_value2 = (input_ptr.add(16) as *const u64).read_unaligned();
        let color_value2 = (input_ptr.add(24) as *const u32).read_unaligned();
        let index_value2 = (input_ptr.add(28) as *const u32).read_unaligned();

        // Third block
        let alpha_value3 = (input_ptr.add(32) as *const u64).read_unaligned();
        let color_value3 = (input_ptr.add(40) as *const u32).read_unaligned();
        let index_value3 = (input_ptr.add(44) as *const u32).read_unaligned();

        // Fourth block
        let alpha_value4 = (input_ptr.add(48) as *const u64).read_unaligned();
        let color_value4 = (input_ptr.add(56) as *const u32).read_unaligned();
        let index_value4 = (input_ptr.add(60) as *const u32).read_unaligned();
        // compiler may reorder this to later (unfortauntely)
        // I tried memory barriers (fence / compiler_fence), but it didn't seem to help
        input_ptr = input_ptr.add(64);

        // Store all blocks
        alphas_ptr.write_unaligned(alpha_value1);
        colours_ptr.write_unaligned(color_value1);
        indices_ptr.write_unaligned(index_value1);

        alphas_ptr.add(1).write_unaligned(alpha_value2);
        colours_ptr.add(1).write_unaligned(color_value2);
        indices_ptr.add(1).write_unaligned(index_value2);

        alphas_ptr.add(2).write_unaligned(alpha_value3);
        colours_ptr.add(2).write_unaligned(color_value3);
        indices_ptr.add(2).write_unaligned(index_value3);

        alphas_ptr.add(3).write_unaligned(alpha_value4);
        colours_ptr.add(3).write_unaligned(color_value4);
        indices_ptr.add(3).write_unaligned(index_value4);

        // Advance pointers
        alphas_ptr = alphas_ptr.add(4);
        colours_ptr = colours_ptr.add(4);
        indices_ptr = indices_ptr.add(4);
    }

    while input_ptr < max_ptr {
        // Split into colours (lower 4 bytes) and indices (upper 4 bytes)
        let alpha_value = (input_ptr as *const u64).read_unaligned();
        let color_value = (input_ptr.add(8) as *const u32).read_unaligned();
        let index_value = (input_ptr.add(12) as *const u32).read_unaligned();
        input_ptr = input_ptr.add(16);

        // Store colours and indices to their respective halves
        alphas_ptr.write_unaligned(alpha_value);
        colours_ptr.write_unaligned(color_value);
        indices_ptr.write_unaligned(index_value);

        alphas_ptr = alphas_ptr.add(1);
        colours_ptr = colours_ptr.add(1);
        indices_ptr = indices_ptr.add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::{
        assert_implementation_matches_reference, generate_bc2_test_data,
        transform_with_reference_implementation,
    };
    use crate::testutils::allocate_align_64;
    use core::ptr::copy_nonoverlapping;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32, "u32 no-unroll")]
    #[case(u32_unroll_2, "u32 unroll_2")]
    #[case(u32_unroll_4, "u32 unroll_4")]
    fn test_portable32_aligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let mut input = allocate_align_64(num_blocks * 16);
            let mut output_expected = allocate_align_64(input.len());
            let mut output_test = allocate_align_64(input.len());

            // Fill the input with test data
            unsafe {
                copy_nonoverlapping(
                    generate_bc2_test_data(num_blocks).as_ptr(),
                    input.as_mut_ptr(),
                    input.len(),
                );
            }

            transform_with_reference_implementation(
                input.as_slice(),
                output_expected.as_mut_slice(),
            );

            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                output_test.as_slice(),
                &format!("{impl_name} (aligned)"),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(u32, "u32 no-unroll")]
    #[case(u32_unroll_2, "u32 unroll_2")]
    #[case(u32_unroll_4, "u32 unroll_4")]
    fn test_portable32_unaligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        for num_blocks in 1..=512 {
            let input = generate_bc2_test_data(num_blocks);

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut input_unaligned = vec![0u8; input.len() + 1];
            input_unaligned[1..].copy_from_slice(input.as_slice());

            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len() + 1];

            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            output_test.as_mut_slice().fill(0);
            unsafe {
                // Use pointers offset by 1 byte to create unaligned access
                permute_fn(
                    input_unaligned.as_ptr().add(1),
                    output_test.as_mut_ptr().add(1),
                    input.len(),
                );
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                &output_test[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}
