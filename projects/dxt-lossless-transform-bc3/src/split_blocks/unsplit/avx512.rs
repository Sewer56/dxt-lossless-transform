#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use crate::split_blocks::unsplit::portable::u32_detransform_with_separate_pointers;

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512vbmi")]
pub unsafe fn avx512_detransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // Non-64bit has an optimized path, and vl + non-vl variant,
    // however it turns out it's negigible in speed difference with 64-bit path, and sometimes slower even.
    // Somehow L1 cache reads are way faster than expected.
    //#[cfg(not(target_arch = "x86_64"))]
    //avx512_detransform_32(input_ptr, output_ptr, len);

    debug_assert!(len % 16 == 0);
    // Process as many 64-byte blocks as possible
    let current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2);
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8);
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12);

    avx512_detransform_separate_components(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len,
    );
}

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512vbmi")]
pub unsafe fn avx512_detransform_32(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(target_feature = "avx512vl")]
    let is_vl_supported = true;

    #[cfg(not(target_feature = "avx512vl"))]
    let is_vl_supported = is_x86_feature_detected!("avx512vl");

    if is_vl_supported {
        avx512_detransform_32_vl(input_ptr, output_ptr, len);
    } else {
        avx512_detransform_32_vbmi(input_ptr, output_ptr, len);
    }
}

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512vl")]
pub unsafe fn avx512_detransform_32_vl(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Process as many 64-byte blocks as possible
    let current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2);
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8);
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12);

    avx512_detransform_separate_components_32_vl(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len,
    );
}

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "avx512vbmi")]
pub unsafe fn avx512_detransform_32_vbmi(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Process as many 64-byte blocks as possible
    let current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2);
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8);
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12);

    avx512_detransform_separate_components_32(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len,
    );
}

