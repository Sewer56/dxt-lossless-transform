use crate::transforms::standard::untransform::portable::u32_untransform_with_separate_pointers;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub(crate) unsafe fn u32_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr as *const u16;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u16;
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const u32;
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const u32;
    let current_output_ptr = output_ptr;

    u32_untransform_with_separate_pointers(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len,
    );
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
pub(crate) unsafe fn u64_untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));
    const BYTES_PER_ITERATION: usize = 32;
    let aligned_len = len - (len % BYTES_PER_ITERATION);

    // Set up input pointers for each section
    // Pointers are doubly sized for unroll.
    let mut alpha_byte_in_ptr = input_ptr as *const u32;
    let mut alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u16;
    let mut color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const u64;
    let mut index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const u64;

    let mut current_output_ptr = output_ptr;

    if aligned_len > 0 {
        let alpha_byte_aligned_end_ptr = input_ptr.add(aligned_len / 16 * 2) as *const u32;
        while alpha_byte_in_ptr < alpha_byte_aligned_end_ptr {
            let alpha_bytes = alpha_byte_in_ptr.read_unaligned();
            let color_bytes = color_byte_in_ptr.read_unaligned();
            let index_bytes = index_byte_in_ptr.read_unaligned();
            alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);
            color_byte_in_ptr = color_byte_in_ptr.add(1);
            index_byte_in_ptr = index_byte_in_ptr.add(1);

            // Alpha bytes (2 bytes). Write in 2 blocks in 1 shot.
            #[cfg(target_endian = "little")]
            {
                (current_output_ptr as *mut u16).write_unaligned(alpha_bytes as u16);
                ((current_output_ptr).add(16) as *mut u16)
                    .write_unaligned((alpha_bytes >> 16) as u16);
            }
            #[cfg(target_endian = "big")]
            {
                (current_output_ptr as *mut u16).write_unaligned((alpha_bytes >> 16) as u16);
                ((current_output_ptr).add(16) as *mut u16).write_unaligned(alpha_bytes as u16);
            }

            // Alpha bits (6 bytes)
            (current_output_ptr.add(2) as *mut u16)
                .write_unaligned(alpha_bit_in_ptr.read_unaligned());
            (current_output_ptr.add(4) as *mut u32)
                .write_unaligned((alpha_bit_in_ptr.add(1) as *const u32).read_unaligned());
            (current_output_ptr.add(2 + 16) as *mut u16)
                .write_unaligned(alpha_bit_in_ptr.add(3).read_unaligned());
            (current_output_ptr.add(4 + 16) as *mut u32)
                .write_unaligned((alpha_bit_in_ptr.add(4) as *const u32).read_unaligned());
            alpha_bit_in_ptr = alpha_bit_in_ptr.add(6);

            // Color bytes (4 bytes)
            #[cfg(target_endian = "little")]
            {
                let color_index_bytes_0 =
                    (color_bytes & 0xFFFFFFFF) | ((index_bytes & 0xFFFFFFFF) << 32);
                let color_index_bytes_1 = (color_bytes >> 32) | ((index_bytes >> 32) << 32);
                (current_output_ptr.add(8) as *mut u64).write_unaligned(color_index_bytes_0);
                (current_output_ptr.add(8 + 16) as *mut u64).write_unaligned(color_index_bytes_1);
            }
            #[cfg(target_endian = "big")]
            {
                let color_index_bytes_0 = (index_bytes >> 32) | ((color_bytes >> 32) << 32);
                let color_index_bytes_1 =
                    (index_bytes & 0xFFFFFFFF) | ((color_bytes & 0xFFFFFFFF) << 32);
                (current_output_ptr.add(8) as *mut u64).write_unaligned(color_index_bytes_0);
                (current_output_ptr.add(8 + 16) as *mut u64).write_unaligned(color_index_bytes_1);
            }

            // Index bytes (4 bytes)
            current_output_ptr = current_output_ptr.add(32);
        }
    }

    // Process remaining bytes if necessary
    u32_untransform_with_separate_pointers(
        alpha_byte_in_ptr as *const u16,
        alpha_bit_in_ptr,
        color_byte_in_ptr as *const u32,
        index_byte_in_ptr as *const u32,
        current_output_ptr,
        len - aligned_len,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_untransform, "u32", 2)]
    #[case(u64_untransform, "u64", 2)]
    fn test_portable_unaligned(
        #[case] untransform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For portable: processes 16 bytes (1 block) per iteration, so max_blocks = 16 bytes × 2 ÷ 16 = 2
        run_standard_untransform_unaligned_test(untransform_fn, max_blocks, impl_name);
    }
}
