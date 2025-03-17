use core::arch::{asm, x86_64::*};

/// Splits the colour endpoints using SSE2 instructions
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
/// - `colors_len_bytes` must be a multiple of 16
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
pub unsafe fn sse2_shift_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 16 && colors_len_bytes % 16 == 0,
        "colors_len_bytes must be at least 16 and a multiple of 16"
    );

    // Setup pointers for processing
    let mut input_ptr = colors as *const u64;
    let mut output0_ptr = colors_out as *mut u64;
    let mut output1_ptr = colors_out.add(colors_len_bytes / 2) as *mut u64;

    // Calculate end pointer for our main loop (process 16 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes) as *const u64;

    while input_ptr < end_ptr {
        // Load 16 bytes (128 bits) from the input array
        // This gives us 2 complete 64-bit color chunks (8 color components)
        let color_chunk = _mm_loadu_si128(input_ptr as *const __m128i);

        // Make a copy of the input for separate processing
        let color_copy = color_chunk;

        // Extract the low 16 bits of each 32-bit chunk:
        // 1. Shift left by 16 bits (moves low 16 bits to high position)
        let shifted_left = _mm_slli_epi32(color_copy, 16);
        // 2. Arithmetic right shift by 16 bits (brings values back with sign extension)
        let low_parts = _mm_srai_epi32(shifted_left, 16);
        // 3. Pack the four 32-bit integers into four 16-bit integers
        let packed_low = _mm_packs_epi32(low_parts, low_parts); // We now have only the color0 components

        // Extract the high 16 bits of each 32-bit chunk:
        // 1. Arithmetic right shift by 16 bits
        let high_parts = _mm_srai_epi32(color_chunk, 16);
        // 2. Pack the four 32-bit integers into four 16-bit integers
        let packed_high = _mm_packs_epi32(high_parts, high_parts); // We now have only the color1 components

        // Store the low 64 bits (4 x 16-bit values) to output0
        _mm_storel_epi64(output0_ptr as *mut __m128i, packed_low);

        // Store the low 64 bits (4 x 16-bit values) to output1
        _mm_storel_epi64(output1_ptr as *mut __m128i, packed_high);

        // Update pointers for the next iteration
        input_ptr = input_ptr.add(2); // Move to the next 16 bytes of input
        output0_ptr = output0_ptr.add(1); // Move to the next 8 bytes (64 bits) of output0
        output1_ptr = output1_ptr.add(1); // Move to the next 8 bytes (64 bits) of output1
    }
}

/// Alternative implementation using pshuflw and pshufhw instructions from SSE2,
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
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
pub unsafe fn sse2_shuf_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 32 && colors_len_bytes % 32 == 0,
        "colors_len_bytes must be at least 32 and a multiple of 32"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 32 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);

    while input_ptr < end_ptr {
        // Load 32 bytes (2 blocks of 16 bytes each)
        let chunk0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let chunk1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);

        // Group the color0 and color1(s) together
        let shuffled0_low = _mm_shufflelo_epi16::<0b11011000>(chunk0);
        let shuffled0 = _mm_shufflehi_epi16::<0b11011000>(shuffled0_low);

        let shuffled1_low = _mm_shufflelo_epi16::<0b11011000>(chunk1);
        let shuffled1 = _mm_shufflehi_epi16::<0b11011000>(shuffled1_low);

        // Interleave them such that lower halves is color0, upper half is color1
        let interleaved_u32s_0 = _mm_shuffle_ps(
            _mm_castsi128_ps(shuffled0),
            _mm_castsi128_ps(shuffled0),
            0b11011000,
        );
        let interleaved_u32s_1 = _mm_shuffle_ps(
            _mm_castsi128_ps(shuffled1),
            _mm_castsi128_ps(shuffled1),
            0b11011000,
        );

        // Now combine them properly into the final color0 and color1
        let colors0 = _mm_shuffle_ps(interleaved_u32s_0, interleaved_u32s_1, 0b01000100);
        let colors1 = _mm_shuffle_ps(interleaved_u32s_0, interleaved_u32s_1, 0b11101110);

        // Store the results
        _mm_storeu_si128(output_low as *mut __m128i, _mm_castps_si128(colors0));
        _mm_storeu_si128(output_high as *mut __m128i, _mm_castps_si128(colors1));

        // Update pointers for the next iteration
        input_ptr = input_ptr.add(32);
        output_low = output_low.add(16);
        output_high = output_high.add(16);
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
    #[case::single(8)] // 16 bytes - two iterations
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
            ("sse2_shift", sse2_shift_impl),
            ("sse2_shuf", sse2_shuf_impl),
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
