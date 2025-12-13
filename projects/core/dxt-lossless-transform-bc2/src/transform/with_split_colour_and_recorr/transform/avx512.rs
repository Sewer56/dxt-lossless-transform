use crate::transform::with_split_colour_and_recorr::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx512bw::{
    decorrelate_ycocg_r_var1_avx512bw, decorrelate_ycocg_r_var2_avx512bw,
    decorrelate_ycocg_r_var3_avx512bw,
};

const PERM_ALPHA_BYTES: [i8; 8] = [0, 2, 4, 6, 8, 10, 12, 14]; // For vpermt2q to gather alpha values

// Constant data for permutation masks
const PERM_COLORS_BYTES: [i8; 16] = [
    0, 4, 8, 12, 2, 6, 10, 14, // + 16 below
    16, 20, 24, 28, 18, 22, 26, 30,
]; // For vpermt2d to gather color values
const PERM_INDICES_BYTES: [i8; 16] = [
    1, 5, 9, 13, 3, 7, 11, 15, // +16 below
    17, 21, 25, 29, 19, 23, 27, 31,
]; // For vpermt2d to gather index values

// Constant data for permutation masks
// Pre-compute epi16 permutation vectors that directly extract colour0/colour1 words.
// Each entry selects a 16-bit lane from either `colors_0` (0-31) or `colors_1` (32-63).
// colour0 = low 16-bits  (even lanes), colour1 = high 16-bits (odd lanes).
#[allow(clippy::identity_op)]
const PERM_COLOR0_EPI16: [i16; 32] = [
    0,
    2,
    4,
    6,
    8,
    10,
    12,
    14,
    16,
    18,
    20,
    22,
    24,
    26,
    28,
    30,
    0 + 32,
    2 + 32,
    4 + 32,
    6 + 32,
    8 + 32,
    10 + 32,
    12 + 32,
    14 + 32,
    16 + 32,
    18 + 32,
    20 + 32,
    22 + 32,
    24 + 32,
    26 + 32,
    28 + 32,
    30 + 32,
];
const PERM_COLOR1_EPI16: [i16; 32] = [
    1,
    3,
    5,
    7,
    9,
    11,
    13,
    15,
    17,
    19,
    21,
    23,
    25,
    27,
    29,
    31,
    1 + 32,
    3 + 32,
    5 + 32,
    7 + 32,
    9 + 32,
    11 + 32,
    13 + 32,
    15 + 32,
    17 + 32,
    19 + 32,
    21 + 32,
    23 + 32,
    25 + 32,
    27 + 32,
    29 + 32,
    31 + 32,
];

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn transform_impl<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alpha_out: *mut u64,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    // Process 32 BC2 blocks at a time = 512 bytes
    let num_iterations = block_count / 32 * 32; // 32 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // 16 bytes per block

    // Load permutation patterns
    let perm_alpha = _mm512_cvtepi8_epi64(_mm_loadl_epi64(PERM_ALPHA_BYTES.as_ptr() as *const _));
    let perm_colors = _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_COLORS_BYTES.as_ptr() as *const _));
    let perm_indices =
        _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_INDICES_BYTES.as_ptr() as *const _));

    let perm_color0_epi16 = _mm512_loadu_si512(PERM_COLOR0_EPI16.as_ptr() as *const __m512i);
    let perm_color1_epi16 = _mm512_loadu_si512(PERM_COLOR1_EPI16.as_ptr() as *const __m512i);

    while input_ptr < input_end {
        // Load 512 bytes (32 blocks)
        let blocks_0 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let blocks_1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        let blocks_2 = _mm512_loadu_si512(input_ptr.add(128) as *const __m512i);
        let blocks_3 = _mm512_loadu_si512(input_ptr.add(192) as *const __m512i);
        let blocks_4 = _mm512_loadu_si512(input_ptr.add(256) as *const __m512i);
        let blocks_5 = _mm512_loadu_si512(input_ptr.add(320) as *const __m512i);
        let blocks_6 = _mm512_loadu_si512(input_ptr.add(384) as *const __m512i);
        let blocks_7 = _mm512_loadu_si512(input_ptr.add(448) as *const __m512i);

        // Update input pointer
        input_ptr = input_ptr.add(512);

        // Filter out the alphas only using vpermt2q
        let alphas_0 = _mm512_permutex2var_epi64(blocks_0, perm_alpha, blocks_1);
        let alphas_1 = _mm512_permutex2var_epi64(blocks_2, perm_alpha, blocks_3);
        let alphas_2 = _mm512_permutex2var_epi64(blocks_4, perm_alpha, blocks_5);
        let alphas_3 = _mm512_permutex2var_epi64(blocks_6, perm_alpha, blocks_7);

        // Lift out colours and indices only
        let colours_indices_only_b0 = _mm512_unpackhi_epi64(blocks_0, blocks_1);
        let colours_indices_only_b1 = _mm512_unpackhi_epi64(blocks_2, blocks_3);
        let colours_indices_only_b2 = _mm512_unpackhi_epi64(blocks_4, blocks_5);
        let colours_indices_only_b3 = _mm512_unpackhi_epi64(blocks_6, blocks_7);

        // Permute to lift out the indices only.
        let indices_only_0 = _mm512_permutex2var_epi32(
            colours_indices_only_b0,
            perm_indices,
            colours_indices_only_b1,
        ); // indices
        let indices_only_1 = _mm512_permutex2var_epi32(
            colours_indices_only_b2,
            perm_indices,
            colours_indices_only_b3,
        ); // indices

        // First separate into colours only registers.
        let colours_only_0 = _mm512_permutex2var_epi32(
            colours_indices_only_b0,
            perm_colors,
            colours_indices_only_b1,
        ); // colours

        let colours_only_1 = _mm512_permutex2var_epi32(
            colours_indices_only_b2,
            perm_colors,
            colours_indices_only_b3,
        ); // colours

        // And now do the fancy x2 permute to split the colours into color0 and color1.
        let color0_only =
            _mm512_permutex2var_epi16(colours_only_0, perm_color0_epi16, colours_only_1);
        let color1_only =
            _mm512_permutex2var_epi16(colours_only_0, perm_color1_epi16, colours_only_1);

        // Apply YCoCg-R decorrelation to color0 and color1
        let decorr_color0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx512bw(color0_only),
            2 => decorrelate_ycocg_r_var2_avx512bw(color0_only),
            3 => decorrelate_ycocg_r_var3_avx512bw(color0_only),
            _ => unreachable_unchecked(),
        };
        let decorr_color1 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx512bw(color1_only),
            2 => decorrelate_ycocg_r_var2_avx512bw(color1_only),
            3 => decorrelate_ycocg_r_var3_avx512bw(color1_only),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm512_storeu_si512(alpha_out as *mut __m512i, alphas_0); // alphas 0
        _mm512_storeu_si512(alpha_out.add(8) as *mut __m512i, alphas_1); // alphas 1 (8 u64s = 64 bytes)
        _mm512_storeu_si512(alpha_out.add(16) as *mut __m512i, alphas_2); // alphas 2 (16 u64s = 128 bytes)
        _mm512_storeu_si512(alpha_out.add(24) as *mut __m512i, alphas_3); // alphas 3 (24 u64s = 192 bytes)

        _mm512_storeu_si512(color0_out as *mut __m512i, decorr_color0); // decorrelated colors
        _mm512_storeu_si512(color1_out as *mut __m512i, decorr_color1); // decorrelated colors

        _mm512_storeu_si512(indices_out as *mut __m512i, indices_only_0); // indices
        _mm512_storeu_si512(indices_out.add(16) as *mut __m512i, indices_only_1); // indices (16 u32s = 64 bytes)

        // Update pointers
        alpha_out = alpha_out.add(32); // 32 u64s = 256 bytes
        color0_out = color0_out.add(32); // 32 u16s = 64 bytes
        color1_out = color1_out.add(32); // 32 u16s = 64 bytes
        indices_out = indices_out.add(32); // 32 u32s = 128 bytes
    }

    // Handle remaining blocks
    let remaining_blocks = block_count % 32;
    if remaining_blocks > 0 {
        let variant_enum = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::transform_with_split_colour_and_recorr(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            remaining_blocks,
            variant_enum,
        );
    }
}

// Assembly wrapper functions for variant-specific optimizations
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    transform_impl::<1>(
        input_ptr,
        alpha_out,
        color0_out,
        color1_out,
        indices_out,
        block_count,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    transform_impl::<2>(
        input_ptr,
        alpha_out,
        color0_out,
        color1_out,
        indices_out,
        block_count,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
pub(crate) unsafe fn transform_decorr_var3(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    transform_impl::<3>(
        input_ptr,
        alpha_out,
        color0_out,
        color1_out,
        indices_out,
        block_count,
    )
}

// Runtime dispatch helper used by parent module
#[inline(always)]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => transform_decorr_var1(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            block_count,
        ),
        YCoCgVariant::Variant2 => transform_decorr_var2(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            block_count,
        ),
        YCoCgVariant::Variant3 => transform_decorr_var3(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            block_count,
        ),
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn avx512_transform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_avx512f() || !has_avx512bw() {
            return;
        }

        // AVX512 currently uses generic implementation, so use low block count
        run_split_colour_and_recorr_transform_roundtrip_test(
            transform_with_split_colour_and_recorr,
            variant,
            2,
            "AVX512",
        );
    }
}
