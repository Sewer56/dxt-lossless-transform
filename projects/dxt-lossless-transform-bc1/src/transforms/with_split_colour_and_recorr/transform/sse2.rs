use crate::transforms::with_split_colour_and_recorr::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::sse2::{
    decorrelate_ycocg_r_var1_sse2, decorrelate_ycocg_r_var2_sse2, decorrelate_ycocg_r_var3_sse2,
};

/// SSE2 implementation for split-colour transform with YCoCg-R decorrelation.
#[target_feature(enable = "sse2")]
unsafe fn transform_impl<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut color0_out: *mut u16,
    mut color1_out: *mut u16,
    mut indices_out: *mut u32,
    block_count: usize,
) {
    let blocks8 = block_count / 8; // round down via division
    let input_end = input_ptr.add(blocks8 * 8 * 8);

    while input_ptr < input_end {
        // Load four 16-byte chunks = 8 blocks
        let data0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);
        input_ptr = input_ptr.add(64);

        // Split colors and indices using shufps patterns
        let colours_0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0x88,
        ));
        let colours_1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0x88,
        ));
        let idx0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data0),
            _mm_castsi128_ps(data1),
            0xDD,
        ));
        let idx1 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(data2),
            _mm_castsi128_ps(data3),
            0xDD,
        ));

        // Now we need to split the colours into their color0 and color1 components.
        // SSE2 is a bit limited here, so we'll use what we can to get by.

        // Shuffle so the first 8 bytes have their color0 and color1 components chunked into u32s
        let colours_u32_grouped_0_lo = _mm_shufflelo_epi16(colours_0, 0b11_01_10_00);
        let colours_u32_grouped_0 = _mm_shufflehi_epi16(colours_u32_grouped_0_lo, 0b11_01_10_00);

        let colours_u32_grouped_1_lo = _mm_shufflelo_epi16(colours_1, 0b11_01_10_00);
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

        let colours_0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colours_0),
            2 => decorrelate_ycocg_r_var2_sse2(colours_0),
            3 => decorrelate_ycocg_r_var3_sse2(colours_0),
            _ => unreachable_unchecked(),
        };

        let colours_1 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colours_1),
            2 => decorrelate_ycocg_r_var2_sse2(colours_1),
            3 => decorrelate_ycocg_r_var3_sse2(colours_1),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm_storeu_si128(color0_out as *mut __m128i, colours_0);
        _mm_storeu_si128(color1_out as *mut __m128i, colours_1);
        _mm_storeu_si128(indices_out as *mut __m128i, idx0);
        _mm_storeu_si128((indices_out as *mut __m128i).add(1), idx1);

        color0_out = color0_out.add(8); // 16 bytes
        color1_out = color1_out.add(8); // 16 bytes
        indices_out = indices_out.add(8); // 32 bytes
    }

    // Remainder
    let remainder_blocks = block_count % 8;
    let variant_enum = match VARIANT {
        1 => YCoCgVariant::Variant1,
        2 => YCoCgVariant::Variant2,
        3 => YCoCgVariant::Variant3,
        _ => unreachable_unchecked(),
    };
    generic::transform_with_split_colour_and_decorr_generic(
        input_ptr,
        color0_out,
        color1_out,
        indices_out,
        remainder_blocks,
        variant_enum,
    );
}

// Wrappers for asm inspection
#[target_feature(enable = "sse2")]
unsafe fn transform_var1(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
) {
    transform_impl::<1>(input_ptr, color0_out, color1_out, indices_out, blocks)
}
#[target_feature(enable = "sse2")]
unsafe fn transform_var2(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
) {
    transform_impl::<2>(input_ptr, color0_out, color1_out, indices_out, blocks)
}
#[target_feature(enable = "sse2")]
unsafe fn transform_var3(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
) {
    transform_impl::<3>(input_ptr, color0_out, color1_out, indices_out, blocks)
}

#[inline(always)]
pub(crate) unsafe fn transform_with_split_colour_and_decorr(
    input_ptr: *const u8,
    color0_out: *mut u16,
    color1_out: *mut u16,
    indices_out: *mut u32,
    blocks: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => {
            transform_var1(input_ptr, color0_out, color1_out, indices_out, blocks)
        }
        YCoCgVariant::Variant2 => {
            transform_var2(input_ptr, color0_out, color1_out, indices_out, blocks)
        }
        YCoCgVariant::Variant3 => {
            transform_var3(input_ptr, color0_out, color1_out, indices_out, blocks)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour_and_recorr::untransform::untransform_with_split_colour_and_recorr;

    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    fn sse2_transform_roundtrip(#[case] variant: YCoCgVariant) {
        if !has_sse2() {
            return;
        }
        for blocks in 1..=128 {
            let input = generate_bc1_test_data(blocks);
            let mut c0_buf = vec![0u16; blocks];
            let mut c1_buf = vec![0u16; blocks];
            let mut idx_buf = vec![0u32; blocks];
            let mut recon = vec![0u8; input.len()];
            unsafe {
                transform_with_split_colour_and_decorr(
                    input.as_ptr(),
                    c0_buf.as_mut_ptr(),
                    c1_buf.as_mut_ptr(),
                    idx_buf.as_mut_ptr(),
                    blocks,
                    variant,
                );
                untransform_with_split_colour_and_recorr(
                    c0_buf.as_ptr(),
                    c1_buf.as_ptr(),
                    idx_buf.as_ptr(),
                    recon.as_mut_ptr(),
                    blocks,
                    variant,
                );
            }
            assert_eq!(
                input.as_slice(),
                recon.as_slice(),
                "SSE2 roundtrip mismatch {variant:?}"
            );
        }
    }
}
