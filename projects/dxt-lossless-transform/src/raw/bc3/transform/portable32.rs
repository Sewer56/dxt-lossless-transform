/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let mut alpha_byte_out_ptr = output_ptr as *mut u16;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2) as *mut u16;
    let mut color_byte_out_ptr = output_ptr.add(len / 16 * 8) as *mut u32;
    let mut index_byte_out_ptr = output_ptr.add(len / 16 * 12) as *mut u32;

    let mut current_input_ptr = input_ptr;
    let alpha_byte_end_ptr = alpha_bit_out_ptr;

    while alpha_byte_out_ptr < alpha_byte_end_ptr {
        // Alpha bytes (2 bytes)
        alpha_byte_out_ptr.write_unaligned((current_input_ptr as *const u16).read_unaligned());
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(1); // 2 bytes forward

        // Alpha bits (6 bytes)
        alpha_bit_out_ptr
            .write_unaligned((current_input_ptr.add(2) as *const u16).read_unaligned());
        (alpha_bit_out_ptr.add(1) as *mut u32)
            .write_unaligned((current_input_ptr.add(4) as *const u32).read_unaligned());
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(3); // 6 bytes forward

        // Color bytes (4 bytes)
        color_byte_out_ptr
            .write_unaligned((current_input_ptr.add(8) as *const u32).read_unaligned());
        color_byte_out_ptr = color_byte_out_ptr.add(1); // 4 bytes forward

        // Index bytes
        index_byte_out_ptr
            .write_unaligned((current_input_ptr.add(12) as *const u32).read_unaligned());
        index_byte_out_ptr = index_byte_out_ptr.add(1); // 4 bytes forward
        current_input_ptr = current_input_ptr.add(16); // 16 bytes forward
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 32 (16 * 2)
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    let mut alpha_byte_out_ptr = output_ptr as *mut u16;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2) as *mut u16;
    let mut color_byte_out_ptr = output_ptr.add(len / 16 * 8) as *mut u32;
    let mut index_byte_out_ptr = output_ptr.add(len / 16 * 12) as *mut u32;

    let mut current_input_ptr = input_ptr;
    let alpha_byte_end_ptr = alpha_bit_out_ptr;

    while alpha_byte_out_ptr < alpha_byte_end_ptr {
        // Block 1
        let alpha_bytes1 = (current_input_ptr as *const u16).read_unaligned();
        let alpha_bits1_a = (current_input_ptr.add(2) as *const u16).read_unaligned();
        let alpha_bits1_b = (current_input_ptr.add(4) as *const u32).read_unaligned();
        let color_bytes1 = (current_input_ptr.add(8) as *const u32).read_unaligned();
        let index_bytes1 = (current_input_ptr.add(12) as *const u32).read_unaligned();

        // Block 2
        let alpha_bytes2 = (current_input_ptr.add(16) as *const u16).read_unaligned();
        let alpha_bits2_a = (current_input_ptr.add(18) as *const u16).read_unaligned();
        let alpha_bits2_b = (current_input_ptr.add(20) as *const u32).read_unaligned();
        let color_bytes2 = (current_input_ptr.add(24) as *const u32).read_unaligned();
        let index_bytes2 = (current_input_ptr.add(28) as *const u32).read_unaligned();

        // Write Block 1
        alpha_byte_out_ptr.write_unaligned(alpha_bytes1);
        alpha_bit_out_ptr.write_unaligned(alpha_bits1_a);
        (alpha_bit_out_ptr.add(1) as *mut u32).write_unaligned(alpha_bits1_b);
        color_byte_out_ptr.write_unaligned(color_bytes1);
        index_byte_out_ptr.write_unaligned(index_bytes1);

        // Write Block 2
        alpha_byte_out_ptr.add(1).write_unaligned(alpha_bytes2);
        alpha_bit_out_ptr.add(3).write_unaligned(alpha_bits2_a);
        (alpha_bit_out_ptr.add(4) as *mut u32).write_unaligned(alpha_bits2_b);
        color_byte_out_ptr.add(1).write_unaligned(color_bytes2);
        index_byte_out_ptr.add(1).write_unaligned(index_bytes2);

        // Advance pointers
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(2);
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(6);
        color_byte_out_ptr = color_byte_out_ptr.add(2);
        index_byte_out_ptr = index_byte_out_ptr.add(2);
        current_input_ptr = current_input_ptr.add(32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc3::transform::tests::generate_bc3_test_data;
    use crate::raw::bc3::transform::tests::transform_with_reference_implementation;
    use rstest::rstest;

    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: TransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::u32(TestCase {
        name: "u32 no-unroll",
        func: u32,
        min_blocks: 1, // 16 bytes, minimum size
        many_blocks: 8,
    })]
    #[case::u32_unroll_2(TestCase {
        name: "u32 unroll-2",
        func: u32_unroll_2,
        min_blocks: 2, // 32 bytes minimum
        many_blocks: 8,
    })]
    fn test_transform(#[case] test_case: TestCase) {
        // Test with minimum blocks
        test_blocks(&test_case, test_case.min_blocks);

        // Test with many blocks
        test_blocks(&test_case, test_case.many_blocks);
    }

    fn test_blocks(test_case: &TestCase, num_blocks: usize) {
        let input = generate_bc3_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Clear the output buffer
        output_test.fill(0);

        // Run the implementation
        unsafe {
            (test_case.func)(input.as_ptr(), output_test.as_mut_ptr(), input.len());
        }

        // Compare results
        assert_eq!(
            output_expected, output_test,
            "{} implementation produced different results than reference for {} blocks.",
            test_case.name, num_blocks
        );
    }
}
