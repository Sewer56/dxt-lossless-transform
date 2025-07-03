/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub unsafe fn u32_unroll_2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

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
    crate::transforms::standard::transform::portable32::u32_with_separate_endpoints(
        current_input_ptr,
        alpha_byte_out_ptr,
        alpha_bit_out_ptr,
        color_byte_out_ptr,
        index_byte_out_ptr,
        alpha_byte_end_ptr,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_unroll_2, "portable32_unroll_2", 2)]
    fn test_portable32_unaligned(
        #[case] transform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For portable32: processes 16 bytes (1 block) per iteration, so max_blocks = 16 bytes × 2 ÷ 16 = 2
        run_standard_transform_unaligned_test(transform_fn, max_blocks, impl_name);
    }
}
