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

    // Setup pointers for alpha components
    let alpha_byte_out_ptr = output_ptr;
    let alpha_bit_out_ptr = output_ptr.add(len / 16 * 2);
    let color_out_ptr = output_ptr.add(len / 16 * 8);
    let index_out_ptr = output_ptr.add(len / 16 * 12);

    avx512_vbmi_with_separate_pointers(
        input_ptr,
        alpha_byte_out_ptr,
        alpha_bit_out_ptr,
        color_out_ptr,
        index_out_ptr,
        len,
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
#[target_feature(enable = "avx512vbmi")]
#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
pub unsafe fn avx512_vbmi_with_separate_pointers(
    input_ptr: *const u8,
    mut alpha_byte_out_ptr: *mut u8,
    mut alpha_bit_out_ptr: *mut u8,
    mut color_out_ptr: *mut u8,
    mut index_out_ptr: *mut u8,
    len: usize,
) {
    // Note: Leaving as intrinsics because the compiler generated form for ancient CPU
    // produces OK code.
    debug_assert!(len % 16 == 0);

    // Process 8 blocks (128 bytes) at a time
    let mut aligned_len = len - (len % 128);
    // The writes to `alpha_bit_out_ptr` overflows as it uses a 64-bit register to write 48-bits
    // of data.
    aligned_len = aligned_len.saturating_sub(128);
    let remaining_len = len - aligned_len;

    let mut current_input_ptr = input_ptr;
    let input_aligned_end_ptr = input_ptr.add(aligned_len);

    // Note(sewer): We need to pre-calculate this because `alpha_byte_out_ptr` will advance later on.
    let alpha_byte_end_ptr = alpha_byte_out_ptr.add(len / 16 * 2);

    // Permute to lift out the alpha bytes from the read blocks.
    #[rustfmt::skip]
    let alpha_bytes_permute_mask: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        1 + (16 * 7),
        0 + (16 * 7), // block 7
        1 + (16 * 6),
        0 + (16 * 6), // block 6
        1 + (16 * 5),
        0 + (16 * 5), // block 5
        1 + (16 * 4),
        0 + (16 * 4), // block 4
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
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        7 + (16 * 7),
        6 + (16 * 7),
        5 + (16 * 7),
        4 + (16 * 7),
        3 + (16 * 7),
        2 + (16 * 7), // block 7
        7 + (16 * 6),
        6 + (16 * 6),
        5 + (16 * 6),
        4 + (16 * 6),
        3 + (16 * 6),
        2 + (16 * 6), // block 6
        7 + (16 * 5),
        6 + (16 * 5),
        5 + (16 * 5),
        4 + (16 * 5),
        3 + (16 * 5),
        2 + (16 * 5), // block 5
        7 + (16 * 4),
        6 + (16 * 4),
        5 + (16 * 4),
        4 + (16 * 4),
        3 + (16 * 4),
        2 + (16 * 4), // block 4
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
       0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        11 + (16 * 7),
        10 + (16 * 7),
        9 + (16 * 7),
        8 + (16 * 7), // block 7
        11 + (16 * 6),
        10 + (16 * 6),
        9 + (16 * 6),
        8 + (16 * 6), // block 6
        11 + (16 * 5),
        10 + (16 * 5),
        9 + (16 * 5),
        8 + (16 * 5), // block 5
        11 + (16 * 4),
        10 + (16 * 4),
        9 + (16 * 4),
        8 + (16 * 4), // block 4
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
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        15 + (16 * 7),
        14 + (16 * 7),
        13 + (16 * 7),
        12 + (16 * 7), // block 7
        15 + (16 * 6),
        14 + (16 * 6),
        13 + (16 * 6),
        12 + (16 * 6), // block 6
        15 + (16 * 5),
        14 + (16 * 5),
        13 + (16 * 5),
        12 + (16 * 5), // block 5
        15 + (16 * 4),
        14 + (16 * 4),
        13 + (16 * 4),
        12 + (16 * 4), // block 4
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

        let alpha_bytes_0 = _mm512_permutex2var_epi8(block_0, alpha_bytes_permute_mask, block_1);
        let alpha_bits_0 = _mm512_permutex2var_epi8(block_0, alpha_bits_permute_mask, block_1);
        let color_bytes_0 = _mm512_permutex2var_epi8(block_0, color_bytes_permute_mask, block_1);
        let index_bytes_0 = _mm512_permutex2var_epi8(block_0, index_bytes_permute_mask, block_1);

        _mm_storeu_si128(
            alpha_byte_out_ptr as *mut __m128i,
            _mm512_castsi512_si128(alpha_bytes_0),
        ); // only 16 to write, so xmm
        _mm512_storeu_si512(alpha_bit_out_ptr as *mut __m512i, alpha_bits_0); // 48 to write, so zmm with a bit of overlap
        _mm256_storeu_si256(
            color_out_ptr as *mut __m256i,
            _mm512_castsi512_si256(color_bytes_0),
        ); // 32 to write, so ymm
        _mm256_storeu_si256(
            index_out_ptr as *mut __m256i,
            _mm512_castsi512_si256(index_bytes_0),
        ); // 32 to write, so ymm

        // Update pointers
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(16);
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(48);
        color_out_ptr = color_out_ptr.add(32);
        index_out_ptr = index_out_ptr.add(32);
    }

    // Process any remaining blocks (less than 8)
    if remaining_len > 0 {
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
        if !dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            return;
        }

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
        if !dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            return;
        }

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

    #[rstest]
    fn avx512_vbmi_with_separate_pointers_matches_avx512_vbmi() {
        if !dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            return;
        }

        for num_blocks in 1..=256 {
            let input = generate_bc3_test_data(num_blocks);
            let len = input.len();

            // Test with the contiguous buffer method
            let mut output_contiguous = allocate_align_64(len);

            // Test with separate pointers
            let mut alpha_bytes = allocate_align_64(len / 8); // 2 bytes per block
            let mut alpha_bits = allocate_align_64(len * 3 / 8); // 6 bytes per block
            let mut colors = allocate_align_64(len / 4); // 4 bytes per block
            let mut indices = allocate_align_64(len / 4); // 4 bytes per block

            unsafe {
                // Reference: contiguous buffer using AVX512
                avx512_vbmi(input.as_ptr(), output_contiguous.as_mut_ptr(), len);

                // Test: separate pointers using AVX512
                avx512_vbmi_with_separate_pointers(
                    input.as_ptr(),
                    alpha_bytes.as_mut_ptr(),
                    alpha_bits.as_mut_ptr(),
                    colors.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    len,
                );
            }

            // Verify that separate pointer results match contiguous buffer layout
            let expected_alpha_bytes = &output_contiguous.as_slice()[0..len / 8];
            let expected_alpha_bits = &output_contiguous.as_slice()[len / 8..len / 2];
            let expected_colors = &output_contiguous.as_slice()[len / 2..len * 3 / 4];
            let expected_indices = &output_contiguous.as_slice()[len * 3 / 4..];

            assert_eq!(
                alpha_bytes.as_slice(),
                expected_alpha_bytes,
                "AVX512 Alpha bytes section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                alpha_bits.as_slice(),
                expected_alpha_bits,
                "AVX512 Alpha bits section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                colors.as_slice(),
                expected_colors,
                "AVX512 Color section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                indices.as_slice(),
                expected_indices,
                "AVX512 Index section mismatch for {num_blocks} blocks"
            );
        }
    }
}
