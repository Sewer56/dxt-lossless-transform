use crate::transform::standard::transform::portable32::u32_with_separate_pointers;
use core::arch::asm;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

const PERM_ALPHA_BYTES: [i8; 8] = [0, 2, 4, 6, 8, 10, 12, 14]; // For vpermt2q to gather alpha values

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - alphas_ptr must be valid for writes of len/2 bytes
/// - colors_ptr must be valid for writes of len/4 bytes
/// - indices_ptr must be valid for writes of len/4 bytes
/// - len must be divisible by 16
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
pub unsafe fn permute_512_with_separate_pointers(
    mut input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len.is_multiple_of(16));

    let mut aligned_len = len - (len % 128);

    // Note(sewer): We need to subtract 32 (half register) as to not write beyond the end of the
    // buffer. We write 64 bytes with vmovdqu64, so the final indices_ptr write goes 32 bytes beyond
    // the limit. We need to guard against this. However, because the indices also represent 4 / 16
    // of the blocks, we need to multiply the amount we subtract by 4 to account for split buffers.
    aligned_len = aligned_len.saturating_sub(32 * 4);

    if aligned_len > 0 {
        let mut aligned_end = input_ptr.add(aligned_len);

        const PERM_COLORS_BYTES: [i8; 8] = [2, 6, 10, 14, 18, 22, 26, 30]; // For vpermt2d to gather color values
        const PERM_INDICES_BYTES: [i8; 8] = [3, 7, 11, 15, 19, 23, 27, 31]; // For vpermt2d to gather index values

        // Load permutation patterns
        let perm_alpha =
            _mm512_cvtepi8_epi64(_mm_loadl_epi64(PERM_ALPHA_BYTES.as_ptr() as *const _));
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadl_epi64(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadl_epi64(PERM_INDICES_BYTES.as_ptr() as *const _));

        asm!(
            ".p2align 5",
            "2:",

            // Load 128 bytes (eight blocks)
            "vmovdqu64 {zmm4}, [{input_ptr}]",      // Load first 4 blocks
            "vmovdqu64 {zmm5}, [{input_ptr} + 64]", // Load next 4 blocks
            "vmovdqa64 {zmm3}, {zmm4}",             // Copy first 4 blocks to zmm3
            "vpermt2q {zmm3}, {perm_alpha}, {zmm5}",// Filter out the alphas from zmm3 (zmm4) + zmm5
            "add {input_ptr}, 128",

            // Permute to separate colors and indices
            "vmovdqa64 {zmm6}, {zmm4}",
            "vpermt2d {zmm6}, {perm_colors}, {zmm5}",  // Gather colors from zmm4 + zmm5
            "vpermt2d {zmm4}, {perm_indices}, {zmm5}", // Gather indices from zmm4 + zmm5

            // Store results
            "vmovdqu64 [{alphas_ptr}], {zmm3}",
            "vmovdqu64 [{colors_ptr}], {zmm6}",
            "vmovdqu64 [{indices_ptr}], {zmm4}",

            // Update pointers
            "add {alphas_ptr}, 64",
            "add {colors_ptr}, 32",
            "add {indices_ptr}, 32",

            // Loop until done
            "cmp {input_ptr}, {aligned_end}",
            "jb 2b",

            input_ptr = inout(reg) input_ptr,
            alphas_ptr = inout(reg) alphas_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            aligned_end = inout(reg) aligned_end,
            perm_alpha = in(zmm_reg) perm_alpha,
            perm_colors = in(zmm_reg) perm_colors,
            perm_indices = in(zmm_reg) perm_indices,
            zmm3 = out(zmm_reg) _,
            zmm4 = out(zmm_reg) _,
            zmm5 = out(zmm_reg) _,
            zmm6 = out(zmm_reg) _,
            options(nostack, preserves_flags)
        );
    }

    // Process any remaining elements
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, remaining);
    }
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
pub unsafe fn permute_512(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    let alphas_ptr = output_ptr as *mut u64;
    let colors_ptr = output_ptr.add(len / 2);
    let indices_ptr = colors_ptr.add(len / 4);

    permute_512_with_separate_pointers(
        input_ptr,
        alphas_ptr,
        colors_ptr as *mut u32,
        indices_ptr as *mut u32,
        len,
    );
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
#[allow(dead_code)]
pub unsafe fn permute_512_intrinsics(mut input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    let mut aligned_len = len - (len % 128);
    let mut alpha_ptr = output_ptr;

    // Note(sewer): We need to subtract 32 (half register) as to not write beyond the end of the
    // buffer. We write 64 bytes with vmovdqu64, so the final indices_ptr write goes 32 bytes beyond
    // the limit. We need to guard against this. However, because the indices also represent 4 / 16
    // of the blocks, we need to multiply the amount we subtract by 4 to account for split buffers.
    aligned_len = aligned_len.saturating_sub(32 * 4);

    let mut colors_ptr = alpha_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    if aligned_len > 0 {
        let aligned_end = input_ptr.add(aligned_len);

        // Constant data for permutation masks
        const PERM_COLORS_BYTES: [i8; 8] = [2, 6, 10, 14, 18, 22, 26, 30]; // For vpermt2d to gather color values
        const PERM_INDICES_BYTES: [i8; 8] = [3, 7, 11, 15, 19, 23, 27, 31]; // For vpermt2d to gather index values

        // Load permutation patterns
        let perm_alpha =
            _mm512_cvtepi8_epi64(_mm_loadl_epi64(PERM_ALPHA_BYTES.as_ptr() as *const _));
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadl_epi64(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadl_epi64(PERM_INDICES_BYTES.as_ptr() as *const _));

        while input_ptr < aligned_end {
            // Load 128 bytes (eight blocks)
            let zmm4 = _mm512_loadu_si512(input_ptr as *const __m512i);
            let zmm5 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);

            // Copy first 4 blocks to zmm3
            let mut zmm3 = zmm4;

            // Filter out the alphas using vpermt2q
            zmm3 = _mm512_permutex2var_epi64(zmm3, perm_alpha, zmm5);

            // Update input pointer
            input_ptr = input_ptr.add(128);

            // Permute to separate colors and indices
            let mut zmm6 = zmm4;
            zmm6 = _mm512_permutex2var_epi32(zmm6, perm_colors, zmm5); // colours
            let zmm4_indices = _mm512_permutex2var_epi32(zmm4, perm_indices, zmm5); // indices

            // Store results
            _mm512_storeu_si512(alpha_ptr as *mut __m512i, zmm3); // alphas
            _mm512_storeu_si512(colors_ptr as *mut __m512i, zmm6); // colours
            _mm512_storeu_si512(indices_ptr as *mut __m512i, zmm4_indices); // indices

            // Update pointers
            alpha_ptr = alpha_ptr.add(64);
            colors_ptr = colors_ptr.add(32);
            indices_ptr = indices_ptr.add(32);
        }
    }

    // Process any remaining elements
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_with_separate_pointers(
            input_ptr,
            alpha_ptr as *mut u64,
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
    #[case(permute_512_intrinsics, "avx512_permute_intrinsics")]
    #[case(permute_512, "avx512_permute_asm")]
    fn test_avx512_unaligned(#[case] permute_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_avx512f() {
            return;
        }

        // AVX512 implementation processes 256 bytes per iteration, so max_blocks = 256 * 2 / 16 = 32
        run_standard_transform_unaligned_test(permute_fn, 32, impl_name);
    }
}
