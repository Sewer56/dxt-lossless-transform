//! # Unsplit Split Colour-Split Blocks Module
//!
//! This module provides optimized functions for combining split color data and block indices
//! back into standard BC1 (DXT1) compressed texture blocks. This is part of the detransformation
//! process that reverses the lossless transformation applied to BC1 data.
//!
//! ## Input Format
//!
//! The module expects three separate arrays as input:
//!
//! ### Color0 Array (`color0_ptr`)
//! - Type: `*const u16`
//! - Contains the first color value for each BC1 block
//!
//! ### Color1 Array (`color1_ptr`)
//! - Type: `*const u16`
//! - Contains the second color value for each BC1 block
//!
//! ### Indices Array (`indices_ptr`)
//! - Type: `*const u32`
//! - Contains the 2-bit per pixel color indices for each BC1 block
//!
//! ## Output Format
//!
//! ### BC1 Blocks (`output_ptr`)
//! - Type: `*mut u8`
//! - Contains standard BC1/DXT1 compressed texture blocks
//! - Each block is 8 bytes in the following format:
//!   ```
//!   Offset | Size | Description
//!   -------|------|------------
//!   0      | 2    | color0 (RGB565, little-endian)
//!   2      | 2    | color1 (RGB565, little-endian)  
//!   4      | 4    | indices (2 bits per pixel, little-endian)
//!   ```

use multiversion::multiversion;

#[cfg(feature = "nightly")]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx512;

#[cfg(not(feature = "no-runtime-cpu-detection"))]
#[cfg(feature = "nightly")]
use dxt_lossless_transform_common::cpu_detect::*;

/// Optimized function to unsplit split colour-split blocks directly to BC1 blocks.
/// This combines the recorrelation of split colors and unsplitting of blocks into a single
/// operation to avoid intermediate memory copies.
///
/// # Arguments
/// * `color0_ptr` - Pointer to the array of color0 values
/// * `color1_ptr` - Pointer to the array of color1 values  
/// * `indices_ptr` - Pointer to the array of 4-byte indices for each block
/// * `output_ptr` - Pointer to the output buffer for BC1 blocks (8 bytes per block)
/// * `block_count` - Number of blocks to process
///
/// # Safety
/// This function is unsafe because it operates on raw pointers. The caller must ensure:
/// - All pointers are valid and properly aligned
/// - `color0_ptr` points to at least `block_count` valid `u16` values
/// - `color1_ptr` points to at least `block_count` valid `u16` values
/// - `indices_ptr` points to at least `block_count` valid `u32` values
/// - `output_ptr` points to at least `block_count * 8` bytes for BC1 blocks
pub(crate) unsafe fn unsplit_split_colour_split_blocks(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        unsplit_split_colour_split_blocks_x86(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        );
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        unsplit_split_colour_split_blocks_generic(
            color0_ptr,
            color1_ptr,
            indices_ptr,
            output_ptr,
            block_count,
        );
    }
}

#[cfg_attr(
    not(feature = "nightly"), 
    multiversion(targets(
        // avx512 only in nightly.
        // x86-64-v3 without lahfsahf
        "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
        // x86-64-v2 without lahfsahf
        "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
    ))
)]
#[cfg_attr(
    feature = "nightly",
    multiversion(targets(
        // x86-64-v4 without lahfsahf
        "x86_64+avx+avx2+avx512bw+avx512cd+avx512dq+avx512f+avx512vl+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
        // x86-64-v3 without lahfsahf
        "x86_64+avx+avx2+bmi1+bmi2+cmpxchg16b+f16c+fma+fxsr+lzcnt+movbe+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3+xsave",
        // x86-64-v2 without lahfsahf
        "x86_64+cmpxchg16b+fxsr+popcnt+sse+sse2+sse3+sse4.1+sse4.2+ssse3",
    ))
)]
pub(crate) unsafe fn unsplit_split_colour_split_blocks_generic(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    // Fallback implementation
    unsafe {
        // Initialize pointers
        let mut color0_ptr = color0_ptr;
        let mut color1_ptr = color1_ptr;
        let mut indices_ptr = indices_ptr;
        let mut output_ptr = output_ptr;

        // Calculate end pointer for color0
        let color0_ptr_end = color0_ptr.add(block_count);

        while color0_ptr < color0_ptr_end {
            // Read the split color values
            let color0 = color0_ptr.read_unaligned();
            let color1 = color1_ptr.read_unaligned();
            let indices = indices_ptr.read_unaligned();

            // Write BC1 block format: [color0: u16, color1: u16, indices: u32]
            // Convert to bytes and write directly
            (output_ptr as *mut u16).write_unaligned(color0);
            (output_ptr.add(2) as *mut u16).write_unaligned(color1);
            (output_ptr.add(4) as *mut u32).write_unaligned(indices);

            // Advance all pointers
            color0_ptr = color0_ptr.add(1);
            color1_ptr = color1_ptr.add(1);
            indices_ptr = indices_ptr.add(1);
            output_ptr = output_ptr.add(8);
        }
    }
}

#[inline(never)]
pub unsafe fn unsplit_split_colour_split_blocks_x86(
    color0_ptr: *const u16,
    color1_ptr: *const u16,
    indices_ptr: *const u32,
    output_ptr: *mut u8,
    block_count: usize,
) {
    #[cfg(not(feature = "no-runtime-cpu-detection"))]
    {
        #[cfg(feature = "nightly")]
        if has_avx512f() & &has_avx512bw() {
            avx512::avx512_unsplit_split_colour_split_blocks(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }
    }

    #[cfg(feature = "no-runtime-cpu-detection")]
    {
        #[cfg(feature = "nightly")]
        if cfg!(target_feature = "avx512f") & &cfg!(target_feature = "avx512bw") {
            avx512::avx512_unsplit_split_colour_split_blocks(
                color0_ptr,
                color1_ptr,
                indices_ptr,
                output_ptr,
                block_count,
            );
            return;
        }
    }

    unsplit_split_colour_split_blocks_generic(
        color0_ptr,
        color1_ptr,
        indices_ptr,
        output_ptr,
        block_count,
    );
}
