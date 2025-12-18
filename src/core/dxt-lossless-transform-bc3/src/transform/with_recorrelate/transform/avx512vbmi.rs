#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use dxt_lossless_transform_common::color_565::YCoCgVariant;
// Use AVX2 decorrelation since we process 8 blocks at a time (32 bytes = 256 bits of color data)
use dxt_lossless_transform_common::intrinsics::color_565::decorrelate::avx2::{
    decorrelate_ycocg_r_var1_avx2, decorrelate_ycocg_r_var2_avx2, decorrelate_ycocg_r_var3_avx2,
};

use super::generic::transform_decorr_var1 as generic_var1;
use super::generic::transform_decorr_var2 as generic_var2;
use super::generic::transform_decorr_var3 as generic_var3;

/// AVX512VBMI implementation for BC3 transform with YCoCg-R decorrelation.
///
/// Processes 8 blocks (128 bytes) at a time using permute instructions for all
/// component extraction, then applies YCoCg-R decorrelation to colors.
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512f")]
#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
unsafe fn transform_decorr<const VARIANT: u8>(
    mut input_ptr: *const u8,
    mut alpha_endpoints_out: *mut u16,
    mut alpha_indices_out: *mut u16,
    mut colors_out: *mut u32,
    mut color_indices_out: *mut u32,
    num_blocks: usize,
) {
    // Process 8 blocks (128 bytes) at a time
    let mut aligned_len = (num_blocks / 8) * 128;
    // The writes to alpha_indices_out overflows as it uses a 64-byte register to write 48 bytes
    // of data, so we need to subtract 128 bytes (8 blocks) to avoid overflowing
    aligned_len = aligned_len.saturating_sub(128);
    let remaining_blocks = num_blocks - (aligned_len / 16);
    let input_aligned_end_ptr = input_ptr.add(aligned_len);

    // Permute to lift out the alpha endpoints from the read blocks.
    #[rustfmt::skip]
    let alpha_bytes_permute_mask: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        1 + (16 * 7),
        0 + (16 * 7), // block 7
        1 + (16 * 6),
        0 + (16 * 6), // block 6
        1 + (16 * 5),
        0 + (16 * 5), // block 5
        1 + (16 * 4),
        0 + (16 * 4), // block 4
        1 + (16 * 3),
        0 + (16 * 3), // block 3
        1 + (16 * 2),
        0 + (16 * 2), // block 2
        1 + (16 * 1),
        0 + (16 * 1), // block 1
        1 + (16 * 0),
        0 + (16 * 0), // block 0
    );

    // Permute to lift out the alpha indices (6 bytes per block)
    #[rustfmt::skip]
    let alpha_bits_permute_mask: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        7 + (16 * 7),
        6 + (16 * 7),
        5 + (16 * 7),
        4 + (16 * 7),
        3 + (16 * 7),
        2 + (16 * 7), // block 7
        7 + (16 * 6),
        6 + (16 * 6),
        5 + (16 * 6),
        4 + (16 * 6),
        3 + (16 * 6),
        2 + (16 * 6), // block 6
        7 + (16 * 5),
        6 + (16 * 5),
        5 + (16 * 5),
        4 + (16 * 5),
        3 + (16 * 5),
        2 + (16 * 5), // block 5
        7 + (16 * 4),
        6 + (16 * 4),
        5 + (16 * 4),
        4 + (16 * 4),
        3 + (16 * 4),
        2 + (16 * 4), // block 4
        7 + (16 * 3),
        6 + (16 * 3),
        5 + (16 * 3),
        4 + (16 * 3),
        3 + (16 * 3),
        2 + (16 * 3), // block 3
        7 + (16 * 2),
        6 + (16 * 2),
        5 + (16 * 2),
        4 + (16 * 2),
        3 + (16 * 2),
        2 + (16 * 2), // block 2
        7 + (16 * 1),
        6 + (16 * 1),
        5 + (16 * 1),
        4 + (16 * 1),
        3 + (16 * 1),
        2 + (16 * 1), // block 1
        7 + (16 * 0),
        6 + (16 * 0),
        5 + (16 * 0),
        4 + (16 * 0),
        3 + (16 * 0),
        2 + (16 * 0), // block 0
    );

    // Permute to lift out the color endpoints (4 bytes per block, offset 8-11)
    #[rustfmt::skip]
    let color_bytes_permute_mask: __m512i = _mm512_set_epi8(
       0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        11 + (16 * 7),
        10 + (16 * 7),
        9 + (16 * 7),
        8 + (16 * 7), // block 7
        11 + (16 * 6),
        10 + (16 * 6),
        9 + (16 * 6),
        8 + (16 * 6), // block 6
        11 + (16 * 5),
        10 + (16 * 5),
        9 + (16 * 5),
        8 + (16 * 5), // block 5
        11 + (16 * 4),
        10 + (16 * 4),
        9 + (16 * 4),
        8 + (16 * 4), // block 4
        11 + (16 * 3),
        10 + (16 * 3),
        9 + (16 * 3),
        8 + (16 * 3), // block 3
        11 + (16 * 2),
        10 + (16 * 2),
        9 + (16 * 2),
        8 + (16 * 2), // block 2
        11 + (16 * 1),
        10 + (16 * 1),
        9 + (16 * 1),
        8 + (16 * 1), // block 1
        11 + (16 * 0),
        10 + (16 * 0),
        9 + (16 * 0),
        8 + (16 * 0), // block 0
    );

    // Permute to lift out the color indices (4 bytes per block, offset 12-15)
    #[rustfmt::skip]
    let index_bytes_permute_mask: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        15 + (16 * 7),
        14 + (16 * 7),
        13 + (16 * 7),
        12 + (16 * 7), // block 7
        15 + (16 * 6),
        14 + (16 * 6),
        13 + (16 * 6),
        12 + (16 * 6), // block 6
        15 + (16 * 5),
        14 + (16 * 5),
        13 + (16 * 5),
        12 + (16 * 5), // block 5
        15 + (16 * 4),
        14 + (16 * 4),
        13 + (16 * 4),
        12 + (16 * 4), // block 4
        15 + (16 * 3),
        14 + (16 * 3),
        13 + (16 * 3),
        12 + (16 * 3), // block 3
        15 + (16 * 2),
        14 + (16 * 2),
        13 + (16 * 2),
        12 + (16 * 2), // block 2
        15 + (16 * 1),
        14 + (16 * 1),
        13 + (16 * 1),
        12 + (16 * 1), // block 1
        15 + (16 * 0),
        14 + (16 * 0),
        13 + (16 * 0),
        12 + (16 * 0), // block 0
    );

    while input_ptr < input_aligned_end_ptr {
        // Read 8 blocks (128 bytes)
        let block_0 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let block_1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        input_ptr = input_ptr.add(128); // Move forward 8 blocks

        // Extract components using permute
        let alpha_bytes = _mm512_permutex2var_epi8(block_0, alpha_bytes_permute_mask, block_1);
        let alpha_bits = _mm512_permutex2var_epi8(block_0, alpha_bits_permute_mask, block_1);
        let color_bytes = _mm512_permutex2var_epi8(block_0, color_bytes_permute_mask, block_1);
        let index_bytes = _mm512_permutex2var_epi8(block_0, index_bytes_permute_mask, block_1);

        // Apply decorrelation to colors (only the lower 256 bits contain data)
        let colors_ymm = _mm512_castsi512_si256(color_bytes);
        let colors_decorr = match VARIANT {
            1 => decorrelate_ycocg_r_var1_avx2(colors_ymm),
            2 => decorrelate_ycocg_r_var2_avx2(colors_ymm),
            3 => decorrelate_ycocg_r_var3_avx2(colors_ymm),
            _ => unreachable_unchecked(),
        };

        // Store alpha endpoints (16 bytes = 2 bytes × 8 blocks)
        _mm_storeu_si128(
            alpha_endpoints_out as *mut __m128i,
            _mm512_castsi512_si128(alpha_bytes),
        );

        // Store alpha indices (48 bytes = 6 bytes × 8 blocks, using zmm with overlap)
        _mm512_storeu_si512(alpha_indices_out as *mut __m512i, alpha_bits);

        // Store decorrelated colors (32 bytes = 4 bytes × 8 blocks)
        _mm256_storeu_si256(colors_out as *mut __m256i, colors_decorr);

        // Store color indices (32 bytes = 4 bytes × 8 blocks)
        _mm256_storeu_si256(
            color_indices_out as *mut __m256i,
            _mm512_castsi512_si256(index_bytes),
        );

        // Update pointers
        alpha_endpoints_out = alpha_endpoints_out.add(8); // 16 bytes = 8 u16s
        alpha_indices_out = alpha_indices_out.add(24); // 48 bytes = 24 u16s
        colors_out = colors_out.add(8); // 32 bytes = 8 u32s
        color_indices_out = color_indices_out.add(8); // 32 bytes = 8 u32s
    }

    // Process any remaining blocks (less than 8)
    if remaining_blocks > 0 {
        match VARIANT {
            1 => generic_var1(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                remaining_blocks,
            ),
            2 => generic_var2(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                remaining_blocks,
            ),
            3 => generic_var3(
                input_ptr,
                alpha_endpoints_out,
                alpha_indices_out,
                colors_out,
                color_indices_out,
                remaining_blocks,
            ),
            _ => unreachable_unchecked(),
        }
    }
}

