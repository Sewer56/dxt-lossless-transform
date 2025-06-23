use crate::transform::with_split_colour::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// AVX2 implementation for split-colour transform.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_with_split_colour(
    mut input_ptr: *const u8,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    let len = block_count * 8; // BC1 block size is 8 bytes
    debug_assert!(len % 8 == 0);

    // Process as many 128-byte blocks as possible (16 BC1 blocks)
    let aligned_len = len - (len % 128);

    // Process aligned blocks using intrinsics
    let aligned_end = input_ptr.add(aligned_len);
    let permute_idx = _mm256_setr_epi32(0, 4, 1, 5, 2, 6, 3, 7);

    while input_ptr < aligned_end {
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

        // Store all results together to utilize store pipeline
        _mm256_storeu_si256(color0_out as *mut __m256i, colors_blocks_0); // Store colors
        _mm256_storeu_si256(color1_out as *mut __m256i, colors_blocks_1); // Store colors
        color0_out = color0_out.add(16); // color0_ptr += 32 bytes / 2 = 16 u16s
        color1_out = color1_out.add(16); // color1_ptr += 32 bytes / 2 = 16 u16s

        _mm256_storeu_si256(indices_out as *mut __m256i, indices_blocks_0); // Store indices
        _mm256_storeu_si256(indices_out.add(8) as *mut __m256i, indices_blocks_1); // Store indices (@ +32 bytes)
        indices_out = indices_out.add(16); // indices_ptr += 64 bytes / 4 = 16 u32s
    }

    // Process any remaining elements after the aligned blocks
    let remaining_blocks = (len - aligned_len) / 8;
    generic::transform_with_split_colour(
        input_ptr,
        color0_out,
        color1_out,
        indices_out,
        remaining_blocks,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn avx2_transform_roundtrip() {
        if !has_avx2() {
            return;
        }

        // 128 bytes processed per main loop iteration (* 2 / 8 == 32)
        run_split_colour_transform_roundtrip_test(transform_with_split_colour, 32, "AVX2");
    }
}
