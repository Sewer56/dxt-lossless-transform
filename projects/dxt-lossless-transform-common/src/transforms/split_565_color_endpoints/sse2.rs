#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::transforms::split_565_color_endpoints::u32_with_separate_endpoints;

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
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
pub unsafe fn sse2_shift_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    // Setup pointers for processing
    let mut input_ptr = colors as *const u64;
    let mut output0_ptr = colors_out as *mut u64;
    let mut output1_ptr = colors_out.add(colors_len_bytes / 2) as *mut u64;

    // Calculate end pointer for our main loop (process 16 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes) as *const u64;
    let aligned_end_ptr = end_ptr.sub(2); // Align to process remaining elements

    while input_ptr < aligned_end_ptr {
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

    // Handle remaining elements
    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output0_ptr as *mut u16,
        output1_ptr as *mut u16,
    );
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
/// - `colors_len_bytes` must be a multiple of 4
/// - Pointers should be 16-byte aligned for best performance
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
pub unsafe fn sse2_shuf_impl(colors: *const u8, colors_out: *mut u8, colors_len_bytes: usize) {
    debug_assert!(
        colors_len_bytes >= 4 && colors_len_bytes % 4 == 0,
        "colors_len_bytes must be at least 4 and a multiple of 4"
    );

    // Setup pointers for processing
    let mut input_ptr = colors;
    let mut output_low = colors_out;
    let mut output_high = colors_out.add(colors_len_bytes / 2);

    // Calculate end pointer for our main loop (process 32 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);
    let aligned_end_ptr = end_ptr.sub(32);

    while input_ptr < aligned_end_ptr {
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

    // Handle remaining elements
    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output_low as *mut u16,
        output_high as *mut u16,
    );
}

/// Alternative implementation using pshuflw and pshufhw instructions from SSE2,
/// processing 64 bytes at once (unroll factor of 2)
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
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
#[allow(unused_assignments)]
pub unsafe fn sse2_shuf_unroll2_impl_asm(
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

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);
    let aligned_end_ptr = end_ptr.sub(64);

    if input_ptr < aligned_end_ptr {
        std::arch::asm!(
            // Loop alignment
            ".p2align 4",

            // Main processing loop
            "2:",  // Label for loop start

            // Load 64 bytes (4 XMM registers)
            "movdqu {xmm0}, xmmword ptr [{src_ptr}]",
            "movdqu {xmm1}, xmmword ptr [{src_ptr} + 16]",
            "movdqu {xmm3}, xmmword ptr [{src_ptr} + 48]",
            "movdqu {xmm2}, xmmword ptr [{src_ptr} + 32]",
            "add {src_ptr}, 64",

            // Shuffle operations for first chunk
            "pshuflw {xmm0}, {xmm0}, 216", // 11011000b = 216
            "pshuflw {xmm1}, {xmm1}, 216",
            "pshuflw {xmm3}, {xmm3}, 216",
            "pshufhw {xmm0}, {xmm0}, 39",  // 00100111b = 39
            "pshufhw {xmm1}, {xmm1}, 39",
            "pshufhw {xmm3}, {xmm3}, 39",
            "pshufd {xmm0}, {xmm0}, 108",  // 01101100b = 108
            "pshufd {xmm1}, {xmm1}, 108",
            "pshufd {xmm3}, {xmm3}, 108",
            "pshufhw {xmm4}, {xmm0}, 30",  // 00011110b = 30
            "pshufhw {xmm5}, {xmm1}, 30",
            "pshuflw {xmm0}, {xmm0}, 180", // 10110100b = 180
            "pshuflw {xmm1}, {xmm1}, 180",
            "punpcklqdq {xmm0}, {xmm1}",
            "pshuflw {xmm1}, {xmm2}, 216",
            "movhlps {xmm5}, {xmm4}",
            "pshufhw {xmm4}, {xmm3}, 30",
            "pshuflw {xmm3}, {xmm3}, 180",
            "pshufhw {xmm1}, {xmm1}, 39",

            // Store first result
            "movdqu xmmword ptr [{out_low}], {xmm0}",
            "pshufd {xmm1}, {xmm1}, 108",
            "pshufhw {xmm2}, {xmm1}, 30",
            "pshuflw {xmm1}, {xmm1}, 180",
            "punpcklqdq {xmm1}, {xmm3}",
            "movhlps {xmm4}, {xmm2}",

            // Store remaining results
            "movdqu xmmword ptr [{out_low} + 16], {xmm1}",
            "movups xmmword ptr [{out_high}], {xmm5}",
            "movups xmmword ptr [{out_high} + 16], {xmm4}",
            "add {out_low}, 32",
            "add {out_high}, 32",

            // Loop condition
            "cmp {src_ptr}, {end_ptr}",
            "jb 2b",

            src_ptr = inout(reg) input_ptr,
            end_ptr = in(reg) aligned_end_ptr,
            out_low = inout(reg) output_low,
            out_high = inout(reg) output_high,

            // XMM registers
            xmm0 = out(xmm_reg) _,
            xmm1 = out(xmm_reg) _,
            xmm2 = out(xmm_reg) _,
            xmm3 = out(xmm_reg) _,
            xmm4 = out(xmm_reg) _,
            xmm5 = out(xmm_reg) _,

            options(preserves_flags, nostack)
        );
    }

    // Handle remaining elements
    u32_with_separate_endpoints(
        end_ptr as *const u32,
        input_ptr as *const u32,
        output_low as *mut u16,
        output_high as *mut u16,
    );
}

