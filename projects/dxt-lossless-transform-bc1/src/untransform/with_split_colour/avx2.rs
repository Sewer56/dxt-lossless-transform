#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[target_feature(enable = "avx2")]
#[allow(clippy::identity_op)]
pub unsafe fn untransform_with_split_colour(
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 16 blocks (128 bytes) at a time with AVX2
    let aligned_count = block_count - (block_count % 16);
    let color0_ptr_aligned_end = color0_ptr.add(aligned_count);

    if aligned_count > 0 {
        while color0_ptr < color0_ptr_aligned_end {
            let color0s = _mm256_loadu_si256(color0_ptr as *const __m256i);
            color0_ptr = color0_ptr.add(16);

            let color1s = _mm256_loadu_si256(color1_ptr as *const __m256i);
            color1_ptr = color1_ptr.add(16);

            let indices_0 = _mm256_loadu_si256(indices_ptr as *const __m256i);
            let indices_1 = _mm256_loadu_si256(indices_ptr.add(8) as *const __m256i);
            indices_ptr = indices_ptr.add(16);

            // Mix the colours back into their c0+c1 pairs
            let colors_0_0 = _mm256_unpacklo_epi16(color0s, color1s);
            let colors_1_0 = _mm256_unpackhi_epi16(color0s, color1s);

            // Because of AVX 'lanes', we meed to permute the upper and lower halves
            let colors_0 = _mm256_permute2x128_si256(colors_0_0, colors_1_0, 0b0010_0000);
            let colors_1 = _mm256_permute2x128_si256(colors_0_0, colors_1_0, 0b0011_0001);

            // Re-combine the colors and indices into the BC1 block format
            let blocks_0 = _mm256_unpacklo_epi32(colors_0, indices_0);
            let blocks_1 = _mm256_unpackhi_epi32(colors_0, indices_0);
            let blocks_2 = _mm256_unpacklo_epi32(colors_1, indices_1);
            let blocks_3 = _mm256_unpackhi_epi32(colors_1, indices_1);

            // We need to combine the lanes to form the final output blocks.
            let output_0 = _mm256_permute2x128_si256(blocks_0, blocks_1, 0b0010_0000);
            let output_1 = _mm256_permute2x128_si256(blocks_0, blocks_1, 0b0011_0001);
            let output_2 = _mm256_permute2x128_si256(blocks_2, blocks_3, 0b0010_0000);
            let output_3 = _mm256_permute2x128_si256(blocks_2, blocks_3, 0b0011_0001);

            _mm256_storeu_si256(output_ptr as *mut __m256i, output_0);
            _mm256_storeu_si256(output_ptr.add(32) as *mut __m256i, output_1);
            _mm256_storeu_si256(output_ptr.add(64) as *mut __m256i, output_2);
            _mm256_storeu_si256(output_ptr.add(96) as *mut __m256i, output_3);

            // Advance output pointer
            output_ptr = output_ptr.add(128);
        }
    }

    // Process any remaining blocks (less than 16) using generic implementation
    let remaining_count = block_count - aligned_count;
    super::generic::untransform_with_split_colour(
        color0_ptr, color1_ptr, indices_ptr, output_ptr, remaining_count
    );
}

#[cfg(test)]
mod tests {
    use super::untransform_with_split_colour;
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::has_avx2;

    #[test]
    fn can_untransform_unaligned() {
        if !has_avx2() {
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
                        decorrelation_mode: YCoCgVariant::None,
                        split_colour_endpoints: true,
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
                untransform_with_split_colour(
                    transformed_unaligned.as_ptr().add(1) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 2) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "untransform_with_split_colour (avx2, unaligned)",
                num_blocks,
            );
        }
    }
}
