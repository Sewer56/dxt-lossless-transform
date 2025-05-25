pub mod portable32;
pub use portable32::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx2::*;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx512;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx512::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            avx512::avx512_vbmi(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::u32_avx2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            avx512::avx512_vbmi(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::u32_avx2(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    u32(input_ptr, output_ptr, len)
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_with_separate_pointers_x86(
    input_ptr: *const u8,
    alpha_byte_ptr: *mut u16,
    alpha_bit_ptr: *mut u16,
    color_ptr: *mut u32,
    index_ptr: *mut u32,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512vbmi() {
            let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
            avx512::avx512_vbmi_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr as *mut u8,
                alpha_bit_ptr as *mut u8,
                color_ptr as *mut u8,
                index_ptr as *mut u8,
                len,
            );
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
            avx2::u32_avx2_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr,
                alpha_bit_ptr as *mut u8,
                color_ptr,
                index_ptr,
                alpha_byte_end_ptr,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512vbmi") {
            let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
            avx512::avx512_vbmi_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr as *mut u8,
                alpha_bit_ptr as *mut u8,
                color_ptr as *mut u8,
                index_ptr as *mut u8,
                len,
            );
            return;
        }

        if cfg!(target_feature = "avx2") {
            let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
            avx2::u32_avx2_with_separate_pointers(
                input_ptr,
                alpha_byte_ptr,
                alpha_bit_ptr as *mut u8,
                color_ptr,
                index_ptr,
                alpha_byte_end_ptr,
            );
            return;
        }
    }

    // Fallback to portable implementation
    let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
    u32_with_separate_endpoints(
        input_ptr,
        alpha_byte_ptr,
        alpha_bit_ptr,
        color_ptr,
        index_ptr,
        alpha_byte_end_ptr,
    )
}

/// Transform bc3 data from standard interleaved format to separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn split_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32(input_ptr, output_ptr, len)
    }
}

/// Transform BC3 data from standard interleaved format to separated component format
/// using separate pointers for each component section.
///
/// # Arguments
///
/// * `input_ptr` - Pointer to the input buffer containing interleaved BC3 block data
/// * `alpha_byte_ptr` - Pointer to the output buffer for alpha endpoint data (2 bytes per block)
/// * `alpha_bit_ptr` - Pointer to the output buffer for alpha index data (6 bytes per block)  
/// * `color_ptr` - Pointer to the output buffer for color endpoint data (4 bytes per block)
/// * `index_ptr` - Pointer to the output buffer for color index data (4 bytes per block)
/// * `len` - The length of the input buffer in bytes
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `alpha_byte_ptr` must be valid for writes of `len * 2 / 16` bytes
/// - `alpha_bit_ptr` must be valid for writes of `len * 6 / 16` bytes
/// - `color_ptr` must be valid for writes of `len * 4 / 16` bytes
/// - `index_ptr` must be valid for writes of `len * 4 / 16` bytes
/// - `len` must be divisible by 16 (BC3 block size)
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - The component buffers must not overlap with each other or the input buffer
#[inline]
pub unsafe fn split_blocks_with_separate_pointers(
    input_ptr: *const u8,
    alpha_byte_ptr: *mut u16,
    alpha_bit_ptr: *mut u16,
    color_ptr: *mut u32,
    index_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_with_separate_pointers_x86(
            input_ptr,
            alpha_byte_ptr,
            alpha_bit_ptr,
            color_ptr,
            index_ptr,
            len,
        )
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        let alpha_byte_end_ptr = alpha_byte_ptr.add(len / 16);
        u32_with_separate_endpoints(
            input_ptr,
            alpha_byte_ptr,
            alpha_bit_ptr,
            color_ptr,
            index_ptr,
            alpha_byte_end_ptr,
        )
    }
}

