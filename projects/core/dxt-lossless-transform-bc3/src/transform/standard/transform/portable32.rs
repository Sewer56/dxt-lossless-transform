#![allow(missing_docs)]

use crate::unaligned_rw::{UnalignedReadWrite, UnalignedWrite};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub(crate) unsafe fn u32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

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
/// - input_ptr must be valid for reads of len bytes
/// - alpha_byte_out_ptr must be valid for writes of len/8 bytes (2 bytes per BC3 block)
/// - alpha_bit_out_ptr must be valid for writes of len*3/8 bytes (6 bytes per BC3 block)
/// - color_out_ptr must be valid for writes of len/4 bytes (4 bytes per BC3 block)
/// - index_out_ptr must be valid for writes of len/4 bytes (4 bytes per BC3 block)
/// - alpha_byte_end_ptr must equal alpha_byte_out_ptr + (len/16) when cast to u16 pointers
/// - All output buffers must not overlap with each other or the input buffer
/// - len must be divisible by 16 (BC3 block size)
pub(crate) unsafe fn u32_with_separate_endpoints(
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
        alpha_byte_out_ptr.write_u16_at(0, current_input_ptr.read_u16_at(0));
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(1); // 2 bytes forward

        // Alpha bits (6 bytes)
        alpha_bit_out_ptr.write_u16_at(0, current_input_ptr.read_u16_at(2));
        alpha_bit_out_ptr.write_u32_at(2, current_input_ptr.read_u32_at(4));
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(3); // 6 bytes forward

        // Color bytes (4 bytes)
        color_byte_out_ptr.write_u32_at(0, current_input_ptr.read_u32_at(8));
        color_byte_out_ptr = color_byte_out_ptr.add(1); // 4 bytes forward

        // Index bytes
        index_byte_out_ptr.write_u32_at(0, current_input_ptr.read_u32_at(12));
        index_byte_out_ptr = index_byte_out_ptr.add(1); // 4 bytes forward
        current_input_ptr = current_input_ptr.add(16); // 16 bytes forward
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32, "portable32", 2)]
    fn test_portable32_unaligned(
        #[case] transform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For portable32: processes 16 bytes (1 block) per iteration, so max_blocks = 16 bytes × 2 ÷ 16 = 2
        run_standard_transform_unaligned_test(transform_fn, max_blocks, impl_name);
    }
}
