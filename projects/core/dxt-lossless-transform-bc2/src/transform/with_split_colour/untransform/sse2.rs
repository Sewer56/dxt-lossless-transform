use crate::transform::with_split_colour::untransform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// SSE2 implementation for split-colour untransform for BC2.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn untransform_with_split_colour(
    mut alpha_ptr: *const u64,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    let blocks8 = block_count / 8 * 8; // round down via division
    let output_end = output_ptr.add(blocks8 * 16); // * 16 bytes per block

    while output_ptr < output_end {
        // Load 8 alpha values (64 bytes)
        let alpha0 = _mm_loadu_si128(alpha_ptr as *const __m128i);
        let alpha1 = _mm_loadu_si128(alpha_ptr.add(2) as *const __m128i);
        let alpha2 = _mm_loadu_si128(alpha_ptr.add(4) as *const __m128i);
        let alpha3 = _mm_loadu_si128(alpha_ptr.add(6) as *const __m128i);

        // Load 8 color0 and 8 color1 values (16 bytes each == 32 bytes)
        let color0s = _mm_loadu_si128(color0_ptr as *const __m128i);
        let color1s = _mm_loadu_si128(color1_ptr as *const __m128i);

        // Load 8 indices values (32 bytes)
        let indices0 = _mm_loadu_si128(indices_ptr as *const __m128i);
        let indices1 = _mm_loadu_si128(indices_ptr.add(4) as *const __m128i);

        // Combine color0 and color1 into color+indices format
        // Each block needs: [color0: u16][color1: u16][indices: u32]

        // Unpack color0 and color1 to 32-bit values for easy manipulation
        let colors_0 = _mm_unpacklo_epi16(color0s, color1s);
        let colors_1 = _mm_unpackhi_epi16(color0s, color1s);

        let colors_indices_0 = _mm_unpacklo_epi32(colors_0, indices0);
        let colors_indices_1 = _mm_unpackhi_epi32(colors_0, indices0);
        let colors_indices_2 = _mm_unpacklo_epi32(colors_1, indices1);
        let colors_indices_3 = _mm_unpackhi_epi32(colors_1, indices1);

        let block0 = _mm_unpacklo_epi64(alpha0, colors_indices_0);
        let block1 = _mm_unpackhi_epi64(alpha0, colors_indices_0);
        let block2 = _mm_unpacklo_epi64(alpha1, colors_indices_1);
        let block3 = _mm_unpackhi_epi64(alpha1, colors_indices_1);
        let block4 = _mm_unpacklo_epi64(alpha2, colors_indices_2);
        let block5 = _mm_unpackhi_epi64(alpha2, colors_indices_2);
        let block6 = _mm_unpacklo_epi64(alpha3, colors_indices_3);
        let block7 = _mm_unpackhi_epi64(alpha3, colors_indices_3);

        _mm_storeu_si128(output_ptr as *mut __m128i, block0);
        _mm_storeu_si128(output_ptr.add(16) as *mut __m128i, block1);
        _mm_storeu_si128(output_ptr.add(32) as *mut __m128i, block2);
        _mm_storeu_si128(output_ptr.add(48) as *mut __m128i, block3);
        _mm_storeu_si128(output_ptr.add(64) as *mut __m128i, block4);
        _mm_storeu_si128(output_ptr.add(80) as *mut __m128i, block5);
        _mm_storeu_si128(output_ptr.add(96) as *mut __m128i, block6);
        _mm_storeu_si128(output_ptr.add(112) as *mut __m128i, block7);

        // Advance input pointers
        alpha_ptr = alpha_ptr.add(8); // 8 * 8 bytes = 64 bytes
        color0_ptr = color0_ptr.add(8); // 8 * 2 bytes = 16 bytes
        color1_ptr = color1_ptr.add(8); // 8 * 2 bytes = 16 bytes
        indices_ptr = indices_ptr.add(8); // 8 * 4 bytes = 32 bytes
        output_ptr = output_ptr.add(128); // 8 blocks * 16 bytes each
    }

    // Handle any remaining blocks using generic fallback
    let rem = block_count % 8;
    generic::untransform_with_split_colour(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        rem,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn sse2_untransform_roundtrip() {
        if !has_sse2() {
            return;
        }

        // 64 bytes processed per main loop iteration (* 2 / 16 == 8)
        run_split_colour_untransform_roundtrip_test(untransform_with_split_colour, 8, "SSE2");
    }
}
