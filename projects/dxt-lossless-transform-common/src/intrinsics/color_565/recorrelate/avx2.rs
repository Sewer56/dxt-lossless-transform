//! These functions are converted from compiler-generated assembly
//! to be used as part of larger assembly routines.

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::color_565::Color565;

#[no_mangle]
pub fn test2(colors: &mut [Color565; 16]) {
    unsafe {
        Color565::recorrelate_ycocg_r_var3_ptr(colors.as_ptr(), colors.as_mut_ptr(), 16);
    }
}

/// Recorrelate a register of [`Color565`] values using an optimized YCoCg-R algorithm
///
/// Takes a `__m256i` register containing 16 [`Color565`] values and returns a register
/// with the colors recorrelated using an optimized YCoCg-R algorithm that operates
/// directly on 16-bit color values.
///
/// # Safety
///
/// Requires `avx2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var1_avx2(colors_raw: __m256i) -> __m256i {
    // === Constants from assembly ===
    let const_32 = _mm256_set1_epi16(32);
    let const_31 = _mm256_set1_epi16(31);
    let const_1984 = _mm256_set1_epi16(1984);
    let mask_15 = _mm256_set1_epi16(15);

    // vmovdqu ymm0, ymmword ptr [rdi] - input colors
    let mut colors = colors_raw; // ymm0

    // vpand ymm3, ymm0, ymmword ptr [rip + .LCPI18_0]
    let ymm3 = _mm256_and_si256(colors, const_32); // ymm3 = ymm0 & const_32

    // vpsrlw ymm4, ymm0, 1; vpand ymm4, ymm4, ymm5
    let mut ymm4 = _mm256_srli_epi16(colors, 1); // ymm4 = ymm0 >> 1
    ymm4 = _mm256_and_si256(ymm4, mask_15); // ymm4 = ymm4 & 15

    // vpsrlw ymm1, ymm0, 11; vpsubw ymm1, ymm1, ymm4
    let mut ymm1 = _mm256_srli_epi16(colors, 11); // ymm1 = ymm0 >> 11 (blue)
    ymm1 = _mm256_sub_epi16(ymm1, ymm4); // ymm1 = ymm1 - ymm4

    // vpsrlw ymm2, ymm0, 6
    let ymm2 = _mm256_srli_epi16(colors, 6); // ymm2 = ymm0 >> 6 (green)

    // vpaddw ymm4, ymm1, ymm0
    ymm4 = _mm256_add_epi16(ymm1, colors); // ymm4 = ymm1 + ymm0

    // vpsrlw ymm0, ymm0, 7; vpand ymm0, ymm0, ymm5
    colors = _mm256_srli_epi16(colors, 7); // ymm0 = ymm0 >> 7
    colors = _mm256_and_si256(colors, mask_15); // ymm0 = ymm0 & 15

    // vpsubw ymm0, ymm1, ymm0
    colors = _mm256_sub_epi16(ymm1, colors); // ymm0 = ymm1 - ymm0

    // vpand ymm1, ymm0, ymmword ptr [rip + .LCPI18_2]
    ymm1 = _mm256_and_si256(colors, const_31); // ymm1 = ymm0 & const_31

    // vpaddw ymm0, ymm0, ymm2; vpsllw ymm0, ymm0, 11
    colors = _mm256_add_epi16(colors, ymm2); // ymm0 = ymm0 + ymm2
    colors = _mm256_slli_epi16(colors, 11); // ymm0 = ymm0 << 11

    // vpor ymm0, ymm0, ymm3
    colors = _mm256_or_si256(colors, ymm3); // ymm0 = ymm0 | ymm3

    // vpor ymm0, ymm0, ymm1
    colors = _mm256_or_si256(colors, ymm1); // ymm0 = ymm0 | ymm1

    // vpsllw ymm1, ymm4, 6; vpand ymm1, ymm1, ymmword ptr [rip + .LCPI18_3]
    ymm1 = _mm256_slli_epi16(ymm4, 6); // ymm1 = ymm4 << 6
    ymm1 = _mm256_and_si256(ymm1, const_1984); // ymm1 = ymm1 & const_1984

    // vpor ymm0, ymm0, ymm1
    _mm256_or_si256(colors, ymm1) // return ymm0 | ymm1
}

