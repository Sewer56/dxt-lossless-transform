#![allow(missing_docs)]

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

/// 32-bit optimized AVX512VBMI implementation processing 4 blocks per iteration.
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512bw")]
pub(crate) unsafe fn untransform_recorr_32<const VARIANT: u8>(
    mut alpha_endpoints_in: *const u16,
    mut alpha_indices_in: *const u16,
    mut colors_in: *const u32,
    mut color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    const BLOCKS_PER_ITERATION: usize = 4;
    const BYTES_PER_ITERATION: usize = BLOCKS_PER_ITERATION * 16;

    // SAFETY: Alpha indices are 6 bytes/block. For 4 blocks we need 24 bytes,
    // but a 256-bit SIMD load reads 32 bytes, extending 8 bytes past alpha_indices.
    //
    // This is safe because transformed data is laid out contiguously as:
    //   [alpha_endpoints | alpha_indices | colors | color_indices]
    // Over-read bytes land in the colors section, not outside the buffer.
    //
    // Rounding down to BLOCKS_PER_ITERATION ensures each loop iteration starts
    // at a valid offset with sufficient data. Remaining blocks use generic code.
    let aligned_blocks = (num_blocks / BLOCKS_PER_ITERATION) * BLOCKS_PER_ITERATION;
    let alpha_endpoints_end = alpha_endpoints_in.add(aligned_blocks);

    // Convert pointers to byte pointers for reading
    let mut alpha_byte_in_ptr = alpha_endpoints_in as *const u8;
    let mut alpha_bit_in_ptr = alpha_indices_in as *const u8;
    let mut color_byte_in_ptr = colors_in as *const u8;
    let mut index_byte_in_ptr = color_indices_in as *const u8;

    // Add the alpha bits to the alpha bytes register
    #[rustfmt::skip]
    let blocks_0_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        23+64,22+64, 21+64,20+64,19+64,18+64, // alpha bits 3
        7,6, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        17+64,16+64,15+64,14+64,13+64,12+64, // alpha bits 2
        5,4, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        11+64,10+64,9+64,8+64,7+64,6+64, // alpha bits 1
        3,2, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        5+64,4+64,3+64,2+64,1+64,0+64, // alpha bits 0
        1,0 // alpha bytes 0
    );

    // Add the colours to the alpha bytes+alpha bits register
    #[rustfmt::skip]
    let blocks_0_perm_colours: __m512i = _mm512_set_epi8(
        0,0,0,0,
        15+64,14+64,13+64,12+64, // colours 3
        55,54, 53,52,51,50, 49,48, // existing bytes 3
        0,0,0,0,
        11+64,10+64,9+64,8+64, // colours 2
        39,38,37,36,35,34, 33,32, // existing bytes 2
        0,0,0,0,
        7+64,6+64,5+64,4+64, // colours 1
        23,22,21,20,19,18, 17,16, // existing bytes 1
        0,0, 0,0,
        3+64,2+64,1+64,0+64, // colours 0
        7,6,5,4,3,2,1,0 // existing bytes 0
    );

    // Add the indices to the alpha bytes+alpha bits+colours register
    #[rustfmt::skip]
    let blocks_0_perm_indices: __m512i = _mm512_set_epi8(
        15+64,14+64,13+64,12+64, // indices 3
        59,58,57,56, 55,54, 53,52,51,50, 49,48, // existing bytes 3
        11+64,10+64,9+64,8+64, // indices 2
        43,42,41,40, 39,38,37,36,35,34, 33,32, // existing bytes 2
        7+64,6+64,5+64,4+64, // indices 1
        27,26,25,24, 23,22,21,20,19,18, 17,16, // existing bytes 1
        3+64,2+64,1+64,0+64, // indices 0
        11,10, 9,8, 7,6,5,4,3,2,1,0 // existing bytes 0
    );

    let mut current_output_ptr = output_ptr;

    while (alpha_byte_in_ptr as *const u16) < alpha_endpoints_end {
        // The alpha bytes for 4 blocks (2 bytes * 4 blocks == 8 bytes)
        let alpha_bytes_0 =
            _mm512_castsi128_si512(_mm_loadu_si128(alpha_byte_in_ptr as *const __m128i));
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(8);

        // The colors for 4 blocks (4 blocks * 4 bytes == 16 bytes) - apply recorrelation
        let colors_0_raw =
            _mm512_castsi128_si512(_mm_loadu_si128(color_byte_in_ptr as *const __m128i));
        let colors_0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(colors_0_raw),
            2 => recorrelate_ycocg_r_var2_avx512bw(colors_0_raw),
            3 => recorrelate_ycocg_r_var3_avx512bw(colors_0_raw),
            _ => unreachable_unchecked(),
        };
        color_byte_in_ptr = color_byte_in_ptr.add(16);

        let indices_0 =
            _mm512_castsi128_si512(_mm_loadu_si128(index_byte_in_ptr as *const __m128i));
        index_byte_in_ptr = index_byte_in_ptr.add(16);

        // The alpha bits for 4 blocks (4 blocks * 6 bytes == 24 bytes)
        let alpha_bit_0 =
            _mm512_castsi256_si512(_mm256_loadu_si256(alpha_bit_in_ptr as *const __m256i));
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(24);

        // Reassemble the 4 blocks
        let mut blocks_0 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_0_perm_alphabits, alpha_bit_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_colours, colors_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_indices, indices_0);

        _mm512_storeu_si512(current_output_ptr as *mut __m512i, blocks_0);
        current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
    }

    // Update pointers for remaining blocks
    alpha_endpoints_in = alpha_byte_in_ptr as *const u16;
    alpha_indices_in = alpha_bit_in_ptr as *const u16;
    colors_in = color_byte_in_ptr as *const u32;
    color_indices_in = index_byte_in_ptr as *const u32;

    // Process remaining blocks with generic implementation
    let remaining_blocks = num_blocks - aligned_blocks;
    if remaining_blocks > 0 {
        super::generic::untransform_with_recorrelate_generic(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            current_output_ptr,
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
