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
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
#[allow(dead_code)] // reference implementation
pub(crate) unsafe fn permute_512_v2_intrinsics(
    mut input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    let aligned_len = len - (len % 256);
    let mut alpha_ptr = output_ptr;

    let mut colors_ptr = alpha_ptr.add(len / 2);
    let mut indices_ptr = colors_ptr.add(len / 4);

    if aligned_len > 0 {
        let aligned_end = input_ptr.add(aligned_len);

        // Constant data for permutation masks
        const PERM_COLORS_BYTES: [i8; 16] = [
            0, 4, 8, 12, 2, 6, 10, 14, // + 16 below
            16, 20, 24, 28, 18, 22, 26, 30,
        ]; // For vpermt2d to gather color values
        const PERM_INDICES_BYTES: [i8; 16] = [
            1, 5, 9, 13, 3, 7, 11, 15, // +16 below
            17, 21, 25, 29, 19, 23, 27, 31,
        ]; // For vpermt2d to gather index values

        // Load permutation patterns
        let perm_alpha =
            _mm512_cvtepi8_epi64(_mm_loadl_epi64(PERM_ALPHA_BYTES.as_ptr() as *const _));
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_INDICES_BYTES.as_ptr() as *const _));

        while input_ptr < aligned_end {
            // Load 256 bytes (16 blocks)
            let blocks_0 = _mm512_loadu_si512(input_ptr as *const __m512i);
            let blocks_1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
            let blocks_2 = _mm512_loadu_si512(input_ptr.add(128) as *const __m512i);
            let blocks_3 = _mm512_loadu_si512(input_ptr.add(192) as *const __m512i);

            // Update input pointer
            input_ptr = input_ptr.add(256);

            // Filter out the alphas only using vpermt2q
            let alphas_0 = _mm512_permutex2var_epi64(blocks_0, perm_alpha, blocks_1);
            let alphas_1 = _mm512_permutex2var_epi64(blocks_2, perm_alpha, blocks_3);

            // Lift out colours and indices only
            let colours_indices_only_b0 = _mm512_unpackhi_epi64(blocks_0, blocks_1);
            let colours_indices_only_b1 = _mm512_unpackhi_epi64(blocks_2, blocks_3);

            // Permute to separate colors and indices
            let colours_only = _mm512_permutex2var_epi32(
                colours_indices_only_b0,
                perm_colors,
                colours_indices_only_b1,
            ); // colours
            let indices_only = _mm512_permutex2var_epi32(
                colours_indices_only_b0,
                perm_indices,
                colours_indices_only_b1,
            ); // indices

            // Store results
            _mm512_storeu_si512(alpha_ptr as *mut __m512i, alphas_0); // alphas 0
            _mm512_storeu_si512(alpha_ptr.add(64) as *mut __m512i, alphas_1); // alphas 1
            _mm512_storeu_si512(colors_ptr as *mut __m512i, colours_only); // colours
            _mm512_storeu_si512(indices_ptr as *mut __m512i, indices_only); // indices

            // Update pointers
            alpha_ptr = alpha_ptr.add(128);
            colors_ptr = colors_ptr.add(64);
            indices_ptr = indices_ptr.add(64);
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

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
pub(crate) unsafe fn permute_512_v2_with_separate_pointers(
    mut input_ptr: *const u8,
    mut alphas_ptr: *mut u64,
    mut colors_ptr: *mut u32,
    mut indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    let aligned_len = len - (len % 256);

    if aligned_len > 0 {
        let aligned_end = input_ptr.add(aligned_len);

        // Constant data for permutation masks
        const PERM_COLORS_BYTES: [i8; 16] = [
            0, 4, 8, 12, 2, 6, 10, 14, // + 16 below
            16, 20, 24, 28, 18, 22, 26, 30,
        ]; // For vpermt2d to gather color values
        const PERM_INDICES_BYTES: [i8; 16] = [
            1, 5, 9, 13, 3, 7, 11, 15, // +16 below
            17, 21, 25, 29, 19, 23, 27, 31,
        ]; // For vpermt2d to gather index values

        // Load permutation patterns
        let perm_alpha =
            _mm512_cvtepi8_epi64(_mm_loadl_epi64(PERM_ALPHA_BYTES.as_ptr() as *const _));
        let perm_colors =
            _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_COLORS_BYTES.as_ptr() as *const _));
        let perm_indices =
            _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_INDICES_BYTES.as_ptr() as *const _));

        asm!(
            ".p2align 5",
            "3:",

            // Load 256 bytes (16 blocks)
            "vmovdqu64 {zmm3}, [{input_ptr}]",
            "vmovdqu64 {zmm4}, [{input_ptr} + 64]",
            "vmovdqu64 {zmm5}, [{input_ptr} + 128]",
            "vmovdqu64 {zmm6}, [{input_ptr} + 192]",
            "add {input_ptr}, 256",

            // Unpack high quadwords and permute
            "vpunpckhqdq {zmm7}, {zmm3}, {zmm4}",
            "vpermt2q {zmm3}, {perm_alpha}, {zmm4}",
            "vpunpckhqdq {zmm4}, {zmm5}, {zmm6}",
            "vpermt2q {zmm5}, {perm_alpha}, {zmm6}",

            // Permute to separate colors and indices
            "vmovdqa64 {zmm6}, {zmm7}",
            "vpermt2d {zmm6}, {perm_colors}, {zmm4}",
            "vpermt2d {zmm7}, {perm_indices}, {zmm4}",

            // Store results
            "vmovdqu64 [{alphas_ptr}], {zmm3}",
            "vmovdqu64 [{alphas_ptr} + 64], {zmm5}",
            "add {alphas_ptr}, 128",
            "vmovdqu64 [{colors_ptr}], {zmm6}",
            "vmovdqu64 [{indices_ptr}], {zmm7}",

            // Update pointers
            "add {colors_ptr}, 64",
            "add {indices_ptr}, 64",

            // Loop until done
            "cmp {input_ptr}, {aligned_end}",
            "jb 3b",

            input_ptr = inout(reg) input_ptr,
            alphas_ptr = inout(reg) alphas_ptr,
            colors_ptr = inout(reg) colors_ptr,
            indices_ptr = inout(reg) indices_ptr,
            aligned_end = in(reg) aligned_end,
            perm_alpha = in(zmm_reg) perm_alpha,
            perm_colors = in(zmm_reg) perm_colors,
            perm_indices = in(zmm_reg) perm_indices,
            zmm3 = out(zmm_reg) _,
            zmm4 = out(zmm_reg) _,
            zmm5 = out(zmm_reg) _,
            zmm6 = out(zmm_reg) _,
            zmm7 = out(zmm_reg) _,
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
pub(crate) unsafe fn permute_512_v2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    let alphas_ptr = output_ptr as *mut u64;
    let colors_ptr = output_ptr.add(len / 2);
    let indices_ptr = colors_ptr.add(len / 4);

    permute_512_v2_with_separate_pointers(
        input_ptr,
        alphas_ptr,
        colors_ptr as *mut u32,
        indices_ptr as *mut u32,
        len,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(permute_512_v2_intrinsics, "avx512_permute_intrinsics_v2")]
    #[case(permute_512_v2, "avx512_permute_asm_v2")]
    fn test_avx512_unaligned(#[case] permute_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_avx512f() {
            return;
        }

        // AVX512 implementation processes 256 bytes per iteration, so max_blocks = 256 * 2 / 16 = 32
        run_standard_transform_unaligned_test(permute_fn, 32, impl_name);
    }
}
