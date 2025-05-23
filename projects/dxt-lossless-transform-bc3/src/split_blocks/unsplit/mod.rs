pub mod portable;
pub use portable::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use sse2::*;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx512::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_blocks_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_detransform(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_arch = "x86_64")]
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_detransform(input_ptr, output_ptr, len);
            return;
        }
    }

    // SSE2 is required by x86-64, so no check needed
    // On i686, this is slower, so skipped.
    #[cfg(target_arch = "x86_64")]
    {
        sse2::u64_detransform_sse2(input_ptr, output_ptr, len);
    }

    #[cfg(target_arch = "x86")]
    {
        u32_detransform_v2(input_ptr, output_ptr, len);
    }
}

/// Transform bc3 data from separated alpha/color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_blocks_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform_v2(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_block_with_separate_pointers_x86(
    alpha_byte_ptr: *const u8,
    alpha_bit_ptr: *const u8,
    color_byte_ptr: *const u8,
    index_byte_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_detransform_separate_components(
                alpha_byte_ptr,
                alpha_bit_ptr,
                color_byte_ptr,
                index_byte_ptr,
                output_ptr,
                len,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(target_arch = "x86_64")]
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_detransform_separate_components(
                alpha_byte_ptr,
                alpha_bit_ptr,
                color_byte_ptr,
                index_byte_ptr,
                output_ptr,
                len,
            );
            return;
        }
    }

    // SSE2 is required by x86-64, so no check needed
    // On i686, this is slower, so skipped.
    #[cfg(target_arch = "x86_64")]
    {
        sse2::u64_detransform_sse2_separate_components(
            alpha_byte_ptr as *const u64,
            alpha_bit_ptr as *const u64,
            color_byte_ptr as *const core::arch::x86_64::__m128i,
            index_byte_ptr as *const core::arch::x86_64::__m128i,
            output_ptr,
            len,
        );
    }

    #[cfg(target_arch = "x86")]
    {
        portable::u32_detransform_with_separate_pointers(
            alpha_byte_ptr as *const u16,
            alpha_bit_ptr as *const u16,
            color_byte_ptr as *const u32,
            index_byte_ptr as *const u32,
            output_ptr,
            len,
        );
    }
}

/// Unsplit BC3 blocks, putting them back into standard interleaved format from separated component pointers
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - alpha_byte_ptr must be valid for reads of len * 2 / 16 bytes (alpha endpoints)
/// - alpha_bit_ptr must be valid for reads of len * 6 / 16 bytes (alpha indices)
/// - color_byte_ptr must be valid for reads of len * 4 / 16 bytes (color endpoints)
/// - index_byte_ptr must be valid for reads of len * 4 / 16 bytes (color indices)
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_block_with_separate_pointers(
    alpha_byte_ptr: *const u8,
    alpha_bit_ptr: *const u8,
    color_byte_ptr: *const u8,
    index_byte_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_block_with_separate_pointers_x86(
            alpha_byte_ptr,
            alpha_bit_ptr,
            color_byte_ptr,
            index_byte_ptr,
            output_ptr,
            len,
        )
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        // Cast pointers to types expected by portable implementation
        portable::u32_detransform_with_separate_pointers(
            alpha_byte_ptr as *const u16,
            alpha_bit_ptr as *const u16,
            color_byte_ptr as *const u32,
            index_byte_ptr as *const u32,
            output_ptr,
            len,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{unsplit_block_with_separate_pointers, unsplit_blocks};
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    /// Helper to assert implementation results match reference implementation
    pub(crate) fn assert_implementation_matches_reference(
        expected: &[u8],
        actual: &[u8],
        impl_name: &str,
        num_blocks: usize,
    ) {
        assert_eq!(
            expected, actual,
            "{impl_name} implementation produced different results than reference for {num_blocks} blocks.\n\
            First differing block will have predictable values:\n\
            Alpha bytes: Sequential 0-1 + (block_num * 2)\n\
            Alpha bits: Sequential 32-37 + (block_num * 6)\n\
            Colors: Sequential 128-131 + (block_num * 4)\n\
            Indices: Sequential 192-195 + (block_num * 4)"
        );
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc3_transformed_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 16);
        let mut data_ptr = data.as_mut_ptr();

        let num_alpha_bytes = data.len() * 2 / 16;
        let num_alpha_bits = data.len() * 6 / 16;
        let num_colors = data.len() * 4 / 16;
        let num_indices = data.len() * 4 / 16;
        unsafe {
            // First the alpha bytes.
            for id in 0..num_alpha_bytes {
                data_ptr.write(id as u8);
                data_ptr = data_ptr.add(1);
            }

            // Now the alpha indices.
            for id in 0..num_alpha_bits {
                data_ptr.write((id + 32) as u8);
                data_ptr = data_ptr.add(1);
            }

            // First the colors.
            for id in 0..num_colors {
                data_ptr.write((id + 128) as u8);
                data_ptr = data_ptr.add(1);
            }

            // Now the indices.
            for id in 0..num_indices {
                data_ptr.write((id + 192) as u8);
                data_ptr = data_ptr.add(1);
            }
        }

        data
    }

    #[test]
    fn validate_bc3_test_data_generator() {
        let expected: Vec<u8> = vec![
            // alpha bytes
            0, 1, 2, 3, 4, 5, // block 1 - 3
            // alpha indices
            32, 33, 34, 35, 36, 37, // block 1
            38, 39, 40, 41, 42, 43, // block 2
            44, 45, 46, 47, 48, 49, // block 3
            // colours
            128, 129, 130, 131, // block 1
            132, 133, 134, 135, // block 2
            136, 137, 138, 139, // block 3
            // indices
            0xC0, 0xC1, 0xC2, 0xC3, // block 1
            0xC4, 0xC5, 0xC6, 0xC7, // block 2
            0xC8, 0xC9, 0xCA, 0xCB, // block 3
        ];
        let output = generate_bc3_transformed_test_data(3);
        assert_eq!(output.as_slice(), expected.as_slice());
    }

    #[test]
    fn unsplit_block_with_separate_pointers_matches_unsplit_blocks() {
        for num_blocks in 1..=512 {
            let mut transformed = generate_bc3_transformed_test_data(num_blocks);
            let len = transformed.len();
            let mut output_ref = allocate_align_64(len);
            let mut output_sep = allocate_align_64(len);

            unsafe {
                unsplit_blocks(transformed.as_mut_ptr(), output_ref.as_mut_ptr(), len);
                unsplit_block_with_separate_pointers(
                    transformed.as_ptr(),
                    transformed.as_ptr().add(len / 8),
                    transformed.as_ptr().add(len / 2),
                    transformed.as_ptr().add(len * 3 / 4),
                    output_sep.as_mut_ptr(),
                    len,
                );
            }

            assert_implementation_matches_reference(
                output_ref.as_slice(),
                output_sep.as_slice(),
                "unsplit_block_with_separate_pointers",
                num_blocks,
            );
        }
    }
}
