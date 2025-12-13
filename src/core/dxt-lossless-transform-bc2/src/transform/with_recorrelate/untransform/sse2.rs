#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::sse2::{
    recorrelate_ycocg_r_var1_sse2, recorrelate_ycocg_r_var2_sse2, recorrelate_ycocg_r_var3_sse2,
};

use crate::transform::with_recorrelate::untransform::generic::untransform_with_recorrelate_generic;

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

#[target_feature(enable = "sse2")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut alphas_in: *const u64,
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Process 4 BC2 blocks at a time using SSE2 SIMD instructions
    let vectorized_blocks = num_blocks & !3; // Round down to multiple of 4
    let colors_end = colors_in.add(vectorized_blocks);

    // Main SIMD processing loop - handles 4 blocks per iteration
    while colors_in < colors_end {
        // Load data for 4 blocks - matching assembly pattern
        let alphas_0 = _mm_loadu_si128(alphas_in as *const __m128i); // First alpha block (xmm0)
        let alphas_1 = _mm_loadu_si128(alphas_in.add(2) as *const __m128i); // Second alpha block (xmm1)
        alphas_in = alphas_in.add(4);

        let colors_0 = _mm_loadu_si128(colors_in as *const __m128i); // Colors (xmm2)
        colors_in = colors_in.add(4);
        let indices_0 = _mm_loadu_si128(indices_in as *const __m128i); // Indices (xmm3)
        indices_in = indices_in.add(4);

        // Apply recorrelation to the colors based on the variant
        let recorrelated_colors_0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_sse2(colors_0),
            2 => recorrelate_ycocg_r_var2_sse2(colors_0),
            3 => recorrelate_ycocg_r_var3_sse2(colors_0),
            _ => unreachable_unchecked(),
        };

        // Current:
        // alphas_0: [A0  - A15]
        // alphas_1: [A16 - A31]
        // recorrelated_colors_0: [C0  - C15] (after recorrelation)
        // indices_0: [I0  - I15]

        // Target:
        // 0       -       7 |   08       -       15
        // block0: [A00 - A07] | [C00 - C03] [I00 - I03]
        // block1: [A08 - A15] | [C04 - C07] [I04 - I07]
        // block2: [A16 - A23] | [C08 - C11] [I08 - I11]
        // block3: [A24 - A31] | [C12 - C15] [I12 - I15]

        // We're going to follow the assembly pattern I wrote before exactly:
        // Let's get [C00 - C03] [I00 - I03] ... inside colors_indices_low (xmm6)
        // Let's get [C08 - C11] [I08 - I11] ... inside colors_indices_high (xmm7)
        let colors_indices_low = _mm_unpacklo_epi32(recorrelated_colors_0, indices_0);
        let colors_indices_high = _mm_unpackhi_epi32(recorrelated_colors_0, indices_0);
        // colors_indices_low: [C00 - C03] [I00 - I03] [C04 - C07] [I04 - I07]
        // colors_indices_high: [C08 - C11] [I08 - I11] [C12 - C15] [I12 - I15]

        // We're gonna now export results to remaining variables
        // block0 = alphas_0
        // block1 = alphas_0_copy
        // block2 = alphas_1
        // block3 = alphas_1_copy

        // Combine alphas with colors+indices using punpcklqdq and punpckhqdq
        let block0 = _mm_unpacklo_epi64(alphas_0, colors_indices_low); // punpcklqdq xmm0, xmm6
        let block2 = _mm_unpacklo_epi64(alphas_1, colors_indices_high); // punpcklqdq xmm1, xmm7
        let block1 = _mm_unpackhi_epi64(alphas_0, colors_indices_low); // punpckhqdq xmm4, xmm6
        let block3 = _mm_unpackhi_epi64(alphas_1, colors_indices_high); // punpckhqdq xmm5, xmm7

        // Store results in the same order as assembly
        _mm_storeu_si128(output_ptr as *mut __m128i, block0);
        _mm_storeu_si128(output_ptr.add(16) as *mut __m128i, block1);
        _mm_storeu_si128(output_ptr.add(32) as *mut __m128i, block2);
        _mm_storeu_si128(output_ptr.add(48) as *mut __m128i, block3);
        output_ptr = output_ptr.add(64);
    }

    // Process any remaining blocks using generic implementation
    let remaining_blocks = num_blocks - vectorized_blocks;
    if remaining_blocks > 0 {
        untransform_with_recorrelate_generic(
            alphas_in,
            colors_in,
            indices_in,
            output_ptr,
            remaining_blocks,
            match VARIANT {
                1 => YCoCgVariant::Variant1,
                2 => YCoCgVariant::Variant2,
                3 => YCoCgVariant::Variant3,
                _ => unreachable_unchecked(),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 8)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 8)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 8)]
    fn sse2_untransform_roundtrip(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_sse2() {
            return;
        }
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "SSE2");
    }
}
