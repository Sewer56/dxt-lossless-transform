#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::sse2::*;

pub(crate) unsafe fn untransform_with_split_colour_and_recorr(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::None => unreachable_unchecked(),
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(color0_in, color1_in, indices_in, output_ptr, block_count)
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(color0_in, color1_in, indices_in, output_ptr, block_count)
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(color0_in, color1_in, indices_in, output_ptr, block_count)
        }
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<1>(color0_in, color1_in, indices_in, output_ptr, block_count)
}

unsafe fn untransform_recorr_var2(
    color0_in: *const u16,
    color1_in: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<2>(color0_in, color1_in, indices_in, output_ptr, block_count)
}

unsafe fn untransform_recorr_var3(
    color0_in: *const u16,
    color1_ptr: *const u16,
    indices_in: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    untransform_recorr::<3>(color0_in, color1_ptr, indices_in, output_ptr, block_count)
}

#[target_feature(enable = "sse2")]
#[allow(clippy::identity_op)]
unsafe fn untransform_recorr<const VARIANT: u8>(
    mut color0_in: *const u16,
    mut color1_in: *const u16,
    mut indices_in: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 8 blocks (64 bytes) at a time with SSE2
    let aligned_count = block_count - (block_count % 8);
    let color0_ptr_aligned_end = color0_in.add(aligned_count);

    while color0_in < color0_ptr_aligned_end {
        let color0s = _mm_loadu_si128(color0_in as *const __m128i);
        color0_in = color0_in.add(8);

        let color1s = _mm_loadu_si128(color1_in as *const __m128i);
        color1_in = color1_in.add(8);

        let indices_0 = _mm_loadu_si128(indices_in as *const __m128i);
        let indices_1 = _mm_loadu_si128(indices_in.add(4) as *const __m128i);
        indices_in = indices_in.add(8);

        // Apply YCoCg-R recorrelation to the colors using the specified variant
        let (recorr_color0s, recorr_color1s) = match VARIANT {
            1 => (
                recorrelate_ycocg_r_var1_sse2(color0s),
                recorrelate_ycocg_r_var1_sse2(color1s),
            ),
            2 => (
                recorrelate_ycocg_r_var2_sse2(color0s),
                recorrelate_ycocg_r_var2_sse2(color1s),
            ),
            3 => (
                recorrelate_ycocg_r_var3_sse2(color0s),
                recorrelate_ycocg_r_var3_sse2(color1s),
            ),
            _ => unreachable_unchecked(),
        };

        // Mix the colours back into their c0+c1 pairs
        let colors_0 = _mm_unpacklo_epi16(recorr_color0s, recorr_color1s);
        let colors_1 = _mm_unpackhi_epi16(recorr_color0s, recorr_color1s);

        // Re-combine the colors and indices into the BC1 block format
        let blocks_0 = _mm_unpacklo_epi32(colors_0, indices_0);
        let blocks_1 = _mm_unpackhi_epi32(colors_0, indices_0);
        let blocks_2 = _mm_unpacklo_epi32(colors_1, indices_1);
        let blocks_3 = _mm_unpackhi_epi32(colors_1, indices_1);

        _mm_storeu_si128(output_ptr as *mut __m128i, blocks_0);
        _mm_storeu_si128(output_ptr.add(16) as *mut __m128i, blocks_1);
        _mm_storeu_si128(output_ptr.add(32) as *mut __m128i, blocks_2);
        _mm_storeu_si128(output_ptr.add(48) as *mut __m128i, blocks_3);

        // Advance output pointer
        output_ptr = output_ptr.add(64);
    }

    // Process any remaining blocks (less than 8) using generic implementation
    let remaining_count = block_count - aligned_count;
    match VARIANT {
        1 => super::generic::untransform_recorr_var1(
            color0_in,
            color1_in,
            indices_in,
            output_ptr,
            remaining_count,
        ),
        2 => super::generic::untransform_recorr_var2(
            color0_in,
            color1_in,
            indices_in,
            output_ptr,
            remaining_count,
        ),
        3 => super::generic::untransform_recorr_var3(
            color0_in,
            color1_in,
            indices_in,
            output_ptr,
            remaining_count,
        ),
        _ => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour_and_recorr::untransform::untransform_with_split_colour_and_recorr;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(#[case] decorr_variant: YCoCgVariant) {
        if !has_sse2() {
            return;
        }

        // 64 bytes processed per main loop iteration (* 2 / 8 == 16)
        run_with_split_colour_and_recorr_generic_untransform_unaligned_test(
            untransform_with_split_colour_and_recorr,
            decorr_variant,
            16,
            "untransform_with_split_colour_and_recorr (sse2, unaligned)",
        );
    }
}
