use crate::transforms::standard::transform::portable32::u32_with_separate_pointers;
use core::arch::asm;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn shuffle_permute_unroll_2(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    let colors_ptr = output_ptr as *mut u32;
    let indices_ptr = output_ptr.add(len / 2) as *mut u32;

    // Call with separate pointers for colors and indices
    shuffle_permute_unroll_2_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8 (BC1 block size)
#[allow(unused_assignments)]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn shuffle_permute_unroll_2_with_separate_pointers(
    mut input_ptr: *const u8,
    mut colors_out: *mut u32,
    mut indices_out: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let aligned_len = len - (len % 128);

    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);
        unsafe {
            asm!(
                // Align the loop's instruction address to 32 bytes.
                // This isn't strictly needed, but more modern processors fetch + execute instructions in
                // 32-byte chunks, as opposed to older ones in 16-byte chunks. Therefore, we can safely-ish
                // assume a processor with AVX2 will be one of those.
                ".p2align 5",
                "2:",

                // Load all 128 bytes first to utilize memory pipeline
                "vmovdqu {ymm0}, [{src_ptr}]",
                "vmovdqu {ymm1}, [{src_ptr} + 32]",
                "vmovdqu {ymm4}, [{src_ptr} + 64]",
                "vmovdqu {ymm5}, [{src_ptr} + 96]",
                "add {src_ptr}, 128",  // src += 128

                // Do all shuffles together to utilize shuffle units
                "vshufps {ymm2}, {ymm0}, {ymm1}, 136",  // colors (0b10001000)
                "vpermpd {ymm2}, {ymm2}, 216",  // arrange colors (0b11011000)
                "vshufps {ymm3}, {ymm0}, {ymm1}, 221",  // indices (0b11011101)
                "vpermpd {ymm3}, {ymm3}, 216",  // arrange indices (0b11011000)
                "vshufps {ymm6}, {ymm4}, {ymm5}, 136",  // colors (0b10001000)
                "vpermpd {ymm6}, {ymm6}, 216",  // arrange colors (0b11011000)
                "vshufps {ymm7}, {ymm4}, {ymm5}, 221",  // indices (0b11011101)
                "vpermpd {ymm7}, {ymm7}, 216",  // arrange indices (0b11011000)

                // Store all results together to utilize store pipeline
                "vmovdqu [{colors_ptr}], {ymm2}",      // Store colors
                "vmovdqu [{indices_ptr}], {ymm3}",      // Store indices
                "vmovdqu [{colors_ptr} + 32], {ymm6}",  // Store colors
                "vmovdqu [{indices_ptr} + 32], {ymm7}",  // Store indices
                "add {colors_ptr}, 64",   // colors_ptr += 64
                "add {indices_ptr}, 64",   // indices_ptr += 64

                // Compare against end address and loop if not done
                "cmp {src_ptr}, {end}",
                "jb 2b",

                src_ptr = inout(reg) input_ptr,
                colors_ptr = inout(reg) colors_out,
                indices_ptr = inout(reg) indices_out,
                end = inout(reg) aligned_end,
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
    u32_with_separate_pointers(input_ptr, colors_out, indices_out, remaining);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(shuffle_permute_unroll_2, "shuffle_permute unroll 2")]
    fn avx2_transform_roundtrip(#[case] permute_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_avx2() {
            return;
        }

        run_standard_transform_roundtrip_test(permute_fn, 32, impl_name);
    }
}