/// Detransforms BC3 block data from separated components using AVX512 instructions.
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
///   - `color_byte_in_ptr` needs `len / 16 * 8` readable bytes.
///   - `index_byte_in_ptr` needs `len / 16 * 8` readable bytes.
/// - `current_output_ptr` must be valid for writes for `len` bytes.
/// - `len` must be a multiple of 16 (the size of a BC3 block).
/// - Pointers do not need to be aligned; unaligned loads/reads are used.
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[target_feature(enable = "avx512vbmi")]
pub unsafe fn avx512_detransform_separate_components(
    mut alpha_byte_in_ptr: *const u8,
    mut alpha_bit_in_ptr: *const u8,
    mut color_byte_in_ptr: *const u8,
    mut index_byte_in_ptr: *const u8,
    mut current_output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);
    const BYTES_PER_ITERATION: usize = 512;
    // We drop some alpha bits, which may lead to an overrun so we should technically subtract,
    // however it's impossible to get a read over the buffer here, as the colours and indices follow
    // right after; so we can just work with aligned len just fine.
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

    while alpha_byte_in_ptr < alpha_byte_end_ptr {
        // Okay, so we can read 64 bytes of alpha bytes (2 byte) at a time.
        // This means 32-blocks read in single read. (1x)
        //
        // For colours and indices, this is 4 bytes at a time.
        // So 16-blocks per read. (2x)
        //
        // For alpha bits (6 bytes), the math does not divide evenly (64 / 6) == ~10.6.
        // So we have to round this down to 8 blocks. (4x)
        // 8 blocks * 48 bits == 384 bits total. 48 bytes per read.
        //
        // What does this mean? To maximize throughput, we read 4x of alpha bits (largest item),
        // then blend with other registers to reproduce the original blocks.

        // Read in the individual components.
        // In AVX512 we got 32 registers, we're going to cook with them all!

        // [4]  9 input registers <- from memory (4 always active)
        // [16] 8+4+4 permutation registers <- from memory (16 always active)
        // [4]  8 output registers <- to memory (4 always active)
        // So around 33 registers used total, but only 24 registers 'active' at any given point.
        // Therefore this fits in the 32 reg limit nicely for AVX512 on x86-64.

        // The alpha bytes for 32 blocks
        let alpha_bytes_0 = _mm512_loadu_si512(alpha_byte_in_ptr as *const __m512i); // 32 blocks, 8 regs
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(64);

        // The colors and indices for 32 blocks (16 blocks per read)
        let colors_0 = _mm512_loadu_si512(color_byte_in_ptr as *const __m512i); // 16 blocks, 4 regs
        let colors_1 = _mm512_loadu_si512(color_byte_in_ptr.add(64) as *const __m512i); // 16 blocks, 4 regs
        color_byte_in_ptr = color_byte_in_ptr.add(128);

        let indices_0 = _mm512_loadu_si512(index_byte_in_ptr as *const __m512i); // 16 blocks, 4 regs
        let indices_1 = _mm512_loadu_si512(index_byte_in_ptr.add(64) as *const __m512i); // 16 blocks, 4 regs
        index_byte_in_ptr = index_byte_in_ptr.add(128);

        // The alpha bits for 32 blocks (8 blocks per read)
        let alpha_bit_0 = _mm512_loadu_si512(alpha_bit_in_ptr as *const __m512i); // 8 blocks, 2 regs
        let alpha_bit_1 = _mm512_loadu_si512(alpha_bit_in_ptr.add(48) as *const __m512i); // 8 blocks, 2 regs
        let alpha_bit_2 = _mm512_loadu_si512(alpha_bit_in_ptr.add(96) as *const __m512i); // 8 blocks, 2 regs
        let alpha_bit_3 = _mm512_loadu_si512(alpha_bit_in_ptr.add(144) as *const __m512i); // 8 blocks, 2 regs
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(192);

        // 9 regs used, let's use another 8
        // and another few for the permutations

        // Now let's reassemble the 32 blocks
        // 64 / 16 == 4 blocks per register, so 8 registers of blocks needed because we got 32 blocks
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

        // Now compiler will swap out `alpha_bit_1` into register of `alpha_bit_0`, hopefully.
        _mm512_storeu_si512(current_output_ptr as *mut __m512i, blocks_0);
        _mm512_storeu_si512(current_output_ptr.add(64) as *mut __m512i, blocks_1);
        _mm512_storeu_si512(current_output_ptr.add(128) as *mut __m512i, blocks_2);
        _mm512_storeu_si512(current_output_ptr.add(192) as *mut __m512i, blocks_3);
        _mm512_storeu_si512(current_output_ptr.add(256) as *mut __m512i, blocks_4);
        _mm512_storeu_si512(current_output_ptr.add(320) as *mut __m512i, blocks_5);
        _mm512_storeu_si512(current_output_ptr.add(384) as *mut __m512i, blocks_6);
        _mm512_storeu_si512(current_output_ptr.add(448) as *mut __m512i, blocks_7);

        // The colors and indices for 8 blocks
        current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
    }

    // Convert pointers to the types expected by u32_detransform_with_separate_pointers
    let alpha_byte_in_ptr_u16 = alpha_byte_in_ptr as *const u16;
    let alpha_bit_in_ptr_u16 = alpha_bit_in_ptr as *const u16;
    let color_byte_in_ptr_u32 = color_byte_in_ptr as *const u32;
    let index_byte_in_ptr_u32 = index_byte_in_ptr as *const u32;

    u32_detransform_with_separate_pointers(
        alpha_byte_in_ptr_u16,
        alpha_bit_in_ptr_u16,
        color_byte_in_ptr_u32,
        index_byte_in_ptr_u32,
        current_output_ptr,
        len - aligned_len,
    );
}

