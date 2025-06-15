#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::arch::*;

use crate::split_blocks::unsplit::portable::u32_detransform_with_separate_pointers;

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
pub unsafe fn u64_detransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    // Process as many 64-byte blocks as possible
    let current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let alpha_byte_in_ptr = input_ptr as *const u64;
    let alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u64;
    let color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const __m128i;
    let index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const __m128i;

    u64_detransform_sse2_separate_components(
        alpha_byte_in_ptr,
        alpha_bit_in_ptr,
        color_byte_in_ptr,
        index_byte_in_ptr,
        current_output_ptr,
        len,
    );
}

/// Detransforms BC3 block data from separated components using SSE2 instructions.
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
pub unsafe fn u64_detransform_sse2_separate_components(
    mut alpha_byte_in_ptr: *const u64,
    mut alpha_bit_in_ptr: *const u64,
    mut color_byte_in_ptr: *const __m128i,
    mut index_byte_in_ptr: *const __m128i,
    mut current_output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);
    const BYTES_PER_ITERATION: usize = 64;
    let aligned_len = len - (len % BYTES_PER_ITERATION);
    if aligned_len > 0 {
        let alpha_byte_end_ptr = alpha_byte_in_ptr.add(aligned_len / 16 * 2 / 8);

        while alpha_byte_in_ptr < alpha_byte_end_ptr {
            let alpha_bytes = alpha_byte_in_ptr.read_unaligned();
            alpha_byte_in_ptr = alpha_byte_in_ptr.add(1);

            // Write alpha bytes for all 4 blocks
            write_u16(current_output_ptr, 0, shift_u64_u16(alpha_bytes, 0));
            write_u16(current_output_ptr, 16, shift_u64_u16(alpha_bytes, 16));
            write_u16(current_output_ptr, 32, shift_u64_u16(alpha_bytes, 32));
            write_u16(current_output_ptr, 48, shift_u64_u16(alpha_bytes, 48));

            // Handle alpha bits - read 8 bytes at a time
            let alpha_bits_0 = alpha_bit_in_ptr.read_unaligned();
            write_u16(current_output_ptr, 2, shift_u64_u16(alpha_bits_0, 0));
            write_u32(current_output_ptr, 4, shift_u64_u32(alpha_bits_0, 16)); // block 0 end
            write_u16(current_output_ptr, 18, shift_u64_u16(alpha_bits_0, 48)); // block 1 start (2/6 bytes), 0 alphabytes left

            let alpha_bits_1 = alpha_bit_in_ptr.add(1).read_unaligned();
            write_u32(current_output_ptr, 20, shift_u64_u32(alpha_bits_1, 0)); // block 1 complete (6/6 bytes), 4 alphabytes left
            write_u32(current_output_ptr, 34, shift_u64_u32(alpha_bits_1, 32)); // block 2 start (4/6 bytes), 0 alphabytes left

            let alpha_bits_2 = alpha_bit_in_ptr.add(2).read_unaligned();
            write_u16(current_output_ptr, 38, shift_u64_u16(alpha_bits_2, 0)); // block 2 end (6/6 bytes), 6 left
            write_u64(current_output_ptr, 50, alpha_bits_2 >> 16); // block 3 atomic write
                                                                   // Note: We overwrite here, but those bytes will be immediately replaced by the SIMD write below

            alpha_bit_in_ptr = alpha_bit_in_ptr.add(3);

            // Load and interleave colors/indices
            let colors = _mm_loadu_si128(color_byte_in_ptr);
            let indices = _mm_loadu_si128(index_byte_in_ptr);

            let low = _mm_unpacklo_epi32(colors, indices);
            let high = _mm_unpackhi_epi32(colors, indices);

            asm!(
                "movq [{out}], {low}",
                "movhps [{out_high}], {low}",
                "movq [{out_mid}], {high}",
                "movhps [{out_high_mid}], {high}",
                out = in(reg) current_output_ptr.add(8),
                out_high = in(reg) current_output_ptr.add(24),
                out_mid = in(reg) current_output_ptr.add(40),
                out_high_mid = in(reg) current_output_ptr.add(56),
                low = in(xmm_reg) low,
                high = in(xmm_reg) high,
                options(nostack, preserves_flags)
            );

            color_byte_in_ptr = color_byte_in_ptr.add(1);
            index_byte_in_ptr = index_byte_in_ptr.add(1);
            current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
        }
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

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
#[target_feature(enable = "sse2")]
pub unsafe fn u32_detransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    const BYTES_PER_ITERATION: usize = 64;
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % BYTES_PER_ITERATION);

    let mut current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let mut alpha_byte_in_ptr = input_ptr as *const u32;
    let mut alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u32;
    let mut color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const __m128i;
    let mut index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const __m128i;

    if aligned_len > 0 {
        let alpha_byte_aligned_end_ptr = input_ptr.add(aligned_len / 16 * 2) as *const u32;
        while alpha_byte_in_ptr < alpha_byte_aligned_end_ptr {
            // Write the alpha bytes.
            let alpha_bytes = alpha_byte_in_ptr.read_unaligned();
            write_u16(current_output_ptr, 0, shift_u32_u16(alpha_bytes, 0));
            write_u16(current_output_ptr, 16, shift_u32_u16(alpha_bytes, 16));
            let alpha_bytes = alpha_byte_in_ptr.add(1).read_unaligned();
            write_u16(current_output_ptr, 32, shift_u32_u16(alpha_bytes, 0));
            write_u16(current_output_ptr, 48, shift_u32_u16(alpha_bytes, 16));
            alpha_byte_in_ptr = alpha_byte_in_ptr.add(2);

            // Write the alpha bits - read 4 bytes at a time
            let alpha_bits = alpha_bit_in_ptr.read_unaligned();
            let alpha_bits_2 = alpha_bit_in_ptr.add(1).read_unaligned();
            write_u32(current_output_ptr, 2, alpha_bits);
            write_u16(current_output_ptr, 6, shift_u32_u16(alpha_bits_2, 0)); // block 0 done
            write_u16(current_output_ptr, 16 + 2, shift_u32_u16(alpha_bits_2, 16));

            let alpha_bits_3 = alpha_bit_in_ptr.add(2).read_unaligned();
            let alpha_bits_4 = alpha_bit_in_ptr.add(3).read_unaligned();
            write_u32(current_output_ptr, 16 + 4, alpha_bits_3); // block 1 done
            write_u32(current_output_ptr, 32 + 2, alpha_bits_4);

            let alpha_bits_5 = alpha_bit_in_ptr.add(4).read_unaligned();
            let alpha_bits_6 = alpha_bit_in_ptr.add(5).read_unaligned();
            write_u16(current_output_ptr, 32 + 6, shift_u32_u16(alpha_bits_5, 0)); // block 2 done
            write_u16(current_output_ptr, 48 + 2, shift_u32_u16(alpha_bits_5, 16));
            write_u32(current_output_ptr, 48 + 4, alpha_bits_6); // block 3 done
            alpha_bit_in_ptr = alpha_bit_in_ptr.add(6);

            // Load and interleave colors/indices
            let colors = _mm_loadu_si128(color_byte_in_ptr);
            let indices = _mm_loadu_si128(index_byte_in_ptr);

            let low = _mm_unpacklo_epi32(colors, indices);
            let high = _mm_unpackhi_epi32(colors, indices);

            asm!(
                "movq [{out}], {low}",
                "movhps [{out_high}], {low}",
                "movq [{out_mid}], {high}",
                "movhps [{out_high_mid}], {high}",
                out = in(reg) current_output_ptr.add(8),
                out_high = in(reg) current_output_ptr.add(24),
                out_mid = in(reg) current_output_ptr.add(40),
                out_high_mid = in(reg) current_output_ptr.add(56),
                low = in(xmm_reg) low,
                high = in(xmm_reg) high,
                options(nostack, preserves_flags)
            );

            color_byte_in_ptr = color_byte_in_ptr.add(1);
            index_byte_in_ptr = index_byte_in_ptr.add(1);
            current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
        }
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

#[inline(always)]
unsafe fn write_u16(ptr: *mut u8, offset: usize, value: u16) {
    (ptr.add(offset) as *mut u16).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u32(ptr: *mut u8, offset: usize, value: u32) {
    (ptr.add(offset) as *mut u32).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u64(ptr: *mut u8, offset: usize, value: u64) {
    (ptr.add(offset) as *mut u64).write_unaligned(value);
}

#[inline(always)]
unsafe fn shift_u64_u16(value: u64, shift: usize) -> u16 {
    (value >> shift) as u16
}

#[inline(always)]
unsafe fn shift_u32_u16(value: u32, shift: usize) -> u16 {
    (value >> shift) as u16
}

#[inline(always)]
unsafe fn shift_u64_u32(value: u64, shift: usize) -> u32 {
    (value >> shift) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_detransform_sse2, "u32", 8)]
    #[case(u64_detransform_sse2, "u64", 8)]
    fn test_sse2_unaligned(
        #[case] detransform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For SSE2: processes 64 bytes (4 blocks) per iteration, so max_blocks = 64 bytes × 2 ÷ 16 = 8
        run_standard_untransform_unaligned_test(detransform_fn, max_blocks, impl_name);
    }
}
