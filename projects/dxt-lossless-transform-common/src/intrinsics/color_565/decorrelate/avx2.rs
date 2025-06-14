//! AVX2 intrinsics for YCoCg-R decorrelation on [`Color565`] values.
//!
//! This module provides AVX2-optimized functions for performing YCoCg-R decorrelation
//! transformations on packed [`Color565`] data in 256-bit registers.
//!
//! # Functions
//!
//! - [`decorrelate_ycocg_r_var1_avx2`] - Applies YCoCg-R variant 1 decorrelation (g_low at bit 5)
//! - [`decorrelate_ycocg_r_var2_avx2`] - Applies YCoCg-R variant 2 decorrelation (g_low at bit 15)  
//! - [`decorrelate_ycocg_r_var3_avx2`] - Applies YCoCg-R variant 3 decorrelation (g_low at bit 0)
//!
//! Each function processes 16 [`Color565`] values simultaneously using AVX2 instructions.
//!
//! [`Color565`]: crate::color_565::Color565

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Applies YCoCg-R variant 1 decorrelation to 16 [`Color565`] values using AVX2.
///
/// This variant places the g_low bit at position 5 in the output format.
/// Output format: `((y << 11) | (co << 6) | (g_low << 5) | cg)`
///
/// # Parameters
///
/// - `colors_raw`: A 256-bit register containing 16 packed [`Color565`] values
///
/// # Returns
///
/// A 256-bit register containing 16 decorrelated [`Color565`] values
///
/// # Safety
///
/// This function requires AVX2 support. Caller must ensure the target CPU supports AVX2.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var1_avx2(colors_raw: __m256i) -> __m256i {
    // Constants
    let mask_5bit = _mm256_set1_epi16(0x1F); // 31 in decimal, 0b11111 in binary
    let mask_1bit = _mm256_set1_epi16(0x01); // 1 in decimal

    // Extract RGB components
    let r = _mm256_srli_epi16(colors_raw, 11); // Red: bits 15-11
    let g = _mm256_srli_epi16(colors_raw, 6); // Green: bits 10-6 (upper 5 bits)
    let g_low = _mm256_and_si256(_mm256_srli_epi16(colors_raw, 5), mask_1bit); // Green low bit
    let b = _mm256_and_si256(colors_raw, mask_5bit); // Blue: bits 4-0

    // Apply YCoCg-R forward transform
    // Step 1: Co = (R - B) & 0x1F
    let co = _mm256_and_si256(_mm256_sub_epi16(r, b), mask_5bit);

    // Step 2: t = (B + (Co >> 1)) & 0x1F
    let t = _mm256_and_si256(_mm256_add_epi16(b, _mm256_srli_epi16(co, 1)), mask_5bit);

    // Step 3: Cg = (G - t) & 0x1F
    let cg = _mm256_and_si256(_mm256_sub_epi16(g, t), mask_5bit);

    // Step 4: Y = (t + (Cg >> 1)) & 0x1F
    let y = _mm256_and_si256(_mm256_add_epi16(t, _mm256_srli_epi16(cg, 1)), mask_5bit);

    // Pack into Color565 format: ((y << 11) | (co << 6) | (g_low << 5) | cg)
    let y_shifted = _mm256_slli_epi16(y, 11);
    let co_shifted = _mm256_slli_epi16(co, 6);
    let g_low_shifted = _mm256_slli_epi16(g_low, 5);

    _mm256_or_si256(
        _mm256_or_si256(y_shifted, co_shifted),
        _mm256_or_si256(g_low_shifted, cg),
    )
}

/// Applies YCoCg-R variant 2 decorrelation to 16 [`Color565`] values using AVX2.
///
/// This variant places the g_low bit at position 15 (top bit) in the output format.
/// Output format: `((g_low << 15) | (y << 10) | (co << 5) | cg)`
///
/// # Parameters
///
/// - `colors_raw`: A 256-bit register containing 16 packed [`Color565`] values
///
/// # Returns
///
/// A 256-bit register containing 16 decorrelated [`Color565`] values
///
/// # Safety
///
/// This function requires AVX2 support. Caller must ensure the target CPU supports AVX2.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var2_avx2(colors_raw: __m256i) -> __m256i {
    // Constants
    let mask_5bit = _mm256_set1_epi16(0x1F); // 31 in decimal, 0b11111 in binary
    let mask_1bit = _mm256_set1_epi16(0x01); // 1 in decimal

    // Extract RGB components
    let r = _mm256_srli_epi16(colors_raw, 11); // Red: bits 15-11
    let g = _mm256_srli_epi16(colors_raw, 6); // Green: bits 10-6 (upper 5 bits)
    let g_low = _mm256_and_si256(_mm256_srli_epi16(colors_raw, 5), mask_1bit); // Green low bit
    let b = _mm256_and_si256(colors_raw, mask_5bit); // Blue: bits 4-0

    // Apply YCoCg-R forward transform
    // Step 1: Co = (R - B) & 0x1F
    let co = _mm256_and_si256(_mm256_sub_epi16(r, b), mask_5bit);

    // Step 2: t = (B + (Co >> 1)) & 0x1F
    let t = _mm256_and_si256(_mm256_add_epi16(b, _mm256_srli_epi16(co, 1)), mask_5bit);

    // Step 3: Cg = (G - t) & 0x1F
    let cg = _mm256_and_si256(_mm256_sub_epi16(g, t), mask_5bit);

    // Step 4: Y = (t + (Cg >> 1)) & 0x1F
    let y = _mm256_and_si256(_mm256_add_epi16(t, _mm256_srli_epi16(cg, 1)), mask_5bit);

    // Pack into Color565 format: ((g_low << 15) | (y << 10) | (co << 5) | cg)
    let g_low_shifted = _mm256_slli_epi16(g_low, 15);
    let y_shifted = _mm256_slli_epi16(y, 10);
    let co_shifted = _mm256_slli_epi16(co, 5);

    _mm256_or_si256(
        _mm256_or_si256(g_low_shifted, y_shifted),
        _mm256_or_si256(co_shifted, cg),
    )
}

