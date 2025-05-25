pub mod portable64;
pub use portable64::*;

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

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_x86(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        // Runtime feature detection
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            permute_512(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            shufps_unroll_4(input_ptr, output_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            permute_512(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if cfg!(target_feature = "sse2") {
            shufps_unroll_4(input_ptr, output_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    u32(input_ptr, output_ptr, len)
}

/// Split BC1 blocks from standard interleaved format to separated color/index format
/// using the best known implementation for the current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn split_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_x86(input_ptr, output_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        u32(input_ptr, output_ptr, len)
    }
}

/// Split BC1 blocks from standard interleaved format to separate color and index pointers
/// using the best known implementation for the current CPU.
///
/// This variant allows direct output to separate buffers for colors and indices, which can
/// be useful when you need the components stored in different memory locations or with
/// different layouts than the standard contiguous separated format.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - colors_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - indices_ptr must be valid for writes of len/2 bytes (4 bytes per block)
/// - len must be divisible by 8 (BC1 block size)
/// - It is recommended that all pointers are at least 16-byte aligned (recommended 32-byte align)
/// - The color and index buffers must not overlap with each other or the input buffer
#[inline]
pub unsafe fn split_blocks_with_separate_pointers(
    input_ptr: *const u8,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
    {
        split_blocks_with_separate_pointers_x86(input_ptr, colors_ptr, indices_ptr, len)
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
    {
        portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len)
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
#[inline(always)]
unsafe fn split_blocks_with_separate_pointers_x86(
    input_ptr: *const u8,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if dxt_lossless_transform_common::cpu_detect::has_avx512f() {
            permute_512_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_avx2() {
            // Future: add AVX2 optimized version for separate pointers
            portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if dxt_lossless_transform_common::cpu_detect::has_sse2() {
            // Future: add SSE2 optimized version for separate pointers
            portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") {
            permute_512_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if cfg!(target_feature = "avx2") {
            portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }

        if cfg!(target_feature = "sse2") {
            portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
            return;
        }
    }

    // Fallback to portable implementation
    portable32::u32_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len)
}

#[cfg(test)]
pub mod tests {
    use dxt_lossless_transform_common::allocate::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    use super::*;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { shift(input.as_ptr(), output.as_mut_ptr(), input.len()) }
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
            Colors: Sequential 0-3 + (block_num * 4)\n\
            Indices: Sequential 128-131 + (block_num * 4)"
        );
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc1_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 8).unwrap();
        let mut data_ptr = data.as_mut_ptr();

        let mut color_byte = 0_u8;
        let mut index_byte = 128_u8;
        unsafe {
            for _ in 0..num_blocks {
                *data_ptr = color_byte.wrapping_add(0);
                *data_ptr.add(1) = color_byte.wrapping_add(1);
                *data_ptr.add(2) = color_byte.wrapping_add(2);
                *data_ptr.add(3) = color_byte.wrapping_add(3);
                color_byte = color_byte.wrapping_add(4);

                *data_ptr.add(4) = index_byte.wrapping_add(0);
                *data_ptr.add(5) = index_byte.wrapping_add(1);
                *data_ptr.add(6) = index_byte.wrapping_add(2);
                *data_ptr.add(7) = index_byte.wrapping_add(3);
                index_byte = index_byte.wrapping_add(4);
                data_ptr = data_ptr.add(8);
            }
        }

        data
    }

    #[test]
    fn test_reference_implementation() {
        let input: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 colours
            0x10, 0x11, 0x12, 0x13, // block 1 indices
            0x04, 0x05, 0x06, 0x07, // block 2 colours
            0x14, 0x15, 0x16, 0x17, // block 2 indices
            0x08, 0x09, 0x0A, 0x0B, // block 3 colours
            0x18, 0x19, 0x1A, 0x1B, // block 3 indices
        ];
        let mut output = vec![0u8; 24];
        transform_with_reference_implementation(&input, &mut output);
        assert_eq!(
            output,
            vec![
                0x00, 0x01, 0x02, 0x03, // colours: block 1
                0x04, 0x05, 0x06, 0x07, // colours: block 2
                0x08, 0x09, 0x0A, 0x0B, // colours: block 3
                0x10, 0x11, 0x12, 0x13, // indices: block 1
                0x14, 0x15, 0x16, 0x17, // indices: block 2
                0x18, 0x19, 0x1A, 0x1B, // indices: block 3
            ]
        );
    }

    #[test]
    fn validate_bc1_test_data_generator() {
        let expected: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 colours
            0x80, 0x81, 0x82, 0x83, // block 1 indices
            0x04, 0x05, 0x06, 0x07, // block 2 colours
            0x84, 0x85, 0x86, 0x87, // block 2 indices
            0x08, 0x09, 0x0A, 0x0B, // block 3 colours
            0x88, 0x89, 0x8A, 0x8B, // block 3 indices
        ];
        let output = generate_bc1_test_data(3);
        assert_eq!(output.as_slice(), expected.as_slice());
    }

    #[test]
    fn split_blocks_with_separate_pointers_matches_split_blocks() {
        for num_blocks in 1..=512 {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();
            let mut output_ref = allocate_align_64(len).unwrap();
            let mut colors_sep = allocate_align_64(len / 2).unwrap();
            let mut indices_sep = allocate_align_64(len / 2).unwrap();

            unsafe {
                // Reference: contiguous output
                split_blocks(input.as_ptr(), output_ref.as_mut_ptr(), len);

                // Test separate pointers variant
                split_blocks_with_separate_pointers(
                    input.as_ptr(),
                    colors_sep.as_mut_ptr() as *mut u32,
                    indices_sep.as_mut_ptr() as *mut u32,
                    len,
                );
            }

            // Compare colors section (first half)
            assert_eq!(
                &output_ref.as_slice()[0..len / 2],
                colors_sep.as_slice(),
                "Colors section doesn't match for {num_blocks} blocks"
            );

            // Compare indices section (second half)
            assert_eq!(
                &output_ref.as_slice()[len / 2..],
                indices_sep.as_slice(),
                "Indices section doesn't match for {num_blocks} blocks"
            );
        }
    }

    #[test]
    fn can_split_blocks_with_separate_pointers() {
        let input: Vec<u8> = vec![
            0x00, 0x01, 0x02, 0x03, // block 1 colours
            0x10, 0x11, 0x12, 0x13, // block 1 indices
            0x04, 0x05, 0x06, 0x07, // block 2 colours
            0x14, 0x15, 0x16, 0x17, // block 2 indices
        ];

        let mut colors = vec![0u8; 8];
        let mut indices = vec![0u8; 8];

        unsafe {
            split_blocks_with_separate_pointers(
                input.as_ptr(),
                colors.as_mut_ptr() as *mut u32,
                indices.as_mut_ptr() as *mut u32,
                input.len(),
            );
        }

        assert_eq!(
            colors,
            vec![
                0x00, 0x01, 0x02, 0x03, // block 1 colours
                0x04, 0x05, 0x06, 0x07, // block 2 colours
            ]
        );

        assert_eq!(
            indices,
            vec![
                0x10, 0x11, 0x12, 0x13, // block 1 indices
                0x14, 0x15, 0x16, 0x17, // block 2 indices
            ]
        );
    }
}
