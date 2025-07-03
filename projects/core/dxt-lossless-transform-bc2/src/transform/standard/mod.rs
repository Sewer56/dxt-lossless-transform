//! # BC2 Block Splitting Module
//!
//! This module provides optimized functions for separating BC2 data into two distinct arrays
//! for better compression efficiency by grouping similar data together.
//!
//! Below is a description of the untransformation process.
//! For transformation, swap the `output` and `input`.
//!
//! ## Input Format
//!
//! The module expects BC2 blocks in standard interleaved format:
//!
//! ### BC2 Blocks (`input_ptr`)
//! - Type: `*const u8`
//! - Contains standard BC2/DXT3 compressed texture blocks
//! - Each block is 16 bytes in the following format:
//!   ```ignore
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 8    | alpha data (4-bit per pixel, not interpolated)
//!   8      | 2    | color0 (RGB565, little-endian)
//!   10     | 2    | color1 (RGB565, little-endian)  
//!   12     | 4    | indices (2 bits per pixel, little-endian)
//!   ```
//!
//! ## Output Format
//!
//! The module outputs two separate arrays:
//!
//! ### Alpha Array (`alpha_ptr`)
//! - Type: `*mut u64`
//! - Contains the alpha data for each BC2 block (8 bytes per block)
//!
//! ### Colors Array (`color_ptr`)
//! - Type: `*mut u32`
//! - Contains the color endpoints for each BC2 block (4 bytes per block)
//!
//! ### Indices Array (`indices_ptr`)
//! - Type: `*mut u32`
//! - Contains the 2-bit per pixel color indices for each BC2 block

pub(crate) mod transform;
pub(crate) mod untransform;

/// Transform BC2 data using standard transform (wrapper around existing implementation)
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn transform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    transform::transform(input_ptr, output_ptr, len);
}

/// Untransform BC2 data using standard untransform (wrapper around existing implementation)
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub(crate) unsafe fn untransform(input_ptr: *const u8, output_ptr: *mut u8, len: usize) {
    untransform::untransform(input_ptr, output_ptr, len);
}
