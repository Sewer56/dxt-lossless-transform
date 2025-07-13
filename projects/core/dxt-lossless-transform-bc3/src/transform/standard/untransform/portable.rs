#![allow(missing_docs)]

use crate::unaligned_rw::{UnalignedReadWrite, UnalignedWrite};

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
pub(crate) unsafe fn u32_untransform_with_separate_pointers(
    mut alpha_byte_in_ptr: *const u16,
    mut alpha_bit_in_ptr: *const u16,
    mut color_byte_in_ptr: *const u32,
    mut index_byte_in_ptr: *const u32,
    mut current_output_ptr: *mut u8,
    len: usize,
) {
    let alpha_byte_end_ptr = alpha_byte_in_ptr.add(len / 16);
    while alpha_byte_in_ptr < alpha_byte_end_ptr {
        // Alpha bytes (2 bytes)
        current_output_ptr.write_u16_at(0, alpha_byte_in_ptr.read_u16_at(0));
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);

        // Alpha bits (6 bytes)
        current_output_ptr.write_u16_at(2, alpha_bit_in_ptr.read_u16_at(0));
        current_output_ptr.write_u32_at(4, alpha_bit_in_ptr.read_u32_at(2));
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(3);

        // Color bytes (4 bytes)
        current_output_ptr.write_u32_at(8, color_byte_in_ptr.read_u32_at(0));
        color_byte_in_ptr = color_byte_in_ptr.add(1);

        // Index bytes (4 bytes)
        current_output_ptr.write_u32_at(12, index_byte_in_ptr.read_u32_at(0));
        index_byte_in_ptr = index_byte_in_ptr.add(1);
        current_output_ptr = current_output_ptr.add(16);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads based on the transformed layout derived from len bytes of output.
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
#[cfg_attr(target_arch = "x86_64", allow(dead_code))] // x86_64 does not use this path.
pub(crate) unsafe fn u32_untransform_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));
    const BYTES_PER_ITERATION: usize = 32;
    let aligned_len = len - (len % BYTES_PER_ITERATION);

    // Set up input pointers for each section
    // Pointers are doubly sized for unroll.
    let mut alpha_byte_in_ptr = input_ptr as *const u32;
    let mut alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u16;
    let mut color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const u32;
    let mut index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const u32;
    let mut current_output_ptr = output_ptr;

    if aligned_len > 0 {
        let alpha_byte_aligned_end_ptr = input_ptr.add(aligned_len / 16 * 2) as *const u32;
        while alpha_byte_in_ptr < alpha_byte_aligned_end_ptr {
            let alpha_bytes = alpha_byte_in_ptr.read_unaligned();
            alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);

            // Alpha bytes (2 bytes). Write in 2 blocks in 1 shot.
            #[cfg(target_endian = "little")]
            {
                current_output_ptr.write_u16_at(0, alpha_bytes as u16);
                current_output_ptr.write_u16_at(16, (alpha_bytes >> 16) as u16);
            }
            #[cfg(target_endian = "big")]
            {
                current_output_ptr.write_u16_at(0, (alpha_bytes >> 16) as u16);
                current_output_ptr.write_u16_at(16, alpha_bytes as u16);
            }

            // Alpha bits (6 bytes)
            current_output_ptr.write_u16_at(2, alpha_bit_in_ptr.read_u16_at(0));
            current_output_ptr.write_u32_at(4, alpha_bit_in_ptr.read_u32_at(2));
            current_output_ptr.write_u16_at(2 + 16, alpha_bit_in_ptr.read_u16_at(6));
            current_output_ptr.write_u32_at(4 + 16, alpha_bit_in_ptr.read_u32_at(8));
            alpha_bit_in_ptr = alpha_bit_in_ptr.add(6);

            // Color bytes (4 bytes)
            current_output_ptr.write_u32_at(8, color_byte_in_ptr.read_u32_at(0));
            current_output_ptr.write_u32_at(8 + 16, color_byte_in_ptr.read_u32_at(4));
            color_byte_in_ptr = color_byte_in_ptr.add(2);

            // Index bytes (4 bytes)
            current_output_ptr.write_u32_at(12, index_byte_in_ptr.read_u32_at(0));
            current_output_ptr.write_u32_at(12 + 16, index_byte_in_ptr.read_u32_at(4));
            index_byte_in_ptr = index_byte_in_ptr.add(2);

            current_output_ptr = current_output_ptr.add(32);
        }
    }

    // Process remaining bytes if necessary
    u32_untransform_with_separate_pointers(
        alpha_byte_in_ptr as *const u16,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len - aligned_len,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_untransform_v2, "u32_v2", 2)]
    fn test_portable_unaligned(
        #[case] untransform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For portable: processes 16 bytes (1 block) per iteration, so max_blocks = 16 bytes × 2 ÷ 16 = 2
        run_standard_untransform_unaligned_test(untransform_fn, max_blocks, impl_name);
    }
}
