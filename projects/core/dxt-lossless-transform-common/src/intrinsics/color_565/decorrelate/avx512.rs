//! AVX512 intrinsics for YCoCg-R decorrelation on [`Color565`] values.
//!
//! This module provides AVX512-optimized functions for performing YCoCg-R decorrelation
//! transformations on packed [`Color565`] data in 512-bit registers.
//!
//! # Functions
//!
//! - [`decorrelate_ycocg_r_var1_avx512`] - Applies YCoCg-R variant 1 decorrelation (g_low at bit 5)
//! - [`decorrelate_ycocg_r_var2_avx512`] - Applies YCoCg-R variant 2 decorrelation (g_low at bit 15)  
//! - [`decorrelate_ycocg_r_var3_avx512`] - Applies YCoCg-R variant 3 decorrelation (g_low at bit 0)
//!
//! Each function processes 32 [`Color565`] values simultaneously using AVX512 instructions.
//!
//! [`Color565`]: crate::color_565::Color565

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Applies YCoCg-R variant 1 decorrelation to 32 [`Color565`] values using AVX512.
///
/// This variant places the g_low bit at position 5 in the output format.
/// Output format: `((y << 11) | (co << 6) | (g_low << 5) | cg)`
///
/// # Parameters
///
/// - `colors_raw`: A 512-bit register containing 32 packed [`Color565`] values
///
/// # Returns
///
/// A 512-bit register containing 32 decorrelated [`Color565`] values
///
/// # Safety
///
/// This function requires AVX512F and AVX512BW support. Caller must ensure the target CPU supports AVX512.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var1_avx512(colors_raw: __m512i) -> __m512i {
    // === Constants from assembly ===
    let mask_31 = _mm512_set1_epi32(0x001F001F); // .LCPI21_1: mask for 5-bit values (31,31)
    let mask_g_low = _mm512_set1_epi32(0x00200020); // .LCPI21_2: mask for g_low bit position 5 (32,32)

    // Load input colors (zmm0 = colors_raw)
    let colors = colors_raw;

    // Extract components through bit manipulation (following assembly exactly)
    // vpsrlw zmm1, zmm0, 11  (Red component)
    let mut red = _mm512_srli_epi16(colors, 11);

    // vpsrlw zmm2, zmm0, 6   (Green component)
    let mut green = _mm512_srli_epi16(colors, 6);

    // Apply YCoCg-R variant 1 decorrelation algorithm
    // vpsubw zmm1, zmm1, zmm0  (Co = R - B, where B is from lower bits of zmm0)
    red = _mm512_sub_epi16(red, colors);

    // vpandd zmm1, zmm1, dword ptr [rip + .LCPI21_1]{1to16}  (Co &= 31)
    red = _mm512_and_si512(red, mask_31);

    // vpsrlw zmm3, zmm1, 1     (Co >> 1)
    let co_half = _mm512_srli_epi16(red, 1);

    // vpsllw zmm1, zmm1, 6     (Co << 6 for final packing)
    let co_shifted = _mm512_slli_epi16(red, 6);

    // vpaddw zmm3, zmm3, zmm0  (t = B + (Co >> 1))
    let t = _mm512_add_epi16(co_half, colors);

    // vpandd zmm0, zmm0, dword ptr [rip + .LCPI21_2]{1to16}  (extract g_low)
    let g_low = _mm512_and_si512(colors, mask_g_low);

    // vpsubw zmm2, zmm2, zmm3  (Cg = G - t)
    green = _mm512_sub_epi16(green, t);

    // vpandd zmm2, zmm2, dword ptr [rip + .LCPI21_1]{1to16}  (Cg &= 31)
    green = _mm512_and_si512(green, mask_31);

    // vpsrlw zmm4, zmm2, 1     (Cg >> 1)
    let cg_half = _mm512_srli_epi16(green, 1);

    // vpaddw zmm3, zmm4, zmm3  (Y = t + (Cg >> 1))
    let y = _mm512_add_epi16(cg_half, t);

    // vpsllw zmm3, zmm3, 11    (Y << 11 for final packing)
    let y_shifted = _mm512_slli_epi16(y, 11);

    // vporq zmm1, zmm3, zmm1   (combine y_shifted and co_shifted)
    let partial_result = _mm512_or_si512(y_shifted, co_shifted);

    // vpternlogd zmm0, zmm2, zmm1, 254  (combine g_low, green(cg), partial_result)
    _mm512_ternarylogic_epi32(g_low, green, partial_result, 254)
}

