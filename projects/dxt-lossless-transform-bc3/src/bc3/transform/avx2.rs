#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::arch::*;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (for eight 16-byte BC3 blocks)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86")]
pub unsafe fn u32_avx2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    // Setup pointers for alpha components
    let mut alpha_byte_out_ptr = output_ptr as *mut u16;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2);
    let mut color_out_ptr = output_ptr.add(len / 16 * 8) as *mut __m256i;
    let mut index_out_ptr = output_ptr.add(len / 16 * 12) as *mut __m256i;

    let mut current_input_ptr = input_ptr;
    let alpha_byte_end_ptr = alpha_bit_out_ptr as *mut u16;

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

    while alpha_byte_out_ptr < alpha_byte_end_ptr {
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
        write_alpha_byte(alpha_byte_out_ptr, current_input_ptr, 0);
        write_alpha_byte(alpha_byte_out_ptr.add(1), current_input_ptr, 16);
        write_alpha_byte(alpha_byte_out_ptr.add(2), current_input_ptr, 32);
        write_alpha_byte(alpha_byte_out_ptr.add(3), current_input_ptr, 48);
        write_alpha_byte(alpha_byte_out_ptr.add(4), current_input_ptr, 64);
        write_alpha_byte(alpha_byte_out_ptr.add(5), current_input_ptr, 80);
        write_alpha_byte(alpha_byte_out_ptr.add(6), current_input_ptr, 96);
        write_alpha_byte(alpha_byte_out_ptr.add(7), current_input_ptr, 112);
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(8);

        // Write out all alpha bit components (2 bytes then 4 bytes for each block), for all 8 blocks
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
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(48);

        // Store results - each register now contains 8 blocks worth of data
        _mm256_storeu_si256(color_out_ptr, colours);
        _mm256_storeu_si256(index_out_ptr, indices);

        // Update pointers
        current_input_ptr = current_input_ptr.add(128); // Move forward 8 blocks
        color_out_ptr = color_out_ptr.add(1);
        index_out_ptr = index_out_ptr.add(1);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (for eight 16-byte BC3 blocks)
/// - pointers must be properly aligned for AVX2 operations (32-byte alignment)
#[target_feature(enable = "avx2")]
#[cfg(target_arch = "x86_64")]
pub unsafe fn u32_avx2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    // Setup pointers for alpha components
    let mut alpha_byte_out_ptr = output_ptr as *mut u16;
    let mut alpha_bit_out_ptr = output_ptr.add(len / 16 * 2);
    let mut color_out_ptr = output_ptr.add(len / 16 * 8) as *mut __m256i;
    let mut index_out_ptr = output_ptr.add(len / 16 * 12) as *mut __m256i;

    let mut current_input_ptr = input_ptr;
    let alpha_byte_end_ptr = alpha_bit_out_ptr as *mut u16;

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

    while alpha_byte_out_ptr < alpha_byte_end_ptr {
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
        write_alpha_byte(alpha_byte_out_ptr, current_input_ptr, 0);
        write_alpha_byte(alpha_byte_out_ptr.add(1), current_input_ptr, 16);
        write_alpha_byte(alpha_byte_out_ptr.add(2), current_input_ptr, 32);
        write_alpha_byte(alpha_byte_out_ptr.add(3), current_input_ptr, 48);
        write_alpha_byte(alpha_byte_out_ptr.add(4), current_input_ptr, 64);
        write_alpha_byte(alpha_byte_out_ptr.add(5), current_input_ptr, 80);
        write_alpha_byte(alpha_byte_out_ptr.add(6), current_input_ptr, 96);
        write_alpha_byte(alpha_byte_out_ptr.add(7), current_input_ptr, 112);
        alpha_byte_out_ptr = alpha_byte_out_ptr.add(8);

        // Write out all alpha bit components (2 bytes then 4 bytes for each block), for all 8 blocks
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
        alpha_bit_out_ptr = alpha_bit_out_ptr.add(48);

        // Store results - each register now contains 8 blocks worth of data
        _mm256_storeu_si256(color_out_ptr, colours);
        _mm256_storeu_si256(index_out_ptr, indices);

        // Update pointers
        current_input_ptr = current_input_ptr.add(128); // Move forward 8 blocks
        color_out_ptr = color_out_ptr.add(1);
        index_out_ptr = index_out_ptr.add(1);
    }
}

#[inline(always)]
unsafe fn write_alpha_byte(out_ptr: *mut u16, in_ptr: *const u8, offset: usize) {
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
#[cfg(target_arch = "x86_64")]
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
    use crate::bc3::transform::tests::generate_bc3_test_data;
    use crate::bc3::transform::tests::transform_with_reference_implementation;
    use rstest::rstest;

    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: TransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::u32_avx2(TestCase {
        name: "u32_avx2 no-unroll",
        func: u32_avx2,
        min_blocks: 8, // 128 bytes, minimum size
        many_blocks: 64,
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