/// Alternative implementation using pshuflw and pshufhw instructions from SSE2,
/// processing 64 bytes at once (unroll factor of 2)
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
/// - CPU must support SSE2 instructions
#[target_feature(enable = "sse2")]
pub unsafe fn sse2_shuf_unroll2_impl(
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

    // Calculate end pointer for our main loop (process 64 bytes at a time)
    let end_ptr = colors.add(colors_len_bytes);
    let aligned_end_ptr = end_ptr.sub(64);

    while input_ptr < aligned_end_ptr {
        // Load 64 bytes (4 blocks of 16 bytes each)
        let chunk0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let chunk1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let chunk2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let chunk3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);

        // Group the color0 and color1(s) together for each chunk
        let shuffled0_low = _mm_shufflelo_epi16::<0b11011000>(chunk0);
        let shuffled0 = _mm_shufflehi_epi16::<0b11011000>(shuffled0_low);

        let shuffled1_low = _mm_shufflelo_epi16::<0b11011000>(chunk1);
        let shuffled1 = _mm_shufflehi_epi16::<0b11011000>(shuffled1_low);

        let shuffled2_low = _mm_shufflelo_epi16::<0b11011000>(chunk2);
        let shuffled2 = _mm_shufflehi_epi16::<0b11011000>(shuffled2_low);

        let shuffled3_low = _mm_shufflelo_epi16::<0b11011000>(chunk3);
        let shuffled3 = _mm_shufflehi_epi16::<0b11011000>(shuffled3_low);

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
        let interleaved_u32s_2 = _mm_shuffle_ps(
            _mm_castsi128_ps(shuffled2),
            _mm_castsi128_ps(shuffled2),
            0b11011000,
        );
        let interleaved_u32s_3 = _mm_shuffle_ps(
            _mm_castsi128_ps(shuffled3),
            _mm_castsi128_ps(shuffled3),
            0b11011000,
        );

        // Now combine them properly into the final color0 and color1
        let colors0 = _mm_shuffle_ps(interleaved_u32s_0, interleaved_u32s_1, 0b01000100);
        let colors1 = _mm_shuffle_ps(interleaved_u32s_2, interleaved_u32s_3, 0b01000100);

        let colors0_2 = _mm_shuffle_ps(interleaved_u32s_0, interleaved_u32s_1, 0b11101110);
        let colors1_2 = _mm_shuffle_ps(interleaved_u32s_2, interleaved_u32s_3, 0b11101110);

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

    // Handle remaining elements
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
    #[case(sse2_shift_impl, "sse2_shift")]
    #[case(sse2_shuf_impl, "sse2_shuf")]
    #[case(sse2_shuf_unroll2_impl, "sse2_shuf_unroll2")]
    #[case(sse2_shuf_unroll2_impl_asm, "sse2_shuf_unroll2_asm")]
    fn test_sse2_aligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
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
    #[case(sse2_shift_impl, "sse2_shift")]
    #[case(sse2_shuf_impl, "sse2_shuf")]
    #[case(sse2_shuf_unroll2_impl, "sse2_shuf_unroll2")]
    #[case(sse2_shuf_unroll2_impl_asm, "sse2_shuf_unroll2_asm")]
    fn test_sse2_unaligned(#[case] implementation: TransformFn, #[case] impl_name: &str) {
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