/// Applies YCoCg-R variant 2 decorrelation to 32 [`Color565`] values using AVX512.
///
/// This variant places the g_low bit at position 15 (top bit) in the output format.
/// Output format: `((g_low << 15) | (y << 10) | (co << 5) | cg)`
///
/// # Parameters
///
/// - `colors_raw`: A 512-bit register containing 32 packed [`Color565`] values
///
/// # Returns
///
/// A 512-bit register containing 32 decorrelated [`Color565`] values
///
/// # Safety
///
/// This function requires AVX512F and AVX512BW support. Caller must ensure the target CPU supports AVX512.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var2_avx512(colors_raw: __m512i) -> __m512i {
    // === Constants from assembly ===
    let mask_31 = _mm512_set1_epi32(0x001F001F); // .LCPI22_1: mask for 5-bit values (31,31)
    let mask_g_low = _mm512_set1_epi32(-2147450880i32); // .LCPI22_2: mask for g_low at bit 15 (0x80008000)
    let mask_y_field = _mm512_set1_epi32(0x7C007C00); // .LCPI22_3: mask for Y field at bit 10

    // Load input colors (zmm0 = colors_raw)
    let colors = colors_raw;

    // Extract components through bit manipulation (following assembly exactly)
    // vpsrlw zmm1, zmm0, 11  (Red component)
    let mut red = _mm512_srli_epi16(colors, 11);

    // vpsrlw zmm2, zmm0, 6   (Green component)
    let mut green = _mm512_srli_epi16(colors, 6);

    // Apply YCoCg-R variant 2 decorrelation algorithm
    // vpsubw zmm1, zmm1, zmm0  (Co = R - B, where B is from lower bits of zmm0)
    red = _mm512_sub_epi16(red, colors);

    // vpandd zmm1, zmm1, dword ptr [rip + .LCPI22_1]{1to16}  (Co &= 31)
    red = _mm512_and_si512(red, mask_31);

    // vpsrlw zmm3, zmm1, 1     (Co >> 1)
    let co_half = _mm512_srli_epi16(red, 1);

    // vpsllw zmm1, zmm1, 5     (Co << 5 for final packing)
    let co_shifted = _mm512_slli_epi16(red, 5);

    // vpaddw zmm3, zmm3, zmm0  (t = B + (Co >> 1))
    let t = _mm512_add_epi16(co_half, colors);

    // vpsllw zmm0, zmm0, 10    (colors << 10 for g_low extraction)
    let colors_shifted = _mm512_slli_epi16(colors, 10);

    // vpternlogd zmm0, zmm1, dword ptr [rip + .LCPI22_2]{1to16}, 236  (combine colors_shifted, co_shifted with g_low)
    let co_with_g_low = _mm512_ternarylogic_epi32(colors_shifted, co_shifted, mask_g_low, 236);

    // vpsubw zmm1, zmm2, zmm3  (Cg = G - t)
    green = _mm512_sub_epi16(green, t);

    // vpandd zmm1, zmm1, dword ptr [rip + .LCPI22_1]{1to16}  (Cg &= 31)
    green = _mm512_and_si512(green, mask_31);

    // vpsrlw zmm2, zmm1, 1     (Cg >> 1)
    let cg_half = _mm512_srli_epi16(green, 1);

    // vpaddw zmm2, zmm2, zmm3  (Y = t + (Cg >> 1))
    let y = _mm512_add_epi16(cg_half, t);

    // vpsllw zmm2, zmm2, 10    (Y << 10 for final packing)
    let y_shifted = _mm512_slli_epi16(y, 10);

    // vpandd zmm2, zmm2, dword ptr [rip + .LCPI22_3]{1to16}  (mask Y field)
    let y_masked = _mm512_and_si512(y_shifted, mask_y_field);

    // vpternlogd zmm2, zmm1, zmm0, 254  (combine y_masked, green(cg), co_with_g_low)
    _mm512_ternarylogic_epi32(y_masked, green, co_with_g_low, 254)
}

