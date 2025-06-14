use crate::transforms::standard::untransform::portable32::u32_detransform_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn unpck_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 64-byte blocks as possible
    let indices_ptr = input_ptr.add(len / 2);
    let colors_ptr = input_ptr;
    unpck_detransform_unroll_2_with_components(output_ptr, len, indices_ptr, colors_ptr);
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/2 bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn unpck_detransform_unroll_2_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut indices_in: *const u8,
    mut colors_in: *const u8,
) {
    debug_assert!(len % 8 == 0);
    let aligned_len = len - (len % 64);
    let colors_aligned_end = colors_in.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load all colors and indices (32 bytes each)
                "movdqu {xmm0}, [{colors_ptr}]",      // colors 0
                "movdqu {xmm3}, [{colors_ptr} + 16]", // colors 1
                "add {colors_ptr}, 32",
                "movdqu {xmm1}, [{indices_ptr}]",     // indices 0
                "movdqu {xmm4}, [{indices_ptr} + 16]", // indices 1
                "add {indices_ptr}, 32",

                // Save copies for high parts
                "movaps {xmm2}, {xmm0}", // colors 0 copy
                "movaps {xmm5}, {xmm3}", // colors 1 copy

                // Unpack all blocks
                "punpckldq {xmm0}, {xmm1}", // color0,index0,color1,index1
                "punpckldq {xmm3}, {xmm4}", // color4,index4,color5,index5
                "punpckhdq {xmm2}, {xmm1}", // color2,index2,color3,index3
                "punpckhdq {xmm5}, {xmm4}", // color6,index6,color7,index7

                // Store all results
                "movdqu [{dst_ptr}], {xmm0}",
                "movdqu [{dst_ptr} + 16], {xmm2}",
                "movdqu [{dst_ptr} + 32], {xmm3}",
                "movdqu [{dst_ptr} + 48], {xmm5}",
                "add {dst_ptr}, 64",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) colors_in,
                indices_ptr = inout(reg) indices_in,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
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
    u32_detransform_with_separate_pointers(
        colors_in as *const u32,
        indices_in as *const u32,
        output_ptr,
        remaining,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(unpck_detransform_unroll_2, "unpck_unroll_2")]
    fn test_sse2_unaligned(#[case] detransform_fn: StandardTransformFn, #[case] impl_name: &str) {
        run_standard_untransform_unaligned_test(detransform_fn, 512, impl_name);
    }
}
