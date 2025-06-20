use crate::transforms::split_565_color_endpoints::portable32::u32_with_separate_endpoints;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Splits the colour endpoints using AVX512VBMI instructions
///
/// # Arguments
///
/// * `colors` - Pointer to the input array of colors
/// * `colors_out` - Pointer to the output array of colors
/// * `colors_len_bytes` - Number of bytes in the input array.
///
/// # Safety
///
/// - `colors` must be valid for reads of `colors_len_bytes` bytes
/// - `colors_out` must be valid for writes of `colors_len_bytes` bytes
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 32-byte aligned for best performance
/// - CPU must support AVX512VBMI instructions
#[target_feature(enable = "avx512vbmi")]
#[allow(clippy::identity_op)]
pub(crate) unsafe fn avx512_impl_unroll2(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    const BYTES_PER_ITERATION: usize = 256;

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let aligned_len = colors_len_bytes - (colors_len_bytes % BYTES_PER_ITERATION);
    let aligned_end_ptr = colors.add(aligned_len);

    // Input data:
    // [00 01], (80 81), [02 03], (82 83), [04 05], (84 85), [06 07], (86 87)
    // [08 09], (88 89), [0A 0B], (8A 8B), [0C 0D], (8C 8D), [0E 0F], (8E 8F)
    // [10 11], (90 91), [12 13], (92 93), [14 15], (94 95), [16 17], (96 97)
    // [18 19], (98 99), [1A 1B], (9A 9B), [1C 1D], (9C 9D), [1E 1F], (9E 9F)

    // Desired format (big endian):
    // [00 01], [02 03], [04 05], [06 07], (80 81), (82 83), (84 85), (86 87)
    // [08 09], [0A 0B], [0C 0D], [0E 0F], (88 89), (8A 8B), (8C 8D), (8E 8F)
    // [10 11], [12 13], [14 15], [16 17], (90 91), (92 93), (94 95), (96 97)
    // [18 19], [1A 1B], [1C 1D], [1E 1F], (98 99), (9A 9B), (9C 9D), (9E 9F)
    #[rustfmt::skip]
    let shuffle_mask_color0 = _mm512_set_epi8(
        125,124,121,120,117,116,113,112,
        109,108,105,104,101,100,97,96,
        93,92,89,88,85,84,81,80,
        77,76,73,72,69,68,65,64, // variable 1
        61,60,57,56,53,52,49,48,
        45,44,41,40,37,36,33,32,
        29,28,25,24,21,20,17,16,
        13,12,9,8,5,4,1,0, // variable 2
    );

    #[rustfmt::skip]
    let shuffle_mask_color1 = _mm512_set_epi8(
        127,126,123,122,119,118,115,114,
        111,110,107,106,103,102,99,98,
        95,94,91,90,87,86,83,82,
        79,78,75,74,71,70,67,66, // variable 1
        63,62,59,58,55,54,51,50,
        47,46,43,42,39,38,35,34,
        31,30,27,26,23,22,19,18,
        15,14,11,10,7,6,3,2, // variable 2
    );

    // Note(sewer): Don't forget the damned little endian!! Flip the order!!
    while input_ptr < aligned_end_ptr {
        // Load 64 bytes (2 blocks of 64 bytes each)
        let chunk1 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let chunk2 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        let chunk3 = _mm512_loadu_si512(input_ptr.add(128) as *const __m512i);
        let chunk4 = _mm512_loadu_si512(input_ptr.add(192) as *const __m512i);
        input_ptr = input_ptr.add(BYTES_PER_ITERATION); // 256

        // Store results
        let color0_rearranged_0 = _mm512_permutex2var_epi8(chunk1, shuffle_mask_color0, chunk2);
        let color1_rearranged_0 = _mm512_permutex2var_epi8(chunk1, shuffle_mask_color1, chunk2);
        let color2_rearranged_1 = _mm512_permutex2var_epi8(chunk3, shuffle_mask_color0, chunk4);
        let color3_rearranged_1 = _mm512_permutex2var_epi8(chunk3, shuffle_mask_color1, chunk4);

        _mm512_storeu_si512(output_low as *mut __m512i, color0_rearranged_0);
        _mm512_storeu_si512(output_low.add(64) as *mut __m512i, color2_rearranged_1);
        output_low = output_low.add(128);

        _mm512_storeu_si512(output_high as *mut __m512i, color1_rearranged_0);
        _mm512_storeu_si512(output_high.add(64) as *mut __m512i, color3_rearranged_1);
        output_high = output_high.add(128);
    }

    // Handle remaining elements
    let end_ptr = colors.add(colors_len_bytes);
    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output_low as *mut u16,
        output_high as *mut u16,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::split_565_color_endpoints::tests::*;
    use rstest::rstest;

    #[rstest]
    #[case(avx512_impl_unroll2, "avx512_impl_unroll2")]
    fn test_avx512_aligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        if !is_x86_feature_detected!("avx512vbmi") {
            return;
        }
        test_implementation_aligned(implementation, impl_name);
    }

    #[rstest]
    #[case(avx512_impl_unroll2, "avx512_impl_unroll2")]
    fn test_avx512_unaligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        if !is_x86_feature_detected!("avx512vbmi") {
            return;
        }
        test_implementation_unaligned(implementation, impl_name);
    }
}
