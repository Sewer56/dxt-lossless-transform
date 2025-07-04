use crate::transform::standard::untransform::portable32::u32_untransform_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub(crate) unsafe fn shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));
    // Process 4 blocks (64 bytes) at a time
    let alpha_ptr = input_ptr;
    let colors_ptr = input_ptr.add(len / 2);
    let indices_ptr = input_ptr.add(len / 2 + len / 4);

    shuffle_with_components(output_ptr, len, alpha_ptr, colors_ptr, indices_ptr);
}

/// # Safety
///
/// - alpha_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/4 bytes
/// - indices_ptr must be valid for reads of len/4 bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub(crate) unsafe fn shuffle_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut alpha_ptr: *const u8,
    mut colors_ptr: *const u8,
    mut indices_ptr: *const u8,
) {
    debug_assert!(
        len.is_multiple_of(16),
        "BC2 shuffle expects `len` to be a multiple of 16 (block size)"
    );

    let aligned_len = len - (len % 64);
    let alpha_ptr_aligned_end = alpha_ptr.add(aligned_len / 2);
    // End pointer for the loop based on aligned length

    if aligned_len > 0 {
        asm!(
            ".p2align 5",
            "2:",

            // Load components
            "movdqu {xmm0}, [{alpha_ptr}]",         // First alpha block
            "movdqu {xmm1}, [{alpha_ptr} + 16]",    // Second alpha block
            "add {alpha_ptr}, 32",
            "movdqu {xmm2}, [{colors_ptr}]",        // Colors
            "add {colors_ptr}, 16",
            "movdqu {xmm3}, [{indices_ptr}]",       // Indices
            "add {indices_ptr}, 16",

            // Current:
            // xmm0: [A0  - A15]
            // xmm1: [A16 - A31]
            // xmm2: [C0  - C15]
            // xmm3: [I0  - I15]

            // Target:
            // 0       -       7 |   08       -       15
            // block0: [A00 - A07] | [C00 - C03] [I00 - I03]
            // block1: [A08 - A15] | [C04 - C07] [I04 - I07]
            // block2: [A16 - A23] | [C08 - C11] [I08 - I11]
            // block3: [A24 - A31] | [C12 - C15] [I12 - I15]

            // Let's get [C00 - C03] [I00 - I03] ... inside XMM6
            // Let's get [C08 - C11] [I08 - I11] ... inside XMM7
            "movaps {xmm6}, {xmm2}",
            "movaps {xmm7}, {xmm2}",
            "punpckldq {xmm6}, {xmm3}", // Interleave 32-bit elements. We did it!
            "punpckhdq {xmm7}, {xmm3}", // Interleave 32-bit elements. We did it!
            // xmm6: [C00 - C03] [I00 - I03] [C04 - C07] [I04 - I07]
            // xmm7: [C08 - C11] [I08 - I11] [C12 - C15] [I12 - I15]

            // We're gonna now export results to remaining xmm registers
            // block0 = xmm0
            // block1 = xmm4
            // block2 = xmm1
            // block3 = xmm5

            // Interleave bottom 64 bits of XMM0 with bottom XMM6 to get block0.
            "movaps {xmm4}, {xmm0}",
            "movaps {xmm5}, {xmm1}",

            "punpcklqdq {xmm0}, {xmm6}", // block0
            "punpcklqdq {xmm1}, {xmm7}", // block2
            "punpckhqdq {xmm4}, {xmm6}", // block1
            "punpckhqdq {xmm5}, {xmm7}", // block3

            // Store results
            "movdqu [{output_ptr}], {xmm0}",
            "movdqu [{output_ptr} + 16], {xmm4}",
            "movdqu [{output_ptr} + 32], {xmm1}",
            "movdqu [{output_ptr} + 48], {xmm5}",
            "add {output_ptr}, 64",

            // Loop until done
            "cmp {alpha_ptr}, {alpha_ptr_aligned_end}",
            "jb 2b",

            alpha_ptr = inout(reg) alpha_ptr,
            output_ptr = inout(reg) output_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            alpha_ptr_aligned_end = in(reg) alpha_ptr_aligned_end,
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,
            xmm6 = out(xmm_reg) _,
            xmm7 = out(xmm_reg) _,
            options(nostack)
        );
    }

    // Process any remaining blocks (less than 4)
    let remaining_len = len - aligned_len;
    if remaining_len > 0 {
        // Pointers `input_ptr`, `colors_ptr`, `indices_ptr`, and `output_ptr` have been updated by the asm block
        u32_untransform_with_separate_pointers(
            alpha_ptr as *const u64,   // Final alpha pointer from asm
            colors_ptr as *const u32,  // Final colors pointer from asm
            indices_ptr as *const u32, // Final indices pointer from asm
            output_ptr,                // Final output pointer from asm
            remaining_len,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case::shuffle(shuffle, "shuffle")]
    fn test_sse2_unaligned(#[case] untransform_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_sse2() {
            return;
        }
        // SSE2 implementation processes 64 bytes per iteration, so max_blocks = 64 * 2 / 16 = 8
        run_standard_untransform_unaligned_test(untransform_fn, 8, impl_name);
    }
}
