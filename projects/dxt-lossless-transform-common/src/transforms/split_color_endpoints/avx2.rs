use core::arch::x86_64::*;

/// Splits the colour endpoints using AVX2 instructions
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len_bytes` must be a multiple of 64
/// - Pointers should be 32-byte aligned for best performance
/// - CPU must support AVX2 instructions
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
pub unsafe fn avx2_shuf_impl_asm(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 64 && colors_len_bytes % 64 == 0,
        "colors_len_bytes must be at least 64 and a multiple of 64"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);

    // Input data:
    // [00 01], (80 81), [02 03], (82 83), [04 05], (84 85), [06 07], (86 87)
    // [08 09], (88 89), [0A 0B], (8A 8B), [0C 0D], (8C 8D), [0E 0F], (8E 8F)
    // [10 11], (90 91), [12 13], (92 93), [14 15], (94 95), [16 17], (96 97)
    // [18 19], (98 99), [1A 1B], (9A 9B), [1C 1D], (9C 9D), [1E 1F], (9E 9F)

    // Desired format (big endian):
    // [00 01], [02 03], [04 05], [06 07], (80 81), (82 83), (84 85), (86 87)
    // [08 09], [0A 0B], [0C 0D], [0E 0F], (88 89), (8A 8B), (8C 8D), (8E 8F)
    // [10 11], [12 13], [14 15], [16 17], (90 91), (92 93), (94 95), (96 97)
    // [18 19], [1A 1B], [1C 1D], [1E 1F], (98 99), (9A 9B), (9C 9D), (9E 9F)
    let shuffle_mask = _mm256_set_epi8(
        13, 12, 9, 8, 15, 14, 11, 10, // xmm high
        5, 4, 1, 0, 7, 6, 3, 2, // xmm low
        13, 12, 9, 8, 15, 14, 11, 10, // xmm high
        5, 4, 1, 0, 7, 6, 3, 2, // xmm low
    );

    std::arch::asm!(
        // Loop alignment
        ".p2align 4",

        // Main processing loop
        "2:",  // Label 1 (loop start)

        // Load 64 bytes (2 YMM registers)
        "vmovdqu {ymm1}, ymmword ptr [{src_ptr}]",
        "vmovdqu {ymm2}, ymmword ptr [{src_ptr} + 32]",
        "add {src_ptr}, 64",

        // Shuffle bytes according to mask
        "vpshufb {ymm1}, {ymm1}, {shuffle_mask}",
        "vpshufb {ymm2}, {ymm2}, {shuffle_mask}",

        // Interleave the results
        "vshufps {ymm3}, {ymm1}, {ymm2}, 221",  // 11011101b
        "vshufps {ymm1}, {ymm1}, {ymm2}, 136",  // 10001000b

        // Rearrange the permuted data
        "vpermpd {ymm3}, {ymm3}, 216",  // 11011000b
        "vpermpd {ymm1}, {ymm1}, 216",  // 11011000b

        // Store results
        "vmovups ymmword ptr [{out_low}], {ymm3}",
        "vmovups ymmword ptr [{out_high}], {ymm1}",
        "add {out_low}, 32",
        "add {out_high}, 32",

        // Loop condition
        "cmp {src_ptr}, {end_ptr}",
        "jb 2b",

        src_ptr = inout(reg) input_ptr,
        out_low = inout(reg) output_low,
        out_high = inout(reg) output_high,
        end_ptr = in(reg) end_ptr,

        // AVX registers that need to be managed
        shuffle_mask = in(ymm_reg) shuffle_mask,
        ymm1 = out(ymm_reg) _,
        ymm2 = out(ymm_reg) _,
        ymm3 = out(ymm_reg) _,

        options(preserves_flags, nostack)
    );
}

/// Splits the colour endpoints using AVX2 instructions
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len_bytes` must be a multiple of 64
/// - Pointers should be 32-byte aligned for best performance
/// - CPU must support AVX2 instructions
///
/// # Remarks
///
/// See the [`avx2_shuf_impl_asm`] for inline assembly implementation, which should be preferred.
#[target_feature(enable = "avx2")]
pub unsafe fn avx2_shuf_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 64 && colors_len_bytes % 64 == 0,
        "colors_len_bytes must be at least 64 and a multiple of 64"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);

    // Input data:
    // [00 01], (80 81), [02 03], (82 83), [04 05], (84 85), [06 07], (86 87)
    // [08 09], (88 89), [0A 0B], (8A 8B), [0C 0D], (8C 8D), [0E 0F], (8E 8F)
    // [10 11], (90 91), [12 13], (92 93), [14 15], (94 95), [16 17], (96 97)
    // [18 19], (98 99), [1A 1B], (9A 9B), [1C 1D], (9C 9D), [1E 1F], (9E 9F)

    // Desired format (big endian):
    // [00 01], [02 03], [04 05], [06 07], (80 81), (82 83), (84 85), (86 87)
    // [08 09], [0A 0B], [0C 0D], [0E 0F], (88 89), (8A 8B), (8C 8D), (8E 8F)
    // [10 11], [12 13], [14 15], [16 17], (90 91), (92 93), (94 95), (96 97)
    // [18 19], [1A 1B], [1C 1D], [1E 1F], (98 99), (9A 9B), (9C 9D), (9E 9F)
    let shuffle_mask = _mm256_set_epi8(
        13, 12, 9, 8, 15, 14, 11, 10, // xmm high
        5, 4, 1, 0, 7, 6, 3, 2, // xmm low
        13, 12, 9, 8, 15, 14, 11, 10, // xmm high
        5, 4, 1, 0, 7, 6, 3, 2, // xmm low
    );

    // Note(sewer): Don't forget the damned little endian!! Flip the order!!

    while input_ptr < end_ptr {
        // Load 64 bytes (2 blocks of 32 bytes each)
        let chunk1 = _mm256_loadu_si256(input_ptr as *const __m256i);
        let chunk2 = _mm256_loadu_si256(input_ptr.add(32) as *const __m256i);

        // Apply the shuffle mask to reorganize bytes
        let shuffled1 = _mm256_shuffle_epi8(chunk1, shuffle_mask);
        let shuffled2 = _mm256_shuffle_epi8(chunk2, shuffle_mask);

        // Extract color0, color1 components from both chunks
        let shuffled1_ps = _mm256_castsi256_ps(shuffled1);
        let shuffled2_ps = _mm256_castsi256_ps(shuffled2);
        let color0_combined = _mm256_shuffle_ps::<0b11_01_11_01>(shuffled1_ps, shuffled2_ps);
        let color1_combined = _mm256_shuffle_ps::<0b10_00_10_00>(shuffled1_ps, shuffled2_ps);

        // Now rearrange them back into right order.
        let shuffled1_pd = _mm256_castps_pd(color0_combined);
        let shuffled2_pd = _mm256_castps_pd(color1_combined);
        let color0_rearranged = _mm256_permute4x64_pd::<0b11_01_10_00>(shuffled1_pd);
        let color1_rearranged = _mm256_permute4x64_pd::<0b11_01_10_00>(shuffled2_pd);

        // Store results
        _mm256_storeu_pd(output_low as *mut f64, color0_rearranged);
        _mm256_storeu_pd(output_high as *mut f64, color1_rearranged);

        // Update pointers for the next iteration
        input_ptr = input_ptr.add(64);
        output_low = output_low.add(32);
        output_high = output_high.add(32);
    }
}

