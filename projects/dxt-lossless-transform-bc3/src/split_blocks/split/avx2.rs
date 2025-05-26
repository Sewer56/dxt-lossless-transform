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
#[target_feature(enable = "avx2")]
pub unsafe fn u32_avx2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Setup pointers for alpha components
    let alpha_byte_out_ptr = output_ptr as *mut u16;
    let alpha_bit_out_ptr = output_ptr.add(len / 16 * 2);
    let color_out_ptr = output_ptr.add(len / 16 * 8) as *mut u32;
    let index_out_ptr = output_ptr.add(len / 16 * 12) as *mut u32;
    let alpha_byte_end_ptr = output_ptr.add(len / 16 * 2) as *mut u16;

    u32_avx2_with_separate_pointers(
        input_ptr,
        alpha_byte_out_ptr,
        alpha_bit_out_ptr,
        color_out_ptr,
        index_out_ptr,
        alpha_byte_end_ptr,
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
#[target_feature(enable = "avx2")]
pub unsafe fn u32_avx2_with_separate_pointers(
    input_ptr: *const u8,
    mut alpha_byte_out_ptr: *mut u16,
    mut alpha_bit_out_ptr: *mut u8,
    mut color_out_ptr: *mut u32,
    mut index_out_ptr: *mut u32,
    alpha_byte_end_ptr: *mut u16,
) {
    let len = (alpha_byte_end_ptr as usize - alpha_byte_out_ptr as usize) * 8; // Convert from u16 count to bytes

    // Process 8 blocks (128 bytes) at a time
    let aligned_len = len - (len % 128);
    let remaining_len = len - aligned_len;

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
    let mut colours: __m256i;
    let mut indices: __m256i;

    while current_input_ptr < input_aligned_end_ptr {
        // Use intrinsics for the gather operations
        unsafe {
            // Gather colors using _mm256_mask_i32gather_epi32 intrinsic
            // Parameters: src, base_addr, vindex, mask, scale
            colours = _mm256_mask_i32gather_epi32::<1>(
                _mm256_setzero_si256(),          // src: source where no elements are gathered
                current_input_ptr as *const i32, // base_addr: base pointer
                colour_offsets,                  // vindex: offsets to gather from
                mask,                            // mask: which elements to gather
            );

            // Gather indices using _mm256_mask_i32gather_epi32 intrinsic
            // Parameters: src, base_addr, vindex, mask, scale
            indices = _mm256_mask_i32gather_epi32::<1>(
                _mm256_setzero_si256(),          // src: source where no elements are gathered
                current_input_ptr as *const i32, // base_addr: base pointer
                indices_offsets,                 // vindex: offsets to gather from
                mask,                            // mask: which elements to gather
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
        _mm256_storeu_si256(color_out_ptr as *mut __m256i, colours);
        _mm256_storeu_si256(index_out_ptr as *mut __m256i, indices);

        // Update pointers
        current_input_ptr = current_input_ptr.add(128); // Move forward 8 blocks
        color_out_ptr = color_out_ptr.add(8); // 8 u32s per m256i
        index_out_ptr = index_out_ptr.add(8); // 8 u32s per m256i
    }

    // Process any remaining blocks (less than 8)
    if remaining_len > 0 {
        u32_with_separate_endpoints(
            current_input_ptr,             // Start of remaining input data
            alpha_byte_out_ptr,            // Start of remaining alpha byte output
            alpha_bit_out_ptr as *mut u16, // Start of alpha bits
            color_out_ptr,                 // Start of remaining color output
            index_out_ptr,                 // Start of remaining index output
            alpha_byte_end_ptr,            // End of alpha byte section
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
        if !dxt_lossless_transform_common::cpu_detect::has_avx2() {
            return;
        }

        for num_blocks in 1..=512 {
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
    }

    #[rstest]
    fn test_avx2_unaligned() {
        if !dxt_lossless_transform_common::cpu_detect::has_avx2() {
            return;
        }

        for num_blocks in 1..=512 {
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

    #[rstest]
    fn avx2_separate_pointers_matches_contiguous() {
        if !dxt_lossless_transform_common::cpu_detect::has_avx2() {
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
                // Reference: contiguous buffer
                u32_avx2(input.as_ptr(), output_contiguous.as_mut_ptr(), len);

                // Test: separate pointers
                u32_avx2_with_separate_pointers(
                    input.as_ptr(),
                    alpha_bytes.as_mut_ptr() as *mut u16,
                    alpha_bits.as_mut_ptr(),
                    colors.as_mut_ptr() as *mut u32,
                    indices.as_mut_ptr() as *mut u32,
                    (alpha_bytes.as_mut_ptr() as *mut u16).add(len / 16),
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
                "AVX2 Alpha bytes section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                alpha_bits.as_slice(),
                expected_alpha_bits,
                "AVX2 Alpha bits section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                colors.as_slice(),
                expected_colors,
                "AVX2 Color section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                indices.as_slice(),
                expected_indices,
                "AVX2 Index section mismatch for {num_blocks} blocks"
            );
        }
    }
}
