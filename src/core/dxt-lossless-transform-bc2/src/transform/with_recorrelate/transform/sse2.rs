use crate::transform::with_recorrelate::transform::generic;
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
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alphas_out: *mut u64,
    mut colors_out: *mut u32,
    mut indices_out: *mut u32,
    num_blocks: usize,
) {
    // Process 4 blocks at a time (64 bytes) with SSE2
    let num_iterations = num_blocks / 4 * 4; // 4 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // * 16 bytes per block

    while input_ptr < input_end {
        // Load four 16-byte BC2 blocks
        let data0 = _mm_loadu_si128(input_ptr as *const __m128i);
        let data1 = _mm_loadu_si128(input_ptr.add(16) as *const __m128i);
        let data2 = _mm_loadu_si128(input_ptr.add(32) as *const __m128i);
        let data3 = _mm_loadu_si128(input_ptr.add(48) as *const __m128i);
        input_ptr = input_ptr.add(64);

        // Extract alphas (first 8 bytes of each block)
        let alphas0 = _mm_unpacklo_epi64(data0, data1); // alpha from block 0 and 1
        let alphas1 = _mm_unpacklo_epi64(data2, data3); // alpha from block 2 and 3

        // Extract colors and indices (last 8 bytes of each block)
        let colors_indices0 = _mm_unpackhi_epi64(data0, data1); // colors+indices from block 0 and 1
        let colors_indices1 = _mm_unpackhi_epi64(data2, data3); // colors+indices from block 2 and 3

        // Split colors and indices using shuffle patterns
        // Each colors_indices contains: [color0_0, color1_0, indices0, color0_1, color1_1, indices1]
        let colors0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices0),
            _mm_castsi128_ps(colors_indices1),
            0x88, // Select lower 32-bit from each 64-bit lane
        ));
        let indices0 = _mm_castps_si128(_mm_shuffle_ps(
            _mm_castsi128_ps(colors_indices0),
            _mm_castsi128_ps(colors_indices1),
            0xDD, // Select upper 32-bit from each 64-bit lane
        ));

        // Apply decorrelation to colors
        let colors0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_sse2(colors0),
            2 => decorrelate_ycocg_r_var2_sse2(colors0),
            3 => decorrelate_ycocg_r_var3_sse2(colors0),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm_storeu_si128(alphas_out as *mut __m128i, alphas0);
        _mm_storeu_si128((alphas_out as *mut __m128i).add(1), alphas1);
        _mm_storeu_si128(colors_out as *mut __m128i, colors0);
        _mm_storeu_si128(indices_out as *mut __m128i, indices0);

        alphas_out = alphas_out.add(4); // 4 u64s = 32 bytes
        colors_out = colors_out.add(4); // 4 u32s = 16 bytes
        indices_out = indices_out.add(4); // 4 u32s = 16 bytes
    }

    // Handle any remaining blocks
    let remaining_blocks = num_blocks % 4;
    if remaining_blocks > 0 {
        let variant_enum = match VARIANT {
            1 => YCoCgVariant::Variant1,
            2 => YCoCgVariant::Variant2,
            3 => YCoCgVariant::Variant3,
            _ => unreachable_unchecked(),
        };
        generic::transform_with_decorrelate_generic(
            input_ptr,
            alphas_out,
            colors_out,
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
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    alphas_out: *mut u64,
    colors_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<1>(input_ptr, alphas_out, colors_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
#[inline]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    alphas_out: *mut u64,
    colors_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<2>(input_ptr, alphas_out, colors_out, indices_out, num_blocks)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "sse2")]
#[inline]
pub(crate) unsafe fn transform_decorr_var3(
    input_ptr: *const u8,
    alphas_out: *mut u64,
    colors_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<3>(input_ptr, alphas_out, colors_out, indices_out, num_blocks)
}

// Runtime dispatcher
#[inline(always)]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    alphas_out: *mut u64,
    colors_out: *mut u32,
    indices_out: *mut u32,
    num_blocks: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => {
            transform_decorr_var1(input_ptr, alphas_out, colors_out, indices_out, num_blocks)
        }
        YCoCgVariant::Variant2 => {
            transform_decorr_var2(input_ptr, alphas_out, colors_out, indices_out, num_blocks)
        }
        YCoCgVariant::Variant3 => {
            transform_decorr_var3(input_ptr, alphas_out, colors_out, indices_out, num_blocks)
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 8)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 8)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 8)]
    fn sse2_transform_roundtrip(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_sse2() {
            return;
        }
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "SSE2");
    }
}