/// Splits the colour endpoints using AVX2 instructions with a single loop unroll
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len_bytes` must be a multiple of 128
/// - Pointers should be 32-byte aligned for best performance
/// - CPU must support AVX2 instructions
#[target_feature(enable = "avx2")]
pub unsafe fn avx2_shuf_impl_unroll_2(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 128 && colors_len_bytes % 128 == 0,
        "colors_len_bytes must be at least 128 and a multiple of 128"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 128 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);

    // Input data:
    // [00 01], (80 81), [02 03], (82 83), [04 05], (84 85), [06 07], (86 87)
    // [08 09], (88 89), [0A 0B], (8A 8B), [0C 0D], (8C 8D), [0E 0F], (8E 8F)
    // [10 11], (90 91), [12 13], (92 93), [14 15], (94 95), [16 17], (96 97)
    // [18 19], (98 99), [1A 1B], (9A 9B), [1C 1D], (9C 9D), [1E 1F], (9E 9F)

    // Desired format (big endian):
    // [00 01], [02 03], [04 05], [06 07], (80 81), (82 83), (84 85), (86 87)
    // [08 09], [0A 0B], [0C 0D], [0E 0F], (88 89), (8A 8B), (8C 8D), (8E 8F)
    // [10 11], [12 13], [14 15], [16 17], (90 91), (92 93), (94 95), (96 97)
    // [18 19], [1A 1B], [1C 1D], [1E 1F], (98 99), (9A 9B), (9C 9D), (9E 9F)
    let shuffle_mask = _mm256_set_epi8(
        13, 12, 9, 8, 15, 14, 11, 10, // xmm high
        5, 4, 1, 0, 7, 6, 3, 2, // xmm low
        13, 12, 9, 8, 15, 14, 11, 10, // xmm high
        5, 4, 1, 0, 7, 6, 3, 2, // xmm low
    );

    // Note(sewer): Don't forget the damned little endian!! Flip the order!!

    while input_ptr < end_ptr {
        // First 64 bytes
        // Load 64 bytes (2 blocks of 32 bytes each)
        let chunk1_a = _mm256_loadu_si256(input_ptr as *const __m256i);
        let chunk2_a = _mm256_loadu_si256(input_ptr.add(32) as *const __m256i);

        // Apply the shuffle mask to reorganize bytes
        let shuffled1_a = _mm256_shuffle_epi8(chunk1_a, shuffle_mask);
        let shuffled2_a = _mm256_shuffle_epi8(chunk2_a, shuffle_mask);

        // Extract color0, color1 components from both chunks
        let shuffled1_ps_a = _mm256_castsi256_ps(shuffled1_a);
        let shuffled2_ps_a = _mm256_castsi256_ps(shuffled2_a);
        let color0_combined_a = _mm256_shuffle_ps::<0b11_01_11_01>(shuffled1_ps_a, shuffled2_ps_a);
        let color1_combined_a = _mm256_shuffle_ps::<0b10_00_10_00>(shuffled1_ps_a, shuffled2_ps_a);

        // Now rearrange them back into right order.
        let shuffled1_pd_a = _mm256_castps_pd(color0_combined_a);
        let shuffled2_pd_a = _mm256_castps_pd(color1_combined_a);
        let color0_rearranged_a = _mm256_permute4x64_pd::<0b11_01_10_00>(shuffled1_pd_a);
        let color1_rearranged_a = _mm256_permute4x64_pd::<0b11_01_10_00>(shuffled2_pd_a);

        // Second 64 bytes (unrolled iteration)
        // Load next 64 bytes (2 blocks of 32 bytes each)
        let chunk1_b = _mm256_loadu_si256(input_ptr.add(64) as *const __m256i);
        let chunk2_b = _mm256_loadu_si256(input_ptr.add(96) as *const __m256i);

        // Apply the shuffle mask to reorganize bytes
        let shuffled1_b = _mm256_shuffle_epi8(chunk1_b, shuffle_mask);
        let shuffled2_b = _mm256_shuffle_epi8(chunk2_b, shuffle_mask);

        // Extract color0, color1 components from both chunks
        let shuffled1_ps_b = _mm256_castsi256_ps(shuffled1_b);
        let shuffled2_ps_b = _mm256_castsi256_ps(shuffled2_b);
        let color0_combined_b = _mm256_shuffle_ps::<0b11_01_11_01>(shuffled1_ps_b, shuffled2_ps_b);
        let color1_combined_b = _mm256_shuffle_ps::<0b10_00_10_00>(shuffled1_ps_b, shuffled2_ps_b);

        // Now rearrange them back into right order.
        let shuffled1_pd_b = _mm256_castps_pd(color0_combined_b);
        let shuffled2_pd_b = _mm256_castps_pd(color1_combined_b);
        let color0_rearranged_b = _mm256_permute4x64_pd::<0b11_01_10_00>(shuffled1_pd_b);
        let color1_rearranged_b = _mm256_permute4x64_pd::<0b11_01_10_00>(shuffled2_pd_b);

        // Store results
        _mm256_storeu_pd(output_low as *mut f64, color0_rearranged_a);
        _mm256_storeu_pd(output_low.add(32) as *mut f64, color0_rearranged_b);
        _mm256_storeu_pd(output_high as *mut f64, color1_rearranged_a);
        _mm256_storeu_pd(output_high.add(32) as *mut f64, color1_rearranged_b);

        // Update pointers for the next iteration
        input_ptr = input_ptr.add(128);
        output_low = output_low.add(64);
        output_high = output_high.add(64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::split_color_endpoints::tests::{
        generate_test_data, transform_with_reference_implementation,
    };
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::single(32)] // 64 bytes - two iterations
    #[case::many_unrolls(64)] // 128 bytes - tests multiple iterations
    #[case::large(512)] // 1024 bytes - large dataset
    fn test_implementations(#[case] num_pairs: usize) {
        let input = generate_test_data(num_pairs);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test the AVX2 implementation
        let implementations: [(&str, TransformFn); 3] = [
            ("avx2_shuf", avx2_shuf_impl),
            ("avx2_shuf_asm", avx2_shuf_impl_asm),
            ("avx2_shuf_unrolled", avx2_shuf_impl_unroll_2),
        ];

        for (impl_name, implementation) in implementations {
            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
            assert_eq!(
                output_expected, output_test,
                "{} implementation produced different results than reference for {} color pairs.\n\
                First differing pair will have predictable values:\n\
                Color0: Sequential bytes 0x00,0x01 + (pair_num * 4)\n\
                Color1: Sequential bytes 0x80,0x81 + (pair_num * 4)",
                impl_name, num_pairs
            );
        }
    }
}
