//! # Block Splitting Process
//!
//! This module contains the code used to split the BC1 blocks into two separate arrays
//! of colours and indices.
//!
//! ## Original BC1 data layout (as received from file)
//!
//! Two 16-bit colours (4 bytes total) followed by 4 bytes of indices:
//!
//! ```text
//! Address: 0       4       8   8      12      16
//!          +-------+-------+   +-------+-------+
//! Data:    | C0-C1 | I0-I3 |   | C2-C3 | I4-I8 |
//!          +-------+-------+   +-------+-------+
//! ```
//!
//! Each 8-byte block contains:
//! - 4 bytes colours (2x RGB565 values)
//! - 4 bytes of packed indices (sixteen 2-bit indices)
//!
//! ## Optimized layout
//!
//! Separates colours and indices into continuous streams:
//!
//! ```text
//! +-------+-------+-------+     +-------+  } colours section
//! |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
//! +-------+-------+-------+     +-------+
//! +-------+-------+-------+     +-------+  } Indices section
//! | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
//! +-------+-------+-------+     +-------+
//! ```
//!
//! This rearrangement improves compression because indices are often very random (high entropy),
//! while colours are more predictable (low entropy).
//!
//! In addition, decompression speed increases (as much as 50%!), as LZ matches are more likely
//! to be in the lower levels (L1, L2) of CPU cache. The match length is often longer, too.
//!
//! ## Requirements
//!
//! A second, separate buffer to receive the results.
//!
//! While doing it in-place is technically possible, and would be beneficial in the sense that there
//! would be improved cache locality; unfortunately, that is not possible to do in a 'single pass'
//! while maintaining the spatial coherency/order.
//!
//! Introducing a second pass meanwhile would be a performance hit.
//!
//! This is possible to do with either allocating half of a buffer, and then copying the other half back,
//! or outputting it all to a single buffer. Outputting all to single buffer is faster.

pub mod split;
pub mod unsplit;

/// Transform BC1 data from standard interleaved format to separated color/index format
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
    split::split_blocks(input_ptr, output_ptr, len);
}

/// Transform BC1 data from separated color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    unsplit::unsplit_blocks(input_ptr, output_ptr, len);
}

#[cfg(test)]
mod tests {
    use crate::split_blocks::split::tests::{
        generate_bc1_test_data, transform_with_reference_implementation,
    };
    use rstest::rstest;

    #[rstest]
    #[case::min_size(1)] // 8 bytes - minimum size
    #[case::min_size(2)] // 16 bytes - SSE Register
    #[case::min_size(4)] // 32 bytes - AVX Register
    #[case::min_size(8)] // 64 bytes - SSE Unrolled Operation
    #[case::min_size(16)] // 128 bytes - AVX Unrolled Operation
    #[case::min_size(32)] // 256 bytes - Multiple Unrolled Operations
    fn test_transform_untransform(#[case] num_blocks: usize) {
        use dxt_lossless_transform_common::allocate::allocate_align_64;

        use crate::split_blocks::split::split_blocks;
        use crate::split_blocks::unsplit::unsplit_blocks;

        let input = generate_bc1_test_data(num_blocks);
        let mut transformed = allocate_align_64(input.len()).unwrap();
        let mut reconstructed = allocate_align_64(input.len()).unwrap();
        let mut reference = allocate_align_64(input.len()).unwrap();

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), reference.as_mut_slice());

        unsafe {
            // Test transform
            split_blocks(input.as_ptr(), transformed.as_mut_ptr(), input.len());
            assert_eq!(
                transformed.as_slice(),
                reference.as_slice(),
                "transform_bc1 produced different results than reference for {num_blocks} blocks"
            );

            // Test untransform
            unsplit_blocks(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
            assert_eq!(
                reconstructed.as_slice(),
                input.as_slice(),
                "untransform_bc1 failed to reconstruct original data for {num_blocks} blocks"
            );
        }
    }
}
