use crate::transforms::standard::untransform::portable32::u32_detransform_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform(mut input_ptr: *const u8, mut output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);
    // Process as many 32-byte blocks as possible
    let aligned_len = len - (len % 32);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load colors and indices (16 bytes each)
                "movdqu {xmm0}, [{colors_ptr}]",    // colors
                "add {colors_ptr}, 16",
                "movdqu {xmm1}, [{indices_ptr}]",   // indices
                "add {indices_ptr}, 16",

                // Interleave the 32-bit values
                "movaps {xmm2}, {xmm0}",
                "unpcklps {xmm0}, {xmm1}",    // Low half: color0,index0,color1,index1
                "unpckhps {xmm2}, {xmm1}",    // High half: color2,index2,color3,index3

                // Store the results
                "movdqu [{dst_ptr}], {xmm0}",
                "movdqu [{dst_ptr} + 16], {xmm2}",
                "add {dst_ptr}, 32",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                xmm0 = out(xmm_reg) _,
                xmm1 = out(xmm_reg) _,
                xmm2 = out(xmm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[cfg(target_arch = "x86_64")]
#[allow(unused_assignments)]
#[target_feature(enable = "sse2")]
pub unsafe fn unpck_detransform_unroll_4(
    mut input_ptr: *const u8,
    mut output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);
    let mut indices_ptr = input_ptr.add(len / 2);
    let colors_aligned_end = input_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load all colors (64 bytes)
                "movdqu {xmm0}, [{colors_ptr}]",      // colors 0
                "movdqu {xmm3}, [{colors_ptr} + 16]", // colors 1
                "movdqu {xmm6}, [{colors_ptr} + 32]", // colors 2
                "movdqu {xmm9}, [{colors_ptr} + 48]", // colors 3
                "add {colors_ptr}, 64",

                // Load all indices (64 bytes)
                "movdqu {xmm1}, [{indices_ptr}]",      // indices 0
                "movdqu {xmm4}, [{indices_ptr} + 16]", // indices 1
                "movdqu {xmm7}, [{indices_ptr} + 32]", // indices 2
                "movdqu {xmm10}, [{indices_ptr} + 48]", // indices 3
                "add {indices_ptr}, 64",

                // Save copies for high parts
                "movaps {xmm2}, {xmm0}",  // colors 0 copy
                "movaps {xmm5}, {xmm3}",  // colors 1 copy
                "movaps {xmm8}, {xmm6}",  // colors 2 copy
                "movaps {xmm11}, {xmm9}", // colors 3 copy

                // Unpack all blocks
                "unpcklps {xmm0}, {xmm1}",
                "unpckhps {xmm2}, {xmm1}",
                "unpcklps {xmm3}, {xmm4}",
                "unpckhps {xmm5}, {xmm4}",
                "unpcklps {xmm6}, {xmm7}",
                "unpckhps {xmm8}, {xmm7}",
                "unpcklps {xmm9}, {xmm10}",
                "unpckhps {xmm11}, {xmm10}",

                // Store all results
                "movdqu [{dst_ptr}], {xmm0}",
                "movdqu [{dst_ptr} + 16], {xmm2}",
                "movdqu [{dst_ptr} + 32], {xmm3}",
                "movdqu [{dst_ptr} + 48], {xmm5}",
                "movdqu [{dst_ptr} + 64], {xmm6}",
                "movdqu [{dst_ptr} + 80], {xmm8}",
                "movdqu [{dst_ptr} + 96], {xmm9}",
                "movdqu [{dst_ptr} + 112], {xmm11}",
                "add {dst_ptr}, 128",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) input_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                xmm0 = out(xmm_reg) _,
                xmm1 = out(xmm_reg) _,
                xmm2 = out(xmm_reg) _,
                xmm3 = out(xmm_reg) _,
                xmm4 = out(xmm_reg) _,
                xmm5 = out(xmm_reg) _,
                xmm6 = out(xmm_reg) _,
                xmm7 = out(xmm_reg) _,
                xmm8 = out(xmm_reg) _,
                xmm9 = out(xmm_reg) _,
                xmm10 = out(xmm_reg) _,
                xmm11 = out(xmm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            input_ptr as *const u32,
            indices_ptr as *const u32,
            output_ptr,
            remaining,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(unpck_detransform, "unpck")]
    #[cfg_attr(
        target_arch = "x86_64",
        case(unpck_detransform_unroll_4, "unpck_unroll_4")
    )]
    fn test_sse2_unaligned(#[case] detransform_fn: StandardTransformFn, #[case] impl_name: &str) {
        run_standard_untransform_unaligned_test(detransform_fn, 512, impl_name);
    }
}
