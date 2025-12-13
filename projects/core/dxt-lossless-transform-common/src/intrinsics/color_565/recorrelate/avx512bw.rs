//! These functions are converted from compiler-generated assembly
//! to be used as part of larger assembly routines.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// Recorrelate a register of [`Color565`] values using an optimized YCoCg-R algorithm
///
/// Takes a `__m512i` register containing 32 [`Color565`] values and returns a register
/// with the colors recorrelated using an optimized YCoCg-R algorithm that operates
/// directly on 16-bit color values.
///
/// # Safety
///
/// Requires `avx512f` and `avx512bw` target features to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var1_avx512bw(colors_raw: __m512i) -> __m512i {
    // === Constants from assembly ===
    let mask_15 = _mm512_set1_epi16(15); // .LCPI18_4: mask for 4-bit values (0x000F)
    let const_1984 = _mm512_set1_epi32(0x07C007C0); // .LCPI18_5: pattern 1984,1984
    let const_32 = _mm512_set1_epi32(0x00200020); // .LCPI18_6: pattern 32,32
    let const_31 = _mm512_set1_epi32(0x001F001F); // .LCPI18_7: pattern 31,31

    // Load input colors (zmm0 = colors_raw)
    let colors = colors_raw;

    // Right shift operations (vpsrlw)
    let shifted_1 = _mm512_srli_epi16(colors, 1); // zmm3 = zmm0 >> 1
    let mut blue_shifted = _mm512_srli_epi16(colors, 11); // zmm1 = zmm0 >> 11 (blue component)
    let shifted_7 = _mm512_srli_epi16(colors, 7); // zmm5 = zmm0 >> 7
    let mut green_shifted = _mm512_srli_epi16(colors, 6); // zmm2 = zmm0 >> 6 (green component)

    // Apply masks (vpandq with mask_15)
    let masked_1 = _mm512_and_si512(shifted_1, mask_15); // zmm3 = zmm3 & 15
    let masked_7 = _mm512_and_si512(shifted_7, mask_15); // zmm4 = zmm5 & 15

    // Subtract operations (vpsubw)
    blue_shifted = _mm512_sub_epi16(blue_shifted, masked_1); // zmm1 = zmm1 - zmm3
    let intermediate = _mm512_add_epi16(blue_shifted, colors); // zmm3 = zmm1 + zmm0
    blue_shifted = _mm512_sub_epi16(blue_shifted, masked_7); // zmm1 = zmm1 - zmm4
    green_shifted = _mm512_add_epi16(blue_shifted, green_shifted); // zmm2 = zmm1 + zmm2

    // Left shift operations (vpsllw)
    let shifted_6 = _mm512_slli_epi16(intermediate, 6); // zmm3 = zmm3 << 6
    let shifted_11 = _mm512_slli_epi16(green_shifted, 11); // zmm2 = zmm2 << 11

    // First ternary logic operation (vpternlogd with 1984 pattern, opcode 236)
    let mut result = _mm512_ternarylogic_epi32(shifted_6, shifted_11, const_1984, 236);

    // Second ternary logic operation (vpternlogd with 32 pattern, opcode 248)
    result = _mm512_ternarylogic_epi32(result, colors, const_32, 248);

    // Third ternary logic operation (vpternlogd with 31 pattern, opcode 248)
    _mm512_ternarylogic_epi32(result, blue_shifted, const_31, 248)
}

/// Recorrelate a register of [`Color565`] values using YCoCg-R variant 2
///
/// Takes a `__m512i` register containing 32 [`Color565`] values and returns a register
/// with the colors recorrelated using YCoCg-R variant 2.
///
/// # Safety
///
/// Requires `avx512f` and `avx512bw` target features to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var2_avx512bw(colors_raw: __m512i) -> __m512i {
    // === Constants from assembly ===
    let mask_15 = _mm512_set1_epi16(15); // .LCPI18_4: mask for 4-bit values (0x000F)
    let const_1984 = _mm512_set1_epi32(0x07C007C0); // .LCPI18_5: pattern 1984,1984
    let const_32 = _mm512_set1_epi32(0x00200020); // .LCPI18_6: pattern 32,32
    let const_31 = _mm512_set1_epi32(0x001F001F); // .LCPI18_7: pattern 31,31

    // Load input colors (zmm0 = colors_raw)
    let mut colors = colors_raw;

    // Right shift operations (vpsrlw)
    let mut shifted_1 = _mm512_srli_epi16(colors, 1); // zmm3 = zmm0 >> 1
    let shifted_10 = _mm512_srli_epi16(colors, 10); // zmm1 = zmm0 >> 10
    let mut shifted_5 = _mm512_srli_epi16(colors, 5); // zmm2 = zmm0 >> 5

    // Apply mask to shifted_1 (vpandq zmm3, zmm3, zmm4)
    shifted_1 = _mm512_and_si512(shifted_1, mask_15); // zmm3 = zmm3 & 15

    // Subtract operations (vpsubw zmm3, zmm1, zmm3)
    shifted_1 = _mm512_sub_epi16(shifted_10, shifted_1); // zmm3 = zmm1 - zmm3

    // Add operation (vpaddw zmm5, zmm3, zmm0)
    let intermediate = _mm512_add_epi16(shifted_1, colors); // zmm5 = zmm3 + zmm0

    // Right shift colors by 6 and mask (vpsrlw zmm0, zmm0, 6; vpandq zmm0, zmm0, zmm4)
    colors = _mm512_srli_epi16(colors, 6); // zmm0 = zmm0 >> 6
    colors = _mm512_and_si512(colors, mask_15); // zmm0 = zmm0 & 15

    // Subtract operation (vpsubw zmm0, zmm3, zmm0)
    colors = _mm512_sub_epi16(shifted_1, colors); // zmm0 = zmm3 - zmm0

    // Left shift intermediate by 6 (vpsllw zmm3, zmm5, 6)
    let shifted_6 = _mm512_slli_epi16(intermediate, 6); // zmm3 = zmm5 << 6

    // Add and left shift operations (vpaddw zmm2, zmm0, zmm2; vpsllw zmm2, zmm2, 11)
    shifted_5 = _mm512_add_epi16(colors, shifted_5); // zmm2 = zmm0 + zmm2
    let shifted_11 = _mm512_slli_epi16(shifted_5, 11); // zmm2 = zmm2 << 11

    // First ternary logic operation (vpternlogd with 1984 pattern, opcode 236)
    let mut result = _mm512_ternarylogic_epi32(shifted_6, shifted_11, const_1984, 236);

    // Second ternary logic operation (vpternlogd with 32 pattern, opcode 248)
    result = _mm512_ternarylogic_epi32(result, shifted_10, const_32, 248);

    // Third ternary logic operation (vpternlogd with 31 pattern, opcode 248)
    _mm512_ternarylogic_epi32(result, colors, const_31, 248)
}

