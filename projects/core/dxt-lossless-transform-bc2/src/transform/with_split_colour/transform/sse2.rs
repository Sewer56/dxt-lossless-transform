use crate::transform::with_split_colour::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// SSE2 implementation for split-colour transform for BC2.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn transform_with_split_colour(
    mut input_ptr: *const u8,
    mut alpha_out: *mut u64,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    // Process 4 blocks at a time (64 bytes) with SSE2
    let num_iterations = block_count / 8 * 8; // 8 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // * 16 bytes per block

    while input_ptr < input_end {
        // Load four 16-byte BC2 blocks
        let data0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);
        let data4 = _mm_loadu_si128(input_ptr.add(64) as *const __m128i);
        let data5 = _mm_loadu_si128(input_ptr.add(80) as *const __m128i);
        let data6 = _mm_loadu_si128(input_ptr.add(96) as *const __m128i);
        let data7 = _mm_loadu_si128(input_ptr.add(112) as *const __m128i);
        input_ptr = input_ptr.add(128);

        // Extract alphas (first 8 bytes of each block)
        let alphas0 = _mm_unpacklo_epi64(data0, data1); // alpha from block 0 and 1
        let alphas1 = _mm_unpacklo_epi64(data2, data3); // alpha from block 2 and 3
        let alphas2 = _mm_unpacklo_epi64(data4, data5); // alpha from block 4 and 5
        let alphas3 = _mm_unpacklo_epi64(data6, data7); // alpha from block 6 and 7

        // Extract colors and indices (last 8 bytes of each block)
        let colors_indices0 = _mm_unpackhi_epi64(data0, data1); // colors+indices from block 0 and 1
        let colors_indices1 = _mm_unpackhi_epi64(data2, data3); // colors+indices from block 2 and 3
        let colors_indices2 = _mm_unpackhi_epi64(data4, data5); // colors+indices from block 4 and 5
        let colors_indices3 = _mm_unpackhi_epi64(data6, data7); // colors+indices from block 6 and 7

        // Now our colors_indices0 and colors_indices1 are essentially BC1 blocks, 4 bytes of colour,
        // and 4 bytes of indices. From here, we can actually reuse the code from the BC1 implementation.
        // i.e. We're working with 4 BC1 blocks.
        let colours_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices0),
            _mm_castsi128_ps(colors_indices1),
            0x88, // Select lower 32-bit from each 64-bit lane
        ));
        let colours_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices2),
            _mm_castsi128_ps(colors_indices3),
            0x88,
        ));
        let indices0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices0),
            _mm_castsi128_ps(colors_indices1),
            0xDD, // Select upper 32-bit from each 64-bit lane
        ));
        let indices1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices2),
            _mm_castsi128_ps(colors_indices3),
            0xDD,
        ));

        // Now we need to split the colours into their color0 and color1 components.
        // SSE2 is a bit limited here, so we'll use what we can to get by.

        // Shuffle so the first 8 bytes have their color0 and color1 components chunked into u32s
        let colours_u32_grouped_0_lo = _mm_shufflelo_epi16(colours_0, 0b11_01_10_00);
        let colours_u32_grouped_0 = _mm_shufflehi_epi16(colours_u32_grouped_0_lo, 0b11_01_10_00);

        let colours_u32_grouped_1_lo = _mm_shufflelo_epi16(colours_1, 0b11_01_10_00);
        let colours_u32_grouped_1 = _mm_shufflehi_epi16(colours_u32_grouped_1_lo, 0b11_01_10_00);

        // Now combine back into single colour registers by shuffling the u32s into their respective positions.
        let colours_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colours_u32_grouped_0),
            _mm_castsi128_ps(colours_u32_grouped_1),
            0b10_00_10_00,
        ));
        let colours_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colours_u32_grouped_0),
            _mm_castsi128_ps(colours_u32_grouped_1),
            0b11_01_11_01,
        ));

        // Store results
        _mm_storeu_si128(alpha_out as *mut __m128i, alphas0);
        _mm_storeu_si128((alpha_out as *mut __m128i).add(1), alphas1);
        _mm_storeu_si128((alpha_out as *mut __m128i).add(2), alphas2);
        _mm_storeu_si128((alpha_out as *mut __m128i).add(3), alphas3);

        _mm_storeu_si128(color0_out as *mut __m128i, colours_0);
        _mm_storeu_si128(color1_out as *mut __m128i, colours_1);

        _mm_storeu_si128(indices_out as *mut __m128i, indices0);
        _mm_storeu_si128((indices_out as *mut __m128i).add(1), indices1);

        alpha_out = alpha_out.add(8); // 8 u64s = 64 bytes
        color0_out = color0_out.add(8); // 8 u16s = 16 bytes
        color1_out = color1_out.add(8); // 8 u16s = 16 bytes
        indices_out = indices_out.add(8); // 8 u32s = 32 bytes
    }

    // Handle any remaining blocks using generic fallback
    let remaining_blocks = block_count % 8;
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
    fn sse2_transform_roundtrip() {
        if !has_sse2() {
            return;
        }

        // 64 bytes processed per main loop iteration (* 2 / 16 == 8)
        run_split_colour_transform_roundtrip_test(transform_with_split_colour, 8, "SSE2");
    }
}
