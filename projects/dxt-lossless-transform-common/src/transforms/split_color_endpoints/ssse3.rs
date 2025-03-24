#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Alternative implementation using pshufb and SSE shuffling,
/// processing 32 bytes at once (unroll factor of 2)
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
/// - `colors_len_bytes` must be a multiple of 32
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 and SSSE3 instructions (for pshufb)
#[target_feature(enable = "ssse3")]
pub unsafe fn ssse3_pshufb_unroll2_impl(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 32 && colors_len_bytes % 32 == 0,
        "colors_len_bytes must be at least 32 and a multiple of 32"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Create shuffle mask for pshufb
    // which creates a pattern of `4 bytes color0` -> `4 bytes color1`,
    // interleaved.
    let shuffle_mask = _mm_set_epi8(
        15, 14, 11, 10, 7, 6, 3, 2, //
        13, 12, 9, 8, 5, 4, 1, 0,
    );

    // Calculate end pointer for our main loop (process 32 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);

    while input_ptr < end_ptr {
        // Load 32 bytes (2 blocks of 16 bytes each)
        let chunk0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let chunk1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);

        // Use pshufb to group color0 and color1, 4 bytes at a time,
        // so we can shuffle them between registers.
        let shuffled0 = _mm_shuffle_epi8(chunk0, shuffle_mask);
        let shuffled1 = _mm_shuffle_epi8(chunk1, shuffle_mask);

        // Copy XMM registers since SSE2 enforces they get replaced.
        let temp0 = shuffled0;
        let temp1 = shuffled1;

        let colors0 = _mm_shuffle_ps(_mm_castsi128_ps(temp0), _mm_castsi128_ps(temp1), 0b01000100);
        let colors1 = _mm_shuffle_ps(_mm_castsi128_ps(temp0), _mm_castsi128_ps(temp1), 0b11101110);

        // Store the results
        _mm_storeu_si128(output_low as *mut __m128i, _mm_castps_si128(colors0));
        _mm_storeu_si128(output_high as *mut __m128i, _mm_castps_si128(colors1));

        // Update pointers for the next iteration
        input_ptr = input_ptr.add(32);
        output_low = output_low.add(16);
        output_high = output_high.add(16);
    }
}

/// Alternative implementation using pshufb and SSE shuffling,
/// processing 64 bytes at once similar to shufps_unroll_4
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
/// - `colors_len_bytes` must be a multiple of 64
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 and SSSE3 instructions (for pshufb)
#[target_feature(enable = "ssse3")]
pub unsafe fn ssse3_pshufb_unroll4_impl(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 64 && colors_len_bytes % 64 == 0,
        "colors_len_bytes must be at least 64 and a multiple of 64"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Create shuffle mask for pshufb
    // which creates a pattern of `4 bytes color0` -> `4 bytes color1`,
    // interleaved.
    let shuffle_mask = _mm_set_epi8(
        15, 14, 11, 10, 7, 6, 3, 2, //
        13, 12, 9, 8, 5, 4, 1, 0,
    );

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);

    while input_ptr < end_ptr {
        // Load 64 bytes (4 blocks of 16 bytes each)
        let chunk0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let chunk1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let chunk2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let chunk3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);

        // Use pshufb to rearrange bytes within each 128-bit register
        let shuffled0 = _mm_shuffle_epi8(chunk0, shuffle_mask);
        let shuffled1 = _mm_shuffle_epi8(chunk1, shuffle_mask);
        let shuffled2 = _mm_shuffle_epi8(chunk2, shuffle_mask);
        let shuffled3 = _mm_shuffle_epi8(chunk3, shuffle_mask);

        // Prepare for the shuffle operation, making an XMM copy.
        let temp0 = shuffled0;
        let temp1 = shuffled1;
        let temp2 = shuffled2;
        let temp3 = shuffled3;

        let colors0 = _mm_shuffle_ps(_mm_castsi128_ps(temp0), _mm_castsi128_ps(temp1), 0b01000100);
        let colors1 = _mm_shuffle_ps(_mm_castsi128_ps(temp2), _mm_castsi128_ps(temp3), 0b01000100);

        let colors0_2 =
            _mm_shuffle_ps(_mm_castsi128_ps(temp0), _mm_castsi128_ps(temp1), 0b11101110);
        let colors1_2 =
            _mm_shuffle_ps(_mm_castsi128_ps(temp2), _mm_castsi128_ps(temp3), 0b11101110);

        // Store the results
        _mm_storeu_si128(output_low as *mut __m128i, _mm_castps_si128(colors0));
        _mm_storeu_si128(
            output_low.add(16) as *mut __m128i,
            _mm_castps_si128(colors1),
        );

        _mm_storeu_si128(output_high as *mut __m128i, _mm_castps_si128(colors0_2));
        _mm_storeu_si128(
            output_high.add(16) as *mut __m128i,
            _mm_castps_si128(colors1_2),
        );

        // Update pointers for the next iteration
        input_ptr = input_ptr.add(64);
        output_low = output_low.add(32);
        output_high = output_high.add(32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transforms::split_color_endpoints::tests::{
        generate_test_data, transform_with_reference_implementation,
    };
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case::single(16)] // 32 bytes - two iterations
    #[case::many_unrolls(64)] // 128 bytes - tests multiple iterations
    #[case::large(512)] // 1024 bytes - large dataset
    fn test_implementations(#[case] num_pairs: usize) {
        let input = generate_test_data(num_pairs);
        let mut output_expected = vec![0u8; input.len()];
        let mut output_test = vec![0u8; input.len()];

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), &mut output_expected);

        // Test the SSE2 implementation
        let implementations: [(&str, TransformFn); 2] = [
            ("ssse3_pshufb_unroll2", ssse3_pshufb_unroll2_impl),
            ("ssse3_pshufb_unroll4", ssse3_pshufb_unroll4_impl),
        ];

        for (impl_name, implementation) in implementations {
            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
            assert_eq!(
                output_expected, output_test,
                "{} implementation produced different results than reference for {} color pairs.\n\
                First differing pair will have predictable values:\n\
                Color0: Sequential bytes 0x00,0x01 + (pair_num * 4)\n\
                Color1: Sequential bytes 0x80,0x81 + (pair_num * 4)",
                impl_name, num_pairs
            );
        }
    }
}
