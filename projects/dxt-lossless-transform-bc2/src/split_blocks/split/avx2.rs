use crate::split_blocks::split::portable32::u32_with_separate_pointers;
use std::arch::asm;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[allow(clippy::unusual_byte_groupings)]
static PERMUTE_MASK: [u32; 8] = [0, 4, 1, 5, 2, 6, 3, 7];

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let aligned_len = len - (len % 128);

    let mut colors_ptr = output_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    // Load the permute mask for 32-bit element reordering
    let permute_mask: __m256i = _mm256_loadu_si256(PERMUTE_MASK.as_ptr() as *const __m256i);

    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);

        asm!(
            ".p2align 5",
            "2:",

            // Load 128 bytes (eight blocks)
            "vmovdqu {ymm0}, [{input_ptr}]",      // First two blocks
            "vmovdqu {ymm1}, [{input_ptr} + 32]", // Second two blocks
            "vmovdqu {ymm3}, [{input_ptr} + 64]", // Third two blocks
            "vmovdqu {ymm4}, [{input_ptr} + 96]", // Fourth two blocks
            "add {input_ptr}, 128",

            // Setup scratch registers.
            "vmovaps {ymm2}, {ymm0}",
            "vmovaps {ymm5}, {ymm3}",
            "vpunpcklqdq {ymm0}, {ymm1}, {ymm0}", // alpha -> ymm0 (out of order)
            "vpunpcklqdq {ymm3}, {ymm4}, {ymm3}", // alpha -> ymm3 (out of order)

            // The registers are like:
            // ymm0: {16, 17, 18, 19, 20, 21, 22, 23, 0, 1, 2, 3, 4, 5, 6, 7, 24, 25, 26, 27, 28, 29, 30, 31, 8, 9, 10, 11, 12, 13, 14, 15}
            // ymm3: {48, 49, 50, 51, 52, 53, 54, 55, 32, 33, 34, 35, 36, 37, 38, 39, 56, 57, 58, 59, 60, 61, 62, 63, 40, 41, 42, 43, 44, 45, 46, 47}
            // Because the block right after last one was in same register.
            // We need to permute them to rearrange items into chronological order:
            "vpermq {ymm0}, {ymm0}, 0x8D", // alpha -> ymm0
            "vpermq {ymm3}, {ymm3}, 0x8D", // alpha -> ymm3

            // Move the colours+indices to ymm2, ymm5
            "vshufps {ymm2}, {ymm2}, {ymm1}, 0xEE",
            "vshufps {ymm5}, {ymm5}, {ymm4}, 0xEE",
            // ymm2 {
            //   [-128, -127, -126, -125],
            //   [-64, -63, -62, -61],
            //   [-120, -119, -118, -117],
            //   [-56, -55, -54, -53],
            //   [-124, -123, -122, -121],
            //   [-60, -59, -58, -57],
            //   [-116, -115, -114, -113],
            //   [-52, -51, -50, -49]
            // }
            // ymm5 {
            //   [-112, -111, -110, -109],
            //   [-48, -47, -46, -45],
            //   [-104, -103, -102, -101],
            //   [-40, -39, -38, -37],
            //   [-108, -107, -106, -105],
            //   [-44, -43, -42, -41],
            //   [-100, -99, -98, -97]
            //   [-36, -35, -34, -33]
            // }

            // Combine colors and indices into separate registers
            "vmovaps {ymm1}, {ymm2}",       // Save ymm2
            "vshufps {ymm2}, {ymm2}, {ymm5}, 0x88", // All colors in ymm2
            "vshufps {ymm1}, {ymm1}, {ymm5}, 0xDD", // All indices in ymm5
            // ymm2 {
            //   [-128, -127, -126, -125],
            //   [-120, -119, -118, -117],
            //   [-112, -111, -110, -109],
            //   [-104, -103, -102, -101],
            //   [-124, -123, -122, -121],
            //   [-116, -115, -114, -113],
            //   [-108, -107, -106, -105],
            //   [-100, -99, -98, -97]
            // }
            // ymm1 {
            //   [-64, -63, -62, -61],
            //   [-56, -55, -54, -53],
            //   [-48, -47, -46, -45],
            //   [-40, -39, -38, -37],
            //   [-60, -59, -58, -57],
            //   [-52, -51, -50, -49],
            //   [-44, -43, -42, -41],
            //   [-36, -35, -34, -33]
            // }
            // We now need to permute across lanes to get our desired output.
            "vpermd {ymm2}, {permute_mask}, {ymm2}", // Permute colors
            "vpermd {ymm1}, {permute_mask}, {ymm1}", // Permute indices

            // Store results
            "vmovdqu [{alpha_ptr}], {ymm0}",
            "vmovdqu [{alpha_ptr} + 32], {ymm3}",
            "add {alpha_ptr}, 64",

            "vmovdqu [{colors_ptr}], {ymm2}",
            "add {colors_ptr}, 32",

            "vmovdqu [{indices_ptr}], {ymm1}",
            "add {indices_ptr}, 32",

            // Loop until done
            "cmp {input_ptr}, {aligned_end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            aligned_end = inout(reg) aligned_end,
            permute_mask = in(ymm_reg) permute_mask, // Pass the mask as a YMM register
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            options(nostack)
        );
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(
            input_ptr,
            output_ptr as *mut u64,
            colors_ptr as *mut u32,
            indices_ptr as *mut u32,
            remaining,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::{
        assert_implementation_matches_reference, generate_bc2_test_data,
        transform_with_reference_implementation,
    };
    use crate::testutils::allocate_align_64;
    use core::ptr::copy_nonoverlapping;
    use rstest::rstest;

    type PermuteFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(shuffle, "avx2_shuffle")]
    fn test_avx2_aligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        if !std::is_x86_feature_detected!("avx2") {
            return;
        }

        for num_blocks in 1..=512 {
            let mut input = allocate_align_64(num_blocks * 16);
            let mut output_expected = allocate_align_64(input.len());
            let mut output_test = allocate_align_64(input.len());

            // Fill the input with test data
            unsafe {
                copy_nonoverlapping(
                    generate_bc2_test_data(num_blocks).as_ptr(),
                    input.as_mut_ptr(),
                    input.len(),
                );
            }

            transform_with_reference_implementation(
                input.as_slice(),
                output_expected.as_mut_slice(),
            );

            output_test.as_mut_slice().fill(0);
            unsafe {
                permute_fn(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                output_test.as_slice(),
                &format!("{impl_name} (aligned)"),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(shuffle, "avx2_shuffle")]
    fn test_avx2_unaligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
        if !std::is_x86_feature_detected!("avx2") {
            return;
        }

        for num_blocks in 1..=512 {
            let input = generate_bc2_test_data(num_blocks);

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut input_unaligned = vec![0u8; input.len() + 1];
            input_unaligned[1..].copy_from_slice(input.as_slice());

            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len() + 1];

            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            output_test.as_mut_slice().fill(0);
            unsafe {
                // Use pointers offset by 1 byte to create unaligned access
                permute_fn(
                    input_unaligned.as_ptr().add(1),
                    output_test.as_mut_ptr().add(1),
                    input.len(),
                );
            }

            assert_implementation_matches_reference(
                output_expected.as_slice(),
                &output_test[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}
