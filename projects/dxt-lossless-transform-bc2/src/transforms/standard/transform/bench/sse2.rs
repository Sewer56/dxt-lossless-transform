use crate::transforms::standard::transform::portable32::u32_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn shuffle_v1(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

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
    debug_assert!(len % 16 == 0);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(shuffle_v1, "shuffle_v1")]
    #[case(shuffle_v1_unroll_2, "shuffle_v1_unroll_2")]
    fn test_sse2_unaligned(#[case] transform_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_sse2() {
            return;
        }
        // SSE2 implementation processes 64 bytes per iteration, so max_blocks = 64 * 2 / 16 = 8
        run_standard_transform_unaligned_test(transform_fn, 8, impl_name);
    }
}
