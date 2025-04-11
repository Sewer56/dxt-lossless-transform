/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let alpha_byte_out_ptr = output_ptr as *mut u16;
    let alpha_bit_out_ptr = output_ptr.add(len / 16 * 2) as *mut u16;
    let color_byte_out_ptr = output_ptr.add(len / 16 * 8) as *mut u32;
    let index_byte_out_ptr = output_ptr.add(len / 16 * 12) as *mut u32;

    u32_with_separate_endpoints(
        input_ptr,
        alpha_byte_out_ptr,
        alpha_bit_out_ptr,
        color_byte_out_ptr,
        index_byte_out_ptr,
        alpha_bit_out_ptr,
    );
}

/// # Safety
///
/// - alpha_byte_out_ptr, alpha_bit_out_ptr, color_byte_out_ptr, and index_byte_out_ptr must be valid for writes  
/// - The distance between pointers must follow the layout expected based on DXT block size  
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32_with_separate_endpoints(
    input_ptr: *const u8,
    mut alpha_byte_out_ptr: *mut u16,
    mut alpha_bit_out_ptr: *mut u16,
    mut color_byte_out_ptr: *mut u32,
    mut index_byte_out_ptr: *mut u32,
    alpha_byte_end_ptr: *mut u16,
) {
    let mut current_input_ptr = input_ptr;

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
/// - len must be divisible by 16
/// - pointers must be properly aligned for u32 access
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let mut alpha_byte_out_ptr = output_ptr as *mut u16;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2) as *mut u16;
    let mut color_byte_out_ptr = output_ptr.add(len / 16 * 8) as *mut u32;
    let mut index_byte_out_ptr = output_ptr.add(len / 16 * 12) as *mut u32;

    let mut current_input_ptr = input_ptr;
    let alpha_byte_end_ptr = output_ptr.add((len / 16 * 2).saturating_sub(16)) as *mut u16;

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

    // Handle remaining blocks
    let alpha_byte_end_ptr = output_ptr.add(len / 16 * 2) as *mut u16;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::{
        assert_implementation_matches_reference, generate_bc3_test_data,
        transform_with_reference_implementation,
    };
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        func: TransformFn,
        name: &'static str,
    }

    #[rstest]
    #[case(TestCase { func: u32, name: "portable32" })]
    fn test_portable32_aligned(#[case] test_case: TestCase) {
        let num_blocks = 64;
        let input = generate_bc3_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len()];
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        let mut output_test = allocate_align_64(input.len());
        output_test.as_mut_slice().fill(0);
        unsafe {
            (test_case.func)(input.as_ptr(), output_test.as_mut_ptr(), input.len());
        }

        assert_implementation_matches_reference(
            output_expected.as_slice(),
            output_test.as_slice(),
            test_case.name,
            num_blocks,
        );
    }

    #[rstest]
    #[case(TestCase { func: u32, name: "portable32" })]
    fn test_portable32_unaligned(#[case] test_case: TestCase) {
        let num_blocks = 64;
        let input = generate_bc3_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len() + 1];
        transform_with_reference_implementation(input.as_slice(), &mut output_expected[1..]);

        let mut output_test = vec![0u8; input.len() + 1];
        output_test.as_mut_slice().fill(0);
        unsafe {
            (test_case.func)(input.as_ptr(), output_test.as_mut_ptr().add(1), input.len());
        }

        assert_implementation_matches_reference(
            &output_expected[1..],
            &output_test[1..],
            test_case.name,
            num_blocks,
        );
    }
}