// Wrappers for asm inspection and variant dispatch
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512f")]
#[inline]
pub(crate) unsafe fn transform_decorr_var1(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<1>(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512f")]
#[inline]
pub(crate) unsafe fn transform_decorr_var2(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<2>(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
    )
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512f")]
#[inline]
pub(crate) unsafe fn transform_decorr_var3(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
) {
    transform_decorr::<3>(
        input_ptr,
        alpha_endpoints_out,
        alpha_indices_out,
        colors_out,
        color_indices_out,
        num_blocks,
    )
}

// Runtime dispatcher
#[inline(always)]
pub(crate) unsafe fn transform_with_decorrelate(
    input_ptr: *const u8,
    alpha_endpoints_out: *mut u16,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    num_blocks: usize,
    variant: YCoCgVariant,
) {
    match variant {
        YCoCgVariant::Variant1 => transform_decorr_var1(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            num_blocks,
        ),
        YCoCgVariant::Variant2 => transform_decorr_var2(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            num_blocks,
        ),
        YCoCgVariant::Variant3 => transform_decorr_var3(
            input_ptr,
            alpha_endpoints_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            num_blocks,
        ),
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
    fn avx512vbmi_transform_roundtrip(
        #[case] func: WithDecorrelateTransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512vbmi() {
            return;
        }
        run_with_decorrelate_transform_roundtrip_test(func, variant, max_blocks, "AVX512VBMI");
    }
}
