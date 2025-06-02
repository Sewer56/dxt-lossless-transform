#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::ptr::{read_unaligned, write_unaligned};
use dxt_lossless_transform_common::color_565::Color565;

#[cfg(feature = "nightly")]
use dxt_lossless_transform_common::intrinsics::color_565::recorrelate::avx512::{
    recorrelate_ycocg_r_variant1_avx512, recorrelate_ycocg_r_variant2_avx512,
    recorrelate_ycocg_r_variant3_avx512,
};

#[cfg(feature = "nightly")]
#[target_feature(enable = "avx512f")]
#[target_feature(enable = "avx512bw")]
#[inline(never)] // improve register budget.
pub(crate) unsafe fn untransform_split_and_decorrelate_variant1_avx512(
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
#[inline(never)] // improve register budget.
pub(crate) unsafe fn untransform_split_and_decorrelate_variant2_avx512(
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
#[inline(never)] // improve register budget.
pub(crate) unsafe fn untransform_split_and_decorrelate_variant3_avx512(
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
