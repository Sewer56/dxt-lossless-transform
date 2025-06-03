#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "x86")]
use core::arch::x86::*;

#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[allow(clippy::identity_op)]
pub unsafe fn untransform_with_split_colour(
    mut color0_ptr: *const u16,
    mut color1_ptr: *const u16,
    mut indices_ptr: *const u32,
    mut output_ptr: *mut u8,
    block_count: usize,
) {
    debug_assert!(block_count > 0);

    // Process 32 blocks (256 bytes) at a time
    let aligned_count = block_count - (block_count % 32);
    let color0_ptr_aligned_end = color0_ptr.add(aligned_count);

    if aligned_count > 0 {
        // Mask for interleaving color0 and color1 into alternating pattern (lower half)
        // This takes 16 u16 color0 values and 16 u16 color1 values and interleaves them

        // Inputs:  color0=[C0_0, C0_1, ..., C0_15], color1=[C1_0, C1_1, ..., C1_15]
        // Output: [C0_0, C1_0, C0_1, C1_1, ..., C0_15, C1_15]
        let perm_color_interleave_low = _mm512_set_epi16(
            15 + 32, // C1_15
            15 + 0,  // C0_15,
            14 + 32, // C1_14
            14 + 0,  // C0_14,
            13 + 32, // C1_13
            13 + 0,  // C0_13,
            12 + 32, // C1_12
            12 + 0,  // C0_12,
            11 + 32, // C1_11
            11 + 0,  // C0_11,
            10 + 32, // C1_10
            10 + 0,  // C0_10,
            9 + 32,  // C1_9
            9 + 0,   // C0_9,
            8 + 32,  // C1_8
            8 + 0,   // C0_8,
            7 + 32,  // C1_7,
            7 + 0,   // C0_7,
            6 + 32,  // C1_6
            6 + 0,   // C0_6,
            5 + 32,  // C1_5
            5 + 0,   // C0_5,
            4 + 32,  // C1_4
            4 + 0,   // C0_4,
            3 + 32,  // C1_3
            3 + 0,   // C0_3,
            2 + 32,  // C1_2
            2 + 0,   // C0_2,
            1 + 32,  // C1_1
            1 + 0,   // C0_1,
            0 + 32,  // C1_0
            0 + 0,   // C0_0,
        );

        let perm_color_interleave_high = _mm512_set_epi16(
            (15 + 32) + 16, // C1_31
            (15 + 0) + 16,  // C0_31,
            (14 + 32) + 16, // C1_30
            (14 + 0) + 16,  // C0_30,
            (13 + 32) + 16, // C1_29
            (13 + 0) + 16,  // C0_29,
            (12 + 32) + 16, // C1_28
            (12 + 0) + 16,  // C0_28,
            (11 + 32) + 16, // C1_27
            (11 + 0) + 16,  // C0_27,
            (10 + 32) + 16, // C1_26
            (10 + 0) + 16,  // C0_26,
            (9 + 32) + 16,  // C1_25
            (9 + 0) + 16,   // C0_25,
            (8 + 32) + 16,  // C1_24
            (8 + 0) + 16,   // C0_24,
            (7 + 32) + 16,  // C1_23,
            (7 + 0) + 16,   // C0_23,
            (6 + 32) + 16,  // C1_22
            (6 + 0) + 16,   // C0_22,
            (5 + 32) + 16,  // C1_21
            (5 + 0) + 16,   // C0_21,
            (4 + 32) + 16,  // C1_20
            (4 + 0) + 16,   // C0_20,
            (3 + 32) + 16,  // C1_19
            (3 + 0) + 16,   // C0_19,
            (2 + 32) + 16,  // C1_18
            (2 + 0) + 16,   // C0_18,
            (1 + 32) + 16,  // C1_17
            (1 + 0) + 16,   // C0_17,
            (0 + 32) + 16,  // C1_16
            (0 + 0) + 16,   // C0_16,
        );

        // Inputs:  colors_0=[C0_0, C1_0, ..., C0_8, C1_8] | indices_0=[I0_0, I0_1 ..., I16_0, I16_1]
        // Output: [C0_0, C1_0, I0_0, I0_1 ..., C0_8, C1_8, I8_0, I8_1]
        let perm_output_0 = _mm512_set_epi16(
            15 + 32, // I7_1
            14 + 32, // I7_0,
            15 + 0,  // C1_7
            14 + 0,  // C0_7,
            13 + 32, // I6_1
            12 + 32, // I6_0,
            13 + 0,  // C1_6
            12 + 0,  // C0_6,
            11 + 32, // I5_1
            10 + 32, // I5_0,
            11 + 0,  // C1_5
            10 + 0,  // C0_5,
            9 + 32,  // I4_1
            8 + 32,  // I4_0,
            9 + 0,   // C1_4
            8 + 0,   // C0_4,
            7 + 32,  // I3_1
            6 + 32,  // I3_0,
            7 + 0,   // C1_3
            6 + 0,   // C0_3,
            5 + 32,  // I2_1
            4 + 32,  // I2_0,
            5 + 0,   // C1_2
            4 + 0,   // C0_2,
            3 + 32,  // I1_1
            2 + 32,  // I1_0,
            3 + 0,   // C1_1
            2 + 0,   // C0_1,
            1 + 32,  // I0_0
            0 + 32,  // I0_0,
            1 + 0,   // C1_0
            0 + 0,   // C0_0,
        );

        let perm_output_1 = _mm512_set_epi16(
            31 + 32, // I15_1
            30 + 32, // I15_0,
            31 + 0,  // C1_15
            30 + 0,  // C0_15,
            29 + 32, // I14_1
            28 + 32, // I14_0,
            29 + 0,  // C1_14
            28 + 0,  // C0_14,
            27 + 32, // I13_1
            26 + 32, // I13_0,
            27 + 0,  // C1_13
            26 + 0,  // C0_13,
            25 + 32, // I12_1
            24 + 32, // I12_0,
            25 + 0,  // C1_12
            24 + 0,  // C0_12,
            23 + 32, // I11_1
            22 + 32, // I11_0,
            23 + 0,  // C1_11
            22 + 0,  // C0_11
            21 + 32, // I10_1
            20 + 32, // I10_0,
            21 + 0,  // C1_10
            20 + 0,  // C0_10,
            19 + 32, // I9_1
            18 + 32, // I9_0,
            19 + 0,  // C1_9
            18 + 0,  // C0_9,
            17 + 32, // I8_1
            16 + 32, // I8_0,
            17 + 0,  // C1_8
            16 + 0,  // C0_8,
        );

        while color0_ptr < color0_ptr_aligned_end {
            // Load 32 blocks worth of data
            // Load 64 bytes (32 u16 values) of color0 data - 1 read
            let color0s = _mm512_loadu_si512(color0_ptr as *const __m512i);
            color0_ptr = color0_ptr.add(32);

            // Load 64 bytes (32 u16 values) of color1 data - 1 read
            let color1s = _mm512_loadu_si512(color1_ptr as *const __m512i);
            color1_ptr = color1_ptr.add(32);

            // Load 128 bytes (32 u32 values) of indices data - 2 reads
            let indices_0 = _mm512_loadu_si512(indices_ptr as *const __m512i);
            let indices_1 = _mm512_loadu_si512(indices_ptr.add(16) as *const __m512i);
            indices_ptr = indices_ptr.add(32);

            // Interleave color0 and color1 into alternating pairs (first 16 blocks)
            let colors_0 = _mm512_permutex2var_epi16(color0s, perm_color_interleave_low, color1s);
            let colors_1 = _mm512_permutex2var_epi16(color0s, perm_color_interleave_high, color1s);

            // Now interleave the color pairs with indices to create final BC1 blocks
            let output_0 = _mm512_permutex2var_epi16(colors_0, perm_output_0, indices_0);
            let output_1 = _mm512_permutex2var_epi16(colors_0, perm_output_1, indices_0);
            let output_2 = _mm512_permutex2var_epi16(colors_1, perm_output_0, indices_1);
            let output_3 = _mm512_permutex2var_epi16(colors_1, perm_output_1, indices_1);

            // Write results - 32 blocks * 8 bytes = 256 bytes total
            _mm512_storeu_si512(output_ptr as *mut __m512i, output_0);
            _mm512_storeu_si512(output_ptr.add(64) as *mut __m512i, output_1);
            _mm512_storeu_si512(output_ptr.add(128) as *mut __m512i, output_2);
            _mm512_storeu_si512(output_ptr.add(192) as *mut __m512i, output_3);

            // Advance output pointer
            output_ptr = output_ptr.add(256);
        }
    }

    // Process any remaining blocks (less than 32) using generic implementation
    let remaining_count = block_count - aligned_count;
    super::generic::untransform_with_split_colour(
        color0_ptr, color1_ptr, indices_ptr, output_ptr, remaining_count
    );
}

