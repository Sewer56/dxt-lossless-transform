//! # Block Splitting Process
//!
//! This module contains the code used to split the BC3/DXT5 blocks into four separate arrays
//! of alpha endpoints, alpha indices, colours and color indices.
//!
//! ## Original BC3 data layout (as received from file)
//!
//! 2 bytes of alpha endpoints followed by 6 bytes of alpha indices, then two 16-bit colours
//! (4 bytes) and 4 bytes of color indices:
//!
//! ```text
//! Address: 0       2        8       12      16  16      18        24      28      32
//!          +-------+--------+-------+-------+   +-------+---------+-------+-------+
//! Data:    | A0-A1 |AIdx0-47| C0-C1 |I0-I15 |   | A2-A3 |AIdx48-95| C2-C3 |I16-I31|
//!          +-------+--------+-------+-------+   +-------+---------+-------+-------+
//! ```
//!
//! Each 16-byte block contains:
//! - 2 bytes of alpha endpoints (min/max alpha values for interpolation)
//! - 6 bytes of alpha indices (sixteen 3-bit indices for alpha interpolation)
//! - 4 bytes colours (2x RGB565 values)
//! - 4 bytes of packed color indices (sixteen 2-bit indices)
//!
//! ## Optimized layout
//!
//! Separates alpha endpoints, alpha indices, colours and indices into continuous streams:
//!
//! ```text
//! +-------+-------+-------+     +-------+  } Alpha endpoints section
//! | A0-A1 | A2-A3 | A4-A5 | ... | AN    |  } (2 bytes per block: 2x 8-bit)
//! +-------+-------+-------+     +-------+
//! +-------+-------+-------+     +-------+  } Alpha indices section
//! |AI0-47 |AI48-95|  ...  | ... |AI N   |  } (6 bytes per block: 16x 3-bit)
//! +-------+-------+-------+     +-------+
//! +-------+-------+-------+     +-------+  } Colours section
//! |C0  C1 |C2  C3 |C4  C5 | ... |CN CN+1|  } (4 bytes per block: 2x RGB565)
//! +-------+-------+-------+     +-------+
//! +-------+-------+-------+     +-------+  } Indices section
//! | Idx0  | Idx1  | Idx2  | ... | IdxN  |  } (4 bytes per block: 16x 2-bit)
//! +-------+-------+-------+     +-------+
//! ```
//!
//! In addition, decompression speed increases, as LZ matches are more likely
//! to be in the lower levels (L1, L2) of CPU cache. The match length is often longer, too.
//!
//! ## Key differences from BC1/DXT1 and BC2/DXT3
//!
//! - Blocks are 16 bytes like BC2
//! - Uses interpolated alpha (8 interpolated values from 2 endpoints) rather than explicit alpha values
//! - Alpha indices use 3 bits per texel (8 possible values) rather than 4 bits in BC2
//! - Color part is identical to BC1/BC2 (4 bytes colors + 4 bytes indices)
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

/// Transform bc3 data from standard interleaved format to separated alpha/color/index format
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
    split::split_blocks(input_ptr, output_ptr, len);
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
    split::split_blocks_with_separate_pointers(
        input_ptr,
        alpha_byte_ptr,
        alpha_bit_ptr,
        color_ptr,
        index_ptr,
        len,
    );
}

/// Transform bc3 data from separated alpha/color/index format back to standard interleaved format
/// using best known implementation for current CPU.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn unsplit_blocks(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len % 16 == 0);
    unsplit::unsplit_blocks(input_ptr, output_ptr, len);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::split_blocks::split::tests::{
        generate_bc3_test_data, transform_with_reference_implementation,
    };
    use crate::testutils::allocate_align_64;
    use rstest::rstest;

    #[rstest]
    #[case::min_size(1)] // 8 bytes - minimum size
    #[case::min_size(2)] // 16 bytes - SSE Register
    #[case::min_size(4)] // 32 bytes - AVX Register
    #[case::min_size(8)] // 64 bytes - SSE Unrolled Operation
    #[case::min_size(16)] // 128 bytes - AVX Unrolled Operation
    #[case::min_size(32)] // 256 bytes - Multiple Unrolled Operations
    fn test_transform_untransform(#[case] num_blocks: usize) {
        let input = generate_bc3_test_data(num_blocks);
        let mut transformed = allocate_align_64(input.len());
        let mut reconstructed = allocate_align_64(input.len());
        let mut reference = allocate_align_64(input.len());

        // Generate reference output
        transform_with_reference_implementation(input.as_slice(), reference.as_mut_slice());

        unsafe {
            // Test transform
            split_blocks(input.as_ptr(), transformed.as_mut_ptr(), input.len());
            assert_eq!(
                transformed.as_slice(),
                reference.as_slice(),
                "transform_bc3 produced different results than reference for {num_blocks} blocks"
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
                "untransform_bc3 failed to reconstruct original data for {num_blocks} blocks"
            );
        }
    }
}
