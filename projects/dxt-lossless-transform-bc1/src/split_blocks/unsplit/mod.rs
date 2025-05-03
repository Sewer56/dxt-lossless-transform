pub mod portable32;
pub use portable32::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_blocks_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if std::is_x86_feature_detected!("avx2") {
            avx2::permd_detransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if std::is_x86_feature_detected!("sse2") {
            sse2::unpck_detransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx2") {
            avx2::permd_detransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::unpck_detransform_unroll_2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    u32_detransform(input_ptr, output_ptr, len)
}

/// Unsplit BC1 blocks, putting them back into standard interleaved format from a separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_blocks_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform(input_ptr, output_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn unsplit_block_with_separate_pointers_x86(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        if std::is_x86_feature_detected!("avx2") {
            avx2::permd_detransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }

        if std::is_x86_feature_detected!("sse2") {
            sse2::unpck_detransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        if cfg!(target_feature = "avx2") {
            avx2::permd_detransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }

        if cfg!(target_feature = "sse2") {
            sse2::unpck_detransform_unroll_2_with_components(
                output_ptr,
                len,
                indices_ptr as *const u8,
                colors_ptr as *const u8,
            );
            return;
        }
    }

    // Fallback to portable implementation
    u32_detransform_with_separate_pointers(colors_ptr, indices_ptr, output_ptr, len)
}

/// Unsplit BC1 blocks, putting them back into standard interleaved format from a separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - colors_ptr must be valid for reads of len/2 bytes
/// - indices_ptr must be valid for reads of len/2 bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that colors_ptr and indices_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_block_with_separate_pointers(
    colors_ptr: *const u32,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        unsplit_block_with_separate_pointers_x86(colors_ptr, indices_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_detransform_with_separate_pointers(colors_ptr, indices_ptr, output_ptr, len)
    }
}

#[cfg(test)]
mod tests {
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    use super::{unsplit_block_with_separate_pointers, unsplit_blocks};

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
            Colors: Sequential 0-3 + (block_num * 4)\n\
            Indices: Sequential 128-131 + (block_num * 4)"
        );
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc1_transformed_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 8);
        let mut data_ptr = data.as_mut_ptr();

        let num_colors = data.len() / 2;
        unsafe {
            // First the colors.
            for id in 0..num_colors {
                data_ptr.write(id as u8);
                data_ptr = data_ptr.add(1);
            }

            // Now the indices.
            for id in 0..num_colors {
                data_ptr.write((id + 128) as u8);
                data_ptr = data_ptr.add(1);
            }
        }

        data
    }

    #[test]
    fn validate_bc1_test_data_generator() {
        let expected: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 colours
            0x04, 0x05, 0x06, 0x07, // block 2 colours
            0x08, 0x09, 0x0A, 0x0B, // block 3 colours
            0x80, 0x81, 0x82, 0x83, // block 1 indices
            0x84, 0x85, 0x86, 0x87, // block 2 indices
            0x88, 0x89, 0x8A, 0x8B, // block 3 indices
        ];
        let output = generate_bc1_transformed_test_data(3);
        assert_eq!(output.as_slice(), expected.as_slice());
    }

    #[test]
    fn unsplit_block_with_separate_pointers_matches_unsplit_blocks() {
        for num_blocks in 1..=512 {
            let mut transformed = generate_bc1_transformed_test_data(num_blocks);
            let len = transformed.len();
            let mut output_ref = allocate_align_64(len);
            let mut output_sep = allocate_align_64(len);

            unsafe {
                // Reference: contiguous pointers
                unsplit_blocks(transformed.as_mut_ptr(), output_ref.as_mut_ptr(), len);
                // Test separate pointers variant
                unsplit_block_with_separate_pointers(
                    transformed.as_ptr() as *const u32,
                    transformed.as_ptr().add(len / 2) as *const u32,
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
