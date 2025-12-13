use crate::transform::with_split_colour_and_recorr::untransform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx512bw::{
    recorrelate_ycocg_r_var1_avx512bw, recorrelate_ycocg_r_var2_avx512bw,
    recorrelate_ycocg_r_var3_avx512bw,
};

/// AVX512 implementation for split-colour and recorrelate untransform for BC2.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_split_colour_recorr_var1(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::Variant2 => {
            untransform_split_colour_recorr_var2(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::Variant3 => {
            untransform_split_colour_recorr_var3(
                alpha_ptr,
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_split_colour_recorr_var1(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<1>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

unsafe fn untransform_split_colour_recorr_var2(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<2>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

unsafe fn untransform_split_colour_recorr_var3(
    alpha_ptr: *const u64,
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_split_colour_recorr::<3>(
        alpha_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    )
}

/// AVX512 implementation for split-colour and recorrelate untransform for BC2.
/// Combines separate arrays of alpha, colour0, colour1 and indices back into standard interleaved BC2 blocks
/// while applying YCoCg recorrelation to the color endpoints.
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[allow(unused_assignments)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::identity_op)]
unsafe fn untransform_split_colour_recorr<const VARIANT: u8>(
    mut alpha_ptr: *const u64,
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    // Process 32 BC2 blocks at a time using AVX512 SIMD instructions
    let vectorized_blocks = block_count & !31; // Round down to multiple of 32
    let alphas_end = alpha_ptr.add(vectorized_blocks);

    // Mask for mixing output_0 (lower half of alpha & `color+index` splits)
    let perm_block_low = _mm512_setr_epi64(
        0,  // alpha 8 bytes
        8,  // colors + indices 8 bytes
        1,  // alpha 8 bytes
        9,  // colors + indices 8 bytes
        2,  // alpha 8 bytes
        10, // colors + indices 8 bytes
        3,  // alpha 8 bytes
        11, // colors + indices 8 bytes
    );
    // Mask for mixing output_1 (upper half of alpha & `color+index` splits)
    let perm_block_high = _mm512_setr_epi64(
        4,  // alpha 8 bytes
        12, // colors + indices 8 bytes
        5,  // alpha 8 bytes
        13, // colors + indices 8 bytes
        6,  // alpha 8 bytes
        14, // colors + indices 8 bytes
        7,  // alpha 8 bytes
        15, // colors + indices 8 bytes
    );

    // perm_color_interleave_low
    // Inputs:  color0=[C0_0, C0_1, ..., C0_15], color1=[C1_0, C1_1, ..., C1_15]
    // Output: [C0_0, C1_0, C0_1, C1_1, ..., C0_15, C1_15]
    let perm_color_interleave_low = _mm512_set_epi16(
        15 + 32, // C1_15
        15 + 0,  // C0_15,
        14 + 32, // C1_14
        14 + 0,  // C0_14,
        13 + 32, // C1_13
        13 + 0,  // C0_13,
        12 + 32, // C1_12
        12 + 0,  // C0_12,
        11 + 32, // C1_11
        11 + 0,  // C0_11,
        10 + 32, // C1_10
        10 + 0,  // C0_10,
        9 + 32,  // C1_9
        9 + 0,   // C0_9,
        8 + 32,  // C1_8
        8 + 0,   // C0_8,
        7 + 32,  // C1_7,
        7 + 0,   // C0_7,
        6 + 32,  // C1_6
        6 + 0,   // C0_6,
        5 + 32,  // C1_5
        5 + 0,   // C0_5,
        4 + 32,  // C1_4
        4 + 0,   // C0_4,
        3 + 32,  // C1_3
        3 + 0,   // C0_3,
        2 + 32,  // C1_2
        2 + 0,   // C0_2,
        1 + 32,  // C1_1
        1 + 0,   // C0_1,
        0 + 32,  // C1_0
        0 + 0,   // C0_0,
    );

    let perm_color_interleave_high = _mm512_set_epi16(
        (15 + 32) + 16, // C1_31
        (15 + 0) + 16,  // C0_31,
        (14 + 32) + 16, // C1_30
        (14 + 0) + 16,  // C0_30,
        (13 + 32) + 16, // C1_29
        (13 + 0) + 16,  // C0_29,
        (12 + 32) + 16, // C1_28
        (12 + 0) + 16,  // C0_28,
        (11 + 32) + 16, // C1_27
        (11 + 0) + 16,  // C0_27,
        (10 + 32) + 16, // C1_26
        (10 + 0) + 16,  // C0_26,
        (9 + 32) + 16,  // C1_25
        (9 + 0) + 16,   // C0_25,
        (8 + 32) + 16,  // C1_24
        (8 + 0) + 16,   // C0_24,
        (7 + 32) + 16,  // C1_23,
        (7 + 0) + 16,   // C0_23,
        (6 + 32) + 16,  // C1_22
        (6 + 0) + 16,   // C0_22,
        (5 + 32) + 16,  // C1_21
        (5 + 0) + 16,   // C0_21,
        (4 + 32) + 16,  // C1_20
        (4 + 0) + 16,   // C0_20,
        (3 + 32) + 16,  // C1_19
        (3 + 0) + 16,   // C0_19,
        (2 + 32) + 16,  // C1_18
        (2 + 0) + 16,   // C0_18,
        (1 + 32) + 16,  // C1_17
        (1 + 0) + 16,   // C0_17,
        (0 + 32) + 16,  // C1_16
        (0 + 0) + 16,   // C0_16,
    );

    // Mask for mixing colors and indices (lower half)
    // rust specifies the args for this call in reverse order, e15 == e0. this is a stdlib blunder
    // Inputs:  colors_0=[C0_0, C1_0, ..., C0_8, C1_8] | indices_0=[I0_0, I0_1 ..., I16_0, I16_1]
    // Output: [C0_0, C1_0, I0_0, I0_1 ..., C0_8, C1_8, I8_0, I8_1]
    let perm_color_index_low = _mm512_set_epi16(
        15 + 32, // I7_1
        14 + 32, // I7_0,
        15 + 0,  // C1_7
        14 + 0,  // C0_7,
        13 + 32, // I6_1
        12 + 32, // I6_0,
        13 + 0,  // C1_6
        12 + 0,  // C0_6,
        11 + 32, // I5_1
        10 + 32, // I5_0,
        11 + 0,  // C1_5
        10 + 0,  // C0_5,
        9 + 32,  // I4_1
        8 + 32,  // I4_0,
        9 + 0,   // C1_4
        8 + 0,   // C0_4,
        7 + 32,  // I3_1
        6 + 32,  // I3_0,
        7 + 0,   // C1_3
        6 + 0,   // C0_3,
        5 + 32,  // I2_1
        4 + 32,  // I2_0,
        5 + 0,   // C1_2
        4 + 0,   // C0_2,
        3 + 32,  // I1_1
        2 + 32,  // I1_0,
        3 + 0,   // C1_1
        2 + 0,   // C0_1,
        1 + 32,  // I0_0
        0 + 32,  // I0_0,
        1 + 0,   // C1_0
        0 + 0,   // C0_0,
    );
    // Mask for mixing colors and indices (upper half)
    let perm_color_index_high = _mm512_set_epi16(
        31 + 32, // I15_1
        30 + 32, // I15_0,
        31 + 0,  // C1_15
        30 + 0,  // C0_15,
        29 + 32, // I14_1
        28 + 32, // I14_0,
        29 + 0,  // C1_14
        28 + 0,  // C0_14,
        27 + 32, // I13_1
        26 + 32, // I13_0,
        27 + 0,  // C1_13
        26 + 0,  // C0_13,
        25 + 32, // I12_1
        24 + 32, // I12_0,
        25 + 0,  // C1_12
        24 + 0,  // C0_12,
        23 + 32, // I11_1
        22 + 32, // I11_0,
        23 + 0,  // C1_11
        22 + 0,  // C0_11
        21 + 32, // I10_1
        20 + 32, // I10_0,
        21 + 0,  // C1_10
        20 + 0,  // C0_10,
        19 + 32, // I9_1
        18 + 32, // I9_0,
        19 + 0,  // C1_9
        18 + 0,  // C0_9,
        17 + 32, // I8_1
        16 + 32, // I8_0,
        17 + 0,  // C1_8
        16 + 0,  // C0_8,
    );

    // Main SIMD processing loop - handles 32 blocks per iteration
    while alpha_ptr < alphas_end {
        // Load 32 blocks worth of data
        // Alpha data: 32 blocks * 8 bytes = 256 bytes
        let alpha_0 = _mm512_loadu_si512(alpha_ptr as *const __m512i);
        let alpha_1 = _mm512_loadu_si512(alpha_ptr.add(8) as *const __m512i);
        let alpha_2 = _mm512_loadu_si512(alpha_ptr.add(16) as *const __m512i);
        let alpha_3 = _mm512_loadu_si512(alpha_ptr.add(24) as *const __m512i);
        alpha_ptr = alpha_ptr.add(32);

        // Colors: 32 blocks * 2 bytes = 64 bytes each
        let color0 = _mm512_loadu_si512(color0_ptr as *const __m512i);
        color0_ptr = color0_ptr.add(32);
        let color1 = _mm512_loadu_si512(color1_ptr as *const __m512i);
        color1_ptr = color1_ptr.add(32);

        // Apply YCoCg recorrelation to colors
        let recorr_color0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(color0),
            2 => recorrelate_ycocg_r_var2_avx512bw(color0),
            3 => recorrelate_ycocg_r_var3_avx512bw(color0),
            _ => unreachable_unchecked(),
        };

        let recorr_color1 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(color1),
            2 => recorrelate_ycocg_r_var2_avx512bw(color1),
            3 => recorrelate_ycocg_r_var3_avx512bw(color1),
            _ => unreachable_unchecked(),
        };

        // Indices: 32 blocks * 4 bytes = 128 bytes
        let indices0 = _mm512_loadu_si512(indices_ptr as *const __m512i);
        let indices1 = _mm512_loadu_si512(indices_ptr.add(16) as *const __m512i);
        indices_ptr = indices_ptr.add(32);

        // Interleave color0 and color1 into alternating pairs (first 16 blocks)
        let colors_0 =
            _mm512_permutex2var_epi16(recorr_color0, perm_color_interleave_low, recorr_color1);
        let colors_1 =
            _mm512_permutex2var_epi16(recorr_color0, perm_color_interleave_high, recorr_color1);

        // re-mix lower & upper colour+index halves
        let colors_indices_0 = _mm512_permutex2var_epi16(colors_0, perm_color_index_low, indices0);
        let colors_indices_1 = _mm512_permutex2var_epi16(colors_0, perm_color_index_high, indices0);
        let colors_indices_2 = _mm512_permutex2var_epi16(colors_1, perm_color_index_low, indices1);
        let colors_indices_3 = _mm512_permutex2var_epi16(colors_1, perm_color_index_high, indices1);

        // re-mix alphas and colour+index halves
        let output_0 = _mm512_permutex2var_epi64(alpha_0, perm_block_low, colors_indices_0);
        let output_1 = _mm512_permutex2var_epi64(alpha_0, perm_block_high, colors_indices_0);
        let output_2 = _mm512_permutex2var_epi64(alpha_1, perm_block_low, colors_indices_1);
        let output_3 = _mm512_permutex2var_epi64(alpha_1, perm_block_high, colors_indices_1);
        let output_4 = _mm512_permutex2var_epi64(alpha_2, perm_block_low, colors_indices_2);
        let output_5 = _mm512_permutex2var_epi64(alpha_2, perm_block_high, colors_indices_2);
        let output_6 = _mm512_permutex2var_epi64(alpha_3, perm_block_low, colors_indices_3);
        let output_7 = _mm512_permutex2var_epi64(alpha_3, perm_block_high, colors_indices_3);

        // Store all 32 blocks (512 bytes total)
        _mm512_storeu_si512(output_ptr as *mut __m512i, output_0);
        _mm512_storeu_si512(output_ptr.add(64) as *mut __m512i, output_1);
        _mm512_storeu_si512(output_ptr.add(128) as *mut __m512i, output_2);
        _mm512_storeu_si512(output_ptr.add(192) as *mut __m512i, output_3);
        _mm512_storeu_si512(output_ptr.add(256) as *mut __m512i, output_4);
        _mm512_storeu_si512(output_ptr.add(320) as *mut __m512i, output_5);
        _mm512_storeu_si512(output_ptr.add(384) as *mut __m512i, output_6);
        _mm512_storeu_si512(output_ptr.add(448) as *mut __m512i, output_7);

        // Advance output pointer
        output_ptr = output_ptr.add(512);
    }

    // Process any remaining blocks using generic implementation
    let remaining_blocks = block_count - vectorized_blocks;
    if remaining_blocks > 0 {
        let variant = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::untransform_with_split_colour_and_recorr(
            alpha_ptr,
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            remaining_blocks,
            variant,
        );
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
    fn avx512_untransform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_avx512f() || !has_avx512bw() {
            return;
        }

        // AVX512 processes 512 bytes per iteration (* 2 / 16 == 64)
        run_split_colour_and_recorr_untransform_roundtrip_test(
            untransform_with_split_colour_and_recorr,
            variant,
            64,
            "AVX512",
        );
    }
}
