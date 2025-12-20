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
#[target_feature(enable = "avx2")]
pub(crate) unsafe fn transform_with_split_alphas(
    input_ptr: *const u8,
    alpha0_out: *mut u8,
    alpha1_out: *mut u8,
    alpha_indices_out: *mut u16,
    colors_out: *mut u32,
    color_indices_out: *mut u32,
    block_count: usize,
) {
    // Process 8 blocks (128 bytes) at a time
    let len = block_count * 16;
    let aligned_len = len - (len % 128);
    let remaining_len = len - aligned_len;

    let mut current_input_ptr = input_ptr;
    let input_aligned_end_ptr = input_ptr.add(aligned_len);
    let mut current_alpha0_out = alpha0_out;
    let mut current_alpha1_out = alpha1_out;
    let mut current_alpha_indices_out = alpha_indices_out as *mut u8;
    let mut current_colors_out = colors_out;
    let mut current_color_indices_out = color_indices_out;

    // Create gather indices for colors (offset 8) and indices (offset 12)
    // For eight blocks, each 16 bytes apart
    let colour_offsets = _mm256_set_epi32(
        120, 104, 88, 72, 56, 40, 24, 8, // Block 8, 7, 6, 5, 4, 3, 2, 1 color offsets
    );
    let indices_offsets = _mm256_set_epi32(
        124, 108, 92, 76, 60, 44, 28, 12, // Block 8, 7, 6, 5, 4, 3, 2, 1 index offsets
    );

    // Create gather mask (all 1s)
    let mask = _mm256_set1_epi32(-1);

    while current_input_ptr < input_aligned_end_ptr {
        // Gather colors using _mm256_mask_i32gather_epi32 intrinsic
        let colours = _mm256_mask_i32gather_epi32::<1>(
            _mm256_setzero_si256(),
            current_input_ptr as *const i32,
            colour_offsets,
            mask,
        );

        // Gather indices using _mm256_mask_i32gather_epi32 intrinsic
        let indices = _mm256_mask_i32gather_epi32::<1>(
            _mm256_setzero_si256(),
            current_input_ptr as *const i32,
            indices_offsets,
            mask,
        );

        // Write out alpha0 bytes as a single u64 (8 bytes from offsets 0, 16, 32, 48, 64, 80, 96, 112)
        write_alpha_bytes_u64(
            current_alpha0_out as *mut u64,
            current_input_ptr,
            0,
            16,
            32,
            48,
            64,
            80,
            96,
            112,
        );
        current_alpha0_out = current_alpha0_out.add(8);

        // Write out alpha1 bytes as a single u64 (8 bytes from offsets 1, 17, 33, 49, 65, 81, 97, 113)
        write_alpha_bytes_u64(
            current_alpha1_out as *mut u64,
            current_input_ptr,
            1,
            17,
            33,
            49,
            65,
            81,
            97,
            113,
        );
        current_alpha1_out = current_alpha1_out.add(8);

        // Write out all alpha indices components (6 bytes per block), for all 8 blocks
        // Resolved and optimized out at compile time!
        if cfg!(target_arch = "x86_64") {
            // Write out all alpha indices components 8 bytes at a time, for all 8 blocks
            write_alpha_bits_u64(current_alpha_indices_out, 0, current_input_ptr, 2);
            write_alpha_bits_u64(current_alpha_indices_out, 6, current_input_ptr, 18);
            write_alpha_bits_u64(current_alpha_indices_out, 12, current_input_ptr, 34);
            write_alpha_bits_u64(current_alpha_indices_out, 18, current_input_ptr, 50);
            write_alpha_bits_u64(current_alpha_indices_out, 24, current_input_ptr, 66);
            write_alpha_bits_u64(current_alpha_indices_out, 30, current_input_ptr, 82);
            write_alpha_bits_u64(current_alpha_indices_out, 36, current_input_ptr, 98);
            // Note: The u64 write overflows by 2 bytes; so on the last write, we need to not overflow, as to
            // not overwrite elements in the next section; so we do a regular write here.
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 42, 114);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 44, 116);
        } else {
            // Write out all alpha indices components (2 bytes then 4 bytes for each block), for all 8 blocks
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 0, 2);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 2, 4);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 6, 18);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 8, 20);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 12, 34);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 14, 36);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 18, 50);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 20, 52);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 24, 66);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 26, 68);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 30, 82);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 32, 84);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 36, 98);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 38, 100);
            write_alpha_bit_u16(current_alpha_indices_out, current_input_ptr, 42, 114);
            write_alpha_bit_u32(current_alpha_indices_out, current_input_ptr, 44, 116);
        }
        current_alpha_indices_out = current_alpha_indices_out.add(48);

        // Store results - each register now contains 8 blocks worth of data
        _mm256_storeu_si256(current_colors_out as *mut __m256i, colours);
        _mm256_storeu_si256(current_color_indices_out as *mut __m256i, indices);

        // Update pointers
        current_input_ptr = current_input_ptr.add(128); // Move forward 8 blocks
        current_colors_out = current_colors_out.add(8); // 8 u32s per m256i
        current_color_indices_out = current_color_indices_out.add(8); // 8 u32s per m256i
    }

    // Process any remaining blocks (less than 8)
    if remaining_len > 0 {
        generic_transform(
            current_input_ptr,
            current_alpha0_out,
            current_alpha1_out,
            current_alpha_indices_out as *mut u16,
            current_colors_out,
            current_color_indices_out,
            remaining_len / 16,
        );
    }
}

