#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::arch::*;

use super::portable32::u32_with_separate_endpoints;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (BC3 block size)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[target_feature(enable = "avx2")]
pub unsafe fn u32_avx2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0, "Length must be a multiple of 16");

    // Process 8 blocks (128 bytes) at a time
    let aligned_len = len - (len % 128);
    let remaining_len = len - aligned_len;

    // Setup pointers for alpha components
    let mut alpha_byte_out_ptr = output_ptr as *mut u16;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2);
    let mut color_out_ptr = output_ptr.add(len / 16 * 8) as *mut __m256i;
    let mut index_out_ptr = output_ptr.add(len / 16 * 12) as *mut __m256i;

    let mut current_input_ptr = input_ptr;
    let input_aligned_end_ptr = input_ptr.add(aligned_len);

    // Create gather indices for colors (offset 8) and indices (offset 12)
    // For eight blocks, each 16 bytes apart
    let colour_offsets = _mm256_set_epi32(
        120, 104, 88, 72, 56, 40, 24, 8, // Block 8, 7, 6, 5, 4, 3, 2, 1 color offsets
    );
    let indices_offsets = _mm256_set_epi32(
        124, 108, 92, 76, 60, 44, 28, 12, // Block 8, 7, 6, 5, 4, 3, 2, 1 index offsets
    );

    // Create gather mask (all 1s)
    let mask = _mm256_set1_epi32(-1);
    let mut colours: __m256i = _mm256_setzero_si256();
    let mut indices: __m256i = _mm256_setzero_si256();

    while current_input_ptr < input_aligned_end_ptr {
        // Use inline assembly for the gather operations
        unsafe {
            asm!(
                "vpgatherdd {colours}, [{current_input_ptr} + {colour_offsets} * 1], {mask}",
                colours = inout(ymm_reg) colours,
                current_input_ptr = in(reg) current_input_ptr,
                colour_offsets = in(ymm_reg) colour_offsets,
                mask = in(ymm_reg) mask,
                options(nostack)
            );

            asm!(
                "vpgatherdd {indices}, [{current_input_ptr} + {indices_offsets} * 1], {mask}",
                indices = inout(ymm_reg) indices,
                current_input_ptr = in(reg) current_input_ptr,
                indices_offsets = in(ymm_reg) indices_offsets,
                mask = in(ymm_reg) mask,
                options(nostack)
            );
        }

        // Write out all alpha bytes first (2 bytes each), for all 8 blocks
        write_alpha_endpoints_u16(alpha_byte_out_ptr, current_input_ptr, 0);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(1), current_input_ptr, 16);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(2), current_input_ptr, 32);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(3), current_input_ptr, 48);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(4), current_input_ptr, 64);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(5), current_input_ptr, 80);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(6), current_input_ptr, 96);
        write_alpha_endpoints_u16(alpha_byte_out_ptr.add(7), current_input_ptr, 112);
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(8);

        // Resolved and optimized out at compile time!
        if cfg!(target_arch = "x86_64") {
            // Write out all alpha indices components 8 bytes at a time, for all 8 blocks
            write_alpha_bits_u64(alpha_bit_out_ptr, 0, current_input_ptr, 2);
            write_alpha_bits_u64(alpha_bit_out_ptr, 6, current_input_ptr, 18);
            write_alpha_bits_u64(alpha_bit_out_ptr, 12, current_input_ptr, 34);
            write_alpha_bits_u64(alpha_bit_out_ptr, 18, current_input_ptr, 50);
            write_alpha_bits_u64(alpha_bit_out_ptr, 24, current_input_ptr, 66);
            write_alpha_bits_u64(alpha_bit_out_ptr, 30, current_input_ptr, 82);
            write_alpha_bits_u64(alpha_bit_out_ptr, 36, current_input_ptr, 98);
            // Note: The u64 write overflows by 2 bytes; so on the last write, we need to not overflow, as to
            // not overwrite elements in the next section; so we do a regular write here.
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 42, 114);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 44, 116);
        } else {
            // Write out all alpha indices components (2 bytes then 4 bytes for each block), for all 8 blocks
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 0, 2);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 2, 4);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 6, 18);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 8, 20);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 12, 34);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 14, 36);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 18, 50);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 20, 52);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 24, 66);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 26, 68);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 30, 82);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 32, 84);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 36, 98);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 38, 100);
            write_alpha_bit_u16(alpha_bit_out_ptr, current_input_ptr, 42, 114);
            write_alpha_bit_u32(alpha_bit_out_ptr, current_input_ptr, 44, 116);
        }

        alpha_bit_out_ptr = alpha_bit_out_ptr.add(48);

        // Store results - each register now contains 8 blocks worth of data
        _mm256_storeu_si256(color_out_ptr, colours);
        _mm256_storeu_si256(index_out_ptr, indices);

        // Update pointers
        current_input_ptr = current_input_ptr.add(128); // Move forward 8 blocks
        color_out_ptr = color_out_ptr.add(1);
        index_out_ptr = index_out_ptr.add(1);
    }

    // Process any remaining blocks (less than 8)
    if remaining_len > 0 {
        let alpha_byte_end_ptr = output_ptr.add(len / 16 * 2);
        u32_with_separate_endpoints(
            current_input_ptr,              // Start of remaining input data
            alpha_byte_out_ptr,             // Start of remaining alpha byte output
            alpha_bit_out_ptr as *mut u16,  // Start of alpha bits
            color_out_ptr as *mut u32,      // Start of remaining color output
            index_out_ptr as *mut u32,      // Start of remaining index output
            alpha_byte_end_ptr as *mut u16, // End of alpha byte section
        );
    }
}

