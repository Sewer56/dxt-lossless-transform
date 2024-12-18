pub mod portable64;
pub use portable64::*;

#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse2")]
pub mod sse2;

#[cfg(target_arch = "x86_64")]
#[cfg(target_feature = "sse2")]
pub use sse2::*;

#[cfg(test)]
mod tests {
    use safe_allocator_api::RawAlloc;

    use crate::raw::dxt1::testutils::allocate_align_64;

    use super::*;

    /// Transforms the input data using a good known reference implementation.
    pub(crate) fn transform_with_reference_implementation(input: &[u8], output: &mut [u8]) {
        unsafe { shift(input.as_ptr(), output.as_mut_ptr(), input.len()) }
    }

    // Helper to generate test data of specified size (in blocks)
    pub(crate) fn generate_dxt1_test_data(num_blocks: usize) -> RawAlloc {
        let mut data = allocate_align_64(num_blocks * 8);
        let data_ptr = data.as_mut_ptr();

        for block_idx in 0..num_blocks {
            // Colors: Sequential bytes 1-64 (ensuring no overlap with indices)
            unsafe {
                *data_ptr.add(block_idx * 4) = (1 + block_idx * 4) as u8;
                *data_ptr.add(block_idx * 4 + 1) = (2 + block_idx * 4) as u8;
                *data_ptr.add(block_idx * 4 + 2) = (3 + block_idx * 4) as u8;
                *data_ptr.add(block_idx * 4 + 3) = (4 + block_idx * 4) as u8;
            }

            // Indices: Sequential bytes 128-191 (well separated from colors)
            unsafe {
                *data_ptr.add(block_idx * 4 + 4) = (128 + block_idx * 4) as u8;
                *data_ptr.add(block_idx * 4 + 5) = (129 + block_idx * 4) as u8;
                *data_ptr.add(block_idx * 4 + 6) = (130 + block_idx * 4) as u8;
                *data_ptr.add(block_idx * 4 + 7) = (131 + block_idx * 4) as u8;
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
}
