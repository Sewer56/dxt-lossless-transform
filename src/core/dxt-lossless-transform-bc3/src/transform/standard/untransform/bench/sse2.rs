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
#[target_feature(enable = "sse2")]
pub(crate) unsafe fn u32_untransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));

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

            // Store interleaved colors+indices for first block half
            _mm_storel_epi64(current_output_ptr.add(8) as *mut __m128i, low);
            // Store interleaved colors+indices for second block half
            _mm_storeh_pd(
                current_output_ptr.add(24) as *mut f64,
                _mm_castsi128_pd(low),
            );
            // Store interleaved colors+indices for third block half
            _mm_storel_epi64(current_output_ptr.add(40) as *mut __m128i, high);
            // Store interleaved colors+indices for fourth block half
            _mm_storeh_pd(
                current_output_ptr.add(56) as *mut f64,
                _mm_castsi128_pd(high),
            );

            color_byte_in_ptr = color_byte_in_ptr.add(1);
            index_byte_in_ptr = index_byte_in_ptr.add(1);
            current_output_ptr = current_output_ptr.add(BYTES_PER_ITERATION);
        }
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

#[inline(always)]
unsafe fn write_u16(ptr: *mut u8, offset: usize, value: u16) {
    (ptr.add(offset) as *mut u16).write_unaligned(value);
}

#[inline(always)]
unsafe fn write_u32(ptr: *mut u8, offset: usize, value: u32) {
    (ptr.add(offset) as *mut u32).write_unaligned(value);
}

#[inline(always)]
unsafe fn shift_u32_u16(value: u32, shift: usize) -> u16 {
    (value >> shift) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[rstest]
    #[case(u32_untransform_sse2, "u32", 8)]
    fn test_sse2_unaligned(
        #[case] untransform_fn: StandardTransformFn,
        #[case] impl_name: &str,
        #[case] max_blocks: usize,
    ) {
        // For SSE2: processes 64 bytes (4 blocks) per iteration, so max_blocks = 64 bytes × 2 ÷ 16 = 8
        run_standard_untransform_unaligned_test(untransform_fn, max_blocks, impl_name);
    }
}
