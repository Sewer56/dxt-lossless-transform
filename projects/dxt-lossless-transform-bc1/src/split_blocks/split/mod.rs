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
        if std::is_x86_feature_detected!("avx512f") {
            permute_512(input_ptr, output_ptr, len);
            return;
        }

        if std::is_x86_feature_detected!("avx2") {
            shuffle_permute_unroll_2(input_ptr, output_ptr, len);
            return;
        }

        if std::is_x86_feature_detected!("sse2") {
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

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
        let mut data = allocate_align_64(num_blocks * 8);
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
}
