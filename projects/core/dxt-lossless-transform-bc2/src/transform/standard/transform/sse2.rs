use crate::transform::standard::transform::portable32::u32_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[cfg_attr(target_arch = "x86_64", allow(dead_code))]
pub(crate) unsafe fn shuffle_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let alphas_ptr = output_ptr as *mut u64;
    let colors_ptr = output_ptr.add(len / 2) as *mut u32;
    let indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    shuffle_v2_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn shuffle_v3(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let alphas_ptr = output_ptr as *mut u64;
    let colors_ptr = output_ptr.add(len / 2) as *mut u32;
    let indices_ptr = output_ptr.add(len / 2 + len / 4) as *mut u32;

    shuffle_v3_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - alphas_ptr must be valid for writes of len / 2 bytes
/// - colors_ptr must be valid for writes of len / 4 bytes  
/// - indices_ptr must be valid for writes of len / 4 bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[cfg(target_arch = "x86_64")]
pub(crate) unsafe fn shuffle_v3_with_separate_pointers(
    mut input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);

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

            // Store results to separate pointers
            "movdqu [{alphas_ptr}], {xmm0}",
            "movdqu [{alphas_ptr} + 16], {xmm3}",
            "movdqu [{alphas_ptr} + 32], {xmm8}",
            "movdqu [{alphas_ptr} + 48], {xmm10}",
            "add {alphas_ptr}, 64",

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
            alphas_ptr = inout(reg) alphas_ptr,
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
        u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, remaining);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - alphas_ptr must be valid for writes of len / 2 bytes
/// - colors_ptr must be valid for writes of len / 4 bytes  
/// - indices_ptr must be valid for writes of len / 4 bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
#[cfg_attr(target_arch = "x86_64", allow(dead_code))]
pub(crate) unsafe fn shuffle_v2_with_separate_pointers(
    mut input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    let aligned_len = len - (len % 64);

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
            "shufps {xmm1}, {xmm5}, 0xDD", // All indices in xmm1, 11_01_11_01

            // Store results to separate pointers
            "movdqu [{alphas_ptr}], {xmm0}",
            "movdqu [{alphas_ptr} + 16], {xmm3}",
            "add {alphas_ptr}, 32",

            "movdqu [{colors_ptr}], {xmm2}",
            "add {colors_ptr}, 16",

            "movdqu [{indices_ptr}], {xmm1}",
            "add {indices_ptr}, 16",

            // Loop until done
            "cmp {input_ptr}, {end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alphas_ptr = inout(reg) alphas_ptr,
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
        u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, remaining);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(shuffle_v2, "shuffle_v2")]
    #[cfg_attr(target_arch = "x86_64", case(shuffle_v3, "shuffle_v3"))]
    fn test_sse2_unaligned(#[case] transform_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_sse2() {
            return;
        }
        // SSE2 implementation processes 64 bytes per iteration, so max_blocks = 64 * 2 / 16 = 8
        run_standard_transform_unaligned_test(transform_fn, 8, impl_name);
    }
}
