#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::hint::unreachable_unchecked;
use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};

#[cfg(feature = "nightly")]
pub(crate) unsafe fn untransform_with_recorrelate(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
    decorrelation_mode: YCoCgVariant,
) {
    match decorrelation_mode {
        YCoCgVariant::Variant1 => {
            untransform_recorr_var1(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant2 => {
            untransform_recorr_var2(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::Variant3 => {
            untransform_recorr_var3(colors_ptr, indices_ptr, output_ptr, num_blocks);
        }
        YCoCgVariant::None => {
            // This should be unreachable based on the calling context
            unreachable_unchecked()
        }
    }
}

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn untransform_recorr_var1(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Note(sewer): Compiler here does not produce good code for this function, unless targeting
    // certain CPUs like znver4. (At least as far as my 9950X3D is concerned)
    // I lifted the below code from a `target-cpu=native` build and cleaned it up a bit. (AI Assisted)

    // === Shuffle Patterns for Interleaving Colors and Indices ===
    // These patterns interleave the recorrelated color data with indices
    // Pattern 1: handles lower half of the output data
    let shuffle_pattern_low = _mm512_set_epi16(
        47, 46, 23, 7, 45, 44, 22, 6, 43, 42, 21, 5, 41, 40, 20, 4, 39, 38, 19, 3, 37, 36, 18, 2,
        35, 34, 17, 1, 33, 32, 16, 0,
    );

    // Pattern 2: handles upper half of the output data
    let shuffle_pattern_high = _mm512_set_epi16(
        63, 62, 31, 15, 61, 60, 30, 14, 59, 58, 29, 13, 57, 56, 28, 12, 55, 54, 27, 11, 53, 52, 26,
        10, 51, 50, 25, 9, 49, 48, 24, 8,
    );

    // === Main Vectorized Loop ===
    // Process 16 blocks at a time using AVX512 SIMD instructions
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !15; // Round down to multiple of 16
    let mut block_index = 0;

    // Main SIMD processing loop - handles 16 blocks per iteration
    while block_index < vectorized_blocks {
        // === Load Input Data ===
        // Load 16 color values (each u32 contains 2 Color565 values)
        let colors_raw = _mm512_loadu_si512(colors_ptr.add(block_index) as *const __m512i);

        // Load 16 index values
        let indices_raw = _mm512_loadu_si512(indices_ptr.add(block_index) as *const __m512i);

        // === YCoCg-R Recorrelation using helper function ===
        let recorrelated_colors = recorrelate_ycocg_r_variant1_avx512(colors_raw);

        // === Interleave Recorrelated Colors with Indices ===
        // Step 6: Create two copies of recorrelated data for different shuffle patterns
        let colors_for_low_shuffle = recorrelated_colors;
        let colors_for_high_shuffle = recorrelated_colors;

        // Step 7: Use permute operations to interleave colors with indices
        // This creates the final output format where each 8-byte block contains:
        // [4 bytes recorrelated colors][4 bytes indices]
        let output_low =
            _mm512_permutex2var_epi16(colors_for_low_shuffle, shuffle_pattern_low, indices_raw);
        let output_high =
            _mm512_permutex2var_epi16(colors_for_high_shuffle, shuffle_pattern_high, indices_raw);

        // === Store Results ===
        // Write the interleaved data to output buffer
        // Each iteration processes 16 blocks, producing 128 bytes of output (16 * 8 bytes)
        _mm512_storeu_si512(
            output_ptr.add(block_index * 8 + 64) as *mut __m512i,
            output_high,
        );
        _mm512_storeu_si512(output_ptr.add(block_index * 8) as *mut __m512i, output_low);

        // Move to next batch of 16 blocks
        block_index += 16;
    }

    // === Scalar Fallback for Remaining Blocks ===
    // Handle any remaining blocks that couldn't be processed in the vectorized loop
    // (when num_blocks is not a multiple of 16)
    for block_idx in vectorized_blocks..num_blocks {
        // Read both values first (better instruction scheduling)
        let color_raw = read_unaligned(colors_ptr.add(block_idx));
        let index_value = read_unaligned(indices_ptr.add(block_idx));

        // Extract both Color565 values from the u32
        let color0 = Color565::from_raw(color_raw as u16);
        let color1 = Color565::from_raw((color_raw >> 16) as u16);

        // Apply recorrelation to both colors
        let recorr_color0 = color0.recorrelate_ycocg_r_var1();
        let recorr_color1 = color1.recorrelate_ycocg_r_var1();

        // Pack both recorrelated colors back into u32
        let recorrelated_colors =
            (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

        // Write both values together
        write_unaligned(
            output_ptr.add(block_idx * 8) as *mut u32,
            recorrelated_colors,
        );
        write_unaligned(output_ptr.add(block_idx * 8 + 4) as *mut u32, index_value);
    }
}

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn untransform_recorr_var2(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Note(sewer): Compiler here does not produce good code for this function, unless targeting
    // certain CPUs like znver4. (At least as far as my 9950X3D is concerned)
    // I lifted the below code from a `target-cpu=native` build and cleaned it up a bit. (AI Assisted)

    // === Shuffle Patterns for Interleaving Colors and Indices ===
    // These patterns are byte-based and will be sign-extended to 16-bit values
    // Pattern 1: handles lower half of the output data
    let shuffle_pattern_low_bytes = _mm256_set_epi8(
        47, 46, 23, 7, 45, 44, 22, 6, 43, 42, 21, 5, 41, 40, 20, 4, 39, 38, 19, 3, 37, 36, 18, 2,
        35, 34, 17, 1, 33, 32, 16, 0,
    );

    // Pattern 2: handles upper half of the output data
    let shuffle_pattern_high_bytes = _mm256_set_epi8(
        63, 62, 31, 15, 61, 60, 30, 14, 59, 58, 29, 13, 57, 56, 28, 12, 55, 54, 27, 11, 53, 52, 26,
        10, 51, 50, 25, 9, 49, 48, 24, 8,
    );

    // Sign-extend byte patterns to 16-bit values (vpmovsxbw equivalent)
    let shuffle_pattern_low = _mm512_cvtepi8_epi16(shuffle_pattern_low_bytes);
    let shuffle_pattern_high = _mm512_cvtepi8_epi16(shuffle_pattern_high_bytes);

    // === Main Vectorized Loop ===
    // Process 16 blocks at a time using AVX512 SIMD instructions
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !15; // Round down to multiple of 16
    let mut block_index = 0;

    // Main SIMD processing loop - handles 16 blocks per iteration
    while block_index < vectorized_blocks {
        // === Load Input Data ===
        // Load 16 color values (each u32 contains 2 [`Color565`] values)
        let colors_raw = _mm512_loadu_si512(colors_ptr.add(block_index) as *const __m512i);

        // Load 16 index values
        let indices_raw = _mm512_loadu_si512(indices_ptr.add(block_index) as *const __m512i);

        // === YCoCg-R Decorrelation using helper function ===
        let recorrelated_colors = recorrelate_ycocg_r_variant2_avx512(colors_raw);

        // === Interleave Recorrelated Colors with Indices ===
        // Step 6: Create two copies of recorrelated data for different shuffle patterns
        let colors_for_low_shuffle = recorrelated_colors;
        let colors_for_high_shuffle = recorrelated_colors;

        // Step 7: Use permute operations to interleave colors with indices
        // This creates the final output format where each 8-byte block contains:
        // [4 bytes recorrelated colors][4 bytes indices]
        let output_low =
            _mm512_permutex2var_epi16(colors_for_low_shuffle, shuffle_pattern_low, indices_raw);
        let output_high =
            _mm512_permutex2var_epi16(colors_for_high_shuffle, shuffle_pattern_high, indices_raw);

        // === Store Results ===
        // Write the interleaved data to output buffer
        // Each iteration processes 16 blocks, producing 128 bytes of output (16 * 8 bytes)
        _mm512_storeu_si512(
            output_ptr.add(block_index * 8 + 64) as *mut __m512i,
            output_high,
        );
        _mm512_storeu_si512(output_ptr.add(block_index * 8) as *mut __m512i, output_low);

        // Move to next batch of 16 blocks
        block_index += 16;
    }

    // === Scalar Fallback for Remaining Blocks ===
    // Handle any remaining blocks that couldn't be processed in the vectorized loop
    // (when num_blocks is not a multiple of 16)
    for block_idx in vectorized_blocks..num_blocks {
        // Read both values first (better instruction scheduling)
        let color_raw = read_unaligned(colors_ptr.add(block_idx));
        let index_value = read_unaligned(indices_ptr.add(block_idx));

        // Extract both [`Color565`] values from the u32
        let color0 = Color565::from_raw(color_raw as u16);
        let color1 = Color565::from_raw((color_raw >> 16) as u16);

        // Apply recorrelation to both colors
        let recorr_color0 = color0.recorrelate_ycocg_r_var2();
        let recorr_color1 = color1.recorrelate_ycocg_r_var2();

        // Pack both recorrelated colors back into u32
        let recorrelated_colors =
            (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

        // Write both values together
        write_unaligned(
            output_ptr.add(block_idx * 8) as *mut u32,
            recorrelated_colors,
        );
        write_unaligned(output_ptr.add(block_idx * 8 + 4) as *mut u32, index_value);
    }
}

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
unsafe fn untransform_recorr_var3(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    num_blocks: usize,
) {
    // Note(sewer): Compiler here does not produce good code for this function, unless targeting
    // certain CPUs like znver4. (At least as far as my 9950X3D is concerned)
    // I lifted the below code from a `target-cpu=native` build and cleaned it up a bit. (AI Assisted)

    // === Shuffle Patterns for Interleaving Colors and Indices ===
    // These patterns are byte-based and will be sign-extended to 16-bit values
    // Pattern 1: handles lower half of the output data
    let shuffle_pattern_low_bytes = _mm256_set_epi8(
        47, 46, 23, 7, 45, 44, 22, 6, 43, 42, 21, 5, 41, 40, 20, 4, 39, 38, 19, 3, 37, 36, 18, 2,
        35, 34, 17, 1, 33, 32, 16, 0,
    );

    // Pattern 2: handles upper half of the output data
    let shuffle_pattern_high_bytes = _mm256_set_epi8(
        63, 62, 31, 15, 61, 60, 30, 14, 59, 58, 29, 13, 57, 56, 28, 12, 55, 54, 27, 11, 53, 52, 26,
        10, 51, 50, 25, 9, 49, 48, 24, 8,
    );

    // Sign-extend byte patterns to 16-bit values (vpmovsxbw equivalent)
    let shuffle_pattern_low = _mm512_cvtepi8_epi16(shuffle_pattern_low_bytes);
    let shuffle_pattern_high = _mm512_cvtepi8_epi16(shuffle_pattern_high_bytes);

    // === Main Vectorized Loop ===
    // Process 16 blocks at a time using AVX512 SIMD instructions
    // Calculate number of blocks that can be processed in vectorized chunks
    let vectorized_blocks = num_blocks & !15; // Round down to multiple of 16
    let mut block_index = 0;

    // Main SIMD processing loop - handles 16 blocks per iteration
    while block_index < vectorized_blocks {
        // === Load Input Data ===
        // Load 16 color values (each u32 contains 2 [`Color565`] values)
        let colors_raw = _mm512_loadu_si512(colors_ptr.add(block_index) as *const __m512i);

        // Load 16 index values
        let indices_raw = _mm512_loadu_si512(indices_ptr.add(block_index) as *const __m512i);

        // === YCoCg-R Decorrelation using helper function ===
        let recorrelated_colors = recorrelate_ycocg_r_variant3_avx512(colors_raw);

        // === Interleave Recorrelated Colors with Indices ===
        // Step 6: Create two copies of recorrelated data for different shuffle patterns
        let colors_for_low_shuffle = recorrelated_colors;
        let colors_for_high_shuffle = recorrelated_colors;

        // Step 7: Use permute operations to interleave colors with indices
        // This creates the final output format where each 8-byte block contains:
        // [4 bytes recorrelated colors][4 bytes indices]
        let output_low =
            _mm512_permutex2var_epi16(colors_for_low_shuffle, shuffle_pattern_low, indices_raw);
        let output_high =
            _mm512_permutex2var_epi16(colors_for_high_shuffle, shuffle_pattern_high, indices_raw);

        // === Store Results ===
        // Write the interleaved data to output buffer
        // Each iteration processes 16 blocks, producing 128 bytes of output (16 * 8 bytes)
        _mm512_storeu_si512(
            output_ptr.add(block_index * 8 + 64) as *mut __m512i,
            output_high,
        );
        _mm512_storeu_si512(output_ptr.add(block_index * 8) as *mut __m512i, output_low);

        // Move to next batch of 16 blocks
        block_index += 16;
    }

    // === Scalar Fallback for Remaining Blocks ===
    // Handle any remaining blocks that couldn't be processed in the vectorized loop
    // (when num_blocks is not a multiple of 16)
    for block_idx in vectorized_blocks..num_blocks {
        // Read both values first (better instruction scheduling)
        let color_raw = read_unaligned(colors_ptr.add(block_idx));
        let index_value = read_unaligned(indices_ptr.add(block_idx));

        // Extract both [`Color565`] values from the u32
        let color0 = Color565::from_raw(color_raw as u16);
        let color1 = Color565::from_raw((color_raw >> 16) as u16);

        // Apply recorrelation to both colors
        let recorr_color0 = color0.recorrelate_ycocg_r_var3();
        let recorr_color1 = color1.recorrelate_ycocg_r_var3();

        // Pack both recorrelated colors back into u32
        let recorrelated_colors =
            (recorr_color0.raw_value() as u32) | ((recorr_color1.raw_value() as u32) << 16);

        // Write both values together
        write_unaligned(
            output_ptr.add(block_idx * 8) as *mut u32,
            recorrelated_colors,
        );
        write_unaligned(output_ptr.add(block_idx * 8 + 4) as *mut u32, index_value);
    }
}

/// # Remarks
///
/// Returned value is not in the order it was received.
/// The values will be interleaved in the upper and lower 256 bit half, e.g.
///
/// {
///     0x0d0c090805040100,
///     0x1d1c191815141110,
///     0x2d2c292825242120,
///     0x3d3c393835343130,
///
///     0x0f0e0b0a07060302,
///     0x1f1e1b1a17161312,
///     0x2f2e2b2a27262322,
///     0x3f3e3b3a37363332
/// }
///
/// First 2 bytes in upper half, then 2 in lower half, then next 2 in upper half, etc.
#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
unsafe fn recorrelate_ycocg_r_variant1_avx512(colors_raw: __m512i) -> __m512i {
    // === Constants for YCoCg-R Recorrelation ===
    // Lifted outside loops by compiler.
    let mask_15 = _mm512_set1_epi16(15); // Mask for extracting 4-bit values
    let const_1984 = _mm512_set1_epi16(1984); // Used in ternary logic operations
    let const_32 = _mm512_set1_epi16(32); // Used in ternary logic operations
    let mask_31 = _mm512_set1_epi16(31); // Mask for extracting 5-bit values

    // === Extract Color Components for YCoCg-R Recorrelation ===
    // Convert lower 16 bits of each color pair to 16-bit values
    let colors_low = _mm512_cvtepi32_epi16(colors_raw);

    // Extract upper 16 bits by shifting and converting
    // These represent the second [`Color565`] value in each pair
    let colors_shifted_16 = _mm512_srli_epi32(colors_raw, 16);
    let colors_high = _mm512_cvtepi32_epi16(colors_shifted_16);

    // Extract specific bit ranges for recorrelation components
    let colors_shifted_17 = _mm512_srli_epi32(colors_raw, 17);
    let colors_shifted_22 = _mm512_srli_epi32(colors_raw, 22);
    let colors_shifted_23 = _mm512_srli_epi32(colors_raw, 23);
    let colors_shifted_27 = _mm512_srli_epi32(colors_raw, 27);

    // Convert shifted values to 16-bit for further processing
    let comp_17_bits = _mm512_cvtepi32_epi16(colors_shifted_17);
    let comp_22_bits = _mm512_cvtepi32_epi16(colors_shifted_22);
    let comp_23_bits = _mm512_cvtepi32_epi16(colors_shifted_23);
    let comp_27_bits = _mm512_cvtepi32_epi16(colors_shifted_27);

    // === Extract YCoCg-R Components from Color Bits ===
    // Extract individual color components by shifting within 16-bit values
    // These shifts extract the RGB components from the RGB565 format
    let red_low_shifted_1 = _mm256_srli_epi16(colors_low, 1); // R component >> 1
    let blue_low_shifted_6 = _mm256_srli_epi16(colors_low, 6); // B component >> 6
    let green_low_shifted_7 = _mm256_srli_epi16(colors_low, 7); // G component >> 7
    let blue_low_shifted_11 = _mm256_srli_epi16(colors_low, 11); // B component >> 11

    // === Combine Low and High Color Components ===
    // Create 512-bit vectors by combining low 256-bit parts with high 256-bit parts
    // This effectively processes both [`Color565`] values from each input u32
    let red_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(red_low_shifted_1), comp_17_bits, 1);
    let green_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(green_low_shifted_7), comp_23_bits, 1);
    let blue_high_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(blue_low_shifted_11), comp_27_bits, 1);
    let colors_full_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(colors_low), colors_high, 1);
    let blue_low_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(blue_low_shifted_6), comp_22_bits, 1);

    // === YCoCg-R Recorrelation Process ===
    // Step 1: Apply bit masks to extract the required component bits
    let red_masked = _mm512_and_si512(red_combined, mask_15); // Mask red with 15 (4 bits)
    let green_masked = _mm512_and_si512(green_combined, mask_15); // Mask green with 15 (4 bits)

    // Step 2: Perform recorrelation arithmetic
    // Co = B - R, Cg = G - R
    let co_component = _mm512_sub_epi16(blue_high_combined, red_masked); // Co = B - R
    let cg_component = _mm512_sub_epi16(co_component, green_masked); // Cg = (B - R) - G = B - R - G

    // Step 3: Reconstruct final color values
    // Y = R + Co + Cg, final color = (Y, Co, Cg)
    let y_component = _mm512_add_epi16(co_component, colors_full_combined); // Y = Co + Original
    let final_cg = _mm512_add_epi16(cg_component, blue_low_combined); // Final Cg value

    // === Pack Components into RGB565 Format ===
    // Step 4: Shift components to their proper bit positions in RGB565
    let y_shifted = _mm512_slli_epi16(y_component, 6); // Y component to bits [10:6]
    let cg_shifted = _mm512_slli_epi16(final_cg, 11); // Cg component to bits [15:11]

    // Step 5: Combine all components using ternary logic operations
    // Build the final RGB565 value by ORing all component parts
    // https://arnaud-carre.github.io/2024-10-06-vpternlogd/
    let recorrelated_colors = _mm512_ternarylogic_epi32(cg_shifted, y_shifted, const_1984, 248); // OR components
    let recorrelated_colors =
        _mm512_ternarylogic_epi32(recorrelated_colors, colors_full_combined, const_32, 248); // OR with base
    _mm512_ternarylogic_epi32(recorrelated_colors, cg_component, mask_31, 248) // Final OR
}

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
unsafe fn recorrelate_ycocg_r_variant2_avx512(colors_raw: __m512i) -> __m512i {
    // === Constants for YCoCg-R Recorrelation ===
    // Lifted outside loops by compiler.
    let mask_15 = _mm512_set1_epi16(15); // Mask for extracting 4-bit values
    let const_1984 = _mm512_set1_epi16(1984); // Used in ternary logic operations
    let const_32 = _mm512_set1_epi16(32); // Used in ternary logic operations
    let mask_31 = _mm512_set1_epi16(31); // Mask for extracting 5-bit values

    // === Extract Color Components for YCoCg-R Recorrelation ===
    // Convert lower 16 bits of each color pair to 16-bit values
    let colors_low = _mm512_cvtepi32_epi16(colors_raw);

    // Extract upper 16 bits by shifting and converting
    // These represent the second [`Color565`] value in each pair
    let colors_shifted_16 = _mm512_srli_epi32(colors_raw, 16);
    let colors_high = _mm512_cvtepi32_epi16(colors_shifted_16);

    // Extract specific bit ranges for recorrelation components
    let colors_shifted_17 = _mm512_srli_epi32(colors_raw, 17);
    let colors_shifted_21 = _mm512_srli_epi32(colors_raw, 21);
    let colors_shifted_22 = _mm512_srli_epi32(colors_raw, 22);
    let colors_shifted_26 = _mm512_srli_epi32(colors_raw, 26);

    // Convert shifted values to 16-bit for further processing
    let comp_17_bits = _mm512_cvtepi32_epi16(colors_shifted_17);
    let comp_21_bits = _mm512_cvtepi32_epi16(colors_shifted_21);
    let comp_22_bits = _mm512_cvtepi32_epi16(colors_shifted_22);
    let comp_26_bits = _mm512_cvtepi32_epi16(colors_shifted_26);

    // === Extract YCoCg-R Components from Color Bits ===
    // Extract individual color components by shifting within 16-bit values
    // These shifts extract the RGB components from the RGB565 format
    let red_low_shifted_1 = _mm256_srli_epi16(colors_low, 1); // R component >> 1
    let blue_low_shifted_5 = _mm256_srli_epi16(colors_low, 5); // B component >> 5
    let green_low_shifted_6 = _mm256_srli_epi16(colors_low, 6); // G component >> 6
    let blue_low_shifted_10 = _mm256_srli_epi16(colors_low, 10); // B component >> 10

    // === Combine Low and High Color Components ===
    // Create 512-bit vectors by combining low 256-bit parts with high 256-bit parts
    // This effectively processes both [`Color565`] values from each input u32
    let red_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(red_low_shifted_1), comp_17_bits, 1);
    let green_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(green_low_shifted_6), comp_22_bits, 1);
    let blue_high_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(blue_low_shifted_10), comp_26_bits, 1);
    let colors_full_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(colors_low), colors_high, 1);
    let blue_low_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(blue_low_shifted_5), comp_21_bits, 1);

    // === YCoCg-R Recorrelation Process ===
    // Step 1: Apply bit masks to extract the required component bits
    let red_masked = _mm512_and_si512(red_combined, mask_15); // Mask red with 15 (4 bits)
    let green_masked = _mm512_and_si512(green_combined, mask_15); // Mask green with 15 (4 bits)

    // Step 2: Perform recorrelation arithmetic
    // Co = B - R, Cg = G - R
    let co_component = _mm512_sub_epi16(blue_high_combined, red_masked); // Co = B - R
    let cg_component = _mm512_sub_epi16(co_component, green_masked); // Cg = (B - R) - G = B - R - G

    // Step 3: Reconstruct final color values
    // Y = R + Co + Cg, final color = (Y, Co, Cg)
    let y_component = _mm512_add_epi16(co_component, colors_full_combined); // Y = Co + Original
    let final_cg = _mm512_add_epi16(cg_component, blue_low_combined); // Final Cg value

    // === Pack Components into RGB565 Format ===
    // Step 4: Shift components to their proper bit positions in RGB565
    let y_shifted = _mm512_slli_epi16(y_component, 6); // Y component to bits [10:6]
    let cg_shifted = _mm512_slli_epi16(final_cg, 11); // Cg component to bits [15:11]

    // Step 5: Combine all components using ternary logic operations
    // Build the final RGB565 value by ORing all component parts
    let recorrelated_colors = _mm512_ternarylogic_epi32(cg_shifted, y_shifted, const_1984, 248); // OR components
    let recorrelated_colors =
        _mm512_ternarylogic_epi32(recorrelated_colors, blue_high_combined, const_32, 248); // OR with base
    _mm512_ternarylogic_epi32(recorrelated_colors, cg_component, mask_31, 248) // Final OR
}

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline]
unsafe fn recorrelate_ycocg_r_variant3_avx512(colors_raw: __m512i) -> __m512i {
    // === Constants for YCoCg-R Recorrelation ===
    // Lifted outside loops by compiler.
    let mask_15 = _mm512_set1_epi16(15); // Mask for extracting 4-bit values
    let const_1984 = _mm512_set1_epi16(1984); // Used in ternary logic operations
    let const_32 = _mm512_set1_epi16(32); // Used in ternary logic operations
    let mask_31 = _mm512_set1_epi16(31); // Mask for extracting 5-bit values

    // === Extract Color Components for YCoCg-R Recorrelation ===
    // Convert lower 16 bits of each color pair to 16-bit values
    let colors_low = _mm512_cvtepi32_epi16(colors_raw);

    // Extract upper 16 bits by shifting and converting
    // These represent the second [`Color565`] value in each pair
    let colors_shifted_16 = _mm512_srli_epi32(colors_raw, 16);
    let colors_high = _mm512_cvtepi32_epi16(colors_shifted_16);

    // Extract specific bit ranges for recorrelation components (variant3 specific shifts)
    let colors_shifted_17 = _mm512_srli_epi32(colors_raw, 17);
    let colors_shifted_18 = _mm512_srli_epi32(colors_raw, 18);
    let colors_shifted_22 = _mm512_srli_epi32(colors_raw, 22);
    let colors_shifted_23 = _mm512_srli_epi32(colors_raw, 23);
    let colors_shifted_27 = _mm512_srli_epi32(colors_raw, 27);

    // Convert shifted values to 16-bit for further processing
    let comp_17_bits = _mm512_cvtepi32_epi16(colors_shifted_17);
    let comp_18_bits = _mm512_cvtepi32_epi16(colors_shifted_18);
    let comp_22_bits = _mm512_cvtepi32_epi16(colors_shifted_22);
    let comp_23_bits = _mm512_cvtepi32_epi16(colors_shifted_23);
    let comp_27_bits = _mm512_cvtepi32_epi16(colors_shifted_27);

    // === Extract YCoCg-R Components from Color Bits ===
    // Extract individual color components by shifting within 16-bit values
    // These shifts extract the RGB components from the RGB565 format
    let red_low_shifted_2 = _mm256_srli_epi16(colors_low, 2); // R component >> 2 (variant3 specific)
    let blue_low_shifted_6 = _mm256_srli_epi16(colors_low, 6); // B component >> 6
    let green_low_shifted_7 = _mm256_srli_epi16(colors_low, 7); // G component >> 7
    let blue_low_shifted_11 = _mm256_srli_epi16(colors_low, 11); // B component >> 11
    let red_low_shifted_1 = _mm256_srli_epi16(colors_low, 1); // R component >> 1

    // === Combine Low and High Color Components ===
    // Create 512-bit vectors by combining low 256-bit parts with high 256-bit parts
    // This effectively processes both [`Color565`] values from each input u32
    let red_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(red_low_shifted_2), comp_18_bits, 1);
    let green_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(green_low_shifted_7), comp_23_bits, 1);
    let blue_high_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(blue_low_shifted_11), comp_27_bits, 1);
    let blue_low_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(blue_low_shifted_6), comp_22_bits, 1);
    let red_shifted_1_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(red_low_shifted_1), comp_17_bits, 1);
    let colors_full_combined =
        _mm512_inserti64x4(_mm512_castsi256_si512(colors_low), colors_high, 1);

    // === YCoCg-R Recorrelation Process ===
    // Step 1: Apply bit masks to extract the required component bits
    let red_masked = _mm512_and_si512(red_combined, mask_15); // Mask red with 15 (4 bits)
    let green_masked = _mm512_and_si512(green_combined, mask_15); // Mask green with 15 (4 bits)

    // Step 2: Perform recorrelation arithmetic
    // Co = B - R, Cg = G - R
    let co_component = _mm512_sub_epi16(blue_high_combined, red_masked); // Co = B - R
    let cg_component = _mm512_sub_epi16(co_component, green_masked); // Cg = (B - R) - G = B - R - G

    // Step 3: Reconstruct final color values
    // Y = R + Co + Cg, final color = (Y, Co, Cg)
    let y_component = _mm512_add_epi16(co_component, red_shifted_1_combined); // Y = Co + R_shifted_1
    let final_cg = _mm512_add_epi16(cg_component, blue_low_combined); // Final Cg value

    // === Pack Components into RGB565 Format ===
    // Step 4: Shift components to their proper bit positions in RGB565
    let y_shifted = _mm512_slli_epi16(y_component, 6); // Y component to bits [10:6]
    let cg_shifted = _mm512_slli_epi16(final_cg, 11); // Cg component to bits [15:11]
    let colors_shifted_5 = _mm512_slli_epi16(colors_full_combined, 5); // Shift colors by 5

    // Step 5: Combine all components using ternary logic operations
    // Build the final RGB565 value by ORing all component parts
    let recorrelated_colors = _mm512_ternarylogic_epi32(cg_shifted, y_shifted, const_1984, 248); // OR components
    let recorrelated_colors =
        _mm512_ternarylogic_epi32(colors_shifted_5, recorrelated_colors, const_32, 236); // Different ternary logic for variant3
    _mm512_ternarylogic_epi32(recorrelated_colors, cg_component, mask_31, 248) // Final OR
}

