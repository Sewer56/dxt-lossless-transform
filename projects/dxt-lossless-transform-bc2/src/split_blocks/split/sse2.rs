use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 32 (two blocks at a time)
/// - pointers must be properly aligned for SSE2 operations
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v1(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 32 == 0);

    unsafe {
        let mut end = input_ptr.add(len);
        let mut colors_ptr = output_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        asm!(
            ".p2align 5",
            "2:",

            // Load 32 bytes (two blocks)
            "movdqu {xmm0}, [{input_ptr}]",      // First block
            "movdqu {xmm1}, [{input_ptr} + 16]", // Second block
            "add {input_ptr}, 32", // input += 2 * 16

            // Current Register Layout:
            // [A0 - A8] [C0 - C4] [I0 - I4]
            // Make copies for unpacking/blending
            "movaps {xmm2}, {xmm0}",

            // Extract all alphas into one register
            // XMM0: [A0 - A16]
            "punpcklqdq {xmm0}, {xmm1}", // Combine both alphas

            // Shuffle XMM0 (xmm2) and XMM1 to group colors and indices back again.
            // Desired XMM2: [C0 - C8] [I0 - I8]

            // XMM2: {0,1,2,3, 4,5,6,7, -128,-127,-126,-125, -64,-63,-62,-61}
            // XMM1: {8,9,10,11, 12,13,14,15, -124,-123,-122,-121, -60,-59,-58,-57}
            // Desired XMM2: {-128,-127,-126,-125, -124,-123,-122,-121, -64,-63,-62,-61, -60,-59,-58,-57}
            "shufps {xmm2}, {xmm1}, 0xEE", // 0b11_10_11_10
            // Current XMM2: {-128,-127,-126,-125, -64,-63,-62,-61, -124,-123,-122,-121, -60,-59,-58,-57}
            "shufps {xmm2}, {xmm2}, 0xD8", // 0b11_01_10_00

            // Store combined alphas
            "movdqu [{alpha_ptr}], {xmm0}",
            "add {alpha_ptr}, 16",

            // Store colours and indices
            "movq [{colors_ptr}], {xmm2}",
            "add {colors_ptr}, 8",
            "movhpd [{indices_ptr}], {xmm2}",
            "add {indices_ptr}, 8",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (four blocks at a time)
/// - pointers must be properly aligned for SSE2 operations
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v1_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        let mut end = input_ptr.add(len);
        let mut colors_ptr = output_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        asm!(
            ".p2align 5",
            "2:",

            // Load 64 bytes (four blocks)
            "movdqu {xmm0}, [{input_ptr}]",      // First block
            "movdqu {xmm1}, [{input_ptr} + 16]", // Second block
            "movdqu {xmm3}, [{input_ptr} + 32]", // Third block
            "movdqu {xmm4}, [{input_ptr} + 48]", // Fourth block
            "add {input_ptr}, 64",

            // Setup scratch registers.
            "movaps {xmm2}, {xmm0}",
            "punpcklqdq {xmm0}, {xmm1}", // alpha -> xmm0
            "movaps {xmm5}, {xmm3}",
            "punpcklqdq {xmm3}, {xmm4}", // alpha -> xmm3

            // XMM2: {0,1,2,3, 4,5,6,7, -128,-127,-126,-125, -64,-63,-62,-61}
            // XMM1: {8,9,10,11, 12,13,14,15, -124,-123,-122,-121, -60,-59,-58,-57}
            // Desired XMM2: {-128,-127,-126,-125, -124,-123,-122,-121, -64,-63,-62,-61, -60,-59,-58,-57}

            // Move the colours + indices to xmm2
            "shufps {xmm2}, {xmm1}, 0xEE",
            "shufps {xmm2}, {xmm2}, 0xD8",

            // Process second pair of blocks
            "shufps {xmm5}, {xmm4}, 0xEE",
            "shufps {xmm5}, {xmm5}, 0xD8",

            // Store results
            "movdqu [{alpha_ptr}], {xmm0}",
            "movdqu [{alpha_ptr} + 16], {xmm3}",
            "add {alpha_ptr}, 32",

            "movq [{colors_ptr}], {xmm2}",
            "movq [{colors_ptr} + 8], {xmm5}",
            "add {colors_ptr}, 16",

            "movhpd [{indices_ptr}], {xmm2}",
            "movhpd [{indices_ptr} + 8], {xmm5}",
            "add {indices_ptr}, 16",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 64 (four blocks at a time)
/// - pointers must be properly aligned for SSE2 operations
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 64 == 0);

    unsafe {
        let mut end = input_ptr.add(len);
        let mut colors_ptr = output_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        asm!(
            ".p2align 5",
            "2:",

            // Load 64 bytes (four blocks)
            "movdqu {xmm0}, [{input_ptr}]",      // First block
            "movdqu {xmm1}, [{input_ptr} + 16]", // Second block
            "movdqu {xmm3}, [{input_ptr} + 32]", // Third block
            "movdqu {xmm4}, [{input_ptr} + 48]", // Fourth block
            "add {input_ptr}, 64",

            // TODO: This can probably be optimized by 1 instruction more.
            // Setup scratch registers.
            "movaps {xmm2}, {xmm0}",
            "movaps {xmm5}, {xmm3}",
            "punpcklqdq {xmm0}, {xmm1}", // alpha -> xmm0
            "punpcklqdq {xmm3}, {xmm4}", // alpha -> xmm3

            // XMM2: {0,1,2,3, 4,5,6,7, -128,-127,-126,-125, -64,-63,-62,-61}
            // XMM1: {8,9,10,11, 12,13,14,15, -124,-123,-122,-121, -60,-59,-58,-57}

            // Move the colours+indices to xmm2, xmm5
            "shufps {xmm2}, {xmm1}, 0xEE", // shuffle block 0, block 1 to combine colours + indices
            "shufps {xmm5}, {xmm4}, 0xEE", // shuffle block 2, block 3 to combine colours + indices
            // Current XMM2, XMM5: {-128,-127,-126,-125, -64,-63,-62,-61, -124,-123,-122,-121, -60,-59,-58,-57}
            // Combine colors and indices into separate registers
            "movaps {xmm1}, {xmm2}",       // Save xmm2
            "shufps {xmm2}, {xmm5}, 0x88", // All colors in xmm2
            "shufps {xmm1}, {xmm5}, 0xDD", // All indices in xmm5, 11_01_11_01

            // Store results
            "movdqu [{alpha_ptr}], {xmm0}",
            "movdqu [{alpha_ptr} + 16], {xmm3}",
            "add {alpha_ptr}, 32",

            "movdqu [{colors_ptr}], {xmm2}",
            "add {colors_ptr}, 16",

            "movdqu [{indices_ptr}], {xmm1}",
            "add {indices_ptr}, 16",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 128 (eight blocks at a time)
/// - pointers must be properly aligned for SSE2 operations
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn shuffle_v3(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 128 == 0);

    unsafe {
        let mut end = input_ptr.add(len);
        let mut colors_ptr = output_ptr.add(len / 2);
        let mut indices_ptr = colors_ptr.add(len / 4);
        asm!(
            ".p2align 5",
            "2:",

            // Load first 64 bytes (four blocks)
            "movdqu {xmm0}, [{input_ptr}]",      // First block
            "movdqu {xmm1}, [{input_ptr} + 16]", // Second block
            "movdqu {xmm3}, [{input_ptr} + 32]", // Third block
            "movdqu {xmm4}, [{input_ptr} + 48]", // Fourth block
            "movdqu {xmm8}, [{input_ptr} + 64]", // Fifth block
            "movdqu {xmm9}, [{input_ptr} + 80]", // Sixth block
            "movdqu {xmm10}, [{input_ptr} + 96]", // Seventh block
            "movdqu {xmm11}, [{input_ptr} + 112]", // Eighth block

            "add {input_ptr}, 128",

            // Extract alpha from all 128 bytes.
            "movaps {xmm2}, {xmm0}",
            "movaps {xmm5}, {xmm3}",
            "movaps {xmm12}, {xmm8}",
            "movaps {xmm13}, {xmm10}",
            "punpcklqdq {xmm0}, {xmm1}", // alpha -> xmm0
            "punpcklqdq {xmm3}, {xmm4}", // alpha -> xmm3
            "punpcklqdq {xmm8}, {xmm9}", // alpha -> xmm8
            "punpcklqdq {xmm10}, {xmm11}", // alpha -> xmm10

            // Move the colours+indices to xmm2, xmm5, xmm12, xmm13
            "shufps {xmm2}, {xmm1}, 0xEE",
            "shufps {xmm5}, {xmm4}, 0xEE",
            "shufps {xmm12}, {xmm9}, 0xEE",
            "shufps {xmm13}, {xmm11}, 0xEE",

            "movaps {xmm1}, {xmm2}",
            "movaps {xmm9}, {xmm12}",

            // Group all colors and indices into separate registers
            "shufps {xmm2}, {xmm5}, 0x88", // All colors in xmm2
            "shufps {xmm1}, {xmm5}, 0xDD", // All indices in xmm1
            "shufps {xmm12}, {xmm13}, 0x88", // All colors in xmm12
            "shufps {xmm9}, {xmm13}, 0xDD", // All indices in xmm9

            // Store results
            "movdqu [{alpha_ptr}], {xmm0}",
            "movdqu [{alpha_ptr} + 16], {xmm3}",
            "movdqu [{alpha_ptr} + 32], {xmm8}",
            "movdqu [{alpha_ptr} + 48], {xmm10}",
            "add {alpha_ptr}, 64",

            // Store colors
            "movdqu [{colors_ptr}], {xmm2}",
            "movdqu [{colors_ptr} + 16], {xmm12}",
            "add {colors_ptr}, 32",

            // Store indices
            "movdqu [{indices_ptr}], {xmm1}",
            "movdqu [{indices_ptr} + 16], {xmm9}",
            "add {indices_ptr}, 32",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alpha_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            end = inout(reg) end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            xmm8 = out(xmm_reg) _,
            xmm9 = out(xmm_reg) _,
            xmm10 = out(xmm_reg) _,
            xmm11 = out(xmm_reg) _,
            xmm12 = out(xmm_reg) _,
            xmm13 = out(xmm_reg) _,
            options(nostack)
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::split_blocks::split::tests::generate_bc2_test_data;
    use crate::split_blocks::split::tests::transform_with_reference_implementation;
    use crate::split_blocks::split::*;
    use rstest::rstest;
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    struct TestCase {
        name: &'static str,
        func: TransformFn,
        min_blocks: usize,
        many_blocks: usize,
    }

    #[rstest]
    #[case::shuffle_v1(TestCase {
        name: "shuffle_v1",
        func: shuffle_v1,
        min_blocks: 2,
        many_blocks: 1024,
    })]
    #[case::shuffle_v1_unroll_2(TestCase {
        name: "shuffle_v1_unroll_2",
        func: shuffle_v1_unroll_2,
        min_blocks: 4,
        many_blocks: 1024,
    })]
    #[case::shuffle_v2(TestCase {
        name: "shuffle_v2",
        func: shuffle_v2,
        min_blocks: 4,
        many_blocks: 1024,
    })]
    #[cfg_attr(target_arch = "x86_64", case::shuffle_v3(TestCase {
        name: "shuffle_v3",
        func: shuffle_v3,
        min_blocks: 8,
        many_blocks: 1024,
    }))]
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