/// Recorrelate a register of [`Color565`] values using YCoCg-R variant 3
///
/// Takes a `__m512i` register containing 32 [`Color565`] values and returns a register
/// with the colors recorrelated using YCoCg-R variant 3.
///
/// # Safety
///
/// Requires `avx512f` and `avx512bw` target features to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var3_avx512bw(colors_raw: __m512i) -> __m512i {
    // === Constants from assembly ===
    let mask_15 = _mm512_set1_epi16(15); // .LCPI18_4: mask for 4-bit values (0x000F)
    let const_1984 = _mm512_set1_epi32(0x07C007C0); // .LCPI18_5: pattern 1984,1984
    let const_32 = _mm512_set1_epi32(0x00200020); // .LCPI18_6: pattern 32,32
    let const_31 = _mm512_set1_epi32(0x001F001F); // .LCPI18_7: pattern 31,31

    // Load input colors (zmm0 = colors_raw)
    let mut colors = colors_raw;

    // Right shift operations (vpsrlw)
    let mut shifted_2 = _mm512_srli_epi16(colors, 2); // zmm4 = zmm0 >> 2
    let mut blue_shifted = _mm512_srli_epi16(colors, 11); // zmm1 = zmm0 >> 11 (blue component)
    let shifted_7 = _mm512_srli_epi16(colors, 7); // zmm6 = zmm0 >> 7
    let mut shifted_1 = _mm512_srli_epi16(colors, 1); // zmm3 = zmm0 >> 1
    let mut green_shifted = _mm512_srli_epi16(colors, 6); // zmm2 = zmm0 >> 6 (green component)

    // Early left shift by 5 (vpsllw zmm0, zmm0, 5)
    colors = _mm512_slli_epi16(colors, 5); // zmm0 = zmm0 << 5

    // Apply mask to shifted_2 (vpandq zmm4, zmm4, zmm5)
    shifted_2 = _mm512_and_si512(shifted_2, mask_15); // zmm4 = zmm4 & 15

    // Subtract operation (vpsubw zmm1, zmm1, zmm4)
    blue_shifted = _mm512_sub_epi16(blue_shifted, shifted_2); // zmm1 = zmm1 - zmm4

    // Apply mask to shifted_7 and reuse shifted_2 variable (vpandq zmm4, zmm6, zmm5)
    shifted_2 = _mm512_and_si512(shifted_7, mask_15); // zmm4 = zmm6 & 15

    // Add operation (vpaddw zmm3, zmm1, zmm3)
    shifted_1 = _mm512_add_epi16(blue_shifted, shifted_1); // zmm3 = zmm1 + zmm3

    // Subtract operation (vpsubw zmm1, zmm1, zmm4)
    blue_shifted = _mm512_sub_epi16(blue_shifted, shifted_2); // zmm1 = zmm1 - zmm4

    // Add operation (vpaddw zmm2, zmm1, zmm2)
    green_shifted = _mm512_add_epi16(blue_shifted, green_shifted); // zmm2 = zmm1 + zmm2

    // Left shift operations (vpsllw)
    let shifted_6 = _mm512_slli_epi16(shifted_1, 6); // zmm3 = zmm3 << 6
    let shifted_11 = _mm512_slli_epi16(green_shifted, 11); // zmm2 = zmm2 << 11

    // First ternary logic operation (vpternlogd with 1984 pattern, opcode 236)
    let combined_result = _mm512_ternarylogic_epi32(shifted_6, shifted_11, const_1984, 236);

    // Second ternary logic operation (vpternlogd with 32 pattern, opcode 236)
    colors = _mm512_ternarylogic_epi32(colors, combined_result, const_32, 236);

    // Third ternary logic operation (vpternlogd with 31 pattern, opcode 248)
    _mm512_ternarylogic_epi32(colors, blue_shifted, const_31, 248)
}
