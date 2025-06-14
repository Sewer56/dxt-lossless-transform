#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[target_feature(enable = "sse2")]
#[allow(clippy::identity_op)]
pub unsafe fn untransform_with_split_colour(
    mut color0_in: *const u16,
    mut color1_in: *const u16,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 8 blocks (64 bytes) at a time with SSE2
    let aligned_count = block_count - (block_count % 8);
    let color0_ptr_aligned_end = color0_in.add(aligned_count);

    while color0_in < color0_ptr_aligned_end {
        let color0s = _mm_loadu_si128(color0_in as *const __m128i);
        color0_in = color0_in.add(8);

        let color1s = _mm_loadu_si128(color1_in as *const __m128i);
        color1_in = color1_in.add(8);

        let indices_0 = _mm_loadu_si128(indices_in as *const __m128i);
        let indices_1 = _mm_loadu_si128(indices_in.add(4) as *const __m128i);
        indices_in = indices_in.add(8);

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

    // Process any remaining blocks (less than 8) using generic implementation
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
        if !has_sse2() {
            return;
        }

        // 64 bytes processed per main loop iteration (* 2 / 8 == 16)
        run_with_split_colour_untransform_unaligned_test(
            untransform_with_split_colour,
            16,
            "untransform_with_split_colour (sse2, unaligned)",
        );
    }
}