#[inline(always)]
#[allow(clippy::too_many_arguments)]
unsafe fn write_alpha_bytes_u64(
    out_ptr: *mut u64,
    in_ptr: *const u8,
    offset0: usize,
    offset1: usize,
    offset2: usize,
    offset3: usize,
    offset4: usize,
    offset5: usize,
    offset6: usize,
    offset7: usize,
) {
    // Read 8 scattered u8 values
    let b0 = in_ptr.add(offset0).read() as u64;
    let b1 = in_ptr.add(offset1).read() as u64;
    let b2 = in_ptr.add(offset2).read() as u64;
    let b3 = in_ptr.add(offset3).read() as u64;
    let b4 = in_ptr.add(offset4).read() as u64;
    let b5 = in_ptr.add(offset5).read() as u64;
    let b6 = in_ptr.add(offset6).read() as u64;
    let b7 = in_ptr.add(offset7).read() as u64;

    // Combine into a single u64 via shifts and OR
    let combined = b0
        | (b1 << 8)
        | (b2 << 16)
        | (b3 << 24)
        | (b4 << 32)
        | (b5 << 40)
        | (b6 << 48)
        | (b7 << 56);

    // Write as a single u64
    out_ptr.write_unaligned(combined);
}

#[inline(always)]
unsafe fn write_alpha_bit_u16(
    out_ptr: *mut u8,
    in_ptr: *const u8,
    out_offset: usize,
    in_offset: usize,
) {
    (out_ptr.add(out_offset) as *mut u16)
        .write_unaligned((in_ptr.add(in_offset) as *const u16).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bit_u32(
    out_ptr: *mut u8,
    in_ptr: *const u8,
    out_offset: usize,
    in_offset: usize,
) {
    (out_ptr.add(out_offset) as *mut u32)
        .write_unaligned((in_ptr.add(in_offset) as *const u32).read_unaligned());
}

#[inline(always)]
unsafe fn write_alpha_bits_u64(
    out_ptr: *mut u8,
    out_offset: usize,
    in_ptr: *const u8,
    in_offset: usize,
) {
    // Read both parts using unaligned loads
    let first_part = (in_ptr.add(in_offset) as *const u16).read_unaligned();
    let second_part = (in_ptr.add(in_offset + 2) as *const u32).read_unaligned();
    let combined_value = ((second_part as u64) << 16) | (first_part as u64);

    // Write using unaligned store
    (out_ptr.add(out_offset) as *mut u64).write_unaligned(combined_value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    fn test_avx2_roundtrip() {
        if !has_avx2() {
            return;
        }

        // For AVX2: processes 128 bytes (8 blocks) per iteration, so max_blocks = 128 bytes × 2 ÷ 16 = 16
        run_split_alphas_transform_roundtrip_test(transform_with_split_alphas, 16, "avx2");
    }
}
