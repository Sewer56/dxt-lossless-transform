use crate::transform::with_recorrelate::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx2::{
    decorrelate_ycocg_r_var1_avx2, decorrelate_ycocg_r_var2_avx2, decorrelate_ycocg_r_var3_avx2,
};

#[allow(clippy::unusual_byte_groupings)]
const PERMUTE_MASK: [u32; 8] = [0, 4, 1, 5, 2, 6, 3, 7];

/// AVX2 implementation for BC2 transform with YCoCg-R decorrelation.
#[target_feature(enable = "avx2")]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alphas_out: *mut u64,
    mut colors_out: *mut u32,
    mut indices_out: *mut u32,
    num_blocks: usize,
) {
    // Process 8 BC2 blocks at a time = 128 bytes
    let num_iterations = num_blocks / 8 * 8; // 8 blocks per iteration. Divide to round down.
    let input_end = input_ptr.add(num_iterations * 16); // 16 bytes per block

    // Load the permute mask for 32-bit element reordering
    let permute_mask = _mm256_loadu_si256(PERMUTE_MASK.as_ptr() as *const __m256i);

    while input_ptr < input_end {
        // Load 8 BC2 blocks = 128 bytes
        let data0 = _mm256_loadu_si256(input_ptr as *const __m256i); // First two blocks
        let data1 = _mm256_loadu_si256(input_ptr.add(32) as *const __m256i); // Second two blocks
        let data3 = _mm256_loadu_si256(input_ptr.add(64) as *const __m256i); // Third two blocks
        let data4 = _mm256_loadu_si256(input_ptr.add(96) as *const __m256i); // Fourth two blocks
        input_ptr = input_ptr.add(128);

        // Setup scratch registers
        let data2 = data0;
        let data5 = data3;

        // Extract alphas using unpack low (vpunpcklqdq)
        let alphas0 = _mm256_unpacklo_epi64(data1, data0); // alpha -> ymm0 (out of order)
        let alphas1 = _mm256_unpacklo_epi64(data4, data3); // alpha -> ymm3 (out of order)

        // Reorder alphas to chronological order (vpermq)
        let alphas_ordered0 = _mm256_permute4x64_epi64(alphas0, 0x8D); // 10_00_11_01 -> [1,3,0,2]
        let alphas_ordered1 = _mm256_permute4x64_epi64(alphas1, 0x8D); // 10_00_11_01 -> [1,3,0,2]

        // Extract colors+indices using shuffle (vshufps)
        let colors_indices0 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data2),
            _mm256_castsi256_ps(data1),
            0xEE, // 11_10_11_10
        ));
        let colors_indices1 = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(data5),
            _mm256_castsi256_ps(data4),
            0xEE, // 11_10_11_10
        ));

        // Separate colors and indices (vshufps)
        let colors_temp = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colors_indices0),
            _mm256_castsi256_ps(colors_indices1),
            0x88, // All colors
        ));
        let indices_temp = _mm256_castps_si256(_mm256_shuffle_ps(
            _mm256_castsi256_ps(colors_indices0),
            _mm256_castsi256_ps(colors_indices1),
            0xDD, // All indices
        ));

        // Permute across lanes to get desired output (vpermd)
        let colors0 = _mm256_permutevar8x32_epi32(colors_temp, permute_mask);
        let indices0 = _mm256_permutevar8x32_epi32(indices_temp, permute_mask);

        // Apply decorrelation to colors
        let colors0 = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colors0),
            2 => decorrelate_ycocg_r_var2_avx2(colors0),
            3 => decorrelate_ycocg_r_var3_avx2(colors0),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm256_storeu_si256(alphas_out as *mut __m256i, alphas_ordered0);
        _mm256_storeu_si256(alphas_out.add(4) as *mut __m256i, alphas_ordered1);
        _mm256_storeu_si256(colors_out as *mut __m256i, colors0);
        _mm256_storeu_si256(indices_out as *mut __m256i, indices0);

        alphas_out = alphas_out.add(8); // 8 u64s = 64 bytes
        colors_out = colors_out.add(8); // 8 u32s = 32 bytes
        indices_out = indices_out.add(8); // 8 u32s = 32 bytes
    }

    // Handle remaining blocks
    let remaining = num_blocks % 8;
    if remaining > 0 {
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
            remaining,
            variant_enum,
        );
    }
}

// Wrappers for asm inspection
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx2")]
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
#[target_feature(enable = "avx2")]
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
#[target_feature(enable = "avx2")]
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
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 16)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 16)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 16)]
    fn avx2_transform_roundtrip(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx2() {
            return;
        }
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "AVX2");
    }
}
