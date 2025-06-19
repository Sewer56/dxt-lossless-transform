use crate::transforms::split_565_color_endpoints::portable32::u32_with_separate_endpoints;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
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
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 32-byte aligned for best performance
/// - CPU must support AVX2 instructions
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
pub(crate) unsafe fn avx2_shuf_impl_asm(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);
    let aligned_end_ptr = end_ptr.sub(64);

    if input_ptr < aligned_end_ptr {
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

        core::arch::asm!(
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
            "cmp {src_ptr}, {aligned_end_ptr}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            out_low = inout(reg) output_low,
            out_high = inout(reg) output_high,
            aligned_end_ptr = in(reg) aligned_end_ptr,

            // AVX registers that need to be managed
            shuffle_mask = in(ymm_reg) shuffle_mask,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,

            options(preserves_flags, nostack)
        );
    }

    // Handle remaining elements
    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output_low as *mut u16,
        output_high as *mut u16,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::split_565_color_endpoints::tests::{
        assert_implementation_matches_reference, generate_test_data,
        transform_with_reference_implementation,
    };
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(avx2_shuf_impl_asm, "avx2_shuf_asm")]
    fn test_avx2_aligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        for num_pairs in 1..=512 {
            let input = generate_test_data(num_pairs);
            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len()];

            // Generate reference output
            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
            assert_implementation_matches_reference(
                &output_expected,
                &output_test,
                &format!("{impl_name} (aligned)"),
                num_pairs,
            );
        }
    }

    #[rstest]
    #[case(avx2_shuf_impl_asm, "avx2_shuf_asm")]
    fn test_avx2_unaligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        for num_pairs in 1..=512 {
            let input = generate_test_data(num_pairs);

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut input_unaligned = vec![0u8; input.len() + 1];
            input_unaligned[1..].copy_from_slice(input.as_slice());

            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len() + 1];

            // Generate reference output
            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                // Use pointers offset by 1 byte to create unaligned access
                implementation(
                    input_unaligned.as_ptr().add(1),
                    output_test.as_mut_ptr().add(1),
                    input.len(),
                );
            }

            // Compare results
            assert_implementation_matches_reference(
                &output_expected,
                &output_test[1..],
                &format!("{impl_name} (unaligned)"),
                num_pairs,
            );
        }
    }
}
