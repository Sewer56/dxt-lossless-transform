#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::portable32::u32_with_separate_endpoints;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (BC3 block size)
#[target_feature(enable = "avx512vbmi")]
#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
pub unsafe fn avx512_vbmi(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Process 8 blocks (128 bytes) at a time
    let mut aligned_len = len - (len % 128);
    // 16 / 2 == block size / alpha_bytes size
    // 128 == loop length
    aligned_len = aligned_len.saturating_sub(16 / 2 * 128); // 1024
    let remaining_len = len - aligned_len;

    // Setup pointers for alpha components
    let mut alpha_byte_out_ptr = output_ptr;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2);
    let mut color_out_ptr = output_ptr.add(len / 16 * 8);
    let mut index_out_ptr = output_ptr.add(len / 16 * 12);

    let mut current_input_ptr = input_ptr;
    let input_aligned_end_ptr = input_ptr.add(aligned_len);

    // Permute to lift out the alpha bytes from the read blocks.
    #[rustfmt::skip]
    let alpha_bytes_permute_mask: __m512i = _mm512_set_epi8(
        0, 0, 0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        1 + (16 * 3),
        0 + (16 * 3), // block 3
        1 + (16 * 2),
        0 + (16 * 2), // block 2
        1 + (16 * 1),
        0 + (16 * 1), // block 1
        1 + (16 * 0),
        0 + (16 * 0), // block 0
    );

    #[rustfmt::skip]
    let alpha_bits_permute_mask: __m512i = _mm512_set_epi8(
        0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        7 + (16 * 3),
        6 + (16 * 3),
        5 + (16 * 3),
        4 + (16 * 3),
        3 + (16 * 3),
        2 + (16 * 3), // block 3
        7 + (16 * 2),
        6 + (16 * 2),
        5 + (16 * 2),
        4 + (16 * 2),
        3 + (16 * 2),
        2 + (16 * 2), // block 2
        7 + (16 * 1),
        6 + (16 * 1),
        5 + (16 * 1),
        4 + (16 * 1),
        3 + (16 * 1),
        2 + (16 * 1), // block 1
        7 + (16 * 0),
        6 + (16 * 0),
        5 + (16 * 0),
        4 + (16 * 0),
        3 + (16 * 0),
        2 + (16 * 0), // block 0
    );

    #[rustfmt::skip]
    let color_bytes_permute_mask: __m512i = _mm512_set_epi8(
        0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        11 + (16 * 3),
        10 + (16 * 3),
        9 + (16 * 3),
        8 + (16 * 3), // block 3
        11 + (16 * 2),
        10 + (16 * 2),
        9 + (16 * 2),
        8 + (16 * 2), // block 2
        11 + (16 * 1),
        10 + (16 * 1),
        9 + (16 * 1),
        8 + (16 * 1), // block 1
        11 + (16 * 0),
        10 + (16 * 0),
        9 + (16 * 0),
        8 + (16 * 0), // block 0
    );

    #[rustfmt::skip]
    let index_bytes_permute_mask: __m512i = _mm512_set_epi8(
        0, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        15 + (16 * 3),
        14 + (16 * 3),
        13 + (16 * 3),
        12 + (16 * 3), // block 3
        15 + (16 * 2),
        14 + (16 * 2),
        13 + (16 * 2),
        12 + (16 * 2), // block 2
        15 + (16 * 1),
        14 + (16 * 1),
        13 + (16 * 1),
        12 + (16 * 1), // block 1
        15 + (16 * 0),
        14 + (16 * 0),
        13 + (16 * 0),
        12 + (16 * 0), // block 0
    );

    while current_input_ptr < input_aligned_end_ptr {
        // Read 8 blocks (128 bytes)
        let block_0 = _mm512_loadu_si512(current_input_ptr as *const __m512i);
        let block_1 = _mm512_loadu_si512(current_input_ptr.add(64) as *const __m512i);
        current_input_ptr = current_input_ptr.add(128); // Move forward 8 blocks

        let alpha_bytes_0 = _mm512_permutexvar_epi8(alpha_bytes_permute_mask, block_0);
        let alpha_bits_0 = _mm512_permutexvar_epi8(alpha_bits_permute_mask, block_0);
        let color_bytes_0 = _mm512_permutexvar_epi8(color_bytes_permute_mask, block_0);
        let index_bytes_0 = _mm512_permutexvar_epi8(index_bytes_permute_mask, block_0);

        let alpha_bytes_1 = _mm512_permutexvar_epi8(alpha_bytes_permute_mask, block_1);
        let alpha_bits_1 = _mm512_permutexvar_epi8(alpha_bits_permute_mask, block_1);
        let color_bytes_1 = _mm512_permutexvar_epi8(color_bytes_permute_mask, block_1);
        let index_bytes_1 = _mm512_permutexvar_epi8(index_bytes_permute_mask, block_1);

        _mm512_storeu_si512(alpha_byte_out_ptr as *mut __m512i, alpha_bytes_0);
        _mm512_storeu_si512(alpha_bit_out_ptr as *mut __m512i, alpha_bits_0);
        _mm512_storeu_si512(color_out_ptr as *mut __m512i, color_bytes_0);
        _mm512_storeu_si512(index_out_ptr as *mut __m512i, index_bytes_0);

        _mm512_storeu_si512(alpha_byte_out_ptr.add(8) as *mut __m512i, alpha_bytes_1);
        _mm512_storeu_si512(alpha_bit_out_ptr.add(24) as *mut __m512i, alpha_bits_1);
        _mm512_storeu_si512(color_out_ptr.add(16) as *mut __m512i, color_bytes_1);
        _mm512_storeu_si512(index_out_ptr.add(16) as *mut __m512i, index_bytes_1);

        // Update pointers
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(16);
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(48);
        color_out_ptr = color_out_ptr.add(32);
        index_out_ptr = index_out_ptr.add(32);
    }

    // Process any remaining blocks (less than 8)
    if remaining_len > 0 {
        let alpha_byte_end_ptr = output_ptr.add(len / 16 * 2);
        u32_with_separate_endpoints(
            current_input_ptr,              // Start of remaining input data
            alpha_byte_out_ptr as *mut u16, // Start of remaining alpha byte output
            alpha_bit_out_ptr as *mut u16,  // Start of alpha bits
            color_out_ptr as *mut u32,      // Start of remaining color output
            index_out_ptr as *mut u32,      // Start of remaining index output
            alpha_byte_end_ptr as *mut u16, // End of alpha byte section
        );
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

    #[rstest]
    fn test_avx512_aligned() {
        for num_blocks in 1..=512 {
            let input = generate_bc3_test_data(num_blocks);
            let mut output_expected = vec![0u8; input.len()];
            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            let mut output_test = allocate_align_64(input.len());
            output_test.as_mut_slice().fill(0);
            unsafe {
                avx512_vbmi(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                output_test.as_slice(),
                "avx512",
                num_blocks,
            );
        }
    }

    #[rstest]
    fn test_avx512_unaligned() {
        for num_blocks in 1..=512 {
            let input = generate_bc3_test_data(num_blocks);
            let mut output_expected = vec![0u8; input.len() + 1];
            transform_with_reference_implementation(input.as_slice(), &mut output_expected[1..]);

            let mut output_test = vec![0u8; input.len() + 1];
            output_test.as_mut_slice().fill(0);
            unsafe {
                avx512_vbmi(input.as_ptr(), output_test.as_mut_ptr().add(1), input.len());
            }

            assert_implementation_matches_reference(
                &output_expected[1..],
                &output_test[1..],
                "avx512",
                num_blocks,
            );
        }
    }
}