/// Recorrelate a register of [`Color565`] values using YCoCg-R variant 2
///
/// Takes a `__m256i` register containing 16 [`Color565`] values and returns a register
/// with the colors recorrelated using YCoCg-R variant 2.
///
/// # Safety
///
/// Requires `avx2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var2_avx2(colors_raw: __m256i) -> __m256i {
    // === Constants from assembly ===
    let const_31 = _mm256_set1_epi16(31);
    let const_1984 = _mm256_set1_epi16(1984);
    let const_32 = _mm256_set1_epi16(32);
    let mask_15 = _mm256_set1_epi16(15);

    // vmovdqu ymm0, ymmword ptr [rdi] - input colors
    let mut colors = colors_raw; // ymm0

    // vpsrlw ymm3, ymm0, 1; vpand ymm3, ymm3, ymm4
    let mut ymm3 = _mm256_srli_epi16(colors, 1); // ymm3 = ymm0 >> 1
    ymm3 = _mm256_and_si256(ymm3, mask_15); // ymm3 = ymm3 & 15

    // vpsrlw ymm1, ymm0, 10; vpsubw ymm3, ymm1, ymm3
    let mut ymm1 = _mm256_srli_epi16(colors, 10); // ymm1 = ymm0 >> 10
    ymm3 = _mm256_sub_epi16(ymm1, ymm3); // ymm3 = ymm1 - ymm3

    // vpsrlw ymm2, ymm0, 5
    let ymm2 = _mm256_srli_epi16(colors, 5); // ymm2 = ymm0 >> 5

    // vpand ymm1, ymm1, ymmword ptr [rip + .LCPI18_3]
    ymm1 = _mm256_and_si256(ymm1, const_32); // ymm1 = ymm1 & const_32

    // vpaddw ymm5, ymm3, ymm0
    let ymm5 = _mm256_add_epi16(ymm3, colors); // ymm5 = ymm3 + ymm0

    // vpsrlw ymm0, ymm0, 6; vpand ymm0, ymm0, ymm4
    colors = _mm256_srli_epi16(colors, 6); // ymm0 = ymm0 >> 6
    colors = _mm256_and_si256(colors, mask_15); // ymm0 = ymm0 & 15

    // vpsubw ymm0, ymm3, ymm0
    colors = _mm256_sub_epi16(ymm3, colors); // ymm0 = ymm3 - ymm0

    // vpand ymm3, ymm0, ymmword ptr [rip + .LCPI18_1]
    ymm3 = _mm256_and_si256(colors, const_31); // ymm3 = ymm0 & const_31

    // vpaddw ymm0, ymm0, ymm2
    colors = _mm256_add_epi16(colors, ymm2); // ymm0 = ymm0 + ymm2

    // vpsllw ymm2, ymm5, 6; vpand ymm2, ymm2, ymmword ptr [rip + .LCPI18_2]
    let mut ymm2 = _mm256_slli_epi16(ymm5, 6); // ymm2 = ymm5 << 6
    ymm2 = _mm256_and_si256(ymm2, const_1984); // ymm2 = ymm2 & const_1984

    // vpsllw ymm0, ymm0, 11
    colors = _mm256_slli_epi16(colors, 11); // ymm0 = ymm0 << 11

    // vpor ymm0, ymm0, ymm1
    colors = _mm256_or_si256(colors, ymm1); // ymm0 = ymm0 | ymm1

    // vpor ymm0, ymm0, ymm3
    colors = _mm256_or_si256(colors, ymm3); // ymm0 = ymm0 | ymm3

    // vpor ymm0, ymm0, ymm2
    _mm256_or_si256(colors, ymm2) // return ymm0 | ymm2
}