/// Detransforms BC3 block data from separated components using AVX512 instructions.
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
///   - `color_byte_in_ptr` needs `len / 16 * 8` readable bytes.
///   - `index_byte_in_ptr` needs `len / 16 * 8` readable bytes.
/// - `current_output_ptr` must be valid for writes for `len` bytes.
/// - `len` must be a multiple of 16 (the size of a BC3 block).
/// - Pointers do not need to be aligned; unaligned loads/reads are used.
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[target_feature(enable = "avx512vbmi")]
pub unsafe fn avx512_detransform_separate_components_32(
    mut alpha_byte_in_ptr: *const u8,
    mut alpha_bit_in_ptr: *const u8,
    mut color_byte_in_ptr: *const u8,
    mut index_byte_in_ptr: *const u8,
    mut current_output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);
    const BYTES_PER_ITERATION: usize = 64;
    // We drop some alpha bits, which may lead to an overrun so we should technically subtract,
    // however it's impossible to get a read over the buffer here, as the colours and indices follow
    // right after; so we can just work with aligned len just fine.
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

        // The alpha bytes, (4 blocks * 2 bytes == 8 bytes)
        let alpha_bytes_0 = _mm512_loadu_si512(alpha_byte_in_ptr as *const __m512i); // 32 blocks, 8 regs
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(8);

        // The colors and indices (4 blocks * 4 bytes == 16 bytes)
        let colors_0 = _mm512_loadu_si512(color_byte_in_ptr as *const __m512i); // 16 blocks, 4 regs
        color_byte_in_ptr = color_byte_in_ptr.add(16);

        let indices_0 = _mm512_loadu_si512(index_byte_in_ptr as *const __m512i); // 16 blocks, 4 regs
        index_byte_in_ptr = index_byte_in_ptr.add(16);

        // The alpha bits (4 blocks * 6 bytes == 24 bytes)
        let alpha_bit_0 = _mm512_loadu_si512(alpha_bit_in_ptr as *const __m512i); // 8 blocks, 2 regs
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(24);

        // 9 regs used, let's use another 8
        // and another few for the permutations

        // Now let's reassemble the 32 blocks
        // 64 / 16 == 4 blocks per register, so 8 registers of blocks needed because we got 32 blocks
        let mut blocks_0 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_0_perm_alphabits, alpha_bit_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_colours, colors_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_indices, indices_0);

        // Now compiler will swap out `alpha_bit_1` into register of `alpha_bit_0`, hopefully.
        _mm512_storeu_si512(current_output_ptr as *mut __m512i, blocks_0);

        // The colors and indices for 8 blocks
        current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
    }

    // Convert pointers to the types expected by u32_detransform_with_separate_pointers
    let alpha_byte_in_ptr_u16 = alpha_byte_in_ptr as *const u16;
    let alpha_bit_in_ptr_u16 = alpha_bit_in_ptr as *const u16;
    let color_byte_in_ptr_u32 = color_byte_in_ptr as *const u32;
    let index_byte_in_ptr_u32 = index_byte_in_ptr as *const u32;

    u32_detransform_with_separate_pointers(
        alpha_byte_in_ptr_u16,
        alpha_bit_in_ptr_u16,
        color_byte_in_ptr_u32,
        index_byte_in_ptr_u32,
        current_output_ptr,
        len - aligned_len,
    );
}

