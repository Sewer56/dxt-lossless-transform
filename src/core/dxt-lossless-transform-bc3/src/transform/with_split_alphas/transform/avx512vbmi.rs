#![allow(missing_docs)]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use super::generic::transform_with_split_alphas as generic_transform;

/// # Safety
///
/// - `input_ptr` must be valid for reads of `block_count * 16` bytes
/// - `alpha0_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha1_out` must be valid for writes of `block_count * 1` bytes
/// - `alpha_indices_out` must be valid for writes of `block_count * 6` bytes
/// - `colors_out` must be valid for writes of `block_count * 4` bytes
/// - `color_indices_out` must be valid for writes of `block_count * 4` bytes
/// - All output buffers must not overlap with each other or the input buffer
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512f")]
#[allow(clippy::identity_op)]
#[allow(clippy::erasing_op)]
pub(crate) unsafe fn transform_with_split_alphas(
    mut input_ptr: *const u8,
    mut alpha0_out: *mut u8,
    mut alpha1_out: *mut u8,
    mut alpha_indices_out: *mut u16,
    mut colors_out: *mut u32,
    mut color_indices_out: *mut u32,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 8 blocks (128 bytes) at a time
    let len = block_count * 16;
    let mut aligned_len = len - (len % 128);
    // The writes to `alpha_indices_out` overflow as it uses a 64-bit register to write 48-bits
    // of data. Guard against this by leaving the last iteration for the generic fallback.
    aligned_len = aligned_len.saturating_sub(128);
    let remaining_blocks = (len - aligned_len) / 16;
    let input_aligned_end_ptr = input_ptr.add(aligned_len);

    // Permute mask to extract alpha0 bytes (offset 0 from each of 8 blocks)
    // Each block is 16 bytes, so alpha0 is at offsets: 0, 16, 32, 48, 64, 80, 96, 112
    // We extract 8 bytes into the low part of the result
    #[rustfmt::skip]
    let alpha0_permute_mask: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0 + (16 * 7), // block 7 alpha0
        0 + (16 * 6), // block 6 alpha0
        0 + (16 * 5), // block 5 alpha0
        0 + (16 * 4), // block 4 alpha0
        0 + (16 * 3), // block 3 alpha0
        0 + (16 * 2), // block 2 alpha0
        0 + (16 * 1), // block 1 alpha0
        0 + (16 * 0), // block 0 alpha0
    );

    // Permute mask to extract alpha1 bytes (offset 1 from each of 8 blocks)
    #[rustfmt::skip]
    let alpha1_permute_mask: __m512i = _mm512_set_epi8(
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,
        1 + (16 * 7), // block 7 alpha1
        1 + (16 * 6), // block 6 alpha1
        1 + (16 * 5), // block 5 alpha1
        1 + (16 * 4), // block 4 alpha1
        1 + (16 * 3), // block 3 alpha1
        1 + (16 * 2), // block 2 alpha1
        1 + (16 * 1), // block 1 alpha1
        1 + (16 * 0), // block 0 alpha1
    );

    // Permute mask to extract alpha indices (offsets 2-7 from each of 8 blocks)
    // 6 bytes per block = 48 bytes total for 8 blocks
    #[rustfmt::skip]
    let alpha_indices_permute_mask: __m512i = _mm512_set_epi8(
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

    // Permute mask to extract colors (offsets 8-11 from each of 8 blocks)
    // 4 bytes per block = 32 bytes total for 8 blocks
    #[rustfmt::skip]
    let colors_permute_mask: __m512i = _mm512_set_epi8(
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

    // Permute mask to extract color indices (offsets 12-15 from each of 8 blocks)
    // 4 bytes per block = 32 bytes total for 8 blocks
    #[rustfmt::skip]
    let color_indices_permute_mask: __m512i = _mm512_set_epi8(
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
        // Read 8 blocks (128 bytes) using two 64-byte loads
        let block_0 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let block_1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        input_ptr = input_ptr.add(128);

        // Extract each component using byte permutation
        let alpha0_bytes = _mm512_permutex2var_epi8(block_0, alpha0_permute_mask, block_1);
        let alpha1_bytes = _mm512_permutex2var_epi8(block_0, alpha1_permute_mask, block_1);
        let alpha_indices = _mm512_permutex2var_epi8(block_0, alpha_indices_permute_mask, block_1);
        let colors = _mm512_permutex2var_epi8(block_0, colors_permute_mask, block_1);
        let color_indices = _mm512_permutex2var_epi8(block_0, color_indices_permute_mask, block_1);

        // Store alpha0 (8 bytes) - use storel to write low 64 bits
        _mm_storel_epi64(
            alpha0_out as *mut __m128i,
            _mm512_castsi512_si128(alpha0_bytes),
        );
        alpha0_out = alpha0_out.add(8);

        // Store alpha1 (8 bytes) - use storel to write low 64 bits
        _mm_storel_epi64(
            alpha1_out as *mut __m128i,
            _mm512_castsi512_si128(alpha1_bytes),
        );
        alpha1_out = alpha1_out.add(8);

        // Store alpha indices (48 bytes) - use full ZMM store (with overflow guard from aligned_len)
        _mm512_storeu_si512(alpha_indices_out as *mut __m512i, alpha_indices);
        alpha_indices_out = (alpha_indices_out as *mut u8).add(48) as *mut u16;

        // Store colors (32 bytes) - use YMM store
        _mm256_storeu_si256(colors_out as *mut __m256i, _mm512_castsi512_si256(colors));
        colors_out = colors_out.add(8);

        // Store color indices (32 bytes) - use YMM store
        _mm256_storeu_si256(
            color_indices_out as *mut __m256i,
            _mm512_castsi512_si256(color_indices),
        );
        color_indices_out = color_indices_out.add(8);
    }

    // Process any remaining blocks using generic fallback
    if remaining_blocks > 0 {
        generic_transform(
            input_ptr,
            alpha0_out,
            alpha1_out,
            alpha_indices_out,
            colors_out,
            color_indices_out,
            remaining_blocks,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn test_avx512_roundtrip() {
        if !has_avx512vbmi() {
            return;
        }

        // For AVX512: processes 128 bytes (8 blocks) per iteration, so max_blocks = 128 bytes × 2 ÷ 16 = 16
        run_split_alphas_transform_roundtrip_test(transform_with_split_alphas, 16, "avx512vbmi");
    }
}
