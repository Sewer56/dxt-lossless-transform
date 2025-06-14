#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[target_feature(enable = "avx2")]
#[allow(clippy::identity_op)]
pub unsafe fn untransform_with_split_colour(
    mut color0_in: *const u16,
    mut color1_in: *const u16,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 16 blocks (128 bytes) at a time with AVX2
    let aligned_count = block_count - (block_count % 16);
    let color0_ptr_aligned_end = color0_in.add(aligned_count);
    while color0_in < color0_ptr_aligned_end {
        let color0s = _mm256_loadu_si256(color0_in as *const __m256i);
        color0_in = color0_in.add(16);

        let color1s = _mm256_loadu_si256(color1_in as *const __m256i);
        color1_in = color1_in.add(16);

        let indices_0 = _mm256_loadu_si256(indices_in as *const __m256i);
        let indices_1 = _mm256_loadu_si256(indices_in.add(8) as *const __m256i);
        indices_in = indices_in.add(16);

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

    // Process any remaining blocks (less than 16) using generic implementation
    let remaining_count = block_count - aligned_count;
    super::generic::untransform_with_split_colour(
        color0_in,
        color1_in,
        indices_in,
        output_ptr,
        remaining_count,
    );
}

#[cfg(test)]
mod tests {
    use super::untransform_with_split_colour;
    use crate::test_prelude::*;

    #[test]
    fn can_untransform_unaligned() {
        if !has_avx2() {
            return;
        }

        // 128 bytes processed per main loop iteration (* 2 / 8 == 32)
        run_with_split_colour_untransform_unaligned_test(
            untransform_with_split_colour,
            32,
            "untransform_with_split_colour (avx2, unaligned)",
        );
    }
}
