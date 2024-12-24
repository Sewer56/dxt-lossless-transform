pub mod portable32;
pub use portable32::*;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod sse2;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub mod avx2;

#[cfg(test)]
mod tests {

    use crate::raw::bc1::testutils::allocate_align_64;
    use safe_allocator_api::RawAlloc;

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
}