/// Recorrelate a register of [`Color565`] values using YCoCg-R variant 3
///
/// Takes a `__m256i` register containing 16 [`Color565`] values and returns a register
/// with the colors recorrelated using YCoCg-R variant 3.
///
/// # Safety
///
/// Requires `avx2` target feature to be enabled.
/// The input register must contain valid [`Color565`] data.
///
/// [`Color565`]: crate::color_565::Color565
#[target_feature(enable = "avx2")]
#[inline]
pub unsafe fn recorrelate_ycocg_r_var3_avx2(colors_raw: __m256i) -> __m256i {
    // === Constants from assembly ===
    let const_31 = _mm256_set1_epi16(31);
    let const_1984 = _mm256_set1_epi16(1984);
    let const_32 = _mm256_set1_epi16(32);
    let mask_15 = _mm256_set1_epi16(15);

    // vmovdqu ymm0, ymmword ptr [rdi] - input colors
    let mut colors = colors_raw; // ymm0

    // vpsrlw ymm4, ymm0, 2; vpand ymm4, ymm4, ymm5
    let mut ymm4 = _mm256_srli_epi16(colors, 2); // ymm4 = ymm0 >> 2
    ymm4 = _mm256_and_si256(ymm4, mask_15); // ymm4 = ymm4 & 15

    // vpsrlw ymm1, ymm0, 11; vpsubw ymm1, ymm1, ymm4
    let mut ymm1 = _mm256_srli_epi16(colors, 11); // ymm1 = ymm0 >> 11 (blue)
    ymm1 = _mm256_sub_epi16(ymm1, ymm4); // ymm1 = ymm1 - ymm4

    // vpsrlw ymm2, ymm0, 6; vpsrlw ymm3, ymm0, 1
    let ymm2 = _mm256_srli_epi16(colors, 6); // ymm2 = ymm0 >> 6 (green)
    let mut ymm3 = _mm256_srli_epi16(colors, 1); // ymm3 = ymm0 >> 1

    // vpsrlw ymm4, ymm0, 7; vpsllw ymm0, ymm0, 5
    ymm4 = _mm256_srli_epi16(colors, 7); // ymm4 = ymm0 >> 7
    colors = _mm256_slli_epi16(colors, 5); // ymm0 = ymm0 << 5

    // vpand ymm0, ymm0, ymmword ptr [rip + .LCPI18_3]
    colors = _mm256_and_si256(colors, const_32); // ymm0 = ymm0 & const_32

    // vpand ymm4, ymm4, ymm5
    ymm4 = _mm256_and_si256(ymm4, mask_15); // ymm4 = ymm4 & 15

    // vpaddw ymm3, ymm1, ymm3
    ymm3 = _mm256_add_epi16(ymm1, ymm3); // ymm3 = ymm1 + ymm3

    // vpsubw ymm1, ymm1, ymm4
    ymm1 = _mm256_sub_epi16(ymm1, ymm4); // ymm1 = ymm1 - ymm4

    // vpand ymm4, ymm1, ymmword ptr [rip + .LCPI18_1]
    ymm4 = _mm256_and_si256(ymm1, const_31); // ymm4 = ymm1 & const_31

    // vpaddw ymm1, ymm1, ymm2
    ymm1 = _mm256_add_epi16(ymm1, ymm2); // ymm1 = ymm1 + ymm2

    // vpsllw ymm2, ymm3, 6; vpand ymm2, ymm2, ymmword ptr [rip + .LCPI18_2]
    let mut ymm2 = _mm256_slli_epi16(ymm3, 6); // ymm2 = ymm3 << 6
    ymm2 = _mm256_and_si256(ymm2, const_1984); // ymm2 = ymm2 & const_1984

    // vpsllw ymm1, ymm1, 11
    ymm1 = _mm256_slli_epi16(ymm1, 11); // ymm1 = ymm1 << 11

    // vpor ymm0, ymm1, ymm0
    colors = _mm256_or_si256(ymm1, colors); // ymm0 = ymm1 | ymm0

    // vpor ymm0, ymm0, ymm4
    colors = _mm256_or_si256(colors, ymm4); // ymm0 = ymm0 | ymm4

    // vpor ymm0, ymm0, ymm2
    _mm256_or_si256(colors, ymm2) // return ymm0 | ymm2
}
