use crate::transform::with_recorrelate::untransform::generic::untransform_with_recorrelate_generic;
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

pub(crate) unsafe fn untransform_with_recorrelate(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(alphas_in, colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    alphas_in: *const u64,
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(alphas_in, colors_in, indices_in, output_ptr, num_blocks)
}

#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[allow(unused_assignments)]
#[allow(clippy::zero_prefixed_literal)]
#[allow(clippy::identity_op)]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut alphas_in: *const u64,
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Process 16 BC2 blocks at a time using AVX512 SIMD instructions
    let vectorized_blocks = num_blocks & !15; // Round down to multiple of 16
    let alphas_end = alphas_in.add(vectorized_blocks);

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
    // Mask for mixing colors and indices (lower half)
    // rust specifies the args for this call in reverse order, e15 == e0. this is a stdlib blunder
    let perm_color_index_low = _mm512_setr_epi32(
        00 + 00, // colors 4 bytes,
        00 + 16, // indices 4 bytes,
        01 + 00, // colors 4 bytes,
        01 + 16, // indices 4 bytes,
        02 + 00, // colors 4 bytes,
        02 + 16, // indices 4 bytes,
        03 + 00, // colors 4 bytes,
        03 + 16, // indices 4 bytes,
        04 + 00, // colors 4 bytes,
        04 + 16, // indices 4 bytes,
        05 + 00, // colors 4 bytes,
        05 + 16, // indices 4 bytes,
        06 + 00, // colors 4 bytes,
        06 + 16, // indices 4 bytes,
        07 + 00, // colors 4 bytes,
        07 + 16, // indices 4 bytes,
    );
    // Mask for mixing colors and indices (upper half)
    let perm_color_index_high = _mm512_setr_epi32(
        08 + 00, // colors 4 bytes,
        08 + 16, // indices 4 bytes,
        09 + 00, // colors 4 bytes,
        09 + 16, // indices 4 bytes,
        10 + 00, // colors 4 bytes,
        10 + 16, // indices 4 bytes,
        11 + 00, // colors 4 bytes,
        11 + 16, // indices 4 bytes,
        12 + 00, // colors 4 bytes,
        12 + 16, // indices 4 bytes,
        13 + 00, // colors 4 bytes,
        13 + 16, // indices 4 bytes,
        14 + 00, // colors 4 bytes,
        14 + 16, // indices 4 bytes,
        15 + 00, // colors 4 bytes,
        15 + 16, // indices 4 bytes,
    );

    // Main SIMD processing loop - handles 16 blocks per iteration
    while alphas_in < alphas_end {
        // Load 16 blocks worth of data
        // Alpha data: 16 blocks * 8 bytes = 128 bytes
        let alpha_0 = _mm512_loadu_si512(alphas_in as *const __m512i);
        let alpha_1 = _mm512_loadu_si512(alphas_in.add(8) as *const __m512i);
        alphas_in = alphas_in.add(16);

        // Colors and indices: 16 blocks * 4 bytes = 64 bytes each
        let mut colors = _mm512_loadu_si512(colors_in as *const __m512i);
        colors_in = colors_in.add(16);
        let indices = _mm512_loadu_si512(indices_in as *const __m512i);
        indices_in = indices_in.add(16);

        // Apply recorrelation to colors based on variant
        match VARIANT {
            1 => colors = recorrelate_ycocg_r_var1_avx512bw(colors),
            2 => colors = recorrelate_ycocg_r_var2_avx512bw(colors),
            3 => colors = recorrelate_ycocg_r_var3_avx512bw(colors),
            _ => unreachable_unchecked(),
        }

        // re-mix lower & upper colour+index halves
        let colors_indices_0 = _mm512_permutex2var_epi32(colors, perm_color_index_low, indices);
        let colors_indices_1 = _mm512_permutex2var_epi32(colors, perm_color_index_high, indices);

        // re-mix alphas and colour+index halves
        let output_0 = _mm512_permutex2var_epi64(alpha_0, perm_block_low, colors_indices_0);
        let output_1 = _mm512_permutex2var_epi64(alpha_0, perm_block_high, colors_indices_0);
        let output_2 = _mm512_permutex2var_epi64(alpha_1, perm_block_low, colors_indices_1);
        let output_3 = _mm512_permutex2var_epi64(alpha_1, perm_block_high, colors_indices_1);

        // Store all 16 blocks (256 bytes total)
        _mm512_storeu_si512(output_ptr as *mut __m512i, output_0);
        _mm512_storeu_si512(output_ptr.add(64) as *mut __m512i, output_1);
        _mm512_storeu_si512(output_ptr.add(128) as *mut __m512i, output_2);
        _mm512_storeu_si512(output_ptr.add(192) as *mut __m512i, output_3);

        // Advance output pointer
        output_ptr = output_ptr.add(256);
    }

    // Process any remaining blocks using generic implementation
    let remaining_blocks = num_blocks - vectorized_blocks;
    if remaining_blocks > 0 {
        // Determine recorrelation mode based on variant
        let recorrelation_mode = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };

        untransform_with_recorrelate_generic(
            alphas_in,
            colors_in,
            indices_in,
            output_ptr,
            remaining_blocks,
            recorrelation_mode,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 32)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 32)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 32)]
    fn avx512_untransform_roundtrip(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512f() || !has_avx512bw() {
            return;
        }
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "AVX512");
    }
}
