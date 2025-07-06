use crate::transform::with_split_colour_and_recorr::untransform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx2::{
    recorrelate_ycocg_r_var1_avx2, recorrelate_ycocg_r_var2_avx2, recorrelate_ycocg_r_var3_avx2,
};

/// AVX2 implementation for split-colour and recorrelate untransform for BC2.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_split_colour_recorr_var1(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::Variant2 => {
            untransform_split_colour_recorr_var2(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::Variant3 => {
            untransform_split_colour_recorr_var3(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_split_colour_recorr_var1(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<1>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

unsafe fn untransform_split_colour_recorr_var2(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<2>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

unsafe fn untransform_split_colour_recorr_var3(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<3>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

/// AVX2 implementation for split-colour and recorrelate untransform for BC2.
/// Combines separate arrays of alpha, colour0, colour1 and indices back into standard interleaved BC2 blocks
/// while applying YCoCg recorrelation to the color endpoints.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
unsafe fn untransform_split_colour_recorr<const VARIANT: u8>(
    mut alpha_ptr: *const u64,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Process 16 blocks at a time (256 bytes)
    let blocks16 = block_count / 16 * 16;
    let output_end = output_ptr.add(blocks16 * 16);

    // Permutation masks for reordering data (same as standard implementation)
    let alpha_permute_mask = _mm256_setr_epi32(0, 1, 4, 5, 2, 3, 6, 7);
    let indcol_permute_mask = _mm256_setr_epi32(0, 4, 2, 6, 1, 5, 3, 7);

    // This implementation is based on my original assembly code,
    // from the standard `avx2.rs`. Aside from the extra step for
    // re-combining the colours and increasing the unroll factor.
    while output_ptr < output_end {
        // === ASSEMBLY EQUIVALENT: Load raw alphas (no permutation yet) ===
        // The standard actually loads with vmovdqu first, then applies vpermd
        // Load 16 alpha values (128 bytes) - 4 registers of 4 u64 each
        let alpha0_raw = _mm256_loadu_si256(alpha_ptr as *const __m256i);
        let alpha1_raw = _mm256_loadu_si256(alpha_ptr.add(4) as *const __m256i);
        let alpha2_raw = _mm256_loadu_si256(alpha_ptr.add(8) as *const __m256i);
        let alpha3_raw = _mm256_loadu_si256(alpha_ptr.add(12) as *const __m256i);
        alpha_ptr = alpha_ptr.add(16);

        // === SPLIT COLOR SPECIFIC: Load and combine separate color0/color1 ===
        // This is unique to split-color - standard has colors already combined
        // Load 16 color0 and 16 color1 values, then combine them
        let color0s = _mm256_loadu_si256(color0_ptr as *const __m256i);
        let color1s = _mm256_loadu_si256(color1_ptr as *const __m256i);
        color0_ptr = color0_ptr.add(16);
        color1_ptr = color1_ptr.add(16);

        // Apply YCoCg recorrelation to colors
        let recorr_color0s = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx2(color0s),
            2 => recorrelate_ycocg_r_var2_avx2(color0s),
            3 => recorrelate_ycocg_r_var3_avx2(color0s),
            _ => unreachable_unchecked(),
        };

        let recorr_color1s = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx2(color1s),
            2 => recorrelate_ycocg_r_var2_avx2(color1s),
            3 => recorrelate_ycocg_r_var3_avx2(color1s),
            _ => unreachable_unchecked(),
        };

        // Load 16 indices values (64 bytes)
        let indices0 = _mm256_loadu_si256(indices_ptr as *const __m256i);
        let indices1 = _mm256_loadu_si256(indices_ptr.add(8) as *const __m256i);
        indices_ptr = indices_ptr.add(16);

        // === SPLIT COLOR SPECIFIC: Combine separate color0/color1 into combined format ===
        // This step is unique to split-color - standard has colors already combined
        // Interleave: [color0[0], color1[0], color0[1], color1[1], ...]
        let colors_combined_laned_0 = _mm256_unpacklo_epi16(recorr_color0s, recorr_color1s);
        let colors_combined_laned_1 = _mm256_unpackhi_epi16(recorr_color0s, recorr_color1s);

        // Restore from separate 'lanes' introduced by the unpack to the correct order
        // as if all the data was read from a single buffer.
        let colors_combined_0 =
            _mm256_permute2x128_si256(colors_combined_laned_0, colors_combined_laned_1, 0x20);
        let colors_combined_1 =
            _mm256_permute2x128_si256(colors_combined_laned_0, colors_combined_laned_1, 0x31);

        // === ASSEMBLY EQUIVALENT: Lines 85-86 (alpha permutation AFTER load) ===
        // "vpermd {ymm0}, {ymm8}, [{alpha_ptr}]"
        // "vpermd {ymm1}, {ymm8}, [{alpha_ptr} + 32]"
        // Apply permutations to alpha values like in standard implementation
        let alpha0_permuted = _mm256_permutevar8x32_epi32(alpha0_raw, alpha_permute_mask);
        let alpha1_permuted = _mm256_permutevar8x32_epi32(alpha1_raw, alpha_permute_mask);
        let alpha2_permuted = _mm256_permutevar8x32_epi32(alpha2_raw, alpha_permute_mask);
        let alpha3_permuted = _mm256_permutevar8x32_epi32(alpha3_raw, alpha_permute_mask);

        // === ASSEMBLY EQUIVALENT: Lines 99-100 from standard/avx2.rs ===
        // "vperm2i128 {ymm4}, {ymm2}, {ymm3}, 0x20"
        // "vperm2i128 {ymm5}, {ymm2}, {ymm3}, 0x31"
        // Combine colors and indices - need to process both color sets
        let colors_indices_0 = _mm256_permute2x128_si256(colors_combined_0, indices0, 0x20);
        let colors_indices_1 = _mm256_permute2x128_si256(colors_combined_0, indices0, 0x31);
        let colors_indices_2 = _mm256_permute2x128_si256(colors_combined_1, indices1, 0x20);
        let colors_indices_3 = _mm256_permute2x128_si256(colors_combined_1, indices1, 0x31);

        // === ASSEMBLY EQUIVALENT: Lines 137-138 from standard/avx2.rs ===
        // "vpermd {ymm4}, {ymm9}, {ymm4}"
        // "vpermd {ymm5}, {ymm9}, {ymm5}"
        // Apply permutation to colors+indices
        let colors_indices_0_permuted =
            _mm256_permutevar8x32_epi32(colors_indices_0, indcol_permute_mask);
        let colors_indices_1_permuted =
            _mm256_permutevar8x32_epi32(colors_indices_1, indcol_permute_mask);
        let colors_indices_2_permuted =
            _mm256_permutevar8x32_epi32(colors_indices_2, indcol_permute_mask);
        let colors_indices_3_permuted =
            _mm256_permutevar8x32_epi32(colors_indices_3, indcol_permute_mask);

        // === ASSEMBLY EQUIVALENT: Lines 142-145 from standard/avx2.rs ===
        // "vpunpcklqdq {ymm2}, {ymm0}, {ymm4}"
        // "vpunpckhqdq {ymm3}, {ymm0}, {ymm4}"
        // "vpunpcklqdq {ymm6}, {ymm1}, {ymm5}"
        // "vpunpckhqdq {ymm7}, {ymm1}, {ymm5}"
        // Interleave alpha with colors+indices
        let result0 = _mm256_unpacklo_epi64(alpha0_permuted, colors_indices_0_permuted);
        let result1 = _mm256_unpackhi_epi64(alpha0_permuted, colors_indices_0_permuted);
        let result2 = _mm256_unpacklo_epi64(alpha1_permuted, colors_indices_1_permuted);
        let result3 = _mm256_unpackhi_epi64(alpha1_permuted, colors_indices_1_permuted);
        let result4 = _mm256_unpacklo_epi64(alpha2_permuted, colors_indices_2_permuted);
        let result5 = _mm256_unpackhi_epi64(alpha2_permuted, colors_indices_2_permuted);
        let result6 = _mm256_unpacklo_epi64(alpha3_permuted, colors_indices_3_permuted);
        let result7 = _mm256_unpackhi_epi64(alpha3_permuted, colors_indices_3_permuted);

        // === ASSEMBLY EQUIVALENT: Lines 159-162 from standard/avx2.rs ===
        // "vmovdqu [{output_ptr}], {ymm2}"
        // "vmovdqu [{output_ptr} + 32], {ymm3}"
        // "vmovdqu [{output_ptr} + 64], {ymm6}"
        // "vmovdqu [{output_ptr} + 96], {ymm7}"
        // "add {output_ptr}, 128"
        // Store results
        _mm256_storeu_si256(output_ptr as *mut __m256i, result0);
        _mm256_storeu_si256(output_ptr.add(32) as *mut __m256i, result1);
        _mm256_storeu_si256(output_ptr.add(64) as *mut __m256i, result2);
        _mm256_storeu_si256(output_ptr.add(96) as *mut __m256i, result3);
        _mm256_storeu_si256(output_ptr.add(128) as *mut __m256i, result4);
        _mm256_storeu_si256(output_ptr.add(160) as *mut __m256i, result5);
        _mm256_storeu_si256(output_ptr.add(192) as *mut __m256i, result6);
        _mm256_storeu_si256(output_ptr.add(224) as *mut __m256i, result7);

        output_ptr = output_ptr.add(256);
    }

    // Handle any remaining blocks using generic fallback
    let rem = block_count % 16;
    if rem > 0 {
        let variant = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::untransform_with_split_colour_and_recorr(
            alpha_ptr,
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            rem,
            variant,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn avx2_untransform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_avx2() {
            return;
        }
        // AVX2 processes 256 bytes per iteration (* 2 / 16 == 32)
        run_split_colour_and_recorr_untransform_roundtrip_test(
            untransform_with_split_colour_and_recorr,
            variant,
            32,
            "AVX2",
        );
    }
}
