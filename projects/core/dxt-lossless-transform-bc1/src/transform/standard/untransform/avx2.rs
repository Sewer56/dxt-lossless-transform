use crate::transform::standard::untransform::portable32::u32_detransform_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn permd_detransform_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let indices_ptr = input_ptr.add(len / 2);
    let colors_ptr = input_ptr;

    permd_detransform_unroll_2_with_components(output_ptr, len, indices_ptr, colors_ptr);
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/2 bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn permd_detransform_unroll_2_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut indices_in: *const u8,
    mut colors_in: *const u8,
) {
    // Explanation in permd_detransform
    debug_assert!(len % 8 == 0, "len must be divisible by 8");
    let aligned_len = len - (len % 128);
    let colors_aligned_end = colors_in.add(aligned_len / 2);

    if aligned_len > 0 {
        unsafe {
            asm!(
                ".p2align 5",
                "2:",
                // Load first set of colors and indices
                "vpermq {ymm0}, [{colors_ptr}], 0xD8",      // First colors block
                "vpermq {ymm1}, [{indices_ptr}], 0xD8",     // First indices block

                // Start loading second set while first set processes
                "vpermq {ymm4}, [{colors_ptr} + 32], 0xD8", // Second colors block
                "vpermq {ymm5}, [{indices_ptr} + 32], 0xD8", // Second indices block
                "add {colors_ptr}, 64",
                "add {indices_ptr}, 64",

                // Process first set
                "vpunpckldq {ymm2}, {ymm0}, {ymm1}",        // Interleave low parts of first set
                "vpunpckhdq {ymm3}, {ymm0}, {ymm1}",        // Interleave high parts of first set

                // Process second set
                "vpunpckldq {ymm6}, {ymm4}, {ymm5}",        // Interleave low parts of second set
                "vpunpckhdq {ymm7}, {ymm4}, {ymm5}",        // Interleave high parts of second set

                // Store results with some spacing to help with memory operations
                "vmovdqu [{dst_ptr}], {ymm2}",             // Store first low part
                "vmovdqu [{dst_ptr} + 32], {ymm3}",        // Store first high part
                "vmovdqu [{dst_ptr} + 64], {ymm6}",        // Store second low part
                "vmovdqu [{dst_ptr} + 96], {ymm7}",        // Store second high part

                // Update pointers for next iteration
                "add {dst_ptr}, 128",

                // Continue if we haven't reached the end
                "cmp {colors_ptr}, {end}",
                "jb 2b",

                colors_ptr = inout(reg) colors_in,
                indices_ptr = inout(reg) indices_in,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) colors_aligned_end,
                ymm0 = out(ymm_reg) _,
                ymm1 = out(ymm_reg) _,
                ymm2 = out(ymm_reg) _,
                ymm3 = out(ymm_reg) _,
                ymm4 = out(ymm_reg) _,
                ymm5 = out(ymm_reg) _,
                ymm6 = out(ymm_reg) _,
                ymm7 = out(ymm_reg) _,
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
    #[case(permd_detransform_unroll_2, "avx_permd_unroll_2")]
    fn test_avx2_unaligned(#[case] detransform_fn: StandardTransformFn, #[case] impl_name: &str) {
        // 128 bytes processed per main loop iteration (* 2 / 8 == 32)
        run_standard_untransform_unaligned_test(detransform_fn, 32, impl_name);
    }
}