#[inline(always)]
unsafe fn write_alpha_endpoints_u16(out_ptr: *mut u16, in_ptr: *const u8, offset: usize) {
    out_ptr.write_unaligned((in_ptr.add(offset) as *const u16).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bit_u16(
    out_ptr: *mut u8,
    in_ptr: *const u8,
    out_offset: usize,
    in_offset: usize,
) {
    (out_ptr.add(out_offset) as *mut u16)
        .write_unaligned((in_ptr.add(in_offset) as *const u16).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bit_u32(
    out_ptr: *mut u8,
    in_ptr: *const u8,
    out_offset: usize,
    in_offset: usize,
) {
    (out_ptr.add(out_offset) as *mut u32)
        .write_unaligned((in_ptr.add(in_offset) as *const u32).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bits_u64(
    out_ptr: *mut u8,
    out_offset: usize,
    in_ptr: *const u8,
    in_offset: usize,
) {
    // Read both parts using unaligned loads
    let first_part = (in_ptr.add(in_offset) as *const u16).read_unaligned();
    let second_part = (in_ptr.add(in_offset + 2) as *const u32).read_unaligned();
    let combined_value = ((second_part as u64) << 16) | (first_part as u64);

    // Write using unaligned store
    (out_ptr.add(out_offset) as *mut u64).write_unaligned(combined_value);
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

    #[rstest]
    fn test_avx2_aligned() {
        let num_blocks = 64;
        let input = generate_bc3_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len()];
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        let mut output_test = allocate_align_64(input.len());
        output_test.as_mut_slice().fill(0);
        unsafe {
            u32_avx2(input.as_ptr(), output_test.as_mut_ptr(), input.len());
        }

        assert_implementation_matches_reference(
            output_expected.as_slice(),
            output_test.as_slice(),
            "avx2",
            num_blocks,
        );
    }

    #[rstest]
    fn test_avx2_unaligned() {
        let num_blocks = 64;
        let input = generate_bc3_test_data(num_blocks);
        let mut output_expected = vec![0u8; input.len() + 1];
        transform_with_reference_implementation(input.as_slice(), &mut output_expected[1..]);

        let mut output_test = vec![0u8; input.len() + 1];
        output_test.as_mut_slice().fill(0);
        unsafe {
            u32_avx2(input.as_ptr(), output_test.as_mut_ptr().add(1), input.len());
        }

        assert_implementation_matches_reference(
            &output_expected[1..],
            &output_test[1..],
            "avx2",
            num_blocks,
        );
    }
}
