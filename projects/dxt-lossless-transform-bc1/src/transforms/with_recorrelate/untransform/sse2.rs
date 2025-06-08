#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::sse2::{
    recorrelate_ycocg_r_var1_sse2, recorrelate_ycocg_r_var2_sse2, recorrelate_ycocg_r_var3_sse2,
};

use crate::transforms::with_recorrelate::untransform::generic::untransform_with_recorrelate_generic;

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

#[target_feature(enable = "sse2")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // === Main Vectorized Loop ===
    // Process 8 blocks at a time using SSE2 SIMD instructions (unroll 2)
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !7; // Round down to multiple of 8
    let mut block_index = 0;

    if vectorized_blocks > 0 {
        // Main SIMD processing loop - handles 8 blocks per iteration (unroll 2)
        while block_index < vectorized_blocks {
            // Load colors and indices (16 bytes each)
            // This corresponds to 8 blocks worth of data
            let colors_0 = _mm_loadu_si128(colors_ptr.add(block_index) as *const __m128i);
            let colors_1 = _mm_loadu_si128(colors_ptr.add(block_index + 4) as *const __m128i);

            let indices_0 = _mm_loadu_si128(indices_ptr.add(block_index) as *const __m128i);
            let indices_1 = _mm_loadu_si128(indices_ptr.add(block_index + 4) as *const __m128i);

            // Apply recorrelation to the colors based on the variant
            let recorrelated_colors_0 = match VARIANT {
                1 => recorrelate_ycocg_r_var1_sse2(colors_0),
                2 => recorrelate_ycocg_r_var2_sse2(colors_0),
                3 => recorrelate_ycocg_r_var3_sse2(colors_0),
                _ => unreachable_unchecked(),
            };

            let recorrelated_colors_1 = match VARIANT {
                1 => recorrelate_ycocg_r_var1_sse2(colors_1),
                2 => recorrelate_ycocg_r_var2_sse2(colors_1),
                3 => recorrelate_ycocg_r_var3_sse2(colors_1),
                _ => unreachable_unchecked(),
            };

            // Save copies for high parts (equivalent to movaps in assembly)
            let colors_0_copy = recorrelated_colors_0;
            let colors_1_copy = recorrelated_colors_1;

            // Unpack all blocks - interleave colors and indices
            // punpckldq: interleave low 32-bit values
            let interleaved_0 = _mm_unpacklo_epi32(recorrelated_colors_0, indices_0); // color0,index0,color1,index1
            let interleaved_1 = _mm_unpacklo_epi32(recorrelated_colors_1, indices_1); // color4,index4,color5,index5

            // punpckhdq: interleave high 32-bit values
            let interleaved_2 = _mm_unpackhi_epi32(colors_0_copy, indices_0); // color2,index2,color3,index3
            let interleaved_3 = _mm_unpackhi_epi32(colors_1_copy, indices_1); // color6,index6,color7,index7

            // Store all results (64 bytes total)
            _mm_storeu_si128(
                output_ptr.add(block_index * 8) as *mut __m128i,
                interleaved_0,
            );
            _mm_storeu_si128(
                output_ptr.add(block_index * 8 + 16) as *mut __m128i,
                interleaved_2,
            );
            _mm_storeu_si128(
                output_ptr.add(block_index * 8 + 32) as *mut __m128i,
                interleaved_1,
            );
            _mm_storeu_si128(
                output_ptr.add(block_index * 8 + 48) as *mut __m128i,
                interleaved_3,
            );

            block_index += 8;
        }
    }

    // === Fallback for Remaining Blocks ===
    // Process any remaining blocks using generic implementation
    let remaining_blocks = num_blocks - block_index;
    if remaining_blocks > 0 {
        untransform_with_recorrelate_generic(
            colors_ptr.add(block_index),
            indices_ptr.add(block_index),
            output_ptr.add(block_index * 8),
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
    use crate::test_prelude::*;
    use super::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: unsafe fn(*const u32, *const u32, *mut u8, usize) -> (),
        #[case] decorr_variant: YCoCgVariant,
    ) {
        if !has_sse2() {
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
                "untransform_with_recorrelate (sse2)",
                num_blocks,
            );
        }
    }
}
