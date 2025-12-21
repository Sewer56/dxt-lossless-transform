#![allow(missing_docs)]

#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::transform::standard::untransform::portable32::u32_untransform_with_separate_pointers;

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512vbmi")]
pub(crate) unsafe fn avx512_untransform_32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

    // Process as many 64-byte blocks as possible
    let current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2);
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8);
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12);

    avx512_untransform_separate_components_32(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len,
    );
}

/// Untransforms BC3 block data from separated components using AVX512 instructions.
/// [32-bit optimized variant]
///
/// # Arguments
///
/// * `alpha_byte_in_ptr` - Pointer to the input buffer containing alpha endpoint pairs (2 bytes per block).
/// * `alpha_bit_in_ptr` - Pointer to the input buffer containing packed alpha indices (6 bytes per block).
/// * `color_byte_in_ptr` - Pointer to the input buffer containing color endpoint pairs (packed RGB565, 4 bytes per block) and unused padding (4 bytes per block). Loaded as `__m128i`.
/// * `index_byte_in_ptr` - Pointer to the input buffer containing color indices (4 bytes per block) and unused padding (4 bytes per block). Loaded as `__m128i`.
/// * `current_output_ptr` - Pointer to the output buffer where the reconstructed BC3 blocks (16 bytes per block) will be written.
/// * `len` - The total number of bytes to write to the output buffer. Must be a multiple of 16.
///
/// # Safety
///
/// - All input pointers must be valid for reads corresponding to `len` bytes of output.
///   - `alpha_byte_in_ptr` needs `len / 16 * 2` readable bytes.
///   - `alpha_bit_in_ptr` needs `len / 16 * 6` readable bytes.
///   - `color_byte_in_ptr` needs `len / 16 * 4` readable bytes.
///   - `index_byte_in_ptr` needs `len / 16 * 4` readable bytes.
/// - `current_output_ptr` must be valid for writes for `len` bytes.
/// - `len` must be a multiple of 16 (the size of a BC3 block).
/// - Pointers do not need to be aligned; unaligned loads/reads are used.
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[target_feature(enable = "avx512vbmi")]
pub(crate) unsafe fn avx512_untransform_separate_components_32(
    mut alpha_byte_in_ptr: *const u8,
    mut alpha_bit_in_ptr: *const u8,
    mut color_byte_in_ptr: *const u8,
    mut index_byte_in_ptr: *const u8,
    mut current_output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len.is_multiple_of(16));
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
    // at a valid offset with sufficient data. Remaining blocks use scalar code.
    let aligned_len = len - (len % BYTES_PER_ITERATION);
    let alpha_byte_end_ptr = alpha_byte_in_ptr.add(aligned_len / 16 * 2);

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

    while alpha_byte_in_ptr < alpha_byte_end_ptr {
        // This is a variant of the 64-bit version that minimizes register usage for x86.
        // Same code as above, but no unroll.
        // Each zmm register stores 4 blocks.

        // The alpha bytes for 4 blocks (2 bytes * 4 blocks == 8 bytes) | (8 bytes unused/leftover)
        let alpha_bytes_0 =
            _mm512_castsi128_si512(_mm_loadu_si128(alpha_byte_in_ptr as *const __m128i));
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(8);

        // The colors and indices for 4 blocks (4 blocks * 4 bytes == 16 bytes)
        let colors_0 = _mm512_castsi128_si512(_mm_loadu_si128(color_byte_in_ptr as *const __m128i));
        color_byte_in_ptr = color_byte_in_ptr.add(16);

        let indices_0 =
            _mm512_castsi128_si512(_mm_loadu_si128(index_byte_in_ptr as *const __m128i));
        index_byte_in_ptr = index_byte_in_ptr.add(16);

        // The alpha bits for 4 blocks (4 blocks * 6 bytes == 24 bytes) | do 32 byte read
        let alpha_bit_0 =
            _mm512_castsi256_si512(_mm256_loadu_si256(alpha_bit_in_ptr as *const __m256i));
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(24);

        // 64 / 16 == 4 blocks per register.
        // Now let's reassemble the 4 blocks
        let mut blocks_0 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_0_perm_alphabits, alpha_bit_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_colours, colors_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_indices, indices_0);

        _mm512_storeu_si512(current_output_ptr as *mut __m512i, blocks_0);

        // The colors and indices for 8 blocks
        current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
    }

    // Convert pointers to the types expected by u32_untransform_with_separate_pointers
    let alpha_byte_in_ptr_u16 = alpha_byte_in_ptr as *const u16;
    let alpha_bit_in_ptr_u16 = alpha_bit_in_ptr as *const u16;
    let color_byte_in_ptr_u32 = color_byte_in_ptr as *const u32;
    let index_byte_in_ptr_u32 = index_byte_in_ptr as *const u32;

    u32_untransform_with_separate_pointers(
        alpha_byte_in_ptr_u16,
        alpha_bit_in_ptr_u16,
        color_byte_in_ptr_u32,
        index_byte_in_ptr_u32,
        current_output_ptr,
        len - aligned_len,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(avx512_untransform_32, "avx512_32", 8)] // _32 variant processes 64 bytes (4 blocks), so max_blocks = 64 × 2 ÷ 16 = 8
    fn test_avx512_unaligned(
        #[case] untransform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        if !has_avx512vbmi() {
            return;
        }

        run_standard_untransform_unaligned_test(untransform_fn, max_blocks, impl_name);
    }
}
