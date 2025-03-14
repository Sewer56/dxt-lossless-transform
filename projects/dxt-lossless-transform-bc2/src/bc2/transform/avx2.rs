use std::arch::asm;

#[allow(clippy::unusual_byte_groupings)]
static PERMUTE_MASK: [u32; 8] = [0, 4, 1, 5, 2, 6, 3, 7];

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (four blocks at a time)
/// - pointers must be properly aligned for AVX2 operations
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut end = input_ptr.add(len);
        let mut colors_ptr = output_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        asm!(
            // Load the permute mask for 32-bit element reordering
            "vmovdqu {ymm6}, [rip + {permute_mask}]",

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
            "vpermd {ymm2}, {ymm6}, {ymm2}", // Permute colors
            "vpermd {ymm1}, {ymm6}, {ymm1}", // Permute indices

            // Store results
            "vmovdqu [{alpha_ptr}], {ymm0}",
            "vmovdqu [{alpha_ptr} + 32], {ymm3}",
            "add {alpha_ptr}, 64",

            "vmovdqu [{colors_ptr}], {ymm2}",
            "add {colors_ptr}, 32",

            "vmovdqu [{indices_ptr}], {ymm1}",
            "add {indices_ptr}, 32",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            permute_mask = sym PERMUTE_MASK,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (four blocks at a time)
/// - pointers must be properly aligned for AVX2 operations
#[target_feature(enable = "avx2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86")]
pub unsafe fn shuffle(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut end = input_ptr.add(len);
        let mut colors_ptr = output_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        asm!(
            // Load the permute mask for 32-bit element reordering
            "vmovdqu {ymm6}, [{permute_mask}]",

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
            "vpermd {ymm2}, {ymm6}, {ymm2}", // Permute colors
            "vpermd {ymm1}, {ymm6}, {ymm1}", // Permute indices

            // Store results
            "vmovdqu [{alpha_ptr}], {ymm0}",
            "vmovdqu [{alpha_ptr} + 32], {ymm3}",
            "add {alpha_ptr}, 64",

            "vmovdqu [{colors_ptr}], {ymm2}",
            "add {colors_ptr}, 32",

            "vmovdqu [{indices_ptr}], {ymm1}",
            "add {indices_ptr}, 32",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            permute_mask = sym PERMUTE_MASK,
            ymm0 = out(ymm_reg) _,
            ymm1 = out(ymm_reg) _,
            ymm2 = out(ymm_reg) _,
            ymm3 = out(ymm_reg) _,
            ymm4 = out(ymm_reg) _,
            ymm5 = out(ymm_reg) _,
            ymm6 = out(ymm_reg) _,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::bc2::transform::tests::generate_bc2_test_data;
    use crate::bc2::transform::tests::transform_with_reference_implementation;
    use crate::bc2::transform::*;
    use rstest::rstest;
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: TransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::avx2_shuffle(TestCase {
        name: "avx2_shuffle",
        func: shuffle,
        min_blocks: 8,
        many_blocks: 1024,
    })]
    fn test_transform(#[case] test_case: TestCase) {
        // Test with minimum blocks
        test_blocks(&test_case, test_case.min_blocks);

        // Test with many blocks
        test_blocks(&test_case, test_case.many_blocks);
    }

    fn test_blocks(test_case: &TestCase, num_blocks: usize) {
        let input = generate_bc2_test_data(num_blocks);
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
