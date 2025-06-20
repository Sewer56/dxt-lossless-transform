#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::sse2::{
    recorrelate_ycocg_r_var1_sse2, recorrelate_ycocg_r_var2_sse2, recorrelate_ycocg_r_var3_sse2,
};

use crate::transforms::with_recorrelate::untransform::generic::untransform_with_recorrelate_generic;

pub(crate) unsafe fn untransform_with_recorrelate(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_in, indices_in, output_ptr, num_blocks);
        }
        YCoCgVariant::None => {
            // This should be unreachable based on the calling context
            unreachable_unchecked()
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`

unsafe fn untransform_recorr_var1(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<1>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var2(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<2>(colors_in, indices_in, output_ptr, num_blocks)
}

unsafe fn untransform_recorr_var3(
    colors_in: *const u32,
    indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    untransform_recorr::<3>(colors_in, indices_in, output_ptr, num_blocks)
}

#[target_feature(enable = "sse2")]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut colors_in: *const u32,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    num_blocks: usize,
) {
    // === Main Vectorized Loop ===
    // Process 8 blocks at a time using SSE2 SIMD instructions (unroll 2)
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !7; // Round down to multiple of 8
    let colors_end = colors_in.add(vectorized_blocks);

    // Main SIMD processing loop - handles 8 blocks per iteration (unroll 2)
    while colors_in < colors_end {
        // Load colors and indices (16 bytes each)
        // This corresponds to 8 blocks worth of data
        let colors_0 = _mm_loadu_si128(colors_in as *const __m128i);
        let colors_1 = _mm_loadu_si128(colors_in.add(4) as *const __m128i);

        let indices_0 = _mm_loadu_si128(indices_in as *const __m128i);
        let indices_1 = _mm_loadu_si128(indices_in.add(4) as *const __m128i);

        // Apply recorrelation to the colors based on the variant
        let recorrelated_colors_0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_sse2(colors_0),
            2 => recorrelate_ycocg_r_var2_sse2(colors_0),
            3 => recorrelate_ycocg_r_var3_sse2(colors_0),
            _ => unreachable_unchecked(),
        };

        let recorrelated_colors_1 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_sse2(colors_1),
            2 => recorrelate_ycocg_r_var2_sse2(colors_1),
            3 => recorrelate_ycocg_r_var3_sse2(colors_1),
            _ => unreachable_unchecked(),
        };

        // Save copies for high parts (equivalent to movaps in assembly)
        let colors_0_copy = recorrelated_colors_0;
        let colors_1_copy = recorrelated_colors_1;

        // Unpack all blocks - interleave colors and indices
        // punpckldq: interleave low 32-bit values
        let interleaved_0 = _mm_unpacklo_epi32(recorrelated_colors_0, indices_0); // color0,index0,color1,index1
        let interleaved_1 = _mm_unpacklo_epi32(recorrelated_colors_1, indices_1); // color4,index4,color5,index5

        // punpckhdq: interleave high 32-bit values
        let interleaved_2 = _mm_unpackhi_epi32(colors_0_copy, indices_0); // color2,index2,color3,index3
        let interleaved_3 = _mm_unpackhi_epi32(colors_1_copy, indices_1); // color6,index6,color7,index7

        // Store all results (64 bytes total)
        _mm_storeu_si128(output_ptr as *mut __m128i, interleaved_0);
        _mm_storeu_si128(output_ptr.add(16) as *mut __m128i, interleaved_2);
        _mm_storeu_si128(output_ptr.add(32) as *mut __m128i, interleaved_1);
        _mm_storeu_si128(output_ptr.add(48) as *mut __m128i, interleaved_3);

        colors_in = colors_in.add(8);
        indices_in = indices_in.add(8);
        output_ptr = output_ptr.add(64);
    }

    // === Fallback for Remaining Blocks ===
    // Process any remaining blocks using generic implementation
    let remaining_blocks = num_blocks - vectorized_blocks;
    untransform_with_recorrelate_generic(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: WithRecorrelateUntransformFn,
        #[case] decorr_variant: YCoCgVariant,
    ) {
        if !has_sse2() {
            return;
        }

        run_with_recorrelate_untransform_unaligned_test(
            function,
            decorr_variant,
            "untransform_with_recorrelate (sse2)",
            16, // 64 bytes tested per main loop iteration (* 2 / 8 == 16)
        );
    }
}
