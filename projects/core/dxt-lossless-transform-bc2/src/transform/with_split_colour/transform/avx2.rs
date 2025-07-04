use crate::transform::with_split_colour::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[allow(clippy::unusual_byte_groupings)]
const PERMUTE_MASK: [u32; 8] = [0, 4, 1, 5, 2, 6, 3, 7];

/// AVX2 implementation for split-colour transform for BC2.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_with_split_colour(
    mut input_ptr: *const u8,
    mut alpha_out: *mut u64,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    // Process 16 BC2 blocks at a time = 256 bytes
    let num_iterations = block_count / 16 * 16; // 16 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // 16 bytes per block

    // Load the permute mask for 32-bit element reordering
    let permute_mask = _mm256_loadu_si256(PERMUTE_MASK.as_ptr() as *const __m256i);

    while input_ptr < input_end {
        // Load 16 BC2 blocks = 256 bytes
        let data0 = _mm256_loadu_si256(input_ptr as *const __m256i); // First two blocks
        let data1 = _mm256_loadu_si256(input_ptr.add(32) as *const __m256i); // Second two blocks
        let data2 = _mm256_loadu_si256(input_ptr.add(64) as *const __m256i); // Third two blocks
        let data3 = _mm256_loadu_si256(input_ptr.add(96) as *const __m256i); // Fourth two blocks
        let data4 = _mm256_loadu_si256(input_ptr.add(128) as *const __m256i); // Fifth two blocks
        let data5 = _mm256_loadu_si256(input_ptr.add(160) as *const __m256i); // Sixth two blocks
        let data6 = _mm256_loadu_si256(input_ptr.add(192) as *const __m256i); // Seventh two blocks
        let data7 = _mm256_loadu_si256(input_ptr.add(224) as *const __m256i); // Eighth two blocks
        input_ptr = input_ptr.add(256);

        // Extract alphas using unpack low (vpunpcklqdq)
        let alphas0 = _mm256_unpacklo_epi64(data1, data0); // alpha -> ymm0 (out of order)
        let alphas1 = _mm256_unpacklo_epi64(data3, data2); // alpha -> ymm3 (out of order)
        let alphas2 = _mm256_unpacklo_epi64(data5, data4); // alpha -> ymm5 (out of order)
        let alphas3 = _mm256_unpacklo_epi64(data7, data6); // alpha -> ymm7 (out of order)

        // Reorder alphas to chronological order (vpermq)
        let alphas_ordered0 = _mm256_permute4x64_epi64(alphas0, 0x8D); // 10_00_11_01 -> [1,3,0,2]
        let alphas_ordered1 = _mm256_permute4x64_epi64(alphas1, 0x8D); // 10_00_11_01 -> [1,3,0,2]
        let alphas_ordered2 = _mm256_permute4x64_epi64(alphas2, 0x8D); // 10_00_11_01 -> [1,3,0,2]
        let alphas_ordered3 = _mm256_permute4x64_epi64(alphas3, 0x8D); // 10_00_11_01 -> [1,3,0,2]

        // Once we have the alphas and colours+indices separated inside separate registers,
        // then we are essentially working with BC1-like blocks inside colors_indices0, colors_indices1,
        // colors_indices2, colors_indices3.

        // We can therefore reuse parts of the BC1 implementation to split the colours into their color0 and color1 components.
        // However, there is an extra important note!!
        //
        // Data will be on different 'lanes' due to gaps introduced by adding the alpha bytes.
        // We need to account for this, do the shuffles a bit different.

        // Extract colors+indices using shuffle (vshufps)
        let colors_indices0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data0),
            _mm256_castsi256_ps(data1),
            0xEE, // 11_10_11_10
        ));
        let colors_indices1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data2),
            _mm256_castsi256_ps(data3),
            0xEE, // 11_10_11_10
        ));
        let colors_indices2 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data4),
            _mm256_castsi256_ps(data5),
            0xEE, // 11_10_11_10
        ));
        let colors_indices3 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data6),
            _mm256_castsi256_ps(data7),
            0xEE, // 11_10_11_10
        ));

        // Separate colors and indices (vshufps)
        let colors_only0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colors_indices0),
            _mm256_castsi256_ps(colors_indices1),
            0x88, // All colors
        ));
        let colors_only1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colors_indices2),
            _mm256_castsi256_ps(colors_indices3),
            0x88, // All colors
        ));

        let colors_only0 = _mm256_permutevar8x32_epi32(colors_only0, permute_mask);
        let colors_only1 = _mm256_permutevar8x32_epi32(colors_only1, permute_mask);

        let indices_shuffled0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colors_indices0),
            _mm256_castsi256_ps(colors_indices1),
            0xDD, // All indices, 0b11_01_11_01
        ));
        let indices_shuffled1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colors_indices2),
            _mm256_castsi256_ps(colors_indices3),
            0xDD, // All indices, 0b11_01_11_01
        ));
        // Note: Arranging the indices here is a bit different.

        // Unlike BC1, we use 32-bit permute to restore lane order at same time as swapping
        let indices_blocks_0 = _mm256_permutevar8x32_epi32(indices_shuffled0, permute_mask);
        let indices_blocks_1 = _mm256_permutevar8x32_epi32(indices_shuffled1, permute_mask);

        // We now group the colours into u32s of color0 and color1 components.
        let colours_u32_grouped_0_lo = _mm256_shufflelo_epi16(colors_only0, 0b11_01_10_00);
        let colours_u32_grouped_0 = _mm256_shufflehi_epi16(colours_u32_grouped_0_lo, 0b11_01_10_00);

        let colours_u32_grouped_1_lo = _mm256_shufflelo_epi16(colors_only1, 0b11_01_10_00);
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

        // So, okay. We processed the colours; but after `let colors_only0 = _mm256_permutevar8x32_epi32(colors_only0, permute_mask);`
        // we actually had them 'slightly' out of order. Namely:
        //
        // Expected:  {0x0706050403020100, 0x1716151413121110, 0x0f0e0d0c0b0a0908, 0x1f1e1d1c1b1a1918}
        // Actual:    {0x0706050403020100, 0x0f0e0d0c0b0a0908, 0x1716151413121110, 0x1f1e1d1c1b1a1918}
        //
        // Our two little lanes were swapped around.
        // We can't undo that swap at `let colors_grouped_0 = _mm256_castps_si256(_mm256_shuffle_ps(`
        // because shuffle is not a cross-lane operation; and we need the data to be split across lanes
        // in order to be able to correctly do the shufflelo / shufflehi operations. So here, we simply
        // unpermute, to restore the lanes to the right order.
        let colors_only0 = _mm256_permute4x64_epi64(colors_grouped_0, 0b11_01_10_00);
        let colors_only1 = _mm256_permute4x64_epi64(colors_grouped_1, 0b11_01_10_00);

        // Store results
        _mm256_storeu_si256(alpha_out as *mut __m256i, alphas_ordered0);
        _mm256_storeu_si256(alpha_out.add(4) as *mut __m256i, alphas_ordered1);
        _mm256_storeu_si256(alpha_out.add(8) as *mut __m256i, alphas_ordered2);
        _mm256_storeu_si256(alpha_out.add(12) as *mut __m256i, alphas_ordered3);
        alpha_out = alpha_out.add(16); // 16 u64s = 128 bytes

        _mm256_storeu_si256(color0_out as *mut __m256i, colors_only0);
        color0_out = color0_out.add(16); // 16 u16s = 32 bytes
        _mm256_storeu_si256(color1_out as *mut __m256i, colors_only1);
        color1_out = color1_out.add(16); // 16 u16s = 32 bytes

        _mm256_storeu_si256(indices_out as *mut __m256i, indices_blocks_0);
        _mm256_storeu_si256((indices_out as *mut __m256i).add(1), indices_blocks_1);
        indices_out = indices_out.add(16); // 16 u32s = 64 bytes
                                           // Increment pointers
    }

    // Handle remaining blocks
    let remaining_blocks = block_count % 16;
    if remaining_blocks > 0 {
        generic::transform_with_split_colour(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            remaining_blocks,
        );
    }
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

        // 256 bytes processed per main loop iteration (* 2 / 16 == 32)
        run_split_colour_transform_roundtrip_test(transform_with_split_colour, 32, "AVX2");
    }
}
