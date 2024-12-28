use std::arch::asm;

#[allow(clippy::unusual_byte_groupings)]
static ALPHA_PERMUTE_MASK: [u32; 8] = [0, 1, 4, 5, 2, 3, 6, 7u32];

#[allow(clippy::unusual_byte_groupings)]
static INDCOL_PERMUTE_MASK: [u32; 8] = [0, 4, 2, 6, 1, 5, 3, 7u32];

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 128
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn avx2_shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut colors_ptr = input_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        let alpha_ptr_end = colors_ptr;

        asm!(
            // Load permutation indices for vpermd, to rearrange blocks.
            "vmovdqu {ymm8}, [{alpha_perm}]",
            "vmovdqu {ymm9}, [{indcol_perm}]",

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
            "cmp {alpha_ptr}, {alpha_ptr_end}",
            "jb 2b",

            alpha_ptr = inout(reg) input_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_end = in(reg) alpha_ptr_end,
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
            alpha_perm = in(reg) &ALPHA_PERMUTE_MASK,
            indcol_perm = in(reg) &INDCOL_PERMUTE_MASK,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for AVX operations
/// - len must be divisible by 128
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86")]
pub unsafe fn avx2_shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut colors_ptr = input_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        let alpha_ptr_end = colors_ptr;

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
            "cmp {alpha_ptr}, {alpha_ptr_end}",
            "jb 2b",

            alpha_ptr = inout(reg) input_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_end = in(reg) alpha_ptr_end,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            ymm7 = out(ymm_reg) _,
            alpha_perm = in(reg) &ALPHA_PERMUTE_MASK,
            indcol_perm = in(reg) &INDCOL_PERMUTE_MASK,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::bc2::transform::tests::generate_bc2_test_data;
    use crate::raw::bc2::transform::u32;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: DetransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::avx2_shuffle(TestCase {
        name: "avx2_shuffle",
        func: avx2_shuffle,
        min_blocks: 8,
        many_blocks: 1024,
    })]
    fn test_detransform(#[case] test_case: TestCase) {
        // Test with minimum blocks
        test_blocks(&test_case, test_case.min_blocks);

        // Test with many blocks
        test_blocks(&test_case, test_case.many_blocks);
    }

    fn test_blocks(test_case: &TestCase, num_blocks: usize) {
        let original = generate_bc2_test_data(num_blocks);
        let mut transformed = vec![0u8; original.len()];
        let mut reconstructed = vec![0u8; original.len()];

        unsafe {
            u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            (test_case.func)(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
        }

        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "{} detransform failed to reconstruct original data for {} blocks",
            test_case.name,
            num_blocks
        );
    }
}
