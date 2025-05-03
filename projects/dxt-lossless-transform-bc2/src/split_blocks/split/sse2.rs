use crate::split_blocks::split::portable32::u32_with_separate_pointers;
use std::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v1(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    let aligned_len = len - (len % 32);
    let mut colors_ptr = output_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    if aligned_len > 0 {
        let mut end = input_ptr.add(aligned_len);

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

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v1_unroll_2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    let aligned_len = len - (len % 64);
    let mut colors_ptr = output_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    if aligned_len > 0 {
        let mut end = input_ptr.add(aligned_len);
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

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v2(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    let aligned_len = len - (len % 64);
    let mut colors_ptr = output_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    if aligned_len > 0 {
        let mut end = input_ptr.add(aligned_len);
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

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn shuffle_v3(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);
    let mut colors_ptr = output_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    if aligned_len > 0 {
        let mut end = input_ptr.add(aligned_len);
        asm!(
            ".p2align 5",
            "2:",
            // Load first 128 bytes (eight blocks)
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
    #[case(shuffle_v1, "shuffle_v1")]
    #[case(shuffle_v1_unroll_2, "shuffle_v1_unroll_2")]
    #[case(shuffle_v2, "shuffle_v2")]
    #[cfg_attr(target_arch = "x86_64", case(shuffle_v3, "shuffle_v3"))]
    fn test_sse2_aligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
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
    #[case(shuffle_v1, "shuffle_v1")]
    #[case(shuffle_v1_unroll_2, "shuffle_v1_unroll_2")]
    #[case(shuffle_v2, "shuffle_v2")]
    #[cfg_attr(target_arch = "x86_64", case(shuffle_v3, "shuffle_v3"))]
    fn test_sse2_unaligned(#[case] permute_fn: PermuteFn, #[case] impl_name: &str) {
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
