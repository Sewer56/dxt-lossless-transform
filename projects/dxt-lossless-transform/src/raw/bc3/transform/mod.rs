pub mod portable32;
pub use portable32::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use avx2::*;

#[cfg(test)]
pub mod tests {
    use crate::raw::bc3::transform::portable32::u32;
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { u32(input.as_ptr(), output.as_mut_ptr(), input.len()) }
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

                *data_ptr.add(2) = alpha_index_byte.wrapping_add(0);
                *data_ptr.add(3) = alpha_index_byte.wrapping_add(1);
                *data_ptr.add(4) = alpha_index_byte.wrapping_add(2);
                *data_ptr.add(5) = alpha_index_byte.wrapping_add(3);
                *data_ptr.add(6) = alpha_index_byte.wrapping_add(4);
                *data_ptr.add(7) = alpha_index_byte.wrapping_add(5);
                alpha_index_byte = alpha_index_byte.wrapping_add(6);

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
}