/// Applies YCoCg-R variant 3 decorrelation to 32 [`Color565`] values using AVX512.
///
/// This variant places the g_low bit at position 0 (bottom bit) in the output format.
/// Output format: `((y << 11) | (co << 6) | (cg << 1) | g_low)`
///
/// This function is translated from compiler-generated assembly for optimal performance.
///
/// # Parameters
///
/// - `colors_raw`: A 512-bit register containing 32 packed [`Color565`] values
///
/// # Returns
///
/// A 512-bit register containing 32 decorrelated [`Color565`] values
///
/// # Safety
///
/// This function requires AVX512F and AVX512BW support. Caller must ensure the target CPU supports AVX512.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var3_avx512(colors_raw: __m512i) -> __m512i {
    // === Constants from assembly ===
    let mask_31 = _mm512_set1_epi16(31); // .LCPI24_2: mask for 5-bit values (0x001F)
    let mask_1 = _mm512_set1_epi32(0x00010001); // .LCPI24_3: pattern 1,1 for g_low

    // Load input colors (zmm0 = colors_raw)
    let mut colors = colors_raw;

    // Extract components through bit manipulation (following assembly exactly)
    // vpsrlw zmm1, zmm0, 11  (Red component)
    let mut red = _mm512_srli_epi16(colors, 11);

    // vpsrlw zmm2, zmm0, 6   (Green component)
    let mut green = _mm512_srli_epi16(colors, 6);

    // vpsrlw zmm3, zmm0, 5   (g_low component)
    let g_low = _mm512_srli_epi16(colors, 5);

    // Apply YCoCg-R variant 3 decorrelation algorithm
    // vpsubw zmm1, zmm1, zmm0  (Co = R - B, where B is from lower bits of zmm0)
    red = _mm512_sub_epi16(red, colors);

    // vpandq zmm1, zmm1, zmm4  (Co &= 31)
    red = _mm512_and_si512(red, mask_31);

    // vpsrlw zmm5, zmm1, 1     (Co >> 1)
    let co_half = _mm512_srli_epi16(red, 1);

    // vpsllw zmm1, zmm1, 6     (Co << 6 for final packing)
    let co_shifted = _mm512_slli_epi16(red, 6);

    // vpaddw zmm0, zmm5, zmm0  (t = B + (Co >> 1))
    colors = _mm512_add_epi16(co_half, colors);

    // vpsubw zmm2, zmm2, zmm0  (Cg = G - t)
    green = _mm512_sub_epi16(green, colors);

    // vpandq zmm2, zmm2, zmm4  (Cg &= 31)
    green = _mm512_and_si512(green, mask_31);

    // vpsrlw zmm4, zmm2, 1     (Cg >> 1)
    let cg_half = _mm512_srli_epi16(green, 1);

    // vpaddw zmm2, zmm2, zmm2  (Cg << 1 for final packing)
    let cg_shifted = _mm512_add_epi16(green, green);

    // vpaddw zmm0, zmm4, zmm0  (Y = t + (Cg >> 1))
    colors = _mm512_add_epi16(cg_half, colors);

    // vpsllw zmm0, zmm0, 11    (Y << 11 for final packing)
    let y_shifted = _mm512_slli_epi16(colors, 11);

    // Pack final result using ternary logic operations (from assembly)
    // vpternlogq zmm2, zmm0, zmm1, 254  (combine cg_shifted, y_shifted, co_shifted)
    let mut result = _mm512_ternarylogic_epi32(cg_shifted, y_shifted, co_shifted, 254);

    // vpternlogd zmm2, zmm3, dword ptr [rip + .LCPI24_3]{1to16}, 248  (add g_low)
    result = _mm512_ternarylogic_epi32(result, g_low, mask_1, 248);

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_565::Color565;
    use rstest::rstest;
    use std::is_x86_feature_detected;

    /// Test data for all decorrelation variants
    fn get_test_colors() -> [Color565; 32] {
        [
            Color565::from_rgb(255, 128, 64),  // Orange
            Color565::from_rgb(128, 255, 192), // Light green
            Color565::from_rgb(64, 128, 255),  // Light blue
            Color565::from_rgb(192, 64, 128),  // Pink
            Color565::from_raw(0x1234),        // Random value 1
            Color565::from_raw(0x5678),        // Random value 2
            Color565::from_raw(0x9ABC),        // Random value 3
            Color565::from_raw(0xDEF0),        // Random value 4
            Color565::from_raw(0x2468),        // Even bits
            Color565::from_raw(0x1357),        // Odd bits
            Color565::from_raw(0xACE0),        // Mixed pattern 1
            Color565::from_raw(0x1359),        // Mixed pattern 2
            Color565::from_raw(0x7531),        // Mixed pattern 3
            Color565::from_raw(0x9BDF),        // Mixed pattern 4
            Color565::from_raw(0x4682),        // Mixed pattern 5
            Color565::from_raw(0xCEA8),        // Mixed pattern 6
            Color565::from_rgb(255, 255, 255), // White
            Color565::from_rgb(0, 0, 0),       // Black
            Color565::from_rgb(255, 0, 0),     // Red
            Color565::from_rgb(0, 255, 0),     // Green
            Color565::from_rgb(0, 0, 255),     // Blue
            Color565::from_rgb(128, 128, 128), // Gray
            Color565::from_raw(0x0000),        // All zeros
            Color565::from_raw(0xFFFF),        // All ones
            Color565::from_raw(0xF800),        // Max red
            Color565::from_raw(0x07E0),        // Max green
            Color565::from_raw(0x001F),        // Max blue
            Color565::from_raw(0x8000),        // MSB only
            Color565::from_raw(0x0001),        // LSB only
            Color565::from_raw(0x5555),        // Alternating bits
            Color565::from_raw(0xAAAA),        // Alternating bits 2
            Color565::from_raw(0x3C78),        // Pattern
        ]
    }

    #[rstest]
    #[case(
        decorrelate_ycocg_r_var1_avx512,
        Color565::decorrelate_ycocg_r_var1_ptr,
        "variant 1"
    )]
    #[case(
        decorrelate_ycocg_r_var2_avx512,
        Color565::decorrelate_ycocg_r_var2_ptr,
        "variant 2"
    )]
    #[case(
        decorrelate_ycocg_r_var3_avx512,
        Color565::decorrelate_ycocg_r_var3_ptr,
        "variant 3"
    )]
    fn test_avx512_vs_reference_implementation(
        #[case] intrinsic_fn: unsafe fn(__m512i) -> __m512i,
        #[case] reference_fn: unsafe fn(*const Color565, *mut Color565, usize),
        #[case] variant_name: &str,
    ) {
        if !is_x86_feature_detected!("avx512f") || !is_x86_feature_detected!("avx512bw") {
            return;
        }

        let test_colors = get_test_colors();

        unsafe {
            let mut reference_results = [Color565::from_raw(0); 32];
            reference_fn(
                test_colors.as_ptr(),
                reference_results.as_mut_ptr(),
                test_colors.len(),
            );

            let input_reg = _mm512_loadu_si512(test_colors.as_ptr() as *const __m512i);
            let result_reg = intrinsic_fn(input_reg);
            let mut intrinsic_results = [Color565::from_raw(0); 32];
            _mm512_storeu_si512(intrinsic_results.as_mut_ptr() as *mut __m512i, result_reg);

            for (x, (&intrinsic_result, &reference_result)) in intrinsic_results
                .iter()
                .zip(reference_results.iter())
                .take(test_colors.len())
                .enumerate()
            {
                assert_eq!(
                    intrinsic_result, reference_result,
                    "AVX512 {variant_name} mismatch at index {x}: intrinsic={intrinsic_result:?}, reference={reference_result:?}"
                );
            }
        }
    }
}