/// Applies YCoCg-R variant 3 decorrelation to 16 [`Color565`] values using AVX2.
///
/// This variant places the g_low bit at position 0 (bottom bit) in the output format.
/// Output format: `((y << 11) | (co << 6) | (cg << 1) | g_low)`
///
/// # Parameters
///
/// - `colors_raw`: A 256-bit register containing 16 packed [`Color565`] values
///
/// # Returns
///
/// A 256-bit register containing 16 decorrelated [`Color565`] values
///
/// # Safety
///
/// This function requires AVX2 support. Caller must ensure the target CPU supports AVX2.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn decorrelate_ycocg_r_var3_avx2(colors_raw: __m256i) -> __m256i {
    // Constants
    let mask_5bit = _mm256_set1_epi16(0x1F); // 31 in decimal, 0b11111 in binary
    let mask_1bit = _mm256_set1_epi16(0x01); // 1 in decimal

    // Extract RGB components
    let r = _mm256_srli_epi16(colors_raw, 11); // Red: bits 15-11
    let g = _mm256_srli_epi16(colors_raw, 6); // Green: bits 10-6 (upper 5 bits)
    let g_low = _mm256_and_si256(_mm256_srli_epi16(colors_raw, 5), mask_1bit); // Green low bit
    let b = _mm256_and_si256(colors_raw, mask_5bit); // Blue: bits 4-0

    // Apply YCoCg-R forward transform
    // Step 1: Co = (R - B) & 0x1F
    let co = _mm256_and_si256(_mm256_sub_epi16(r, b), mask_5bit);

    // Step 2: t = (B + (Co >> 1)) & 0x1F
    let t = _mm256_and_si256(_mm256_add_epi16(b, _mm256_srli_epi16(co, 1)), mask_5bit);

    // Step 3: Cg = (G - t) & 0x1F
    let cg = _mm256_and_si256(_mm256_sub_epi16(g, t), mask_5bit);

    // Step 4: Y = (t + (Cg >> 1)) & 0x1F
    let y = _mm256_and_si256(_mm256_add_epi16(t, _mm256_srli_epi16(cg, 1)), mask_5bit);

    // Pack into Color565 format: ((y << 11) | (co << 6) | (cg << 1) | g_low)
    let y_shifted = _mm256_slli_epi16(y, 11);
    let co_shifted = _mm256_slli_epi16(co, 6);
    let cg_shifted = _mm256_slli_epi16(cg, 1);

    _mm256_or_si256(
        _mm256_or_si256(y_shifted, co_shifted),
        _mm256_or_si256(cg_shifted, g_low),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color_565::Color565;
    use rstest::rstest;

    /// Test data for all decorrelation variants
    fn get_test_colors() -> [Color565; 16] {
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
        ]
    }

    #[rstest]
    #[case(
        decorrelate_ycocg_r_var1_avx2,
        Color565::decorrelate_ycocg_r_var1_ptr,
        "variant 1"
    )]
    #[case(
        decorrelate_ycocg_r_var2_avx2,
        Color565::decorrelate_ycocg_r_var2_ptr,
        "variant 2"
    )]
    #[case(
        decorrelate_ycocg_r_var3_avx2,
        Color565::decorrelate_ycocg_r_var3_ptr,
        "variant 3"
    )]
    fn test_avx2_vs_reference_implementation(
        #[case] intrinsic_fn: unsafe fn(__m256i) -> __m256i,
        #[case] reference_fn: unsafe fn(*const Color565, *mut Color565, usize),
        #[case] variant_name: &str,
    ) {
        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let test_colors = get_test_colors();

        unsafe {
            let mut reference_results = [Color565::from_raw(0); 16];
            reference_fn(
                test_colors.as_ptr(),
                reference_results.as_mut_ptr(),
                test_colors.len(),
            );

            let input_reg = _mm256_loadu_si256(test_colors.as_ptr() as *const __m256i);
            let result_reg = intrinsic_fn(input_reg);
            let mut intrinsic_results = [Color565::from_raw(0); 16];
            _mm256_storeu_si256(intrinsic_results.as_mut_ptr() as *mut __m256i, result_reg);

            for (x, (&intrinsic_result, &reference_result)) in intrinsic_results
                .iter()
                .zip(reference_results.iter())
                .take(test_colors.len())
                .enumerate()
            {
                assert_eq!(
                    intrinsic_result, reference_result,
                    "AVX2 {variant_name} mismatch at index {x}: intrinsic={intrinsic_result:?}, reference={reference_result:?}"
                );
            }
        }
    }
}
