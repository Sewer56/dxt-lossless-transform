#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx2::{
    recorrelate_ycocg_r_var1_avx2, recorrelate_ycocg_r_var2_avx2, recorrelate_ycocg_r_var3_avx2,
};

// Permutation masks matching the original assembly implementation
#[allow(clippy::unusual_byte_groupings)]
static ALPHA_PERMUTE_MASK: [u32; 8] = [0, 1, 4, 5, 2, 3, 6, 7u32];

#[allow(clippy::unusual_byte_groupings)]
static INDCOL_PERMUTE_MASK: [u32; 8] = [0, 4, 2, 6, 1, 5, 3, 7u32];

pub(crate) unsafe fn untransform_with_recorrelate(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

#[target_feature(enable = "avx2")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut alphas_in: *const u64,
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Process 8 BC2 blocks at a time using AVX2 SIMD instructions
    let vectorized_blocks = num_blocks & !7; // Round down to multiple of 8
    let alphas_end = alphas_in.add(vectorized_blocks);

    // Load permutation masks
    let alpha_perm_mask = _mm256_loadu_si256(ALPHA_PERMUTE_MASK.as_ptr() as *const __m256i);
    let indcol_perm_mask = _mm256_loadu_si256(INDCOL_PERMUTE_MASK.as_ptr() as *const __m256i);

    // Main SIMD processing loop - handles 8 blocks per iteration
    while alphas_in < alphas_end {
        // Load alpha data (64 bytes = 8 blocks * 8 bytes each)
        let alphas_raw_0 = _mm256_loadu_si256(alphas_in as *const __m256i); // First 32 bytes
        let alphas_raw_1 = _mm256_loadu_si256(alphas_in.add(4) as *const __m256i); // Next 32 bytes
        alphas_in = alphas_in.add(8);

        // Apply alpha permutation - matching the original assembly's vpermd instructions
        let alphas_0 = _mm256_permutevar8x32_epi32(alphas_raw_0, alpha_perm_mask);
        let alphas_1 = _mm256_permutevar8x32_epi32(alphas_raw_1, alpha_perm_mask);

        // Load colors and indices (32 bytes each = 8 blocks * 4 bytes each)
        let colors_raw = _mm256_loadu_si256(colors_in as *const __m256i);
        colors_in = colors_in.add(8);
        let indices_raw = _mm256_loadu_si256(indices_in as *const __m256i);
        indices_in = indices_in.add(8);

        // Apply recorrelation to the colors based on the variant
        let colors_recorrelated = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx2(colors_raw),
            2 => recorrelate_ycocg_r_var2_avx2(colors_raw),
            3 => recorrelate_ycocg_r_var3_avx2(colors_raw),
            _ => unreachable_unchecked(),
        };

        // Combine colors and indices using vperm2i128 - matching the original assembly
        let colors_indices_4 = _mm256_permute2x128_si256(colors_recorrelated, indices_raw, 0x20);
        let colors_indices_5 = _mm256_permute2x128_si256(colors_recorrelated, indices_raw, 0x31);

        // Apply indices/colors permutation - matching the original assembly's vpermd instructions
        let colors_indices_permuted_4 =
            _mm256_permutevar8x32_epi32(colors_indices_4, indcol_perm_mask);
        let colors_indices_permuted_5 =
            _mm256_permutevar8x32_epi32(colors_indices_5, indcol_perm_mask);

        // Interleave alphas with colors/indices using vpunpcklqdq and vpunpckhqdq
        // This matches the original assembly's interleaving pattern
        let result_0 = _mm256_unpacklo_epi64(alphas_0, colors_indices_permuted_4); // blocks 0+1
        let result_1 = _mm256_unpackhi_epi64(alphas_0, colors_indices_permuted_4); // blocks 2+3
        let result_2 = _mm256_unpacklo_epi64(alphas_1, colors_indices_permuted_5); // blocks 4+5
        let result_3 = _mm256_unpackhi_epi64(alphas_1, colors_indices_permuted_5); // blocks 6+7

        // Store all 8 blocks (128 bytes total)
        _mm256_storeu_si256(output_ptr as *mut __m256i, result_0);
        _mm256_storeu_si256(output_ptr.add(32) as *mut __m256i, result_1);
        _mm256_storeu_si256(output_ptr.add(64) as *mut __m256i, result_2);
        _mm256_storeu_si256(output_ptr.add(96) as *mut __m256i, result_3);

        // Advance pointers
        output_ptr = output_ptr.add(128);
    }

    // Process any remaining blocks using generic implementation
    let remaining_blocks = num_blocks - vectorized_blocks;
    if remaining_blocks > 0 {
        super::generic::untransform_with_recorrelate_generic(
            alphas_in,
            colors_in,
            indices_in,
            output_ptr,
            remaining_blocks,
            match VARIANT {
                1 => YCoCgVariant::Variant1,
                2 => YCoCgVariant::Variant2,
                3 => YCoCgVariant::Variant3,
                _ => unreachable_unchecked(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 16)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 16)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 16)]
    fn avx2_untransform_roundtrip(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx2() {
            return;
        }
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "AVX2");
    }
}