/// Detransforms BC3 block data from separated components using AVX512 instructions.
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
///   - `color_byte_in_ptr` needs `len / 16 * 8` readable bytes.
///   - `index_byte_in_ptr` needs `len / 16 * 8` readable bytes.
/// - `current_output_ptr` must be valid for writes for `len` bytes.
/// - `len` must be a multiple of 16 (the size of a BC3 block).
/// - Pointers do not need to be aligned; unaligned loads/reads are used.
#[allow(clippy::erasing_op)]
#[allow(clippy::identity_op)]
#[target_feature(enable = "avx512vbmi")]
#[target_feature(enable = "avx512vl")]
pub unsafe fn avx512_detransform_separate_components_32_vl(
    mut alpha_byte_in_ptr: *const u8,
    mut alpha_bit_in_ptr: *const u8,
    mut color_byte_in_ptr: *const u8,
    mut index_byte_in_ptr: *const u8,
    mut current_output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);
    const BYTES_PER_ITERATION: usize = 64;
    // We drop some alpha bits, which may lead to an overrun so we should technically subtract,
    // however it's impossible to get a read over the buffer here, as the colours and indices follow
    // right after; so we can just work with aligned len just fine.
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

        // The alpha bytes, (4 blocks * 2 bytes == 8 bytes)
        let alpha_bytes_0 =
            _mm512_castsi128_si512(_mm_loadu_si128(alpha_byte_in_ptr as *const __m128i)); // 32 blocks, 8 regs
        alpha_byte_in_ptr = alpha_byte_in_ptr.add(8);

        // The colors and indices (4 blocks * 4 bytes == 16 bytes)
        let colors_0 = _mm512_castsi128_si512(_mm_loadu_si128(color_byte_in_ptr as *const __m128i)); // 16 blocks, 4 regs
        color_byte_in_ptr = color_byte_in_ptr.add(16);

        let indices_0 =
            _mm512_castsi128_si512(_mm_loadu_si128(index_byte_in_ptr as *const __m128i)); // 16 blocks, 4 regs
        index_byte_in_ptr = index_byte_in_ptr.add(16);

        // The alpha bits (4 blocks * 6 bytes == 24 bytes)
        let alpha_bit_0 =
            _mm512_castsi256_si512(_mm256_loadu_si256(alpha_bit_in_ptr as *const __m256i)); // 8 blocks, 2 regs
        alpha_bit_in_ptr = alpha_bit_in_ptr.add(24);

        // 9 regs used, let's use another 8
        // and another few for the permutations

        // Now let's reassemble the 32 blocks
        // 64 / 16 == 4 blocks per register, so 8 registers of blocks needed because we got 32 blocks
        let mut blocks_0 =
            _mm512_permutex2var_epi8(alpha_bytes_0, blocks_0_perm_alphabits, alpha_bit_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_colours, colors_0);
        blocks_0 = _mm512_permutex2var_epi8(blocks_0, blocks_0_perm_indices, indices_0);

        // Now compiler will swap out `alpha_bit_1` into register of `alpha_bit_0`, hopefully.
        _mm512_storeu_si512(current_output_ptr as *mut __m512i, blocks_0);

        // The colors and indices for 8 blocks
        current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
    }

    // Convert pointers to the types expected by u32_detransform_with_separate_pointers
    let alpha_byte_in_ptr_u16 = alpha_byte_in_ptr as *const u16;
    let alpha_bit_in_ptr_u16 = alpha_bit_in_ptr as *const u16;
    let color_byte_in_ptr_u32 = color_byte_in_ptr as *const u32;
    let index_byte_in_ptr_u32 = index_byte_in_ptr as *const u32;

    u32_detransform_with_separate_pointers(
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
    use crate::split_blocks::split::tests::generate_bc3_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(avx512_detransform, "avx512")]
    #[case(avx512_detransform_32_vl, "avx512_32_vl")]
    #[case(avx512_detransform_32_vbmi, "avx512_32_vbmi")]
    fn test_avx512_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        if !is_x86_feature_detected!("avx512vbmi") || !is_x86_feature_detected!("avx512vl") {
            return;
        }

        for num_blocks in 1..=512 {
            let original = generate_bc3_test_data(num_blocks);
            let mut transformed = allocate_align_64(original.len());
            let mut reconstructed = allocate_align_64(original.len());

            unsafe {
                // Transform using standard implementation
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());

                // Reconstruct using the implementation being tested
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                reconstructed.as_slice(),
                &format!("{impl_name} (aligned)"),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(avx512_detransform, "avx512")]
    #[case(avx512_detransform_32_vbmi, "avx512_32_vbmi")]
    #[case(avx512_detransform_32_vl, "avx512_32_vl")]
    fn test_avx512_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
        if !is_x86_feature_detected!("avx512vbmi") || !is_x86_feature_detected!("avx512vl") {
            return;
        }

        for num_blocks in 1..=512 {
            let original = generate_bc3_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            unsafe {
                u32(original.as_ptr(), transformed.as_mut_ptr(), original.len());
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);

            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                detransform_fn(
                    transformed_unaligned.as_ptr().add(1),
                    reconstructed.as_mut_ptr().add(1),
                    transformed.len(),
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}
