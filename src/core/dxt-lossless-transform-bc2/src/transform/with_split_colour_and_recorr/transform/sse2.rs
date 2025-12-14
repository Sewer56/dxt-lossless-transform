use crate::transform::with_split_colour_and_recorr::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::sse2::{
    decorrelate_ycocg_r_var1_sse2, decorrelate_ycocg_r_var2_sse2, decorrelate_ycocg_r_var3_sse2,
};

// Const-generic worker
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
unsafe fn transform_split_colour_and_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alpha_out: *mut u64,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    // Process 8 blocks at a time (128 bytes) with SSE2
    let num_iterations = block_count / 8 * 8; // 8 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // * 16 bytes per block

    while input_ptr < input_end {
        // Load eight 16-byte BC2 blocks
        let data0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);
        let data4 = _mm_loadu_si128(input_ptr.add(64) as *const __m128i);
        let data5 = _mm_loadu_si128(input_ptr.add(80) as *const __m128i);
        let data6 = _mm_loadu_si128(input_ptr.add(96) as *const __m128i);
        let data7 = _mm_loadu_si128(input_ptr.add(112) as *const __m128i);
        input_ptr = input_ptr.add(128);

        // Extract alphas (first 8 bytes of each block)
        let alphas0 = _mm_unpacklo_epi64(data0, data1); // alpha from block 0 and 1
        let alphas1 = _mm_unpacklo_epi64(data2, data3); // alpha from block 2 and 3
        let alphas2 = _mm_unpacklo_epi64(data4, data5); // alpha from block 4 and 5
        let alphas3 = _mm_unpacklo_epi64(data6, data7); // alpha from block 6 and 7

        // Extract colors and indices (last 8 bytes of each block)
        let colors_indices0 = _mm_unpackhi_epi64(data0, data1); // colors+indices from block 0 and 1
        let colors_indices1 = _mm_unpackhi_epi64(data2, data3); // colors+indices from block 2 and 3
        let colors_indices2 = _mm_unpackhi_epi64(data4, data5); // colors+indices from block 4 and 5
        let colors_indices3 = _mm_unpackhi_epi64(data6, data7); // colors+indices from block 6 and 7

        // Extract just the colors for decorrelation (first 4 bytes of each colors_indices)
        let colors_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices0),
            _mm_castsi128_ps(colors_indices1),
            0x88, // Select lower 32-bit from each 64-bit lane
        ));
        let colors_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices2),
            _mm_castsi128_ps(colors_indices3),
            0x88,
        ));

        // Apply decorrelation to the packed colors
        let decorr_colors_0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colors_0),
            2 => decorrelate_ycocg_r_var2_sse2(colors_0),
            3 => decorrelate_ycocg_r_var3_sse2(colors_0),
            _ => unreachable_unchecked(),
        };
        let decorr_colors_1 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colors_1),
            2 => decorrelate_ycocg_r_var2_sse2(colors_1),
            3 => decorrelate_ycocg_r_var3_sse2(colors_1),
            _ => unreachable_unchecked(),
        };

        // Split the decorrelated colors into color0 and color1 components
        // Shuffle so the first 8 bytes have their color0 and color1 components chunked into u32s
        let colours_u32_grouped_0_lo = _mm_shufflelo_epi16(decorr_colors_0, 0b11_01_10_00);
        let colours_u32_grouped_0 = _mm_shufflehi_epi16(colours_u32_grouped_0_lo, 0b11_01_10_00);

        let colours_u32_grouped_1_lo = _mm_shufflelo_epi16(decorr_colors_1, 0b11_01_10_00);
        let colours_u32_grouped_1 = _mm_shufflehi_epi16(colours_u32_grouped_1_lo, 0b11_01_10_00);

        // Now combine back into single colour registers by shuffling the u32s into their respective positions.
        let colours_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colours_u32_grouped_0),
            _mm_castsi128_ps(colours_u32_grouped_1),
            0b10_00_10_00,
        ));
        let colours_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colours_u32_grouped_0),
            _mm_castsi128_ps(colours_u32_grouped_1),
            0b11_01_11_01,
        ));

        // Extract indices
        let indices0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices0),
            _mm_castsi128_ps(colors_indices1),
            0xDD, // Select upper 32-bit from each 64-bit lane
        ));
        let indices1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices2),
            _mm_castsi128_ps(colors_indices3),
            0xDD,
        ));

        // Store results
        _mm_storeu_si128(alpha_out as *mut __m128i, alphas0);
        _mm_storeu_si128((alpha_out as *mut __m128i).add(1), alphas1);
        _mm_storeu_si128((alpha_out as *mut __m128i).add(2), alphas2);
        _mm_storeu_si128((alpha_out as *mut __m128i).add(3), alphas3);

        _mm_storeu_si128(color0_out as *mut __m128i, colours_0);
        _mm_storeu_si128(color1_out as *mut __m128i, colours_1);

        _mm_storeu_si128(indices_out as *mut __m128i, indices0);
        _mm_storeu_si128((indices_out as *mut __m128i).add(1), indices1);

        alpha_out = alpha_out.add(8); // 8 u64s = 64 bytes
        color0_out = color0_out.add(8); // 8 u16s = 16 bytes
        color1_out = color1_out.add(8); // 8 u16s = 16 bytes
        indices_out = indices_out.add(8); // 8 u32s = 32 bytes
    }

    // Handle any remaining blocks
    let remaining_blocks = block_count % 8;
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

// Wrappers for asm inspection
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
#[inline]
pub(crate) unsafe fn transform_split_colour_and_decorr_var1(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    transform_split_colour_and_decorr::<1>(
        input_ptr,
        alpha_out,
        color0_out,
        color1_out,
        indices_out,
        block_count,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
#[inline]
pub(crate) unsafe fn transform_split_colour_and_decorr_var2(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    transform_split_colour_and_decorr::<2>(
        input_ptr,
        alpha_out,
        color0_out,
        color1_out,
        indices_out,
        block_count,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
#[inline]
pub(crate) unsafe fn transform_split_colour_and_decorr_var3(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
) {
    transform_split_colour_and_decorr::<3>(
        input_ptr,
        alpha_out,
        color0_out,
        color1_out,
        indices_out,
        block_count,
    )
}

// Runtime dispatcher
#[inline(always)]
pub(crate) unsafe fn transform_with_split_colour_and_recorr(
    input_ptr: *const u8,
    alpha_out: *mut u64,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    block_count: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => transform_split_colour_and_decorr_var1(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            block_count,
        ),
        YCoCgVariant::Variant2 => transform_split_colour_and_decorr_var2(
            input_ptr,
            alpha_out,
            color0_out,
            color1_out,
            indices_out,
            block_count,
        ),
        YCoCgVariant::Variant3 => transform_split_colour_and_decorr_var3(
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
    fn sse2_transform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_sse2() {
            return;
        }

        run_split_colour_and_recorr_transform_roundtrip_test(
            transform_with_split_colour_and_recorr,
            variant,
            16,
            "SSE2",
        );
    }
}
