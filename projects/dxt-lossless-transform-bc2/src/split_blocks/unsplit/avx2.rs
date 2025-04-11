use crate::split_blocks::unsplit::portable32::u32_detransform_with_separate_pointers;
use std::arch::asm;

#[allow(clippy::unusual_byte_groupings)]
static ALPHA_PERMUTE_MASK: [u32; 8] = [0, 1, 4, 5, 2, 3, 6, 7u32];

#[allow(clippy::unusual_byte_groupings)]
static INDCOL_PERMUTE_MASK: [u32; 8] = [0, 4, 2, 6, 1, 5, 3, 7u32];

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn avx2_shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process 8 blocks (128 bytes) at a time
    let aligned_len = len - (len % 128);

    let mut colors_ptr = input_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);
    let alpha_ptr_aligned_end = input_ptr.add(aligned_len / 2); // End pointer for the loop based on aligned length

    if aligned_len > 0 {
        asm!(
            // Load permutation indices for vpermd, to rearrange blocks.
            "vmovdqu {ymm8}, [rip + {alpha_perm}]",
            "vmovdqu {ymm9}, [rip + {indcol_perm}]",

            ".p2align 5",
            "2:",

            // Load ymm0 and ymm1 in format ready for interleaving with colours/indices.
            // ymm0 {
            //     [0, 1, 2, 3], [BLOCK0 ALPHA]
            //     [4, 5, 6, 7],
            //     [8, 9, 10, 11], [BLOCK1 ALPHA]
            //     [12, 13, 14, 15],
            //
            //     [16, 17, 18, 19], [BLOCK2 ALPHA]
            //     [20, 21, 22, 23],
            //     [24, 25, 26, 27], [BLOCK3 ALPHA]
            //     [28, 29, 30, 31]
            // }

            // =>

            // ymm0 {
            //     [0, 1, 2, 3], [BLOCK0 ALPHA] 0
            //     [4, 5, 6, 7], [BLOCK0 ALPHA] 1
            //     [16, 17, 18, 19], [BLOCK2 ALPHA] 4
            //     [20, 21, 22, 23], [BLOCK2 ALPHA] 5
            //
            //     [8, 9, 10, 11], [BLOCK1 ALPHA] 2
            //     [12, 13, 14, 15], [BLOCK1 ALPHA] 3
            //     [24, 25, 26, 27], [BLOCK3 ALPHA] 6
            //     [28, 29, 30, 31], [BLOCK3 ALPHA] 7
            // }

            // Same for YMM1
            "vpermd {ymm0}, {ymm8}, [{alpha_ptr}]",  // Select low half from ymm0, low half from ymm1
            "vpermd {ymm1}, {ymm8}, [{alpha_ptr} + 32]",  // Select high half from ymm0, high half from ymm1
            "add {alpha_ptr}, 64",

            "vmovdqu {ymm2}, [{colors_ptr}]",        // Colors
            "add {colors_ptr}, 32",
            "vmovdqu {ymm3}, [{indices_ptr}]",       // Indices
            "add {indices_ptr}, 32",

            // Current:
            // ymm0: [A0  - A31]
            // ymm1: [A32 - A63]
            // ymm2: [C0  - C31]
            // ymm3: [I0  - I31]
            "vperm2i128 {ymm4}, {ymm2}, {ymm3}, 0x20",  // Select low half from ymm0, low half from ymm1
            "vperm2i128 {ymm5}, {ymm2}, {ymm3}, 0x31",  // Select high half from ymm0, high half from ymm1

            // Desired permute (due to interleave with vpunpcklqdq / vpunpckhqdq):
            // reg:
            //     [Block0]
            //     [Block2]
            //     [Block1]
            //     [Block3]
            // We will achieve this permute on both alphas and colours/indices

            // ymm4 {
            //     [-128, -127, -126, -125], [BLOCK0 COL]
            //     [-124, -123, -122, -121], [BLOCK1 COL]
            //     [-120, -119, -118, -117], [BLOCK2 COL]
            //     [-116, -115, -114, -113], [BLOCK3 COL]
            //
            //     [-64, -63, -62, -61], [BLOCK0 IND]
            //     [-60, -59, -58, -57], [BLOCK1 IND]
            //     [-56, -55, -54, -53], [BLOCK2 IND]
            //     [-52, -51, -50, -49], [BLOCK3 IND]
            // }

            // =>

            // ymm4 target {
            //     [-128, -127, -126, -125], [BLOCK0 COL] 0
            //     [-64, -63, -62, -61], [BLOCK0 IND] 4
            //     [-120, -119, -118, -117], [BLOCK2 COL] 2
            //     [-56, -55, -54, -53], [BLOCK2 IND] 6
            //
            //     [-124, -123, -122, -121], [BLOCK1 COL] 1
            //     [-60, -59, -58, -57], [BLOCK1 IND] 5
            //     [-116, -115, -114, -113], [BLOCK3 COL] 3
            //     [-52, -51, -50, -49], [BLOCK3 IND] 7
            // }

            // Same for YMM5
            "vpermd {ymm4}, {ymm9}, {ymm4}",  // Select low half from ymm0, low half from ymm1
            "vpermd {ymm5}, {ymm9}, {ymm5}",  // Select high half from ymm0, high half from ymm1

            // We're gonna now interleave the registers now that they're aligned.
            // By matching BLOCKX patterns.
            "vpunpcklqdq {ymm2}, {ymm0}, {ymm4}", // block0+1
            "vpunpckhqdq {ymm3}, {ymm0}, {ymm4}", // block2+3
            "vpunpcklqdq {ymm6}, {ymm1}, {ymm5}", // block4+5
            "vpunpckhqdq {ymm7}, {ymm1}, {ymm5}", // block6+7

            // == Interleaved ==
            // ymm0 [0, 1, 2, 3], [BLOCK0]
            //      [4, 5, 6, 7],
            //      [-128, -127, -126, -125], [BLOCK0]
            //      [-64, -63, -62, -61],
            //
            //      [8, 9, 10, 11], [BLOCK1]
            //      [12, 13, 14, 15],
            //      [-124, -123, -122, -121], [BLOCK1]
            //      [-60, -59, -58, -57],

            // Store results
            "vmovdqu [{output_ptr}], {ymm2}",
            "vmovdqu [{output_ptr} + 32], {ymm3}",
            "vmovdqu [{output_ptr} + 64], {ymm6}",
            "vmovdqu [{output_ptr} + 96], {ymm7}",
            "add {output_ptr}, 128",

            // Loop until done
            "cmp {alpha_ptr}, {alpha_ptr_aligned_end}",
            "jb 2b",

            alpha_ptr = inout(reg) input_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_aligned_end = in(reg) alpha_ptr_aligned_end,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            ymm7 = out(ymm_reg) _,
            // x64-only, alpha and index/colour permutes.
            ymm8 = out(ymm_reg) _,
            ymm9 = out(ymm_reg) _,
            alpha_perm = sym ALPHA_PERMUTE_MASK,
            indcol_perm = sym INDCOL_PERMUTE_MASK,
            options(nostack)
        );
    }

    // Process any remaining blocks (less than 8)
    let remaining_len = len - aligned_len;
    if remaining_len > 0 {
        // Pointers `input_ptr`, `colors_ptr`, `indices_ptr`, and `output_ptr` have been updated by the asm block
        u32_detransform_with_separate_pointers(
            input_ptr as *const u64, // Final alpha pointer from asm (or initial if aligned_len == 0)
            colors_ptr as *const u32, // Final colors pointer from asm (or initial)
            indices_ptr as *const u32, // Final indices pointer from asm (or initial)
            output_ptr,              // Final output pointer from asm (or initial)
            remaining_len,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86")]
pub unsafe fn avx2_shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process 8 blocks (128 bytes) at a time
    let aligned_len = len - (len % 128);

    let mut colors_ptr = input_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);
    let alpha_ptr_aligned_end = input_ptr.add(aligned_len / 2); // End pointer for the loop based on aligned length

    if aligned_len > 0 {
        asm!(
            // Load permutation indices for vpermd, to rearrange blocks.
            "vmovdqu {ymm6}, [{alpha_perm}]",
            "vmovdqu {ymm7}, [{indcol_perm}]",

            ".p2align 5",
            "2:",

            // Load ymm0 and ymm1 in format ready for interleaving with colours/indices.
            // ymm0 {
            //     [0, 1, 2, 3], [BLOCK0 ALPHA]
            //     [4, 5, 6, 7],
            //     [8, 9, 10, 11], [BLOCK1 ALPHA]
            //     [12, 13, 14, 15],
            //
            //     [16, 17, 18, 19], [BLOCK2 ALPHA]
            //     [20, 21, 22, 23],
            //     [24, 25, 26, 27], [BLOCK3 ALPHA]
            //     [28, 29, 30, 31]
            // }

            // =>

            // ymm0 {
            //     [0, 1, 2, 3], [BLOCK0 ALPHA] 0
            //     [4, 5, 6, 7], [BLOCK0 ALPHA] 1
            //     [16, 17, 18, 19], [BLOCK2 ALPHA] 4
            //     [20, 21, 22, 23], [BLOCK2 ALPHA] 5
            //
            //     [8, 9, 10, 11], [BLOCK1 ALPHA] 2
            //     [12, 13, 14, 15], [BLOCK1 ALPHA] 3
            //     [24, 25, 26, 27], [BLOCK3 ALPHA] 6
            //     [28, 29, 30, 31], [BLOCK3 ALPHA] 7
            // }

            // Same for YMM1
            "vpermd {ymm0}, {ymm6}, [{alpha_ptr}]",  // Select low half from ymm0, low half from ymm1
            "vpermd {ymm1}, {ymm6}, [{alpha_ptr} + 32]",  // Select high half from ymm0, high half from ymm1
            "add {alpha_ptr}, 64",

            "vmovdqu {ymm2}, [{colors_ptr}]",        // Colors
            "add {colors_ptr}, 32",
            "vmovdqu {ymm3}, [{indices_ptr}]",       // Indices
            "add {indices_ptr}, 32",

            // Current:
            // ymm0: [A0  - A31]
            // ymm1: [A32 - A63]
            // ymm2: [C0  - C31]
            // ymm3: [I0  - I31]
            "vperm2i128 {ymm4}, {ymm2}, {ymm3}, 0x20",  // Select low half from ymm0, low half from ymm1
            "vperm2i128 {ymm5}, {ymm2}, {ymm3}, 0x31",  // Select high half from ymm0, high half from ymm1

            // Desired permute (due to interleave with vpunpcklqdq / vpunpckhqdq):
            // reg:
            //     [Block0]
            //     [Block2]
            //     [Block1]
            //     [Block3]
            // We will achieve this permute on both alphas and colours/indices

            // ymm4 {
            //     [-128, -127, -126, -125], [BLOCK0 COL]
            //     [-124, -123, -122, -121], [BLOCK1 COL]
            //     [-120, -119, -118, -117], [BLOCK2 COL]
            //     [-116, -115, -114, -113], [BLOCK3 COL]
            //
            //     [-64, -63, -62, -61], [BLOCK0 IND]
            //     [-60, -59, -58, -57], [BLOCK1 IND]
            //     [-56, -55, -54, -53], [BLOCK2 IND]
            //     [-52, -51, -50, -49], [BLOCK3 IND]
            // }

            // =>

            // ymm4 target {
            //     [-128, -127, -126, -125], [BLOCK0 COL] 0
            //     [-64, -63, -62, -61], [BLOCK0 IND] 4
            //     [-120, -119, -118, -117], [BLOCK2 COL] 2
            //     [-56, -55, -54, -53], [BLOCK2 IND] 6
            //
            //     [-124, -123, -122, -121], [BLOCK1 COL] 1
            //     [-60, -59, -58, -57], [BLOCK1 IND] 5
            //     [-116, -115, -114, -113], [BLOCK3 COL] 3
            //     [-52, -51, -50, -49], [BLOCK3 IND] 7
            // }

            // Same for YMM5
            "vpermd {ymm4}, {ymm7}, {ymm4}",  // Select low half from ymm0, low half from ymm1
            "vpermd {ymm5}, {ymm7}, {ymm5}",  // Select high half from ymm0, high half from ymm1

            // We're gonna now interleave the registers now that they're aligned.
            // By matching BLOCKX patterns.
            // In x86, we need to split this up into 2, to save ymm registers, as
            // we don't have enough unlike x86-64
            "vpunpcklqdq {ymm2}, {ymm0}, {ymm4}", // block0+1
            "vpunpckhqdq {ymm3}, {ymm0}, {ymm4}", // block2+3
            "vmovdqu [{output_ptr}], {ymm2}",
            "vmovdqu [{output_ptr} + 32], {ymm3}",

            "vpunpcklqdq {ymm2}, {ymm1}, {ymm5}", // block4+5
            "vpunpckhqdq {ymm3}, {ymm1}, {ymm5}", // block6+7
            "vmovdqu [{output_ptr} + 64], {ymm2}",
            "vmovdqu [{output_ptr} + 96], {ymm3}",
            "add {output_ptr}, 128",

            // Loop until done
            "cmp {alpha_ptr}, {alpha_ptr_aligned_end}",
            "jb 2b",

            alpha_ptr = inout(reg) input_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_aligned_end = in(reg) alpha_ptr_aligned_end,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            ymm7 = out(ymm_reg) _,
            alpha_perm = sym ALPHA_PERMUTE_MASK,
            indcol_perm = sym INDCOL_PERMUTE_MASK,
            options(nostack)
        );
    }

    // Process any remaining blocks (less than 8)
    let remaining_len = len - aligned_len;
    if remaining_len > 0 {
        // Pointers `input_ptr`, `colors_ptr`, `indices_ptr`, and `output_ptr` have been updated by the asm block
        u32_detransform_with_separate_pointers(
            input_ptr as *const u64, // Final alpha pointer from asm (or initial if aligned_len == 0)
            colors_ptr as *const u32, // Final colors pointer from asm (or initial)
            indices_ptr as *const u32, // Final indices pointer from asm (or initial)
            output_ptr,              // Final output pointer from asm (or initial)
            remaining_len,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::generate_bc2_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::avx2_shuffle(avx2_shuffle, "avx2_shuffle")]
    fn test_avx2_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        // Test with different block counts to ensure they all work correctly
        for block_count in 1..=512 {
            // Generate test data
            let original = generate_bc2_test_data(block_count);
            let mut transformed = allocate_align_64(original.len());
            let mut reconstructed = allocate_align_64(original.len());

            unsafe {
                // Transform the original test data
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Re-transform it back using the implementation under test
                (detransform_fn)(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            // Verify the results match
            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                impl_name,
                block_count,
            );
        }
    }

    #[rstest]
    #[case::avx2_shuffle(avx2_shuffle, "avx2_shuffle")]
    fn test_avx2_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        // Test with different block counts to ensure they all work correctly
        for block_count in 1..=512 {
            // Generate test data
            let original = generate_bc2_test_data(block_count);

            // Create unaligned buffers (allocate an extra byte and offset by 1)
            let mut unaligned_transformed = vec![0u8; original.len() + 1];
            let mut unaligned_reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Transform the original test data
                u32(
                    original.as_ptr(),
                    unaligned_transformed.as_mut_ptr().add(1),
                    original.len(),
                );

                // Re-transform it back using the implementation under test
                (detransform_fn)(
                    unaligned_transformed.as_mut_ptr().add(1),
                    unaligned_reconstructed.as_mut_ptr().add(1),
                    unaligned_transformed.len() - 1,
                );
            }

            // Verify the results match
            assert_implementation_matches_reference(
                original.as_slice(),
                &unaligned_reconstructed[1..],
                impl_name,
                block_count,
            );
        }
    }
}