#[cfg(test)]
mod tests {
    use crate::normalize_blocks::ColorNormalizationMode;
    use crate::split_blocks::split::tests::assert_implementation_matches_reference;

    use crate::with_recorrelate::avx512::*;
    use crate::{
        split_blocks::split::tests::generate_bc1_test_data, transform_bc1, Bc1TransformDetails,
    };
    use dxt_lossless_transform_common::color_565::YCoCgVariant;
    use dxt_lossless_transform_common::cpu_detect::{has_avx512bw, has_avx512f};
    use rstest::rstest;

    #[rstest]
    #[case(untransform_recorr_var1, YCoCgVariant::Variant1)]
    #[case(untransform_recorr_var2, YCoCgVariant::Variant2)]
    #[case(untransform_recorr_var3, YCoCgVariant::Variant3)]
    fn can_untransform_unaligned(
        #[case] function: unsafe fn(*const u32, *const u32, *mut u8, usize) -> (),
        #[case] decorr_variant: YCoCgVariant,
    ) {
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
                        decorrelation_mode: decorr_variant,
                        split_colour_endpoints: false,
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
                function(
                    transformed_unaligned.as_ptr().add(1) as *const u32,
                    transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                );
            }

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                "unaligned untransform with recorrelation (avx512)",
                num_blocks,
            );
        }
    }
}
