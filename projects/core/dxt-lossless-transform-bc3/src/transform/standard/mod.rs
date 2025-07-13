//! # BC3 Block Splitting Module
//!
//! This module provides optimized functions for separating BC3 data into four distinct arrays
//! for better compression efficiency by grouping similar data together.
//!
//! Below is a description of the untransformation process.
//! For transformation, swap the `output` and `input`.
//!
//! ## Input Format
//!
//! The module expects BC3 blocks in standard interleaved format:
//!
//! ### BC3 Blocks (`input_ptr`)
//! - Type: `*const u8`
//! - Contains standard BC3/DXT5 compressed texture blocks
//! - Each block is 16 bytes in the following format:
//!   ```ignore
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 2    | alpha endpoints (2x 8-bit values for interpolation)
//!   2      | 6    | alpha indices (16x 3-bit indices for alpha interpolation)
//!   8      | 2    | color0 (RGB565, little-endian)
//!   10     | 2    | color1 (RGB565, little-endian)
//!   12     | 4    | color indices (2 bits per pixel, little-endian)
//!   ```
//!
//! ## Output Format
//!
//! The module outputs four separate arrays:
//!
//! ### Alpha Endpoints Array (`alpha_byte_ptr`)
//! - Type: `*mut u16`
//! - Contains the alpha endpoint pairs for each BC3 block (2 bytes per block)
//!
//! ### Alpha Indices Array (`alpha_bit_ptr`)
//! - Type: `*mut u16`
//! - Contains the alpha indices for each BC3 block (6 bytes per block)
//!
//! ### Colors Array (`color_ptr`)
//! - Type: `*mut u32`
//! - Contains the color endpoints for each BC3 block (4 bytes per block)
//!
//! ### Color Indices Array (`index_ptr`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC3 block

/// See [`super`] for the exact details.
pub mod transform;

/// See [`super`] for the exact details.
pub mod untransform;

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
pub unsafe fn transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));
    transform::transform(input_ptr, output_ptr, len);
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
pub unsafe fn untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    debug_assert!(len.is_multiple_of(16));
    untransform::untransform(input_ptr, output_ptr, len);
}
