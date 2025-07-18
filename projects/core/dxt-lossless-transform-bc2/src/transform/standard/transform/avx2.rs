use crate::transform::standard::transform::portable32::u32_with_separate_pointers;
use core::arch::asm;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[allow(clippy::unusual_byte_groupings)]
const PERMUTE_MASK: [u32; 8] = [0, 4, 1, 5, 2, 6, 3, 7];

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - alphas_ptr must be valid for writes of len/2 bytes
/// - colors_ptr must be valid for writes of len/4 bytes
/// - indices_ptr must be valid for writes of len/4 bytes
/// - len must be divisible by 16
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
pub(crate) unsafe fn shuffle_with_separate_pointers(
    mut input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len.is_multiple_of(16));

    let aligned_len = len - (len % 128);

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
            // ymm0: {
            //          [16, 17, 18, 19],   // 00
            //          [20, 21, 22, 23],
            //          [0, 1, 2, 3],       // 01
            //          [4, 5, 6, 7],
            //          [24, 25, 26, 27],   // 10
            //          [28, 29, 30, 31],
            //          [8, 9, 10, 11],     // 11
            //          [12, 13, 14, 15]
            // }
            // ymm3: {48, 49, 50, 51, 52, 53, 54, 55, 32, 33, 34, 35, 36, 37, 38, 39, 56, 57, 58, 59, 60, 61, 62, 63, 40, 41, 42, 43, 44, 45, 46, 47}
            // Because the block right after last one was in same register.
            // We need to permute them to rearrange items into chronological order:
            "vpermq {ymm0}, {ymm0}, 0x8D", // alpha -> ymm0 | 0x8D -> 10_00_11_01 | ymm0 = ymm0[1,3,0,2]
            "vpermq {ymm3}, {ymm3}, 0x8D", // alpha -> ymm3 | 0x8D -> 10_00_11_01 | ymm3 = ymm3[1,3,0,2]
            // ymm0 is now [0, 1, 2, 3, 4, 5, 6 ... etc.]

            // Move the colours+indices to ymm2, ymm5
            "vshufps {ymm2}, {ymm2}, {ymm1}, 0xEE", // 11_10_11_10
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
            alpha_ptr = inout(reg) alphas_ptr,
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
        u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, remaining);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
pub(crate) unsafe fn shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    let alphas_ptr = output_ptr as *mut u64;
    let colors_ptr = output_ptr.add(len / 2) as *mut u32;
    let indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    shuffle_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(shuffle, "avx2_shuffle")]
    fn test_avx2_unaligned(#[case] transform_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_avx2() {
            return;
        }

        // AVX2 implementation processes 128 bytes per iteration, so max_blocks = 128 * 2 / 16 = 16
        run_standard_transform_unaligned_test(transform_fn, 16, impl_name);
    }
}
