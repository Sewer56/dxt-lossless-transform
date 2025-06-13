//! These functions are converted from compiler-generated assembly
//! to be used as part of larger assembly routines.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Decorrelate a register of [`Color565`] values using YCoCg-R variant 1
///
/// Takes a `__m128i` register containing 8 [`Color565`] values and returns a register
/// with the colors decorrelated using YCoCg-R variant 1 algorithm that operates
/// directly on 16-bit color values.
///
/// # Safety
///
/// Requires `sse2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var1_sse2(colors_raw: __m128i) -> __m128i {
    // Constants
    let mask_32 = _mm_set1_epi16(32);
    let mask_31 = _mm_set1_epi16(31);

    // Load input colors
    let xmm1 = colors_raw;

    // Extract blue component (AND with mask 32)
    let xmm2 = _mm_and_si128(xmm1, mask_32);

    // Extract red component and start YCoCg-R algorithm
    let mut xmm0 = _mm_srli_epi16(xmm1, 11); // Red component
    xmm0 = _mm_sub_epi16(xmm0, xmm1); // R - input
    xmm0 = _mm_and_si128(xmm0, mask_31); // AND with mask 31

    // Calculate intermediate value
    let mut xmm4 = _mm_srli_epi16(xmm0, 1); // Divide by 2
    xmm4 = _mm_add_epi16(xmm4, xmm1); // Add input colors

    // Process green component
    let mut xmm1 = _mm_srli_epi16(xmm1, 6); // Green component
    xmm1 = _mm_sub_epi16(xmm1, xmm4); // Subtract intermediate
    xmm1 = _mm_and_si128(xmm1, mask_31); // AND with mask 31

    // Combine first part of result
    xmm0 = _mm_slli_epi16(xmm0, 6); // Shift left by 6
    xmm0 = _mm_or_si128(xmm0, xmm2); // OR with blue component
    xmm0 = _mm_or_si128(xmm0, xmm1); // OR with processed green

    // Final calculation for Y component
    xmm1 = _mm_srli_epi16(xmm1, 1); // Shift right by 1
    xmm1 = _mm_add_epi16(xmm1, xmm4); // Add intermediate
    xmm1 = _mm_slli_epi16(xmm1, 11); // Shift to Y position (red field)

    // Final combination
    _mm_or_si128(xmm0, xmm1)
}

/// Decorrelate a register of [`Color565`] values using YCoCg-R variant 2
///
/// Takes a `__m128i` register containing 8 [`Color565`] values and returns a register
/// with the colors decorrelated using YCoCg-R variant 2 algorithm that operates
/// directly on 16-bit color values.
///
/// # Safety
///
/// Requires `sse2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var2_sse2(colors_raw: __m128i) -> __m128i {
    // Constants based on the scalar assembly analysis
    let mask_31 = _mm_set1_epi16(31);
    let mask_g_low = _mm_set1_epi16(-32768); // Mask for preserving g_low bit in position 15 (0x8000)

    // Load input colors
    let input = colors_raw;

    // Extract red component (bits 15-11)
    let red = _mm_srli_epi16(input, 11);

    // Extract green component (bits 10-6, ignoring bit 5)
    let green = _mm_srli_epi16(input, 6);

    // Step 1: Co = R - input (modulo 32)
    let mut co = _mm_sub_epi16(red, input);
    co = _mm_and_si128(co, mask_31);

    // Step 2: t = B + (Co >> 1)
    // B is the lower 5 bits of input, Co >> 1 is the half of co
    let co_half = _mm_srli_epi16(co, 1);
    let t = _mm_add_epi16(input, co_half); // input contains B in lower bits

    // Step 3: Cg = G - t (modulo 32)
    let mut cg = _mm_sub_epi16(green, t);
    cg = _mm_and_si128(cg, mask_31);

    // Step 4: Y = t + (Cg >> 1)
    let cg_half = _mm_srli_epi16(cg, 1);
    let mut y = _mm_add_epi16(t, cg_half);
    y = _mm_and_si128(y, mask_31);

    // Packing based on variant 2 format:
    // Bit 15: g_low (preserved from original green bit 5)
    // Bits 14-10: Y (5 bits)
    // Bits 9-5: Co (5 bits)
    // Bits 4-0: Cg (5 bits)

    // Extract and preserve g_low bit (bit 5 of original input)
    // Based on assembly: shl input, 10 then and with 0x3FFE000 to preserve bit 5 -> bit 15
    let mut g_low_preserved = _mm_slli_epi16(input, 10);
    g_low_preserved = _mm_and_si128(g_low_preserved, mask_g_low);

    // Shift Y to position 14-10 (red field)
    let mut y_shifted = _mm_slli_epi16(y, 10);
    let mask_y = _mm_set1_epi16(0x7C00); // Mask for bits 14-10
    y_shifted = _mm_and_si128(y_shifted, mask_y);

    // Shift Co to position 9-5 (green field)
    let co_shifted = _mm_slli_epi16(co, 5);

    // Combine all components
    let mut result = _mm_or_si128(g_low_preserved, y_shifted);
    result = _mm_or_si128(result, co_shifted);
    result = _mm_or_si128(result, cg);

    result
}

