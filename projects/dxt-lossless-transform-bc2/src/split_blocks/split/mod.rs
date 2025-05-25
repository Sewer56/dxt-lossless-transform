pub mod portable32;
pub use portable32::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use sse2::*;

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

/// Transform BC2 data from standard interleaved format to separated alpha/color/index format
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
        split_blocks_bc2_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32(input_ptr, output_ptr, len)
    }
}

/// Transform BC2 data from standard interleaved format to separated alpha/color/index format
/// using separate pointers for each component section.
///
/// # Arguments
///
/// * `input_ptr` - Pointer to the input buffer containing interleaved BC2 block data
/// * `alphas_ptr` - Pointer to the output buffer for alpha data (8 bytes per block)
/// * `colors_ptr` - Pointer to the output buffer for color data (4 bytes per block)
/// * `indices_ptr` - Pointer to the output buffer for index data (4 bytes per block)
/// * `len` - The length of the input buffer in bytes
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `alphas_ptr` must be valid for writes of `len / 2` bytes
/// - `colors_ptr` must be valid for writes of `len / 4` bytes
/// - `indices_ptr` must be valid for writes of `len / 4` bytes
/// - `len` must be divisible by 16 (BC2 block size)
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn split_blocks_with_separate_pointers(
    input_ptr: *const u8,
    alphas_ptr: *mut u64,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 16 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_with_separate_pointers_x86(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_bc2_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    // Note(sewer): The AVX512 implementation is disabled because a bunch of CPUs throttle on it,
    // leading to it being slower.,
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            avx512::permute_512(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::shuffle(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v3(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v2(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            avx512::permute_512(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::shuffle(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v3(input_ptr, output_ptr, len);
            return;
        }

        #[cfg(target_arch = "x86")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v2(input_ptr, output_ptr, len);
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
    alphas_ptr: *mut u64,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    // Note(sewer): The AVX512 implementation is disabled because a bunch of CPUs throttle on it,
    // leading to it being slower.,
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            permute_512_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            avx2::shuffle_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v3_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86")]
        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            sse2::shuffle_v2_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            permute_512_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            avx2::shuffle_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86_64")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v3_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }

        #[cfg(target_arch = "x86")]
        if cfg!(target_feature = "sse2") {
            sse2::shuffle_v2_with_separate_pointers(
                input_ptr,
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                len,
            );
            return;
        }
    }

    // Fallback to portable implementation
    u32_with_separate_pointers(input_ptr, alphas_ptr, colors_ptr, indices_ptr, len)
}

#[cfg(test)]
pub mod tests {
    use super::split_blocks_with_separate_pointers;
    use crate::split_blocks::split::portable32::u32;
    use dxt_lossless_transform_common::allocate::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { u32(input.as_ptr(), output.as_mut_ptr(), input.len()) }
    }

    /// Helper to assert implementation results match reference implementation
    pub(crate) fn assert_implementation_matches_reference(
        output_expected: &[u8],
        output_test: &[u8],
        impl_name: &str,
        num_blocks: usize,
    ) {
        assert_eq!(
            output_expected, output_test,
            "{impl_name} implementation produced different results than reference for {num_blocks} blocks.\n\
            First differing block will have predictable values:\n\
            Alpha: Sequential 0-7 + (block_num * 8)\n\
            Colors: Sequential 0x80-0x83 + (block_num * 4)\n\
            Indices: Sequential 0xC0-0xC3 + (block_num * 4)"
        );
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc2_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 16).unwrap();
        let mut data_ptr = data.as_mut_ptr();

        // Reference byte ranges to make testing easy:
        // alpha: 0x00 - 0x80
        // colors: 0x80 - 0xC0
        // indices: 0xC0 - 0xFF
        let mut alpha_byte = 0_u8;
        let mut color_byte = 0x80_u8;
        let mut index_byte = 0xC0_u8;
        unsafe {
            for _ in 0..num_blocks {
                *data_ptr.add(0) = alpha_byte.wrapping_add(0);
                *data_ptr.add(1) = alpha_byte.wrapping_add(1);
                *data_ptr.add(2) = alpha_byte.wrapping_add(2);
                *data_ptr.add(3) = alpha_byte.wrapping_add(3);
                *data_ptr.add(4) = alpha_byte.wrapping_add(4);
                *data_ptr.add(5) = alpha_byte.wrapping_add(5);
                *data_ptr.add(6) = alpha_byte.wrapping_add(6);
                *data_ptr.add(7) = alpha_byte.wrapping_add(7);
                alpha_byte = alpha_byte.wrapping_add(8);

                *data_ptr.add(8) = color_byte.wrapping_add(0);
                *data_ptr.add(9) = color_byte.wrapping_add(1);
                *data_ptr.add(10) = color_byte.wrapping_add(2);
                *data_ptr.add(11) = color_byte.wrapping_add(3);
                color_byte = color_byte.wrapping_add(4);

                *data_ptr.add(12) = index_byte.wrapping_add(0);
                *data_ptr.add(13) = index_byte.wrapping_add(1);
                *data_ptr.add(14) = index_byte.wrapping_add(2);
                *data_ptr.add(15) = index_byte.wrapping_add(3);
                index_byte = index_byte.wrapping_add(4);
                data_ptr = data_ptr.add(16);
            }
        }

        data
    }

    #[test]
    fn validate_bc2_test_data_generator() {
        let expected: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 alpha
            0x04, 0x05, 0x06, 0x07, // block 1 alpha
            0x80, 0x81, 0x82, 0x83, // block 1 colours
            0xC0, 0xC1, 0xC2, 0xC3, // block 1 indices
            // block 2
            0x08, 0x09, 0x0A, 0x0B, // block 2 alpha
            0x0C, 0x0D, 0x0E, 0x0F, // block 2 alpha
            0x84, 0x85, 0x86, 0x87, // block 2 colours
            0xC4, 0xC5, 0xC6, 0xC7, // block 2 indices
            // block 3
            0x10, 0x11, 0x12, 0x13, // block 3 alpha
            0x14, 0x15, 0x16, 0x17, // block 3 alpha
            0x88, 0x89, 0x8A, 0x8B, // block 3 colours
            0xC8, 0xC9, 0xCA, 0xCB, // block 3 indices
        ];
        let output = generate_bc2_test_data(3);
        assert_eq!(output.as_slice(), expected.as_slice());
    }

    #[test]
    fn test_reference_implementation() {
        let input: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 alpha
            0x04, 0x05, 0x06, 0x07, // block 1 alpha
            0x80, 0x81, 0x82, 0x83, // block 1 colours
            0xC0, 0xC1, 0xC2, 0xC3, // block 1 indices
            // block 2
            0x08, 0x09, 0x0A, 0x0B, // block 2 alpha
            0x0C, 0x0D, 0x0E, 0x0F, // block 2 alpha
            0x84, 0x85, 0x86, 0x87, // block 2 colours
            0xC4, 0xC5, 0xC6, 0xC7, // block 2 indices
            // block 3
            0x10, 0x11, 0x12, 0x13, // block 3 alpha
            0x14, 0x15, 0x16, 0x17, // block 3 alpha
            0x88, 0x89, 0x8A, 0x8B, // block 3 colours
            0xC8, 0xC9, 0xCA, 0xCB, // block 3 indices
        ];
        let mut output = vec![0u8; 48];
        transform_with_reference_implementation(&input, &mut output);
        assert_eq!(
            output,
            vec![
                // alpha
                0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, // block 1
                0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, // block 2
                0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, // block 3
                // colours
                0x80, 0x81, 0x82, 0x83, // block 1
                0x84, 0x85, 0x86, 0x87, // block 2
                0x88, 0x89, 0x8A, 0x8B, // block 3
                // indices
                0xC0, 0xC1, 0xC2, 0xC3, // block 1
                0xC4, 0xC5, 0xC6, 0xC7, // block 2
                0xC8, 0xC9, 0xCA, 0xCB, // block 3
            ]
        );
    }

    #[test]
    fn can_split_blocks_with_separate_pointers() {
        let input: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 alpha
            0x04, 0x05, 0x06, 0x07, // block 1 alpha
            0x80, 0x81, 0x82, 0x83, // block 1 colours
            0xC0, 0xC1, 0xC2, 0xC3, // block 1 indices
            // block 2
            0x08, 0x09, 0x0A, 0x0B, // block 2 alpha
            0x0C, 0x0D, 0x0E, 0x0F, // block 2 alpha
            0x84, 0x85, 0x86, 0x87, // block 2 colours
            0xC4, 0xC5, 0xC6, 0xC7, // block 2 indices
        ];

        // Allocate separate buffers
        let mut alphas = vec![0u64; 2]; // 2 blocks * 8 bytes per block = 16 bytes total
        let mut colors = vec![0u32; 2]; // 2 blocks * 4 bytes per block = 8 bytes total
        let mut indices = vec![0u32; 2]; // 2 blocks * 4 bytes per block = 8 bytes total

        unsafe {
            split_blocks_with_separate_pointers(
                input.as_ptr(),
                alphas.as_mut_ptr(),
                colors.as_mut_ptr(),
                indices.as_mut_ptr(),
                input.len(),
            );
        }

        // Verify alphas (8 bytes per block, read as u64)
        assert_eq!(
            alphas[0],
            u64::from_le_bytes([0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07])
        );
        assert_eq!(
            alphas[1],
            u64::from_le_bytes([0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F])
        );

        // Verify colors (4 bytes per block, read as u32)
        assert_eq!(colors[0], u32::from_le_bytes([0x80, 0x81, 0x82, 0x83]));
        assert_eq!(colors[1], u32::from_le_bytes([0x84, 0x85, 0x86, 0x87]));

        // Verify indices (4 bytes per block, read as u32)
        assert_eq!(indices[0], u32::from_le_bytes([0xC0, 0xC1, 0xC2, 0xC3]));
        assert_eq!(indices[1], u32::from_le_bytes([0xC4, 0xC5, 0xC6, 0xC7]));
    }

    #[test]
    fn split_blocks_with_separate_pointers_matches_split_blocks() {
        for num_blocks in 1..=256 {
            let input = generate_bc2_test_data(num_blocks);
            let len = input.len();

            // Test with the contiguous buffer method
            let mut output_contiguous = allocate_align_64(len).unwrap();

            // Test with separate pointers
            let mut alphas = allocate_align_64(len / 2).unwrap(); // 8 bytes per block
            let mut colors = allocate_align_64(len / 4).unwrap(); // 4 bytes per block
            let mut indices = allocate_align_64(len / 4).unwrap(); // 4 bytes per block

            unsafe {
                // Reference: contiguous buffer
                super::split_blocks(input.as_ptr(), output_contiguous.as_mut_ptr(), len);

                // Test: separate pointers
                super::split_blocks_with_separate_pointers(
                    input.as_ptr(),
                    alphas.as_mut_ptr() as *mut u64,
                    colors.as_mut_ptr() as *mut u32,
                    indices.as_mut_ptr() as *mut u32,
                    len,
                );
            }

            // Verify that separate pointer results match contiguous buffer layout
            let expected_alphas = &output_contiguous.as_slice()[0..len / 2];
            let expected_colors = &output_contiguous.as_slice()[len / 2..len / 2 + len / 4];
            let expected_indices = &output_contiguous.as_slice()[len / 2 + len / 4..];

            assert_eq!(
                alphas.as_slice(),
                expected_alphas,
                "Alpha section mismatch for {num_blocks} blocks"
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
