//! These functions are converted from compiler-generated assembly
//! to be used as part of larger assembly routines.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Recorrelate a register of [`Color565`] values using an optimized YCoCg-R algorithm
///
/// Takes a `__m128i` register containing 8 [`Color565`] values and returns a register
/// with the colors recorrelated using an optimized YCoCg-R algorithm that operates
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
pub unsafe fn recorrelate_ycocg_r_var1_sse2(colors_raw: __m128i) -> __m128i {
    // Constants
    let mask_15 = _mm_set1_epi16(15);
    let mask_32 = _mm_set1_epi16(32);
    let mask_31 = _mm_set1_epi16(31);
    let mask_1984 = _mm_set1_epi16(1984);

    // Extract components through bit manipulation
    let xmm3 = _mm_and_si128(colors_raw, mask_32); // Blue component mask
    let xmm4 = _mm_and_si128(_mm_srli_epi16(colors_raw, 1), mask_15);
    let xmm0 = _mm_srli_epi16(colors_raw, 11); // Red component
    let xmm2 = _mm_srli_epi16(colors_raw, 6); // Green component

    // YCoCg-R variant 1 algorithm
    let xmm0 = _mm_sub_epi16(xmm0, xmm4);
    let xmm4 = _mm_add_epi16(xmm0, colors_raw);
    let xmm1 = _mm_and_si128(_mm_srli_epi16(colors_raw, 7), mask_15);
    let xmm0 = _mm_sub_epi16(xmm0, xmm1);
    let xmm5 = _mm_and_si128(xmm0, mask_31);
    let xmm0 = _mm_add_epi16(xmm0, xmm2);
    let xmm0 = _mm_slli_epi16(xmm0, 11);
    let xmm4 = _mm_and_si128(_mm_slli_epi16(xmm4, 6), mask_1984);

    // Combine components
    let result = _mm_or_si128(xmm0, xmm3);
    let result = _mm_or_si128(result, xmm5);
    _mm_or_si128(result, xmm4)
}

/// Recorrelate a register of [`Color565`] values using YCoCg-R variant 2
///
/// Takes a `__m128i` register containing 8 [`Color565`] values and returns a register
/// with the colors recorrelated using YCoCg-R variant 2.
///
/// # Safety
///
/// Requires `sse2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var2_sse2(colors_raw: __m128i) -> __m128i {
    // Constants
    let mask_15 = _mm_set1_epi16(15);
    let mask_31 = _mm_set1_epi16(31);
    let mask_32 = _mm_set1_epi16(32);
    let mask_1984 = _mm_set1_epi16(1984);

    // Extract components through bit manipulation
    let xmm3 = _mm_and_si128(_mm_srli_epi16(colors_raw, 1), mask_15);
    let xmm0 = _mm_srli_epi16(colors_raw, 10);
    let xmm2 = _mm_srli_epi16(colors_raw, 5);
    let xmm5 = xmm0;
    let xmm0 = _mm_and_si128(xmm0, mask_32);

    // YCoCg-R variant 2 algorithm
    let xmm5 = _mm_sub_epi16(xmm5, xmm3);
    let xmm3 = _mm_add_epi16(xmm5, colors_raw);
    let xmm1 = _mm_and_si128(_mm_srli_epi16(colors_raw, 6), mask_15);
    let xmm3 = _mm_and_si128(_mm_slli_epi16(xmm3, 6), mask_1984);
    let xmm5 = _mm_sub_epi16(xmm5, xmm1);
    let xmm4 = _mm_and_si128(xmm5, mask_31);
    let xmm5 = _mm_add_epi16(xmm5, xmm2);
    let xmm5 = _mm_slli_epi16(xmm5, 11);

    // Combine components
    let result = _mm_or_si128(xmm0, xmm5);
    let result = _mm_or_si128(result, xmm4);
    _mm_or_si128(result, xmm3)
}

/// Recorrelate a register of [`Color565`] values using YCoCg-R variant 3
///
/// Takes a `__m128i` register containing 8 [`Color565`] values and returns a register
/// with the colors recorrelated using YCoCg-R variant 3.
///
/// # Safety
///
/// Requires `sse2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "sse2")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var3_sse2(colors_raw: __m128i) -> __m128i {
    // Constants
    let mask_15 = _mm_set1_epi16(15);
    let mask_31 = _mm_set1_epi16(31);
    let mask_1984 = _mm_set1_epi16(1984);
    let mask_32 = _mm_set1_epi16(32);

    // Extract and process components
    let xmm4 = _mm_and_si128(_mm_srli_epi16(colors_raw, 2), mask_15);
    let xmm6 = _mm_and_si128(_mm_srli_epi16(colors_raw, 7), mask_15);
    let xmm1 = _mm_srli_epi16(colors_raw, 11); // Red component
    let xmm3 = _mm_srli_epi16(colors_raw, 6); // Green component
    let xmm0 = _mm_and_si128(_mm_slli_epi16(colors_raw, 5), mask_32); // Blue component shifted
    let xmm2 = _mm_srli_epi16(colors_raw, 1);

    // YCoCg-R variant 3 algorithm
    let xmm1 = _mm_sub_epi16(xmm1, xmm4);
    let xmm2 = _mm_add_epi16(xmm2, xmm1);
    let xmm1 = _mm_sub_epi16(xmm1, xmm6);
    let xmm2 = _mm_and_si128(_mm_slli_epi16(xmm2, 6), mask_1984);
    let xmm5 = _mm_and_si128(xmm1, mask_31);
    let xmm1 = _mm_add_epi16(xmm1, xmm3);
    let xmm1 = _mm_slli_epi16(xmm1, 11);

    // Combine components
    let result = _mm_or_si128(xmm0, xmm1);
    let result = _mm_or_si128(result, xmm5);
    _mm_or_si128(result, xmm2)
}
