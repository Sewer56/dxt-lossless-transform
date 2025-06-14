use crate::transforms::with_split_colour_and_recorr::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx2::{
    decorrelate_ycocg_r_var1_avx2, decorrelate_ycocg_r_var2_avx2, decorrelate_ycocg_r_var3_avx2,
};

/// AVX2 implementation for split-colour transform with YCoCg-R decorrelation.
#[target_feature(enable = "avx2")]
unsafe fn transform_impl<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    let permute_idx = _mm256_setr_epi32(0, 4, 1, 5, 2, 6, 3, 7);
    let blocks16 = block_count / 16;
    let input_end = input_ptr.add(blocks16 * 16 * 8);

    while input_ptr < input_end {
        // Load all 128 bytes first to utilize memory pipeline
        let input_0 = _mm256_loadu_si256(input_ptr as *const __m256i);
        let input_1 = _mm256_loadu_si256(input_ptr.add(32) as *const __m256i);
        let input_2 = _mm256_loadu_si256(input_ptr.add(64) as *const __m256i);
        let input_3 = _mm256_loadu_si256(input_ptr.add(96) as *const __m256i);
        input_ptr = input_ptr.add(128);

        // Do all shuffles together to utilize shuffle units
        // Colors: vshufps with 0b10001000 (136) extracts elements 0,2 from each 128-bit lane
        // We separate only the color components from the input blocks.
        let colors_only_0 = _mm256_shuffle_ps(
            _mm256_castsi256_ps(input_0),
            _mm256_castsi256_ps(input_1),
            0b10001000,
        );

        let colors_only_1 = _mm256_shuffle_ps(
            _mm256_castsi256_ps(input_2),
            _mm256_castsi256_ps(input_3),
            0b10001000,
        );

        // Indices: vshufps with 0b11011101 (221) extracts elements 1,3 from each 128-bit lane
        let indices_shuffled_0 = _mm256_shuffle_ps(
            _mm256_castsi256_ps(input_0),
            _mm256_castsi256_ps(input_1),
            0b11011101,
        );
        let indices_blocks_0 =
            _mm256_permute4x64_epi64(_mm256_castps_si256(indices_shuffled_0), 0b11011000); // arrange indices (216)

        let indices_shuffled_1 = _mm256_shuffle_ps(
            _mm256_castsi256_ps(input_2),
            _mm256_castsi256_ps(input_3),
            0b11011101,
        );
        let indices_blocks_1 =
            _mm256_permute4x64_epi64(_mm256_castps_si256(indices_shuffled_1), 0b11011000); // arrange indices (216)

        // We now group the colours into u32s of color0 and color1 components.
        let colours_u32_grouped_0_lo =
            _mm256_shufflelo_epi16(_mm256_castps_si256(colors_only_0), 0b11_01_10_00);
        let colours_u32_grouped_0 = _mm256_shufflehi_epi16(colours_u32_grouped_0_lo, 0b11_01_10_00);

        let colours_u32_grouped_1_lo =
            _mm256_shufflelo_epi16(_mm256_castps_si256(colors_only_1), 0b11_01_10_00);
        let colours_u32_grouped_1 = _mm256_shufflehi_epi16(colours_u32_grouped_1_lo, 0b11_01_10_00);

        let colors_grouped_0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colours_u32_grouped_0),
            _mm256_castsi256_ps(colours_u32_grouped_1),
            0b10_00_10_00,
        ));
        let colors_grouped_1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colours_u32_grouped_0),
            _mm256_castsi256_ps(colours_u32_grouped_1),
            0b11_01_11_01,
        ));

        // We now have the correct output, but the data is interleaved across lanes.
        // i.e. First u32 in first lane, then first u32 in second lane, etc.
        // We need to merge these u32s across lanes. Permute + unpack seems to be the fastest way.
        // Re-order the 32-bit words to `[0,4,1,5,2,6,3,7]` so the two 128-bit lanes are interleaved.
        let colors_blocks_0 = _mm256_permutevar8x32_epi32(colors_grouped_0, permute_idx);
        let colors_blocks_1 = _mm256_permutevar8x32_epi32(colors_grouped_1, permute_idx);

        let colors_blocks_0: __m256i = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colors_blocks_0),
            2 => decorrelate_ycocg_r_var2_avx2(colors_blocks_0),
            3 => decorrelate_ycocg_r_var3_avx2(colors_blocks_0),
            _ => unreachable_unchecked(),
        };

        let colors_blocks_1: __m256i = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colors_blocks_1),
            2 => decorrelate_ycocg_r_var2_avx2(colors_blocks_1),
            3 => decorrelate_ycocg_r_var3_avx2(colors_blocks_1),
            _ => unreachable_unchecked(),
        };

        // Store all results together to utilize store pipeline
        _mm256_storeu_si256(color0_out as *mut __m256i, colors_blocks_0); // Store colors
        _mm256_storeu_si256(color1_out as *mut __m256i, colors_blocks_1); // Store colors
        color0_out = color0_out.add(16); // color0_ptr += 32 bytes / 2 = 16 u16s
        color1_out = color1_out.add(16); // color1_ptr += 32 bytes / 2 = 16 u16s

        _mm256_storeu_si256(indices_out as *mut __m256i, indices_blocks_0); // Store indices
        _mm256_storeu_si256(indices_out.add(8) as *mut __m256i, indices_blocks_1); // Store indices (@ +32 bytes)
        indices_out = indices_out.add(16); // indices_ptr += 64 bytes / 4 = 16 u32s
    }

    // Remainder blocks handled by generic implementation
    let rem = block_count % 16;
    let variant_enum = match VARIANT {
        1 => YCoCgVariant::Variant1,
        2 => YCoCgVariant::Variant2,
        3 => YCoCgVariant::Variant3,
        _ => unreachable_unchecked(),
    };
    generic::transform_with_split_colour_and_decorr_generic(
        input_ptr,
        color0_out,
        color1_out,
        indices_out,
        rem,
        variant_enum,
    );
}

