use crate::transforms::with_split_colour::transform::generic;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

/// AVX512 implementation for split‐colour transform.
///
/// This uses the same permutation approach as [`permute_512_with_separate_pointers`]
/// but additionally splits the joined colour dword into separate `color0` and
/// `color1` arrays.
///
/// Safety invariants are identical to the generic implementation.
#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[allow(clippy::identity_op)]
#[no_mangle]
pub(crate) unsafe fn transform_with_split_colour(
    mut input_ptr: *const u8,
    mut color0_ptr: *mut u16,
    mut color1_ptr: *mut u16,
    mut indices_ptr: *mut u32,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Number of blocks that can be processed per 256-byte iteration (32 BC1 blocks).
    const BLOCKS_PER_ITER: usize = 32;

    // Byte permutation indices for vpermt2d to gather colour and index dwords.
    // Same values as the standard AVX512 split implementation.
    const PERM_COLORS_BYTES: [i8; 16] = [0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30];
    const PERM_INDICES_BYTES: [i8; 16] =
        [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 27, 29, 31];

    // Pre-compute permutation vectors for dword gathers (extend i8 -> i32 as required by vpermt2d).
    let perm_colors = _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_COLORS_BYTES.as_ptr() as *const _));
    let perm_indices =
        _mm512_cvtepi8_epi32(_mm_loadu_si128(PERM_INDICES_BYTES.as_ptr() as *const _));

    // Pre-compute epi16 permutation vectors that directly extract colour0/colour1 words.
    // Each entry selects a 16-bit lane from either `colors_0` (0-31) or `colors_1` (32-63).
    // colour0 = low 16-bits  (even lanes), colour1 = high 16-bits (odd lanes).
    const PERM_COLOR0_EPI16: [i16; 32] = [
        0,
        2,
        4,
        6,
        8,
        10,
        12,
        14,
        16,
        18,
        20,
        22,
        24,
        26,
        28,
        30,
        0 + 32,
        2 + 32,
        4 + 32,
        6 + 32,
        8 + 32,
        10 + 32,
        12 + 32,
        14 + 32,
        16 + 32,
        18 + 32,
        20 + 32,
        22 + 32,
        24 + 32,
        26 + 32,
        28 + 32,
        30 + 32,
    ];
    const PERM_COLOR1_EPI16: [i16; 32] = [
        1,
        3,
        5,
        7,
        9,
        11,
        13,
        15,
        17,
        19,
        21,
        23,
        25,
        27,
        29,
        31,
        1 + 32,
        3 + 32,
        5 + 32,
        7 + 32,
        9 + 32,
        11 + 32,
        13 + 32,
        15 + 32,
        17 + 32,
        19 + 32,
        21 + 32,
        23 + 32,
        25 + 32,
        27 + 32,
        29 + 32,
        31 + 32,
    ];

    let perm_color0_epi16 = _mm512_loadu_si512(PERM_COLOR0_EPI16.as_ptr() as *const __m512i);
    let perm_color1_epi16 = _mm512_loadu_si512(PERM_COLOR1_EPI16.as_ptr() as *const __m512i);

    // Aligned block count that fits full iterations.
    let aligned_blocks = block_count - (block_count % BLOCKS_PER_ITER);
    let aligned_end_input = input_ptr.add(aligned_blocks * 8); // 8 bytes per BC1 block

    while input_ptr < aligned_end_input {
        // Load 256 bytes (4 × 64-byte ZMM registers) = 32 BC1 blocks.
        let in0 = _mm512_loadu_si512(input_ptr as *const __m512i);
        let in1 = _mm512_loadu_si512(input_ptr.add(64) as *const __m512i);
        let in2 = _mm512_loadu_si512(input_ptr.add(128) as *const __m512i);
        let in3 = _mm512_loadu_si512(input_ptr.add(192) as *const __m512i);
        input_ptr = input_ptr.add(256);

        // Extract colours and indices into separate registers.
        let colors_0 = _mm512_permutex2var_epi32(in0, perm_colors, in1);
        let colors_1 = _mm512_permutex2var_epi32(in2, perm_colors, in3);

        let indices_0 = _mm512_permutex2var_epi32(in0, perm_indices, in1);
        let indices_1 = _mm512_permutex2var_epi32(in2, perm_indices, in3);

        // Group extracted colours into color0 and color1 components.
        let color0_only = _mm512_permutex2var_epi16(colors_0, perm_color0_epi16, colors_1);
        let color1_only = _mm512_permutex2var_epi16(colors_0, perm_color1_epi16, colors_1);

        // Store color0 and color1 (64 bytes each).
        _mm512_storeu_si512(color0_ptr as *mut __m512i, color0_only);
        _mm512_storeu_si512(color1_ptr as *mut __m512i, color1_only);
        color0_ptr = color0_ptr.add(BLOCKS_PER_ITER);
        color1_ptr = color1_ptr.add(BLOCKS_PER_ITER);

        // Store indices (two 64-byte stores = 32 u32 values).
        _mm512_storeu_si512(indices_ptr as *mut __m512i, indices_0);
        _mm512_storeu_si512(indices_ptr.add(16) as *mut __m512i, indices_1);
        indices_ptr = indices_ptr.add(BLOCKS_PER_ITER);
    }

    // Process any remaining blocks via the portable fallback.
    let remaining_blocks = block_count - aligned_blocks;
    generic::transform_with_split_colour(
        input_ptr,
        color0_ptr,
        color1_ptr,
        indices_ptr,
        remaining_blocks,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use crate::transforms::with_split_colour::untransform::untransform_with_split_colour;

    #[rstest]
    fn avx512_transform_roundtrip() {
        if !has_avx512f() || !has_avx512bw() {
            return;
        }

        for num_blocks in 1..=128 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut colour0 = vec![0u16; num_blocks];
            let mut colour1 = vec![0u16; num_blocks];
            let mut indices = vec![0u32; num_blocks];
            let mut reconstructed = vec![0u8; len];
            unsafe {
                transform_with_split_colour(
                    input.as_ptr(),
                    colour0.as_mut_ptr(),
                    colour1.as_mut_ptr(),
                    indices.as_mut_ptr(),
                    num_blocks,
                );
                untransform_with_split_colour(
                    colour0.as_ptr(),
                    colour1.as_ptr(),
                    indices.as_ptr(),
                    reconstructed.as_mut_ptr(),
                    num_blocks,
                );
            }
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "Mismatch AVX512 roundtrip split-colour",
            );
        }
    }
}