#[cfg(test)]
pub mod tests {
    use crate::split_blocks::split::portable32::u32;
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { u32(input.as_ptr(), output.as_mut_ptr(), input.len()) }
    }

    /// Helper to assert implementation results match reference implementation
    pub(crate) fn assert_implementation_matches_reference(
        expected: &[u8],
        actual: &[u8],
        impl_name: &str,
        num_blocks: usize,
    ) {
        assert_eq!(
            expected, actual,
            "BC3 {impl_name} implementation produced different results than reference for {num_blocks} blocks.\n\
            First differing block will have predictable values:\n\
            Alpha: Sequential 00-31\n\
            Alpha Indices: Sequential 32-127\n\
            Colors: Sequential 128-191\n\
            Indices: Sequential 192-255"
        );
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc3_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 16);
        let mut data_ptr = data.as_mut_ptr();

        // Reference byte ranges to make testing easy:
        // alpha: 00 - 31
        // alpha_indices: 32 - 127
        // colors: 128 - 191
        // indices: 192 - 255
        let mut alpha_byte: u8 = 0_u8;
        let mut alpha_index_byte = 32_u8;
        let mut color_byte = 128_u8;
        let mut index_byte = 192_u8;
        unsafe {
            for _ in 0..num_blocks {
                *data_ptr.add(0) = alpha_byte.wrapping_add(0);
                *data_ptr.add(1) = alpha_byte.wrapping_add(1);
                alpha_byte = alpha_byte.wrapping_add(2);
                if alpha_byte >= 32 {
                    alpha_byte = alpha_byte.wrapping_sub(32);
                }

                *data_ptr.add(2) = alpha_index_byte.wrapping_add(0);
                *data_ptr.add(3) = alpha_index_byte.wrapping_add(1);
                *data_ptr.add(4) = alpha_index_byte.wrapping_add(2);
                *data_ptr.add(5) = alpha_index_byte.wrapping_add(3);
                *data_ptr.add(6) = alpha_index_byte.wrapping_add(4);
                *data_ptr.add(7) = alpha_index_byte.wrapping_add(5);
                alpha_index_byte = alpha_index_byte.wrapping_add(6);
                if alpha_index_byte >= 128 {
                    alpha_index_byte = alpha_index_byte.wrapping_sub(96);
                }

                *data_ptr.add(8) = color_byte.wrapping_add(0);
                *data_ptr.add(9) = color_byte.wrapping_add(1);
                *data_ptr.add(10) = color_byte.wrapping_add(2);
                *data_ptr.add(11) = color_byte.wrapping_add(3);
                color_byte = color_byte.wrapping_add(4);
                if color_byte >= 192 {
                    color_byte = color_byte.wrapping_sub(64);
                }

                *data_ptr.add(12) = index_byte.wrapping_add(0);
                *data_ptr.add(13) = index_byte.wrapping_add(1);
                *data_ptr.add(14) = index_byte.wrapping_add(2);
                *data_ptr.add(15) = index_byte.wrapping_add(3);
                index_byte = index_byte.wrapping_add(4);
                if index_byte < 192 {
                    index_byte = index_byte.wrapping_sub(64);
                }

                data_ptr = data_ptr.add(16);
            }
        }

        data
    }

    #[test]
    fn validate_bc3_test_data_generator() {
        let expected: Vec<u8> = vec![
            0, 1, // block 1 alpha
            32, 33, 34, 35, 36, 37, // block 1 alpha indices
            128, 129, 130, 131, // block 1 colours
            192, 193, 194, 195, // block 1 indices
            // block 2
            2, 3, // block 2 alpha
            38, 39, 40, 41, 42, 43, // block 2 alpha indices
            132, 133, 134, 135, // block 2 colours
            196, 197, 198, 199, // block 2 indices
            // block 3
            4, 5, // block 3 alpha
            44, 45, 46, 47, 48, 49, // block 3 alpha indices
            136, 137, 138, 139, // block 3 colours
            200, 201, 202, 203, // block 3 indices
        ];
        let output = generate_bc3_test_data(3);
        assert_eq!(output.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_reference_implementation() {
        let input = vec![
            0, 1, // block 1 alpha
            32, 33, 34, 35, 36, 37, // block 1 alpha indices
            128, 129, 130, 131, // block 1 colours
            192, 193, 194, 195, // block 1 indices
            // block 2
            2, 3, // block 2 alpha
            38, 39, 40, 41, 42, 43, // block 2 alpha indices
            132, 133, 134, 135, // block 2 colours
            196, 197, 198, 199, // block 2 indices
            // block 3
            4, 5, // block 3 alpha
            44, 45, 46, 47, 48, 49, // block 3 alpha indices
            136, 137, 138, 139, // block 3 colours
            200, 201, 202, 203, // block 3 indices
        ];
        let mut output = vec![0u8; 48];
        transform_with_reference_implementation(&input, &mut output);
        assert_eq!(
            output,
            vec![
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
            ]
        );
    }

    #[test]
    fn split_blocks_with_separate_pointers_matches_split_blocks() {
        for num_blocks in 1..=256 {
            let input = generate_bc3_test_data(num_blocks);
            let len = input.len();

            // Test with the contiguous buffer method
            let mut output_contiguous = allocate_align_64(len);

            // Test with separate pointers
            let mut alpha_bytes = allocate_align_64(len / 8); // 2 bytes per block
            let mut alpha_bits = allocate_align_64(len * 3 / 8); // 6 bytes per block
            let mut colors = allocate_align_64(len / 4); // 4 bytes per block
            let mut indices = allocate_align_64(len / 4); // 4 bytes per block

            unsafe {
                // Reference: contiguous buffer
                super::split_blocks(input.as_ptr(), output_contiguous.as_mut_ptr(), len);

                // Test: separate pointers
                super::split_blocks_with_separate_pointers(
                    input.as_ptr(),
                    alpha_bytes.as_mut_ptr() as *mut u16,
                    alpha_bits.as_mut_ptr() as *mut u16,
                    colors.as_mut_ptr() as *mut u32,
                    indices.as_mut_ptr() as *mut u32,
                    len,
                );
            }

            // Verify that separate pointer results match contiguous buffer layout
            let expected_alpha_bytes = &output_contiguous.as_slice()[0..len / 8];
            let expected_alpha_bits = &output_contiguous.as_slice()[len / 8..len / 2];
            let expected_colors = &output_contiguous.as_slice()[len / 2..len * 3 / 4];
            let expected_indices = &output_contiguous.as_slice()[len * 3 / 4..];

            assert_eq!(
                alpha_bytes.as_slice(),
                expected_alpha_bytes,
                "Alpha bytes section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                alpha_bits.as_slice(),
                expected_alpha_bits,
                "Alpha bits section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                colors.as_slice(),
                expected_colors,
                "Color section mismatch for {num_blocks} blocks"
            );
            assert_eq!(
                indices.as_slice(),
                expected_indices,
                "Index section mismatch for {num_blocks} blocks"
            );
        }
    }
}