// Wrappers for asm inspection
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_var1(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
) {
    transform_impl::<1>(input_ptr, color0_out, color1_out, indices_out, blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_var2(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
) {
    transform_impl::<2>(input_ptr, color0_out, color1_out, indices_out, blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_var3(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
) {
    transform_impl::<3>(input_ptr, color0_out, color1_out, indices_out, blocks)
}

// Runtime dispatcher for AVX2 path
#[inline(always)]
pub(crate) unsafe fn transform_with_split_colour_and_decorr(
    input_ptr: *const u8,
    color0_ptr: *mut u16,
    color1_ptr: *mut u16,
    indices_ptr: *mut u32,
    block_count: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => {
            transform_var1(input_ptr, color0_ptr, color1_ptr, indices_ptr, block_count)
        }
        YCoCgVariant::Variant2 => {
            transform_var2(input_ptr, color0_ptr, color1_ptr, indices_ptr, block_count)
        }
        YCoCgVariant::Variant3 => {
            transform_var3(input_ptr, color0_ptr, color1_ptr, indices_ptr, block_count)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour_and_recorr::untransform::untransform_with_split_colour_and_recorr;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn avx2_transform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_avx2() {
            return;
        }
        for blocks in 1..=128 {
            let input = generate_bc1_test_data(blocks);
            let mut colour0 = vec![0u16; blocks];
            let mut colour1 = vec![0u16; blocks];
            let mut indices = vec![0u32; blocks];
            let mut reconstructed = vec![0u8; input.len()];
            unsafe {
                transform_with_split_colour_and_decorr(
                    input.as_ptr(),
                    colour0.as_mut_ptr(),
                    colour1.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    blocks,
                    variant,
                );
                untransform_with_split_colour_and_recorr(
                    colour0.as_ptr(),
                    colour1.as_ptr(),
                    indices.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    blocks,
                    variant,
                );
            }
            assert_eq!(
                input.as_slice(),
                reconstructed.as_slice(),
                "AVX2 roundtrip mismatch {variant:?}"
            );
        }
    }
}
