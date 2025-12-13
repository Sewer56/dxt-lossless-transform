use crate::transform::with_recorrelate::transform::generic;
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

// For colors: take 3rd dword from each block
const PERM_COLORS_BYTES: [i8; 16] = [
    0, 4, 8, 12, 2, 6, 10, 14, // + 16 below
    16, 20, 24, 28, 18, 22, 26, 30,
]; // For vpermt2d to gather color values
const PERM_INDICES_BYTES: [i8; 16] = [
    1, 5, 9, 13, 3, 7, 11, 15, // +16 below
    17, 21, 25, 29, 19, 23, 27, 31,
]; // For vpermt2d to gather index values

#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alphas_out: *mut u64,
    mut colors_out: *mut u32,
    mut indices_out: *mut u32,
    num_blocks: usize,
) {
    // Load permutation patterns
    let perm_alpha = _mm512_cvtepi8_epi64(_mm_loadl_epi64(PERM_ALPHA_BYTES.as_ptr() as *const _));
    let perm_colors = _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_COLORS_BYTES.as_ptr() as *const _));
    let perm_indices =
        _mm512_cvtepi8_epi32(_mm_loadu_epi8(PERM_INDICES_BYTES.as_ptr() as *const _));

    // Process 16 BC2 blocks at a time = 256 bytes
    let vectorized_blocks = num_blocks & !15; // Round down to multiple of 16
    let input_end = input_ptr.add(vectorized_blocks * 16); // 16 bytes per block

    while input_ptr < input_end {
        // Load 256 bytes (16 blocks)
        let blocks_0 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let blocks_1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        let blocks_2 = _mm512_loadu_si512(input_ptr.add(128) as *const __m512i);
        let blocks_3 = _mm512_loadu_si512(input_ptr.add(192) as *const __m512i);

        // Update input pointer
        input_ptr = input_ptr.add(256);

        // Filter out the alphas only using vpermt2q
        let alphas_0 = _mm512_permutex2var_epi64(blocks_0, perm_alpha, blocks_1);
        let alphas_1 = _mm512_permutex2var_epi64(blocks_2, perm_alpha, blocks_3);

        // Lift out colours and indices only
        let colours_indices_only_b0 = _mm512_unpackhi_epi64(blocks_0, blocks_1);
        let colours_indices_only_b1 = _mm512_unpackhi_epi64(blocks_2, blocks_3);

        // Permute to separate colors and indices
        let colours_only = _mm512_permutex2var_epi32(
            colours_indices_only_b0,
            perm_colors,
            colours_indices_only_b1,
        ); // colours
        let indices_only = _mm512_permutex2var_epi32(
            colours_indices_only_b0,
            perm_indices,
            colours_indices_only_b1,
        ); // indices

        // Decorrelate colors
        let colors_decorr = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx512bw(colours_only),
            2 => decorrelate_ycocg_r_var2_avx512bw(colours_only),
            3 => decorrelate_ycocg_r_var3_avx512bw(colours_only),
            _ => unreachable_unchecked(),
        };

        // Store results
        _mm512_storeu_si512(alphas_out as *mut __m512i, alphas_0); // alphas 0
        _mm512_storeu_si512(
            alphas_out.add(64 / size_of::<u64>()) as *mut __m512i,
            alphas_1,
        ); // alphas 1
        _mm512_storeu_si512(colors_out as *mut __m512i, colors_decorr); // colours
        _mm512_storeu_si512(indices_out as *mut __m512i, indices_only); // indices

        // Update pointers
        alphas_out = alphas_out.add(16); // 16 blocks
        colors_out = colors_out.add(16);
        indices_out = indices_out.add(16);
    }

    // Handle remaining blocks
    let remaining_blocks = num_blocks % 16;
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
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
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
    #[case(transform_decorr_var1, YCoCgVariant::Variant1, 32)]
    #[case(transform_decorr_var2, YCoCgVariant::Variant2, 32)]
    #[case(transform_decorr_var3, YCoCgVariant::Variant3, 32)]
    fn avx512_transform_roundtrip(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512bw() {
            return;
        }
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "AVX512");
    }
}
