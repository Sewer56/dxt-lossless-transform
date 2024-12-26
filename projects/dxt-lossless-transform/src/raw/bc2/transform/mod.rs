pub mod portable32;
pub use portable32::*;

#[cfg(test)]
pub mod tests {
    use crate::raw::bc2::transform::portable32::u32;
    use crate::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { u32(input.as_ptr(), output.as_mut_ptr(), input.len()) }
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_bc2_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 16);
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
}
