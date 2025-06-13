use crate::transforms::with_split_colour::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// SSE2 implementation for split-colour transform.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn transform_with_split_colour(
    mut input_ptr: *const u8,
    mut color0_ptr: *mut u16,
    mut color1_ptr: *mut u16,
    mut indices_ptr: *mut u32,
    block_count: usize,
) {
    let blocks8 = block_count / 8;
    let input_end = input_ptr.add(blocks8 * 8 * 8); // blocks8 * 8 blocks per iteration * 8 bytes per block
    while input_ptr < input_end {
        // Load four 16-byte chunks = 8 blocks
        let data0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);
        input_ptr = input_ptr.add(64);

        // Split colors and indices using shufps patterns
        let colours_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0x88,
        ));
        let colours_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0x88,
        ));
        let idx0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0xDD,
        ));
        let idx1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0xDD,
        ));

        // Now we need to split the colours into their color0 and color1 components.
        // SSE2 is a bit limited here, so we'll use what we can to get by.

        // Shuffle so the first 8 bytes have their color0 and color1 components chunked into u32s
        let col0_grouped_lo = _mm_shufflelo_epi16(colours_0, 0b11_01_10_00);
        let col0_grouped = _mm_shufflehi_epi16(col0_grouped_lo, 0b11_01_10_00);

        let col1_grouped_lo = _mm_shufflelo_epi16(colours_1, 0b11_01_10_00);
        let col1_grouped = _mm_shufflehi_epi16(col1_grouped_lo, 0b11_01_10_00);

        // Now combine back into single colour registers by shuffling the u32s into their respective positions.
        let colours_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(col0_grouped),
            _mm_castsi128_ps(col1_grouped),
            0b10_00_10_00,
        ));
        let colours_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(col0_grouped),
            _mm_castsi128_ps(col1_grouped),
            0b11_01_11_01,
        ));

        // Store results
        _mm_storeu_si128(color0_ptr as *mut __m128i, colours_0);
        _mm_storeu_si128(color1_ptr as *mut __m128i, colours_1);
        _mm_storeu_si128(indices_ptr as *mut __m128i, idx0);
        _mm_storeu_si128((indices_ptr as *mut __m128i).add(1), idx1);

        color0_ptr = color0_ptr.add(8); // 16 bytes
        color1_ptr = color1_ptr.add(8); // 16 bytes
        indices_ptr = indices_ptr.add(8); // 32 bytes
    }
    // Handle any remaining blocks using generic fallback
    let rem = block_count % 8;
    generic::transform_with_split_colour(input_ptr, color0_ptr, color1_ptr, indices_ptr, rem);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour::untransform::untransform_with_split_colour;

    #[rstest]
    fn sse2_transform_roundtrip() {
        for num_blocks in 1..=128 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut colour0 = vec![0u16; num_blocks];
            let mut colour1 = vec![0u16; num_blocks];
            let mut indices = vec![0u32; num_blocks];
            let mut reconstructed = vec![0u8; len];
            unsafe {
                transform_with_split_colour(
                    input.as_ptr(),
                    colour0.as_mut_ptr(),
                    colour1.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    num_blocks,
                );
                untransform_with_split_colour(
                    colour0.as_ptr(),
                    colour1.as_ptr(),
                    indices.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    num_blocks,
                );
            }
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "Mismatch SSE2 roundtrip split-colour",
            );
        }
    }
}
