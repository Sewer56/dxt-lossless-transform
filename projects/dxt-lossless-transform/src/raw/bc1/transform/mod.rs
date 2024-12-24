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

#[cfg(test)]
pub mod tests {

    use super::*;
    use crate::raw::bc1::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { shift(input.as_ptr(), output.as_mut_ptr(), input.len()) }
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
