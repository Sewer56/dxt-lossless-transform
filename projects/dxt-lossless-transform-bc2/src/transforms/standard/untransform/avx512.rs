use crate::transforms::standard::untransform::portable32::u32_detransform_with_separate_pointers;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[cfg(target_arch = "x86_64")]
use core::arch::*;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
pub unsafe fn avx512_shuffle(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);
    let alpha_ptr = input_ptr;
    let colors_ptr = alpha_ptr.add(len / 2);
    let indices_ptr = colors_ptr.add(len / 4);

    #[cfg(target_arch = "x86_64")]
    avx512_shuffle_with_components(output_ptr, len, alpha_ptr, colors_ptr, indices_ptr);

    #[cfg(target_arch = "x86")]
    avx512_shuffle_with_components_intrinsics(output_ptr, len, alpha_ptr, colors_ptr, indices_ptr);
}

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
pub unsafe fn avx512_shuffle_intrinsics(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);
    let alpha_ptr = input_ptr;
    let colors_ptr = alpha_ptr.add(len / 2);
    let indices_ptr = colors_ptr.add(len / 4);

    avx512_shuffle_with_components_intrinsics(output_ptr, len, alpha_ptr, colors_ptr, indices_ptr);
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - alpha_ptr must be valid for reads of len / 2 bytes
/// - colors_ptr must be valid for reads of len / 4 bytes
/// - indices_ptr must be valid for reads of len / 4 bytes
#[target_feature(enable = "avx512f")]
#[allow(unused_assignments)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::identity_op)]
pub unsafe fn avx512_shuffle_with_components_intrinsics(
    mut output_ptr: *mut u8,
    len: usize,
    mut alpha_ptr: *const u8,
    mut colors_ptr: *const u8,
    mut indices_ptr: *const u8,
) {
    debug_assert!(len % 16 == 0);
    // Process 16 blocks (256 bytes) at a time
    let aligned_len = len - (len % 256);
    let alpha_ptr_aligned_end = alpha_ptr.add(aligned_len / 2);
    // End pointer for the loop based on aligned length

    if aligned_len > 0 {
        // Mask for mixing output_0 (lower half of alpha & `color+index` splits)
        let perm_block_low = _mm512_setr_epi64(
            0,  // alpha 8 bytes
            8,  // colors + indices 8 bytes
            1,  // alpha 8 bytes
            9,  // colors + indices 8 bytes
            2,  // alpha 8 bytes
            10, // colors + indices 8 bytes
            3,  // alpha 8 bytes
            11, // colors + indices 8 bytes
        );
        // Mask for mixing output_1 (upper half of alpha & `color+index` splits)
        let perm_block_high = _mm512_setr_epi64(
            4,  // alpha 8 bytes
            12, // colors + indices 8 bytes
            5,  // alpha 8 bytes
            13, // colors + indices 8 bytes
            6,  // alpha 8 bytes
            14, // colors + indices 8 bytes
            7,  // alpha 8 bytes
            15, // colors + indices 8 bytes
        );
        // Mask for mixing colors and indices (lower half)
        // rust specifies the args for this call in reverse order, e15 == e0. this is a stdlib blunder
        let perm_color_index_low = _mm512_setr_epi32(
            00 + 00, // colors 4 bytes,
            00 + 16, // indices 4 bytes,
            01 + 00, // colors 4 bytes,
            01 + 16, // indices 4 bytes,
            02 + 00, // colors 4 bytes,
            02 + 16, // indices 4 bytes,
            03 + 00, // colors 4 bytes,
            03 + 16, // indices 4 bytes,
            04 + 00, // colors 4 bytes,
            04 + 16, // indices 4 bytes,
            05 + 00, // colors 4 bytes,
            05 + 16, // indices 4 bytes,
            06 + 00, // colors 4 bytes,
            06 + 16, // indices 4 bytes,
            07 + 00, // colors 4 bytes,
            07 + 16, // indices 4 bytes,
        );
        // Mask for mixing colors and indices (upper half)
        let perm_color_index_high = _mm512_setr_epi32(
            08 + 00, // colors 4 bytes,
            08 + 16, // indices 4 bytes,
            09 + 00, // colors 4 bytes,
            09 + 16, // indices 4 bytes,
            10 + 00, // colors 4 bytes,
            10 + 16, // indices 4 bytes,
            11 + 00, // colors 4 bytes,
            11 + 16, // indices 4 bytes,
            12 + 00, // colors 4 bytes,
            12 + 16, // indices 4 bytes,
            13 + 00, // colors 4 bytes,
            13 + 16, // indices 4 bytes,
            14 + 00, // colors 4 bytes,
            14 + 16, // indices 4 bytes,
            15 + 00, // colors 4 bytes,
            15 + 16, // indices 4 bytes,
        );

        while alpha_ptr < alpha_ptr_aligned_end {
            // Load 256 bytes (16 blocks)
            // Read in the individual components.
            let alpha_0 = _mm512_loadu_si512(alpha_ptr as *const __m512i);
            let alpha_1 = _mm512_loadu_si512(alpha_ptr.add(64) as *const __m512i);
            let colors = _mm512_loadu_si512(colors_ptr as *const __m512i);
            let indices = _mm512_loadu_si512(indices_ptr as *const __m512i);
            alpha_ptr = alpha_ptr.add(128);
            colors_ptr = colors_ptr.add(64);
            indices_ptr = indices_ptr.add(64);

            // re-mix lower & upper colour+index halves
            let colors_indices_0 = _mm512_permutex2var_epi32(colors, perm_color_index_low, indices);
            let colors_indices_1 =
                _mm512_permutex2var_epi32(colors, perm_color_index_high, indices);

            // re-mix alphas and colour+index halves
            let output_0 = _mm512_permutex2var_epi64(alpha_0, perm_block_low, colors_indices_0);
            let output_1 = _mm512_permutex2var_epi64(alpha_0, perm_block_high, colors_indices_0);
            let output_2 = _mm512_permutex2var_epi64(alpha_1, perm_block_low, colors_indices_1);
            let output_3 = _mm512_permutex2var_epi64(alpha_1, perm_block_high, colors_indices_1);

            // Write results
            _mm512_storeu_si512(output_ptr as *mut __m512i, output_0);
            _mm512_storeu_si512(output_ptr.add(64) as *mut __m512i, output_1);
            _mm512_storeu_si512(output_ptr.add(128) as *mut __m512i, output_2);
            _mm512_storeu_si512(output_ptr.add(192) as *mut __m512i, output_3);

            // Advance pointer after writing output
            output_ptr = output_ptr.add(256);
        }
    }

    // Process any remaining blocks (less than 8)
    let remaining_len = len - aligned_len;
    if remaining_len > 0 {
        // Pointers `alpha_ptr`, `colors_ptr`, `indices_ptr`, and `output_ptr` have been updated by the asm block
        u32_detransform_with_separate_pointers(
            alpha_ptr as *const u64, // Final alpha pointer from asm (or initial if aligned_len == 0)
            colors_ptr as *const u32, // Final colors pointer from asm (or initial)
            indices_ptr as *const u32, // Final indices pointer from asm (or initial)
            output_ptr,              // Final output pointer from asm (or initial)
            remaining_len,
        );
    }
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - alpha_ptr must be valid for reads of len / 2 bytes
/// - colors_ptr must be valid for reads of len / 4 bytes
/// - indices_ptr must be valid for reads of len / 4 bytes
#[allow(unused_assignments)]
#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::identity_op)]
#[cfg(target_arch = "x86_64")]
pub unsafe fn avx512_shuffle_with_components(
    mut output_ptr: *mut u8,
    len: usize,
    mut alpha_ptr: *const u8,
    mut colors_ptr: *const u8,
    mut indices_ptr: *const u8,
) {
    debug_assert!(len % 16 == 0);
    // Process 16 blocks (256 bytes) at a time
    let aligned_len = len - (len % 256);
    let alpha_ptr_aligned_end = alpha_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        // Mask for mixing output_0 (lower half of alpha & `color+index` splits)
        let perm_block_low = _mm512_setr_epi64(
            0,  // alpha 8 bytes
            8,  // colors + indices 8 bytes
            1,  // alpha 8 bytes
            9,  // colors + indices 8 bytes
            2,  // alpha 8 bytes
            10, // colors + indices 8 bytes
            3,  // alpha 8 bytes
            11, // colors + indices 8 bytes
        );
        // Mask for mixing output_1 (upper half of alpha & `color+index` splits)
        let perm_block_high = _mm512_setr_epi64(
            4,  // alpha 8 bytes
            12, // colors + indices 8 bytes
            5,  // alpha 8 bytes
            13, // colors + indices 8 bytes
            6,  // alpha 8 bytes
            14, // colors + indices 8 bytes
            7,  // alpha 8 bytes
            15, // colors + indices 8 bytes
        );
        // Mask for mixing colors and indices (lower half)
        // rust specifies the args for this call in reverse order, e15 == e0. this is a stdlib blunder
        let perm_color_index_low = _mm512_setr_epi32(
            00 + 00, // colors 4 bytes,
            00 + 16, // indices 4 bytes,
            01 + 00, // colors 4 bytes,
            01 + 16, // indices 4 bytes,
            02 + 00, // colors 4 bytes,
            02 + 16, // indices 4 bytes,
            03 + 00, // colors 4 bytes,
            03 + 16, // indices 4 bytes,
            04 + 00, // colors 4 bytes,
            04 + 16, // indices 4 bytes,
            05 + 00, // colors 4 bytes,
            05 + 16, // indices 4 bytes,
            06 + 00, // colors 4 bytes,
            06 + 16, // indices 4 bytes,
            07 + 00, // colors 4 bytes,
            07 + 16, // indices 4 bytes,
        );
        // Mask for mixing colors and indices (upper half)
        let perm_color_index_high = _mm512_setr_epi32(
            08 + 00, // colors 4 bytes,
            08 + 16, // indices 4 bytes,
            09 + 00, // colors 4 bytes,
            09 + 16, // indices 4 bytes,
            10 + 00, // colors 4 bytes,
            10 + 16, // indices 4 bytes,
            11 + 00, // colors 4 bytes,
            11 + 16, // indices 4 bytes,
            12 + 00, // colors 4 bytes,
            12 + 16, // indices 4 bytes,
            13 + 00, // colors 4 bytes,
            13 + 16, // indices 4 bytes,
            14 + 00, // colors 4 bytes,
            14 + 16, // indices 4 bytes,
            15 + 00, // colors 4 bytes,
            15 + 16, // indices 4 bytes,
        );

        unsafe {
            asm!(
                // Align the loop's instruction address to 16 bytes
                ".p2align 4",
                "2:",  // Loop label

                // Load alpha, colors, and indices
                "vmovdqu64 {zmm6}, [{colors_ptr}]",        // Load colors
                "vmovdqu64 {zmm7}, [{indices_ptr}]",       // Load indices
                "vmovdqu64 {zmm4}, [{alpha_ptr}]",         // Load first alpha block
                "vmovdqu64 {zmm5}, [{alpha_ptr} + 64]",    // Load second alpha block
                "add {alpha_ptr}, 128",                   // Increment alpha pointer
                "add {colors_ptr}, 64",                    // Increment colors pointer
                "add {indices_ptr}, 64",                   // Increment indices pointer

                // Re-mix colors and indices
                "vmovdqa64 {zmm8}, {zmm6}",                // Copy colors for permutation
                "vpermt2d {zmm8}, {perm_color_index_low}, {zmm7}",    // Permute with low pattern
                "vpermt2d {zmm6}, {perm_color_index_high}, {zmm7}",   // Permute with high pattern

                // Re-mix alphas and color+index halves
                "vmovdqa64 {zmm7}, {zmm4}",                // Copy alpha for permutation
                "vpermt2q {zmm7}, {perm_block_low}, {zmm8}", // Permute with low pattern (output_0)
                "vpermt2q {zmm4}, {perm_block_high}, {zmm8}", // Permute with high pattern (output_1)

                "vmovdqa64 {zmm8}, {zmm5}",                // Copy alpha for second permutation
                "vpermt2q {zmm8}, {perm_block_low}, {zmm6}", // Permute with low pattern (output_2)
                "vpermt2q {zmm5}, {perm_block_high}, {zmm6}", // Permute with high pattern (output_3)

                // Store results
                "vmovdqu64 [{dst_ptr}], {zmm7}",           // Store first block (output_0)
                "vmovdqu64 [{dst_ptr} + 64], {zmm4}",      // Store second block (output_1)
                "vmovdqu64 [{dst_ptr} + 128], {zmm8}",     // Store third block (output_2)
                "vmovdqu64 [{dst_ptr} + 192], {zmm5}",     // Store fourth block (output_3)

                // Update pointer and loop
                "add {dst_ptr}, 256",                      // Increment destination pointer
                "cmp {alpha_ptr}, {end}",                  // Compare with end pointer
                "jb 2b",                                   // Jump back if not at end

                alpha_ptr = inout(reg) alpha_ptr,
                colors_ptr = inout(reg) colors_ptr,
                indices_ptr = inout(reg) indices_ptr,
                dst_ptr = inout(reg) output_ptr,
                end = in(reg) alpha_ptr_aligned_end,
                perm_color_index_low = in(zmm_reg) perm_color_index_low,
                perm_color_index_high = in(zmm_reg) perm_color_index_high,
                perm_block_low = in(zmm_reg) perm_block_low,
                perm_block_high = in(zmm_reg) perm_block_high,
                zmm4 = out(zmm_reg) _,
                zmm5 = out(zmm_reg) _,
                zmm6 = out(zmm_reg) _,
                zmm7 = out(zmm_reg) _,
                zmm8 = out(zmm_reg) _,
                options(nostack)
            );
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        avx512_shuffle_with_components_intrinsics(
            output_ptr,
            remaining,
            alpha_ptr,
            colors_ptr,
            indices_ptr,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case::avx512_shuffle(avx512_shuffle, "avx512_shuffle")]
    #[case::avx512_shuffle_intrinsics(avx512_shuffle_intrinsics, "avx512_shuffle_intrinsics")]
    fn test_avx512_unaligned(#[case] detransform_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_avx512f() {
            return;
        }

        // AVX512 implementation processes 256 bytes per iteration, so max_blocks = 256 * 2 / 16 = 32
        run_standard_untransform_unaligned_test(detransform_fn, 32, impl_name);
    }
}
