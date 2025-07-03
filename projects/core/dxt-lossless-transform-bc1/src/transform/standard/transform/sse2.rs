use crate::transform::standard::transform::portable32::u32_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn shufps_unroll_4(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    let colors_ptr = output_ptr;
    let indices_ptr = output_ptr.add(len / 2);
    shufps_unroll_4_with_separate_pointers(
        input_ptr,
        colors_ptr as *mut u32,
        indices_ptr as *mut u32,
        len,
    );
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes
/// - indices_ptr must be valid for writes of len/2 bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn shufps_unroll_4_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colors_out: *mut u32,
    mut indices_out: *mut u32,
    len: usize,
) {
    debug_assert!(len.is_multiple_of(8));
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

    let mut aligned_end = input_ptr.add(aligned_len);
    if aligned_len > 0 {
        unsafe {
            asm!(
                // Modern CPUs fetch instructions in 32 byte blocks (or greater), not 16 like the
                // CPUs of older. So we can gain a little here by aligning heavier than Rust would.
                ".p2align 5",
                "2:",

                // Load 4 blocks (64 bytes)
                "movdqu {xmm0}, [{src_ptr}]",
                "movdqu {xmm1}, [{src_ptr} + 16]",
                "movdqu {xmm2}, [{src_ptr} + 32]",
                "movdqu {xmm3}, [{src_ptr} + 48]",
                "add {src_ptr}, 64",   // src += 4 * 16

                "movaps {xmm4}, {xmm0}",
                "movaps {xmm5}, {xmm2}",

                // Shuffle the pairs to rearrange
                "shufps {xmm0}, {xmm1}, 0xDD",    // Indices (0b11011101)
                "shufps {xmm2}, {xmm3}, 0xDD",    // Indices (0b11011101)
                "shufps {xmm4}, {xmm1}, 0x88",    // Colors (0b10001000)
                "shufps {xmm5}, {xmm3}, 0x88",    // Colors (0b10001000)

                // Store colors and indices
                "movdqu [{indices_ptr}], {xmm0}",
                "movdqu [{indices_ptr} + 16], {xmm2}",
                "add {indices_ptr}, 32",   // indices_ptr += 4 * 8
                "movdqu [{colors_ptr}], {xmm4}",
                "movdqu [{colors_ptr} + 16], {xmm5}",
                "add {colors_ptr}, 32",   // colors_ptr += 4 * 8

                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) colors_out,
                indices_ptr = inout(reg) indices_out,
                end = inout(reg) aligned_end,
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

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    u32_with_separate_pointers(input_ptr, colors_out, indices_out, remaining);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(shufps_unroll_4, "SSE2 shuffle unroll-4")]
    fn sse2_transform_roundtrip(#[case] permute_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_sse2() {
            return;
        }

        run_standard_transform_roundtrip_test(permute_fn, 16, impl_name);
    }
}
