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

pub mod transform;
pub mod untransform;

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
pub unsafe fn transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    transform::transform(input_ptr, output_ptr, len);
}

/// Transform BC1 data from standard interleaved format to separate color and index pointers
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
pub(crate) unsafe fn transform_with_separate_pointers(
    input_ptr: *const u8,
    colors_ptr: *mut u32,
    indices_ptr: *mut u32,
    len: usize,
) {
    transform::transform_with_separate_pointers(input_ptr, colors_ptr, indices_ptr, len);
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
pub(crate) unsafe fn untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    untransform::untransform(input_ptr, output_ptr, len);
}
