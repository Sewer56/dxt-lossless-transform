#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx512bw::{
    recorrelate_ycocg_r_var1_avx512bw, recorrelate_ycocg_r_var2_avx512bw,
    recorrelate_ycocg_r_var3_avx512bw,
};

pub(crate) unsafe fn untransform_with_recorrelate(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => {
            // This should be unreachable based on the calling context
            unreachable_unchecked()
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`

unsafe fn untransform_recorr_var1(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(colors_in, indices_in, output_ptr, num_blocks)
}

#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // === Main Vectorized Loop ===
    // Process 32 blocks at a time using AVX512 SIMD instructions (unroll 2)
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !31; // Round down to multiple of 32

    // Set up permutation patterns for interleaving colors and indices (moved outside loop)
    const PERM_LOW_BYTES: [i8; 16] = [0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23];
    const PERM_HIGH_BYTES: [i8; 16] =
        [8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31];

    let perm_low = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_LOW_BYTES.as_ptr() as *const _));
    let perm_high = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_HIGH_BYTES.as_ptr() as *const _));

    let colors_end = colors_in.add(vectorized_blocks);
    let mut current_colors_ptr = colors_in;
    let mut current_indices_ptr = indices_in;
    let mut current_output_ptr = output_ptr;

    // Main SIMD processing loop - handles 32 blocks per iteration (unroll 2)
    while current_colors_ptr < colors_end {
        // Load first set of colors and indices (64 bytes each)
        let colors_0 = _mm512_loadu_si512(current_colors_ptr as *const __m512i);
        let colors_1 = _mm512_loadu_si512(current_colors_ptr.add(16) as *const __m512i);

        let indices_0 = _mm512_loadu_si512(current_indices_ptr as *const __m512i);
        let indices_1 = _mm512_loadu_si512(current_indices_ptr.add(16) as *const __m512i);

        // Apply recorrelation to the colors based on the variant
        let recorrelated_colors_0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(colors_0),
            2 => recorrelate_ycocg_r_var2_avx512bw(colors_0),
            3 => recorrelate_ycocg_r_var3_avx512bw(colors_0),
            _ => unreachable_unchecked(),
        };
        let recorrelated_colors_1 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(colors_1),
            2 => recorrelate_ycocg_r_var2_avx512bw(colors_1),
            3 => recorrelate_ycocg_r_var3_avx512bw(colors_1),
            _ => unreachable_unchecked(),
        };

        // Apply permutations to interleave colors and indices (equivalent to vpermt2d instructions)
        let output_0 = _mm512_permutex2var_epi32(recorrelated_colors_0, perm_low, indices_0);
        let output_1 = _mm512_permutex2var_epi32(recorrelated_colors_0, perm_high, indices_0);
        let output_2 = _mm512_permutex2var_epi32(recorrelated_colors_1, perm_low, indices_1);
        let output_3 = _mm512_permutex2var_epi32(recorrelated_colors_1, perm_high, indices_1);

        // Store results (equivalent to vmovdqu64 instructions)
        _mm512_storeu_si512(current_output_ptr as *mut __m512i, output_0);
        _mm512_storeu_si512(current_output_ptr.add(64) as *mut __m512i, output_1);
        _mm512_storeu_si512(current_output_ptr.add(128) as *mut __m512i, output_2);
        _mm512_storeu_si512(current_output_ptr.add(192) as *mut __m512i, output_3);

        current_colors_ptr = current_colors_ptr.add(32);
        current_indices_ptr = current_indices_ptr.add(32);
        current_output_ptr = current_output_ptr.add(256);
    }

    // === Scalar Fallback for Remaining Blocks ===
    // Handle any remaining blocks that couldn't be processed in the vectorized loop
    // (when num_blocks is not a multiple of 32) using generic implementation
    let remaining_count = num_blocks - vectorized_blocks;
    let variant = match VARIANT {
        1 => YCoCgVariant::Variant1,
        2 => YCoCgVariant::Variant2,
        3 => YCoCgVariant::Variant3,
        _ => unreachable_unchecked(),
    };
    super::generic::untransform_with_recorrelate_generic(
        colors_in.add(vectorized_blocks),
        indices_in.add(vectorized_blocks),
        output_ptr.add(vectorized_blocks * 8),
        remaining_count,
        variant,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: WithRecorrelateUntransformFn,
        #[case] decorr_variant: YCoCgVariant,
    ) {
        if !(has_avx512f() && has_avx512bw()) {
            return;
        }

        run_with_recorrelate_untransform_unaligned_test(
            function,
            decorr_variant,
            "untransform_with_recorrelate (avx512)",
            64, // 256 bytes tested per main loop iteration (* 2 / 8 == 64)
        );
    }
}
