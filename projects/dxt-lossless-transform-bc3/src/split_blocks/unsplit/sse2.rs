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
/// - pointers must be properly aligned for SSE2 access (16-byte alignment)
#[target_feature(enable = "sse2")]
pub unsafe fn u64_detransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

    let mut current_output_ptr = output_ptr;

    // Set up input pointers for each section
    let mut alpha_byte_in_ptr = input_ptr as *const u64;
    let mut alpha_bit_in_ptr = input_ptr.add(len / 16 * 2) as *const u64;
    let mut color_byte_in_ptr = input_ptr.add(len / 16 * 8) as *const __m128i;
    let mut index_byte_in_ptr = input_ptr.add(len / 16 * 12) as *const __m128i;

    if aligned_len > 0 {
        let alpha_byte_end_ptr = input_ptr.add(aligned_len / 16 * 2) as *const u64;

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
            current_output_ptr = current_output_ptr.add(64);
        }
    }

    // Process remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        // Convert pointers to the types expected by u32_detransform_with_separate_pointers
        let alpha_byte_in_ptr_u16 = alpha_byte_in_ptr as *const u16;
        let alpha_bit_in_ptr_u16 = alpha_bit_in_ptr as *const u16;
        let color_byte_in_ptr_u32 = color_byte_in_ptr as *const u32;
        let index_byte_in_ptr_u32 = index_byte_in_ptr as *const u32;
        let alpha_byte_end_ptr = input_ptr.add(len / 16 * 2) as *const u16;

        u32_detransform_with_separate_pointers(
            alpha_byte_in_ptr_u16,
            alpha_bit_in_ptr_u16,
            color_byte_in_ptr_u32,
            index_byte_in_ptr_u32,
            current_output_ptr,
            alpha_byte_end_ptr,
        );
    }
}

/// # Safety
///
/// - Same safety requirements as the scalar version:
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - pointers must be properly aligned for SSE2 access (16-byte alignment)
#[target_feature(enable = "sse2")]
pub unsafe fn u32_detransform_sse2(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // Process as many 64-byte blocks as possible
    let aligned_len = len - (len % 64);

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
            current_output_ptr = current_output_ptr.add(64);
        }
    }

    // Process remaining elements after the aligned blocks
    let remaining = len - aligned_len;
    if remaining > 0 {
        // Convert pointers to the types expected by u32_detransform_with_separate_pointers
        let alpha_byte_in_ptr_u16 = alpha_byte_in_ptr as *const u16;
        let alpha_bit_in_ptr_u16 = alpha_bit_in_ptr as *const u16;
        let color_byte_in_ptr_u32 = color_byte_in_ptr as *const u32;
        let index_byte_in_ptr_u32 = index_byte_in_ptr as *const u32;
        let alpha_byte_end_ptr = input_ptr.add(len / 16 * 2) as *const u16;

        u32_detransform_with_separate_pointers(
            alpha_byte_in_ptr_u16,
            alpha_bit_in_ptr_u16,
            color_byte_in_ptr_u32,
            index_byte_in_ptr_u32,
            current_output_ptr,
            alpha_byte_end_ptr,
        );
    }
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
    use crate::split_blocks::split::tests::generate_bc3_test_data;
    use crate::split_blocks::split::u32;
    use crate::split_blocks::unsplit::tests::assert_implementation_matches_reference;
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    type DetransformFn = unsafe fn(*const u8, *mut u8, usize);

    #[rstest]
    #[case(u32_detransform_sse2, "u32")]
    #[case(u64_detransform_sse2, "u64")]
    fn test_sse2_aligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
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
                &format!("{} (aligned)", impl_name),
                num_blocks,
            );
        }
    }

    #[rstest]
    #[case(u32_detransform_sse2, "u32")]
    #[case(u64_detransform_sse2, "u64")]
    fn test_sse2_unaligned(#[case] detransform_fn: DetransformFn, #[case] impl_name: &str) {
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
                &format!("{} (unaligned)", impl_name),
                num_blocks,
            );
        }
    }
}