/// Decorrelate a register of [`Color565`] values using YCoCg-R variant 3
///
/// Takes a `__m128i` register containing 8 [`Color565`] values and returns a register
/// with the colors decorrelated using YCoCg-R variant 3 algorithm that operates
/// directly on 16-bit color values.
///
/// # Safety
///
/// Requires `sse2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var3_sse2(colors_raw: __m128i) -> __m128i {
    // Constants for variant 3
    let mask_31 = _mm_set1_epi16(31);
    let mask_1 = _mm_set1_epi16(1);

    // Load input colors
    let input = colors_raw;

    // Extract RGB components
    let r = _mm_srli_epi16(input, 11); // Red component (5 bits)
    let g = _mm_srli_epi16(input, 6); // Green component (will be masked to 5 bits)
    let g_low = _mm_and_si128(_mm_srli_epi16(input, 5), mask_1); // Extract g_low bit
    let b = _mm_and_si128(input, mask_31); // Blue component (5 bits)

    // Apply YCoCg-R variant 3 algorithm
    // Step 1: Co = (R - B) & 0x1F
    let co = _mm_and_si128(_mm_sub_epi16(r, b), mask_31);

    // Step 2: t = B + (Co >> 1)
    let t = _mm_add_epi16(b, _mm_srli_epi16(co, 1));

    // Step 3: Cg = (G - t) & 0x1F
    let cg = _mm_and_si128(_mm_sub_epi16(_mm_and_si128(g, mask_31), t), mask_31);

    // Step 4: Y = t + (Cg >> 1)
    let y = _mm_add_epi16(t, _mm_srli_epi16(cg, 1));

    // Pack into Color565 variant 3 format:
    // Y (5 bits) in bits 15-11: Y << 11
    // Co (5 bits) in bits 10-6: Co << 6
    // Cg (5 bits) in bits 5-1: Cg << 1
    // g_low (1 bit) in bit 0: g_low
    let y_shifted = _mm_slli_epi16(y, 11);
    let co_shifted = _mm_slli_epi16(co, 6);
    let cg_shifted = _mm_slli_epi16(cg, 1);

    // Combine all components
    let result = _mm_or_si128(y_shifted, co_shifted);
    let result = _mm_or_si128(result, cg_shifted);
    _mm_or_si128(result, g_low)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_565::Color565;
    use rstest::rstest;

    /// Test data for all decorrelation variants
    fn get_test_colors() -> [Color565; 8] {
        [
            Color565::from_rgb(255, 128, 64),  // Orange
            Color565::from_rgb(128, 255, 192), // Light green
            Color565::from_rgb(64, 128, 255),  // Light blue
            Color565::from_rgb(192, 64, 128),  // Pink
            Color565::from_raw(0x1234),        // Random value 1
            Color565::from_raw(0x5678),        // Random value 2
            Color565::from_raw(0x9ABC),        // Random value 3
            Color565::from_raw(0xDEF0),        // Random value 4
        ]
    }

    #[rstest]
    #[case(
        decorrelate_ycocg_r_var1_sse2,
        Color565::decorrelate_ycocg_r_var1_ptr,
        "SSE2",
        "variant 1"
    )]
    #[case(
        decorrelate_ycocg_r_var2_sse2,
        Color565::decorrelate_ycocg_r_var2_ptr,
        "SSSE3",
        "variant 2"
    )]
    #[case(
        decorrelate_ycocg_r_var3_sse2,
        Color565::decorrelate_ycocg_r_var3_ptr,
        "SSE2",
        "variant 3"
    )]
    fn test_sse_vs_reference_implementation(
        #[case] intrinsic_fn: unsafe fn(__m128i) -> __m128i,
        #[case] reference_fn: unsafe fn(*const Color565, *mut Color565, usize),
        #[case] arch_name: &str,
        #[case] variant_name: &str,
    ) {
        // Check feature availability based on the function being tested
        let feature_available = match arch_name {
            "SSE2" => is_x86_feature_detected!("sse2"),
            "SSSE3" => is_x86_feature_detected!("ssse3"),
            _ => false,
        };

        if !feature_available {
            return;
        }

        let test_colors = get_test_colors();

        unsafe {
            let mut reference_results = [Color565::from_raw(0); 8];
            reference_fn(
                test_colors.as_ptr(),
                reference_results.as_mut_ptr(),
                test_colors.len(),
            );

            let input_reg = _mm_loadu_si128(test_colors.as_ptr() as *const __m128i);
            let result_reg = intrinsic_fn(input_reg);
            let mut intrinsic_results = [Color565::from_raw(0); 8];
            _mm_storeu_si128(intrinsic_results.as_mut_ptr() as *mut __m128i, result_reg);

            for (x, (&intrinsic_result, &reference_result)) in intrinsic_results
                .iter()
                .zip(reference_results.iter())
                .take(test_colors.len())
                .enumerate()
            {
                assert_eq!(
                    intrinsic_result, reference_result,
                    "{arch_name} {variant_name} mismatch at index {x}: intrinsic={intrinsic_result:?}, reference={reference_result:?}"
                );
            }
        }
    }
}
