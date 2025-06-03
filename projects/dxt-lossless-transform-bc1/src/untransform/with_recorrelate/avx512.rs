#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx512::{
    recorrelate_ycocg_r_var1_avx512, recorrelate_ycocg_r_var2_avx512,
    recorrelate_ycocg_r_var3_avx512,
};

pub(crate) unsafe fn untransform_with_recorrelate(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::None => {
            // This should be unreachable based on the calling context
            unreachable_unchecked()
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`

unsafe fn untransform_recorr_var1(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(colors_ptr, indices_ptr, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(colors_ptr, indices_ptr, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(colors_ptr, indices_ptr, output_ptr, num_blocks)
}

#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // === Main Vectorized Loop ===
    // Process 32 blocks at a time using AVX512 SIMD instructions (unroll 2)
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !31; // Round down to multiple of 32
    let mut block_index = 0;

    if vectorized_blocks > 0 {
        // Set up permutation patterns for interleaving colors and indices (moved outside loop)
        const PERM_LOW_BYTES: [i8; 16] = [0, 16, 1, 17, 2, 18, 3, 19, 4, 20, 5, 21, 6, 22, 7, 23];
        const PERM_HIGH_BYTES: [i8; 16] =
            [8, 24, 9, 25, 10, 26, 11, 27, 12, 28, 13, 29, 14, 30, 15, 31];

        let perm_low = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_LOW_BYTES.as_ptr() as *const _));
        let perm_high = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_HIGH_BYTES.as_ptr() as *const _));

        // Main SIMD processing loop - handles 32 blocks per iteration (unroll 2)
        while block_index < vectorized_blocks {
            // Load first set of colors and indices (64 bytes each)
            let colors_0 = _mm512_loadu_si512(colors_ptr.add(block_index) as *const __m512i);
            let colors_1 = _mm512_loadu_si512(colors_ptr.add(block_index + 16) as *const __m512i);

            let indices_0 = _mm512_loadu_si512(indices_ptr.add(block_index) as *const __m512i);
            let indices_1 = _mm512_loadu_si512(indices_ptr.add(block_index + 16) as *const __m512i);

            // Apply recorrelation to the colors based on the variant
            let recorrelated_colors_0 = match VARIANT {
                1 => recorrelate_ycocg_r_var1_avx512(colors_0),
                2 => recorrelate_ycocg_r_var2_avx512(colors_0),
                3 => recorrelate_ycocg_r_var3_avx512(colors_0),
                _ => unreachable_unchecked(),
            };
            let recorrelated_colors_1 = match VARIANT {
                1 => recorrelate_ycocg_r_var1_avx512(colors_1),
                2 => recorrelate_ycocg_r_var2_avx512(colors_1),
                3 => recorrelate_ycocg_r_var3_avx512(colors_1),
                _ => unreachable_unchecked(),
            };

            // Apply permutations to interleave colors and indices (equivalent to vpermt2d instructions)
            let output_0 = _mm512_permutex2var_epi32(recorrelated_colors_0, perm_low, indices_0);
            let output_1 = _mm512_permutex2var_epi32(recorrelated_colors_0, perm_high, indices_0);
            let output_2 = _mm512_permutex2var_epi32(recorrelated_colors_1, perm_low, indices_1);
            let output_3 = _mm512_permutex2var_epi32(recorrelated_colors_1, perm_high, indices_1);

            // Store results (equivalent to vmovdqu64 instructions)
            _mm512_storeu_si512(output_ptr.add(block_index * 8) as *mut __m512i, output_0);
            _mm512_storeu_si512(
                output_ptr.add(block_index * 8 + 64) as *mut __m512i,
                output_1,
            );
            _mm512_storeu_si512(
                output_ptr.add(block_index * 8 + 128) as *mut __m512i,
                output_2,
            );
            _mm512_storeu_si512(
                output_ptr.add(block_index * 8 + 192) as *mut __m512i,
                output_3,
            );

            block_index += 32;
        }
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
        colors_ptr.add(vectorized_blocks),
        indices_ptr.add(vectorized_blocks),
        output_ptr.add(vectorized_blocks * 8),
        remaining_count,
        variant,
    );
}

#[cfg(test)]
mod tests {
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;

    use crate::with_recorrelate::avx512::*;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::{has_avx512bw, has_avx512f};
    use rstest::rstest;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: unsafe fn(*const u32, *const u32, *mut u8, usize) -> (),
        #[case] decorr_variant: YCoCgVariant,
    ) {
        if !has_avx512f() & has_avx512bw() {
            return;
        }

        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            let mut work = vec![0u8; original.len()];
            unsafe {
                transform_bc1(
                    original.as_ptr(),
                    transformed.as_mut_ptr(),
                    work.as_mut_ptr(),
                    original.len(),
                    Bc1TransformDetails {
                        color_normalization_mode: ColorNormalizationMode::None,
                        decorrelation_mode: decorr_variant,
                        split_colour_endpoints: false,
                    },
                );
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);
            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                function(
                    transformed_unaligned.as_ptr().add(1) as *const u32,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "untransform_with_recorrelate (avx512)",
                num_blocks,
            );
        }
    }
}
