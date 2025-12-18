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

/// AVX512VBMI implementation of BC3 untransform with YCoCg-R recorrelation.
///
/// # Safety
///
/// - alpha_endpoints_in must be valid for reads of num_blocks * 2 bytes
/// - alpha_indices_in must be valid for reads of num_blocks * 6 bytes
/// - colors_in must be valid for reads of num_blocks * 4 bytes
/// - color_indices_in must be valid for reads of num_blocks * 4 bytes
/// - output_ptr must be valid for writes of num_blocks * 16 bytes
/// - recorrelation_mode must be a valid [`YCoCgVariant`]
#[inline]
pub(crate) unsafe fn untransform_with_recorrelate(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    recorrelation_mode: YCoCgVariant,
) {
    match recorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(
                alpha_endpoints_in,
                alpha_indices_in,
                colors_in,
                color_indices_in,
                output_ptr,
                num_blocks,
            );
        }
        YCoCgVariant::None => unreachable_unchecked(),
    }
}

// Wrapper functions for assembly inspection using `cargo asm`
unsafe fn untransform_recorr_var1(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(target_arch = "x86_64")]
    {
        untransform_recorr_64::<1>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
    #[cfg(target_arch = "x86")]
    {
        untransform_recorr_32::<1>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
}

unsafe fn untransform_recorr_var2(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(target_arch = "x86_64")]
    {
        untransform_recorr_64::<2>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
    #[cfg(target_arch = "x86")]
    {
        untransform_recorr_32::<2>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
}

unsafe fn untransform_recorr_var3(
    alpha_endpoints_in: *const u16,
    alpha_indices_in: *const u16,
    colors_in: *const u32,
    color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    #[cfg(target_arch = "x86_64")]
    {
        untransform_recorr_64::<3>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
    #[cfg(target_arch = "x86")]
    {
        untransform_recorr_32::<3>(
            alpha_endpoints_in,
            alpha_indices_in,
            colors_in,
            color_indices_in,
            output_ptr,
            num_blocks,
        )
    }
}

/// 64-bit optimized AVX512VBMI implementation processing 32 blocks per iteration.
#[cfg(any(target_arch = "x86_64", feature = "bench", test))]
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[target_feature(enable = "avx512vbmi")]
unsafe fn untransform_recorr_64<const VARIANT: u8>(
    mut alpha_endpoints_in: *const u16,
    mut alpha_indices_in: *const u16,
    mut colors_in: *const u32,
    mut color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    const BYTES_PER_ITERATION: usize = 512; // 32 blocks * 16 bytes
    let aligned_blocks = (num_blocks / 32) * 32;
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

    #[rustfmt::skip]
    let blocks_1_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        47+64,46+64, 45+64,44+64,43+64,42+64, // alpha bits 3
        15,14, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        41+64,40+64,39+64,38+64,37+64,36+64, // alpha bits 2
        13,12, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        35+64,34+64,33+64,32+64,31+64,30+64, // alpha bits 1
        11,10, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        29+64,28+64,27+64,26+64,25+64,24+64, // alpha bits 0
        9,8 // alpha bytes 0
    );

    #[rustfmt::skip]
    let blocks_2_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        23+64,22+64, 21+64,20+64,19+64,18+64, // alpha bits 3
        23,22, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        17+64,16+64,15+64,14+64,13+64,12+64, // alpha bits 2
        21,20, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        11+64,10+64,9+64,8+64,7+64,6+64, // alpha bits 1
        19,18, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        5+64,4+64,3+64,2+64,1+64,0+64, // alpha bits 0
        17,16 // alpha bytes 0
    );

    #[rustfmt::skip]
    let blocks_3_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        47+64,46+64, 45+64,44+64,43+64,42+64, // alpha bits 3
        31,30, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        41+64,40+64,39+64,38+64,37+64,36+64, // alpha bits 2
        29,28, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        35+64,34+64,33+64,32+64,31+64,30+64, // alpha bits 1
        27,26, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        29+64,28+64,27+64,26+64,25+64,24+64, // alpha bits 0
        25,24 // alpha bytes 0
    );

    #[rustfmt::skip]
    let blocks_4_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        23+64,22+64, 21+64,20+64,19+64,18+64, // alpha bits 3
        39,38, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        17+64,16+64,15+64,14+64,13+64,12+64, // alpha bits 2
        37,36, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        11+64,10+64,9+64,8+64,7+64,6+64, // alpha bits 1
        35,34, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        5+64,4+64,3+64,2+64,1+64,0+64, // alpha bits 0
        33,32 // alpha bytes 0
    );

    #[rustfmt::skip]
    let blocks_5_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        47+64,46+64, 45+64,44+64,43+64,42+64, // alpha bits 3
        47,46, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        41+64,40+64,39+64,38+64,37+64,36+64, // alpha bits 2
        45,44, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        35+64,34+64,33+64,32+64,31+64,30+64, // alpha bits 1
        43,42, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        29+64,28+64,27+64,26+64,25+64,24+64, // alpha bits 0
        41,40 // alpha bytes 0
    );

    #[rustfmt::skip]
    let blocks_6_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        23+64,22+64, 21+64,20+64,19+64,18+64, // alpha bits 3
        55,54, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        17+64,16+64,15+64,14+64,13+64,12+64, // alpha bits 2
        53,52, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        11+64,10+64,9+64,8+64,7+64,6+64, // alpha bits 1
        51,50, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        5+64,4+64,3+64,2+64,1+64,0+64, // alpha bits 0
        49,48 // alpha bytes 0
    );

    #[rustfmt::skip]
    let blocks_7_perm_alphabits: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,
        47+64,46+64, 45+64,44+64,43+64,42+64, // alpha bits 3
        63,62, // alpha bytes 3
        0,0,0,0,0,0,0,0,
        41+64,40+64,39+64,38+64,37+64,36+64, // alpha bits 2
        61,60, // alpha bytes 2
        0,0,0,0,0,0,0,0,
        35+64,34+64,33+64,32+64,31+64,30+64, // alpha bits 1
        59,58, // alpha bytes 1
        0,0, 0,0,0,0,0,0,
        29+64,28+64,27+64,26+64,25+64,24+64, // alpha bits 0
        57,56 // alpha bytes 0
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

    #[rustfmt::skip]
    let blocks_1_perm_colours: __m512i = _mm512_set_epi8(
        0,0,0,0,
        31+64,30+64,29+64,28+64, // colours 3
        55,54, 53,52,51,50, 49,48, // existing bytes 3
        0,0,0,0,
        27+64,26+64,25+64,24+64, // colours 2
        39,38,37,36,35,34, 33,32, // existing bytes 2
        0,0,0,0,
        23+64,22+64,21+64,20+64, // colours 1
        23,22,21,20,19,18, 17,16, // existing bytes 1
        0,0, 0,0,
        19+64,18+64,17+64,16+64, // colours 0
        7,6,5,4,3,2,1,0 // existing bytes 0
    );

    #[rustfmt::skip]
    let blocks_2_perm_colours: __m512i = _mm512_set_epi8(
        0,0,0,0,
        47+64,46+64,45+64,44+64, // colours 3
        55,54, 53,52,51,50, 49,48, // existing bytes 3
        0,0,0,0,
        43+64,42+64,41+64,40+64, // colours 2
        39,38,37,36,35,34, 33,32, // existing bytes 2
        0,0,0,0,
        39+64,38+64,37+64,36+64, // colours 1
        23,22,21,20,19,18, 17,16, // existing bytes 1
        0,0, 0,0,
        35+64,34+64,33+64,32+64, // colours 0
        7,6,5,4,3,2,1,0 // existing bytes 0
    );

    #[rustfmt::skip]
    let blocks_3_perm_colours: __m512i = _mm512_set_epi8(
        0,0,0,0,
        63+64,62+64,61+64,60+64, // colours 3
        55,54, 53,52,51,50, 49,48, // existing bytes 3
        0,0,0,0,
        59+64,58+64,57+64,56+64, // colours 2
        39,38,37,36,35,34, 33,32, // existing bytes 2
        0,0,0,0,
        55+64,54+64,53+64,52+64, // colours 1
        23,22,21,20,19,18, 17,16, // existing bytes 1
        0,0, 0,0,
        51+64,50+64,49+64,48+64, // colours 0
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

    #[rustfmt::skip]
    let blocks_1_perm_indices: __m512i = _mm512_set_epi8(
        31+64,30+64,29+64,28+64, // indices 3
        59,58,57,56, 55,54, 53,52,51,50, 49,48, // existing bytes 3
        27+64,26+64,25+64,24+64, // indices 2
        43,42,41,40, 39,38,37,36,35,34, 33,32, // existing bytes 2
        23+64,22+64,21+64,20+64, // indices 1
        27,26,25,24, 23,22,21,20,19,18, 17,16, // existing bytes 1
        19+64,18+64,17+64,16+64, // indices 0
        11,10, 9,8, 7,6,5,4,3,2,1,0 // existing bytes 0
    );

    #[rustfmt::skip]
    let blocks_2_perm_indices: __m512i = _mm512_set_epi8(
        47+64,46+64,45+64,44+64, // indices 3
        59,58,57,56, 55,54, 53,52,51,50, 49,48, // existing bytes 3
        43+64,42+64,41+64,40+64, // indices 2
        43,42,41,40, 39,38,37,36,35,34, 33,32, // existing bytes 2
        39+64,38+64,37+64,36+64, // indices 1
        27,26,25,24, 23,22,21,20,19,18, 17,16, // existing bytes 1
        35+64,34+64,33+64,32+64, // indices 0
        11,10, 9,8, 7,6,5,4,3,2,1,0 // existing bytes 0
    );

    #[rustfmt::skip]
    let blocks_3_perm_indices: __m512i = _mm512_set_epi8(
        63+64,62+64,61+64,60+64, // indices 3
        59,58,57,56, 55,54, 53,52,51,50, 49,48, // existing bytes 3
        59+64,58+64,57+64,56+64, // indices 2
        43,42,41,40, 39,38,37,36,35,34, 33,32, // existing bytes 2
        55+64,54+64,53+64,52+64, // indices 1
        27,26,25,24, 23,22,21,20,19,18, 17,16, // existing bytes 1
        51+64,50+64,49+64,48+64, // indices 0
        11,10, 9,8, 7,6,5,4,3,2,1,0 // existing bytes 0
    );

    let mut current_output_ptr = output_ptr;

    while (alpha_byte_in_ptr as *const u16) < alpha_endpoints_end {
        // The alpha bytes for 32 blocks
        let alpha_bytes_0 = _mm512_loadu_si512(alpha_byte_in_ptr as *const __m512i);
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(64);

        // The colors for 32 blocks (16 blocks per read) - apply recorrelation
        let colors_0_raw = _mm512_loadu_si512(color_byte_in_ptr as *const __m512i);
        let colors_1_raw = _mm512_loadu_si512(color_byte_in_ptr.add(64) as *const __m512i);

        // Apply recorrelation to colors
        let colors_0 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(colors_0_raw),
            2 => recorrelate_ycocg_r_var2_avx512bw(colors_0_raw),
            3 => recorrelate_ycocg_r_var3_avx512bw(colors_0_raw),
            _ => unreachable_unchecked(),
        };
        let colors_1 = match VARIANT {
            1 => recorrelate_ycocg_r_var1_avx512bw(colors_1_raw),
            2 => recorrelate_ycocg_r_var2_avx512bw(colors_1_raw),
            3 => recorrelate_ycocg_r_var3_avx512bw(colors_1_raw),
            _ => unreachable_unchecked(),
        };
        color_byte_in_ptr = color_byte_in_ptr.add(128);

        let indices_0 = _mm512_loadu_si512(index_byte_in_ptr as *const __m512i);
        let indices_1 = _mm512_loadu_si512(index_byte_in_ptr.add(64) as *const __m512i);
        index_byte_in_ptr = index_byte_in_ptr.add(128);

        // The alpha bits for 32 blocks (8 blocks per read)
        let alpha_bit_0 = _mm512_loadu_si512(alpha_bit_in_ptr as *const __m512i);
        let alpha_bit_1 = _mm512_loadu_si512(alpha_bit_in_ptr.add(48) as *const __m512i);
        let alpha_bit_2 = _mm512_loadu_si512(alpha_bit_in_ptr.add(96) as *const __m512i);
        let alpha_bit_3 = _mm512_loadu_si512(alpha_bit_in_ptr.add(144) as *const __m512i);
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(192);

        // Reassemble the 32 blocks
        let mut blocks_0 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_0_perm_alphabits, alpha_bit_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_colours, colors_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_indices, indices_0);

        let mut blocks_1 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_1_perm_alphabits, alpha_bit_0);
        blocks_1 = _mm512_permutex2var_epi8(blocks_1, blocks_1_perm_colours, colors_0);
        blocks_1 = _mm512_permutex2var_epi8(blocks_1, blocks_1_perm_indices, indices_0);

        let mut blocks_2 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_2_perm_alphabits, alpha_bit_1);
        blocks_2 = _mm512_permutex2var_epi8(blocks_2, blocks_2_perm_colours, colors_0);
        blocks_2 = _mm512_permutex2var_epi8(blocks_2, blocks_2_perm_indices, indices_0);

        let mut blocks_3 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_3_perm_alphabits, alpha_bit_1);
        blocks_3 = _mm512_permutex2var_epi8(blocks_3, blocks_3_perm_colours, colors_0);
        blocks_3 = _mm512_permutex2var_epi8(blocks_3, blocks_3_perm_indices, indices_0);

        let mut blocks_4 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_4_perm_alphabits, alpha_bit_2);
        blocks_4 = _mm512_permutex2var_epi8(blocks_4, blocks_0_perm_colours, colors_1);
        blocks_4 = _mm512_permutex2var_epi8(blocks_4, blocks_0_perm_indices, indices_1);

        let mut blocks_5 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_5_perm_alphabits, alpha_bit_2);
        blocks_5 = _mm512_permutex2var_epi8(blocks_5, blocks_1_perm_colours, colors_1);
        blocks_5 = _mm512_permutex2var_epi8(blocks_5, blocks_1_perm_indices, indices_1);

        let mut blocks_6 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_6_perm_alphabits, alpha_bit_3);
        blocks_6 = _mm512_permutex2var_epi8(blocks_6, blocks_2_perm_colours, colors_1);
        blocks_6 = _mm512_permutex2var_epi8(blocks_6, blocks_2_perm_indices, indices_1);

        let mut blocks_7 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_7_perm_alphabits, alpha_bit_3);
        blocks_7 = _mm512_permutex2var_epi8(blocks_7, blocks_3_perm_colours, colors_1);
        blocks_7 = _mm512_permutex2var_epi8(blocks_7, blocks_3_perm_indices, indices_1);

        // Store all 32 blocks
        _mm512_storeu_si512(current_output_ptr as *mut __m512i, blocks_0);
        _mm512_storeu_si512(current_output_ptr.add(64) as *mut __m512i, blocks_1);
        _mm512_storeu_si512(current_output_ptr.add(128) as *mut __m512i, blocks_2);
        _mm512_storeu_si512(current_output_ptr.add(192) as *mut __m512i, blocks_3);
        _mm512_storeu_si512(current_output_ptr.add(256) as *mut __m512i, blocks_4);
        _mm512_storeu_si512(current_output_ptr.add(320) as *mut __m512i, blocks_5);
        _mm512_storeu_si512(current_output_ptr.add(384) as *mut __m512i, blocks_6);
        _mm512_storeu_si512(current_output_ptr.add(448) as *mut __m512i, blocks_7);

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

/// 32-bit optimized AVX512VBMI implementation processing 4 blocks per iteration.
#[cfg(any(target_arch = "x86", feature = "bench", test))]
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[allow(dead_code)] // Used only on x86, not x86_64
#[target_feature(enable = "avx512vbmi")]
unsafe fn untransform_recorr_32<const VARIANT: u8>(
    mut alpha_endpoints_in: *const u16,
    mut alpha_indices_in: *const u16,
    mut colors_in: *const u32,
    mut color_indices_in: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    const BYTES_PER_ITERATION: usize = 64; // 4 blocks * 16 bytes
    let aligned_blocks = (num_blocks / 4) * 4;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1, 64)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2, 64)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3, 64)]
    fn avx512vbmi_untransform_roundtrip(
        #[case] func: WithRecorrelateUntransformFn,
        #[case] variant: YCoCgVariant,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512vbmi() {
            return;
        }
        run_with_recorrelate_untransform_roundtrip_test(func, variant, max_blocks, "AVX512VBMI");
    }
}