#[cfg(test)]
mod tests {
    use super::untransform_with_split_colour;
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::{has_avx512bw, has_avx512f};

    #[test]
    fn can_untransform_unaligned() {
        if !has_avx512f() & has_avx512bw() {
            return;
        }

        for num_blocks in 1..=512 {
            let original = generate_bc1_test_data(num_blocks);

            // Transform using standard implementation
            let mut transformed = vec![0u8; original.len()];
            let mut work = vec![0u8; original.len()];
            unsafe {
                transform_bc1(
                    original.as_ptr(),
                    transformed.as_mut_ptr(),
                    work.as_mut_ptr(),
                    original.len(),
                    Bc1TransformDetails {
                        color_normalization_mode: ColorNormalizationMode::None,
                        decorrelation_mode: YCoCgVariant::None,
                        split_colour_endpoints: true,
                    },
                );
            }

            // Add 1 extra byte at the beginning to create misaligned buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);
            let mut reconstructed = vec![0u8; original.len() + 1];

            unsafe {
                // Reconstruct using the implementation being tested with unaligned pointers
                reconstructed.as_mut_slice().fill(0);
                untransform_with_split_colour(
                    transformed_unaligned.as_ptr().add(1) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 2) as *const u16,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "untransform_with_split_colour (avx512, unaligned)",
                num_blocks,
            );
        }
    }
}
