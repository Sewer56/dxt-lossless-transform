//! # Block Splitting Process
//!
//! This module contains the code used to split the BC2/DXT3 blocks into three separate arrays
//! of alpha values, colours and indices.
//!
//! ## Original BC2 data layout (as received from file)
//!
//! 8 bytes of alpha values followed by two 16-bit colours (4 bytes) and 4 bytes of color indices:
//!
//! ```text
//! Address: 0       8       12      16  16      24      28      32
//!          +-------+-------+-------+   +-------+-------+--------+
//! Data:    |A0-A15 | C0-C1 | I0-I15 |  |A16-A31| C2-C3 | I6-I31 |
//!          +-------+-------+-------+   +-------+-------+--------+
//! ```
//!
//! Each 16-byte block contains:
//! - 8 bytes of explicit alpha (sixteen 4-bit alpha values)
//! - 4 bytes colours (2x RGB565 values)
//! - 4 bytes of packed color indices (sixteen 2-bit indices)
//!
//! ## Optimized layout
//!
//! Separates alpha, colours and indices into continuous streams:
//!
//! ```text
//! +-------+-------+-------+     +-------+  } Alpha section
//! | A0    | A1    | A2    | ... | AN    |  } (8 bytes per block: 16x 4-bit)
//! +-------+-------+-------+     +-------+
//! +-------+-------+-------+     +-------+  } Colours section
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
//! In addition, decompression speed increases, as LZ matches are more likely
//! to be in the lower levels (L1, L2) of CPU cache. The match length is often longer, too.
//!
//! ## Key differences from BC1/DXT1
//!
//! - Blocks are 16 bytes instead of 8 bytes
//! - Includes explicit 4-bit alpha values (no alpha interpolation)
//! - No special "transparent black" color combinations
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
//!
pub mod transform;
pub mod untransform;

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
    transform::split_blocks(input_ptr, output_ptr, len);
}

/// Transform BC2 data from separated alpha/color/index format back to standard interleaved format
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
    untransform::unsplit_blocks(input_ptr, output_ptr, len);
}
