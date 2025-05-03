#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::transforms::split_565_color_endpoints::u32_with_separate_endpoints;

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
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 and SSSE3 instructions (for pshufb)
#[target_feature(enable = "ssse3")]
pub unsafe fn ssse3_pshufb_unroll2_impl(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
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
    let aligned_end_ptr = end_ptr.sub(32);

    while input_ptr < aligned_end_ptr {
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

    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output_low as *mut u16,
        output_high as *mut u16,
    );
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
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 and SSSE3 instructions (for pshufb)
#[target_feature(enable = "ssse3")]
pub unsafe fn ssse3_pshufb_unroll4_impl(
    colors: *const u8,
    colors_out: *mut u8,
    colors_len_bytes: usize,
) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
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

    let aligned_end_ptr = end_ptr.sub(64);

    while input_ptr < aligned_end_ptr {
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
    use crate::transforms::split_565_color_endpoints::tests::{
        assert_implementation_matches_reference, generate_test_data,
        transform_with_reference_implementation,
    };
    use rstest::rstest;

    // Define the function pointer type
    type TransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(ssse3_pshufb_unroll2_impl, "ssse3_pshufb_unroll2")]
    #[case(ssse3_pshufb_unroll4_impl, "ssse3_pshufb_unroll4")]
    fn test_ssse3_aligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        for num_pairs in 1..=512 {
            let input = generate_test_data(num_pairs);
            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len()];

            // Generate reference output
            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                implementation(input.as_ptr(), output_test.as_mut_ptr(), input.len());
            }

            // Compare results
            assert_implementation_matches_reference(
                &output_expected,
                &output_test,
                &format!("{impl_name} (aligned)"),
                num_pairs,
            );
        }
    }

    #[rstest]
    #[case(ssse3_pshufb_unroll2_impl, "ssse3_pshufb_unroll2")]
    #[case(ssse3_pshufb_unroll4_impl, "ssse3_pshufb_unroll4")]
    fn test_ssse3_unaligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
        for num_pairs in 1..=512 {
            let input = generate_test_data(num_pairs);

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut input_unaligned = vec![0u8; input.len() + 1];
            input_unaligned[1..].copy_from_slice(input.as_slice());

            let mut output_expected = vec![0u8; input.len()];
            let mut output_test = vec![0u8; input.len() + 1];

            // Generate reference output
            transform_with_reference_implementation(input.as_slice(), &mut output_expected);

            // Clear the output buffer
            output_test.fill(0);

            // Run the implementation
            unsafe {
                // Use pointers offset by 1 byte to create unaligned access
                implementation(
                    input_unaligned.as_ptr().add(1),
                    output_test.as_mut_ptr().add(1),
                    input.len(),
                );
            }

            // Compare results
            assert_implementation_matches_reference(
                &output_expected,
                &output_test[1..],
                &format!("{impl_name} (unaligned)"),
                num_pairs,
            );
        }
    }
}
