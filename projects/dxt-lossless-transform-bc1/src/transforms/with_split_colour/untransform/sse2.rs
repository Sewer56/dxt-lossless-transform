#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[target_feature(enable = "sse2")]
#[allow(clippy::identity_op)]
pub unsafe fn untransform_with_split_colour(
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 8 blocks (64 bytes) at a time with SSE2
    let aligned_count = block_count - (block_count % 8);
    let color0_ptr_aligned_end = color0_ptr.add(aligned_count);

    if aligned_count > 0 {
        while color0_ptr < color0_ptr_aligned_end {
            let color0s = _mm_loadu_si128(color0_ptr as *const __m128i);
            color0_ptr = color0_ptr.add(8);

            let color1s = _mm_loadu_si128(color1_ptr as *const __m128i);
            color1_ptr = color1_ptr.add(8);

            let indices_0 = _mm_loadu_si128(indices_ptr as *const __m128i);
            let indices_1 = _mm_loadu_si128(indices_ptr.add(4) as *const __m128i);
            indices_ptr = indices_ptr.add(8);

            // Mix the colours back into their c0+c1 pairs
            let colors_0 = _mm_unpacklo_epi16(color0s, color1s);
            let colors_1 = _mm_unpackhi_epi16(color0s, color1s);

            // Re-combine the colors and indices into the BC1 block format
            let blocks_0 = _mm_unpacklo_epi32(colors_0, indices_0);
            let blocks_1 = _mm_unpackhi_epi32(colors_0, indices_0);
            let blocks_2 = _mm_unpacklo_epi32(colors_1, indices_1);
            let blocks_3 = _mm_unpackhi_epi32(colors_1, indices_1);

            _mm_storeu_si128(output_ptr as *mut __m128i, blocks_0);
            _mm_storeu_si128(output_ptr.add(16) as *mut __m128i, blocks_1);
            _mm_storeu_si128(output_ptr.add(32) as *mut __m128i, blocks_2);
            _mm_storeu_si128(output_ptr.add(48) as *mut __m128i, blocks_3);

            // Advance output pointer
            output_ptr = output_ptr.add(64);
        }
    }

    // Process any remaining blocks (less than 8) using generic implementation
    let remaining_count = block_count - aligned_count;
    super::generic::untransform_with_split_colour(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        remaining_count,
    );
}

#[cfg(test)]
mod tests {
    use super::untransform_with_split_colour;
    use crate::experimental::normalize_blocks::*;
    use crate::transforms::standard::transform::tests::{
        assert_implementation_matches_reference, generate_bc1_test_data,
    };
    use crate::{transform_bc1, Bc1TransformDetails};
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::*;

    #[test]
    fn can_untransform_unaligned() {
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
                "untransform_with_split_colour (sse2, unaligned)",
                num_blocks,
            );
        }
    }
}
