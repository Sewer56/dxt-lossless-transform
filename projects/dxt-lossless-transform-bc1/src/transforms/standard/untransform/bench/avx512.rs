use crate::transforms::standard::untransform::portable32::u32_detransform_with_separate_pointers;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[allow(unused_assignments)]
#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_detransform_unroll_2_intrinsics(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);
    // Process as many 128-byte blocks as possible
    let indices_ptr = input_ptr.add(len / 2);
    let colors_ptr = input_ptr;

    permute_512_detransform_unroll_2_with_components_intrinsics(
        output_ptr,
        len,
        indices_ptr,
        colors_ptr,
    );
}

/// # Safety
///
/// - output_ptr must be valid for writes of len bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - colors_ptr must be valid for reads of len/2 bytes
#[allow(unused_assignments)]
#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
pub unsafe fn permute_512_detransform_unroll_2_with_components_intrinsics(
    mut output_ptr: *mut u8,
    len: usize,
    mut indices_ptr: *const u8,
    mut colors_ptr: *const u8,
) {
    debug_assert!(len % 8 == 0, "len must be divisible by 8");
    let aligned_len = len - (len % 256);
    let colors_aligned_end = colors_ptr.add(aligned_len / 2);

    if aligned_len > 0 {
        // Define permutation constants for [`vpermt2d`]
        // For gathering low dwords (0,16,1,17,etc.)
        const PERM_LOW_BYTES: [i8; 16] = [0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23];
        // For gathering high dwords (8,24,9,25,etc.)
        const PERM_HIGH_BYTES: [i8; 16] =
            [8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31];

        // Load permutation patterns using vpmovsxbd (sign-extend bytes to dwords)
        let perm_low = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_LOW_BYTES.as_ptr() as *const _));
        let perm_high = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_HIGH_BYTES.as_ptr() as *const _));

        // Main processing loop - translates assembly 1:1
        while colors_ptr < colors_aligned_end {
            let colors_0 = _mm512_loadu_si512(colors_ptr as *const __m512i);
            let colors_1 = _mm512_loadu_si512(colors_ptr.add(64) as *const __m512i);
            colors_ptr = colors_ptr.add(128);

            let indices_0 = _mm512_loadu_si512(indices_ptr as *const __m512i);
            let indices_1 = _mm512_loadu_si512(indices_ptr.add(64) as *const __m512i);
            indices_ptr = indices_ptr.add(128);

            // Apply permutations (equivalent to vpermt2d instructions)
            let output_0 = _mm512_permutex2var_epi32(colors_0, perm_low, indices_0);
            let output_1 = _mm512_permutex2var_epi32(colors_0, perm_high, indices_0);
            let output_2 = _mm512_permutex2var_epi32(colors_1, perm_low, indices_1);
            let output_3 = _mm512_permutex2var_epi32(colors_1, perm_high, indices_1);

            // Store results (equivalent to vmovdqu64 instructions)
            _mm512_storeu_si512(output_ptr as *mut __m512i, output_0);
            _mm512_storeu_si512(output_ptr.add(64) as *mut __m512i, output_1);
            _mm512_storeu_si512(output_ptr.add(128) as *mut __m512i, output_2);
            _mm512_storeu_si512(output_ptr.add(192) as *mut __m512i, output_3);

            // Update pointer (equivalent to add instruction)
            output_ptr = output_ptr.add(256);
        }
    }

    // Process any remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        u32_detransform_with_separate_pointers(
            colors_ptr as *const u32,
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
    #[case(
        permute_512_detransform_unroll_2_intrinsics,
        "avx512_permute_unroll_2_intrinsics"
    )]
    fn test_avx512_unaligned(#[case] detransform_fn: StandardTransformFn, #[case] impl_name: &str) {
        if !has_avx512f() {
            return;
        }

        run_standard_untransform_unaligned_test(detransform_fn, 512, impl_name);
    }
}
