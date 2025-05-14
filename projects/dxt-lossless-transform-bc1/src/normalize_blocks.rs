//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC1 blocks to improve compression ratio
//! by making solid color blocks and transparent blocks have consistent representations.
//!
//! ## BC1 Block Format
//!
//! First, let's recall the BC1 block format:
//!
//! ```text
//! Address: 0        2        4       8
//!          +--------+--------+--------+
//! Data:    | Color0 | Color1 | Indices|
//!          +--------+--------+--------+
//! ```
//!
//! Where:
//! - `Color0` and `Color1` are 16-bit RGB565 color values (2 bytes each)
//! - `Indices` are 4 bytes containing sixteen 2-bit indices (one for each pixel in the 4x4 block)
//!
//! ## Normalization Rules
//!
//! The normalization process applies the following rules to improve compression:
//!
//! ### 1. Solid Color Blocks
//!
//! When an entire block represents a single solid color with a clean conversion between RGBA8888
//! and RGB565, we standardize the representation:
//!
//! ```text
//! +--------+--------+--------+
//! | Color  | 0x0000 |  0x00  |
//! +--------+--------+--------+
//! ```
//!
//! We place the block's color in `Color0`, set `Color1` to zero, and set all indices to zero
//! (represented by all-zero bytes in the indices section). Can also be repeat of same byte.
//!
//! The implementation checks for this case by:
//! 1. Decoding the block to get all 16 pixels
//! 2. Checking that all pixels have the same color
//! 3. Verifying the color can be cleanly round-tripped through RGB565 encoding
//! 4. Constructing a new normalized block with the pattern above
//!
//! ### 2. Fully Transparent Blocks
//!
//! For blocks that are completely transparent (common in textures with alpha), we standardize
//! the representation to all 1's:
//!
//! ```text
//! +--------+--------+--------+
//! | 0xFFFF | 0xFFFF | 0xFFFF |
//! +--------+--------+--------+
//! ```
//!
//! The implementation detects transparent blocks by:
//! 1. Decoding all 16 pixels in the block
//! 2. Checking if all pixels have alpha=0 (check if first pixel is transparent, after checking if all are equal)
//! 3. Setting the entire block content to 0xFF bytes
//!
//! ### 3. Mixed Transparency Blocks
//!
//! In BC1, when `Color0 <= Color1`, the block is in "punch-through alpha" mode, where index `11`
//! represents a transparent pixel. Blocks containing both opaque and transparent pixels
//! (mixed alpha) use this mode.
//!
//! For these blocks, we can't apply significant normalization without changing the visual
//! appearance, so we preserve them unchanged.
//!
//! ## Implementation Details
//!
//! The normalization process uses the BC1 decoder to analyze the block content, then rebuilds
//! blocks according to the rules above.
//!
//! When normalizing blocks, we:
//!
//! 1. Look at the RGB565 color values to determine if we're in alpha mode (`Color0 <= Color1`)
//! 2. Decode the block to get the 16 pixels with their colors
//! 3. Apply one of the three normalization cases based on the block properties
//! 4. Write the normalized block to the output

use crate::util::decode_bc1_block;
use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform_common::{color_565::Color565, decoded_4x4_block::Decoded4x4Block};
use likely_stable::unlikely;

/// Reads an input of blocks from `input_ptr` and writes the normalized blocks to `output_ptr`.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `color_mode`: How to normalize color values
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - input_ptr and output_ptr must not overlap
///
/// # Remarks
///
/// This function identifies and normalizes BC1 blocks based on their content:
/// - Solid color blocks are normalized to a standard format with the color in color0
/// - Fully transparent blocks are normalized to all 0xFF bytes
/// - Mixed color/alpha blocks are preserved as-is
///
/// Normalization improves compression ratios by ensuring that similar visual blocks
/// have identical binary representations, reducing entropy in the data.
///
/// See the module-level documentation for more details on the normalization process.
#[inline]
pub unsafe fn normalize_blocks(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    color_mode: ColorNormalizationMode,
) {
    debug_assert!(
        input_ptr.add(len) <= output_ptr || output_ptr.add(len) <= input_ptr as *mut u8,
        "Input and output memory regions must not overlap"
    );

    // Skip normalization if mode is None
    if color_mode == ColorNormalizationMode::None {
        copy_nonoverlapping(input_ptr, output_ptr, len);
        return;
    }

    // Setup destination pointer
    let mut dst_block_ptr = output_ptr;

    normalize_blocks_impl(
        input_ptr,
        len,
        |src_block_ptr, _decoded_block, block_case, color565| {
            match block_case {
                BlockCase::Transparent => {
                    // Case 2: Fully transparent block - fill with 0xFF
                    core::ptr::write_bytes(dst_block_ptr, 0xFF, 8);
                }
                BlockCase::SolidColorRoundtrippable => {
                    // Can be normalized - use the helper function to write the block
                    write_normalized_solid_color_block(
                        dst_block_ptr,
                        src_block_ptr,
                        color565,
                        color_mode,
                    );
                }
                BlockCase::Other => {
                    // Cannot normalize, copy source block as-is
                    copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
                }
            }

            // Advance destination pointer
            dst_block_ptr = dst_block_ptr.add(8);
        },
    );
}

/// Generic implementation for normalizing blocks with customizable output handling.
///
/// This internal function encapsulates the common logic for block analysis
/// and delegates the output writing to a closure.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `handle_output`: A closure that handles writing the output. The closure receives:
///   - The source block pointer
///   - A reference to the decoded block
///   - A block processing case (transparent, solid color w/ roundtrip, or other)
///   - The color in RGB565 format (valid only for solid color blocks)
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - len must be divisible by 8
/// - The closure must handle memory safety for all output operations
#[inline]
unsafe fn normalize_blocks_impl<F>(input_ptr: *const u8, len: usize, mut handle_output: F)
where
    F: FnMut(*const u8, &Decoded4x4Block, BlockCase, Color565),
{
    debug_assert!(len % 8 == 0);

    // Calculate pointers to current block
    let mut src_block_ptr = input_ptr;
    let src_end_ptr = input_ptr.add(len);

    // Process each block
    while src_block_ptr < src_end_ptr {
        // Decode the block to analyze its content
        let decoded_block = decode_bc1_block(src_block_ptr);

        // Check if all pixels in the block are identical
        if decoded_block.has_identical_pixels() {
            // Get the first pixel (they're all the same)
            let pixel = decoded_block.pixels[0];

            // Check if the block is fully transparent
            if unlikely(pixel.a == 0) {
                // Case 2: Fully transparent block
                handle_output(
                    src_block_ptr,
                    &decoded_block,
                    BlockCase::Transparent,
                    Color565::default(),
                );
            } else {
                // Case 1: Solid color block
                // Convert the color to RGB565
                let color565 = pixel.to_color_565();

                // Check if color can be round-tripped cleanly through RGB565
                let color8888 = color565.to_color_8888();

                if unlikely(color8888 == pixel) {
                    // Can be normalized
                    handle_output(
                        src_block_ptr,
                        &decoded_block,
                        BlockCase::SolidColorRoundtrippable,
                        color565,
                    );
                } else {
                    // Case 3: Cannot normalize
                    handle_output(src_block_ptr, &decoded_block, BlockCase::Other, color565);
                }
            }
        } else {
            // Case 3: Mixed colors
            handle_output(
                src_block_ptr,
                &decoded_block,
                BlockCase::Other,
                Color565::default(),
            );
        }

        // Move to the next block
        src_block_ptr = src_block_ptr.add(8);
    }
}

/// Block processing case for the normalization functions
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BlockCase {
    /// Fully transparent block (all pixels have alpha=0)
    Transparent,
    /// Solid color block with clean RGB565 roundtrip
    SolidColorRoundtrippable,
    /// Any other block type (mixed colors, non-roundtrippable solid color)
    Other,
}

/// Helper function to write a solid color block with the specified normalization mode.
///
/// # Parameters
///
/// - `dst_block_ptr`: Pointer to the destination block
/// - `src_block_ptr`: Pointer to the source block (used for None mode)
/// - `color565`: The RGB565 color to write
/// - `color_mode`: The normalization mode to use
///
/// # Safety
///
/// - dst_block_ptr must be valid for writes of 8 bytes
/// - src_block_ptr must be valid for reads of 8 bytes if color_mode is None
#[inline]
unsafe fn write_normalized_solid_color_block(
    dst_block_ptr: *mut u8,
    src_block_ptr: *const u8,
    color565: Color565,
    color_mode: ColorNormalizationMode,
) {
    // Can be normalized - write the standard pattern:
    // Color0 = the color, Color1 = 0, indices = 0
    let color_bytes = color565.raw_value().to_le_bytes();

    // Write Color0 (the solid color)
    *dst_block_ptr = color_bytes[0];
    *dst_block_ptr.add(1) = color_bytes[1];

    // Write Color1 and indices based on the mode
    match color_mode {
        ColorNormalizationMode::None => {
            // For None mode, copy the original source block
            copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
        }
        ColorNormalizationMode::Color0Only => {
            // Write Color1 = 0
            *dst_block_ptr.add(2) = 0;
            *dst_block_ptr.add(3) = 0;

            // Write indices = 0
            *dst_block_ptr.add(4) = 0;
            *dst_block_ptr.add(5) = 0;
            *dst_block_ptr.add(6) = 0;
            *dst_block_ptr.add(7) = 0;
        }
        ColorNormalizationMode::ReplicateColor => {
            // Write Color1 = same as Color0
            *dst_block_ptr.add(2) = color_bytes[0];
            *dst_block_ptr.add(3) = color_bytes[1];

            // Write indices = 0
            *dst_block_ptr.add(4) = 0;
            *dst_block_ptr.add(5) = 0;
            *dst_block_ptr.add(6) = 0;
            *dst_block_ptr.add(7) = 0;
        }
    }
}

/// Reads an input of blocks from `input_ptr` and writes the normalized blocks to multiple output pointers,
/// one for each available [`ColorNormalizationMode`].
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptrs`: An array of output pointers, one for each [`ColorNormalizationMode`]
/// - `len`: The length of the input data in bytes
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - each pointer in output_ptrs must be valid for writes of len bytes
/// - len must be divisible by 8
/// - input_ptr and output_ptrs must not overlap
///
/// # Remarks
///
/// This function processes each block once and writes it to multiple output buffers,
/// applying a different normalization mode to each output. This allows you to compare
/// the results of different normalization strategies for the same input.
///
/// The output_ptrs array must contain exactly one pointer for each variant in [`ColorNormalizationMode`],
/// in the same order as they are defined in the enum.
///
/// See the module-level documentation for more details on the normalization process.
#[inline]
pub unsafe fn normalize_blocks_all_modes(
    input_ptr: *const u8,
    output_ptrs: &mut [*mut u8; ColorNormalizationMode::all_values().len()],
    len: usize,
) {
    debug_assert!(len % 8 == 0);

    normalize_blocks_impl(
        input_ptr,
        len,
        |src_block_ptr, _decoded_block, block_case, color565| {
            match block_case {
                BlockCase::Transparent => {
                    // Case 2: Fully transparent block - fill with 0xFF in all output buffers
                    for dst_ptr in output_ptrs.iter() {
                        core::ptr::write_bytes(*dst_ptr, 0xFF, 8);
                    }
                }
                BlockCase::SolidColorRoundtrippable => {
                    // Can be normalized - write the standard pattern for each mode
                    for (x, dst_ptr) in output_ptrs.iter().enumerate() {
                        let mode = ColorNormalizationMode::all_values()[x];
                        write_normalized_solid_color_block(*dst_ptr, src_block_ptr, color565, mode);
                    }
                }
                BlockCase::Other => {
                    // Cannot normalize, copy source block as-is to all output buffers
                    for dst_ptr in output_ptrs.iter() {
                        copy_nonoverlapping(src_block_ptr, *dst_ptr, 8);
                    }
                }
            }

            // Advance all destination pointers
            for dst_ptr in output_ptrs.iter_mut() {
                *dst_ptr = dst_ptr.add(8);
            }
        },
    );
}

/// Defines how colors should be normalized for BC1 blocks
///
/// BC1 blocks can represent solid colors in multiple ways. This enum
/// defines the strategies for normalizing these representations to improve compression.
#[derive(Debug, Copy, Clone, PartialEq, Eq, derive_enum_all_values::AllValues)]
pub enum ColorNormalizationMode {
    /// No color normalization, preserves original color data
    None,

    /// For solid color blocks, put color in C0, zeroes in C1 and indices
    /// Creates a pattern of `color,0,0,0,0,0,0,0` for the color component
    /// This results in a nice repetition of `0x00` across 6 bytes
    Color0Only,

    /// For solid color blocks, replicate color in both C0 and C1, zeroes in indices
    /// Creates a pattern of `color,color,0,0,0,0` for the color component
    /// In some cases, this performs better in compression.
    ReplicateColor,
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// Test normalizing a solid color block
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_normalize_solid_color_block(#[case] color_mode: ColorNormalizationMode) {
        // Red in RGB565: (31, 0, 0) -> 0xF800
        // This cleanly round trips into (255, 0, 0) and back to (31, 0, 0).
        let red565 = 0xF800u16.to_le_bytes(); // Little endian: [0x00, 0xF8]

        // This creates a BC1 block with the following characteristics:
        // - Color0 = Red (RGB565)
        // - Color1 = Another color (doesn't matter as long as Color0 > Color1)
        // - Indices = All pointing to Color0 (all 0b00)
        let mut block = [0u8; 8];
        block[0] = red565[0]; // Color0 (low byte)
        block[1] = red565[1]; // Color0 (high byte)
        block[2] = 0x01; // Color1 (low byte) - smaller than Color0 to avoid punch-through alpha mode
        block[3] = 0x01; // Color1 (high byte)

        // All indices = 0, pointing to Color0
        block[4] = 0x00;
        block[5] = 0x00;
        block[6] = 0x00;
        block[7] = 0x00;

        // Expected normalized block:
        // - Color0 = the color (0xF800)
        // - Color1 = 0 or same as Color0 (depending on mode)
        // - All indices = 0
        let mut expected = [0u8; 8];
        expected[0] = red565[0];
        expected[1] = red565[1];

        // Set Color1 based on mode
        if color_mode == ColorNormalizationMode::ReplicateColor {
            expected[2] = red565[0];
            expected[3] = red565[1];
        } else if color_mode == ColorNormalizationMode::Color0Only {
            expected[2] = 0;
            expected[3] = 0;
        }

        // Output buffer for normalized block
        let mut output = [0u8; 8];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 8, color_mode);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Solid color block normalization failed");
    }

    /// Test normalizing a fully transparent block
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_normalize_transparent_block(#[case] color_mode: ColorNormalizationMode) {
        // Create a BC1 block that decodes to all transparent pixels
        // In BC1, when Color0 <= Color1, index 3 refers to transparent

        // So we'll create a block with:
        // - Color0 = some value
        // - Color1 = larger value (making Color0 <= Color1)
        // - All indices = 3 (transparent)
        let mut block = [0u8; 8];
        block[0] = 0x00; // Color0 (low byte)
        block[1] = 0x80; // Color0 (high byte) = 0x8000
        block[2] = 0x00; // Color1 (low byte)
        block[3] = 0xF8; // Color1 (high byte) = 0xF800, greater than Color0

        // Set all indices to 3 (0b11)
        block[4] = 0xFF; // 0b11111111
        block[5] = 0xFF; // 0b11111111
        block[6] = 0xFF; // 0b11111111
        block[7] = 0xFF; // 0b11111111

        // Expected normalized block: all FF
        let expected = [0xFF; 8];

        // Output buffer for normalized block
        let mut output = [0u8; 8];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 8, color_mode);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Transparent block normalization failed");
    }

    /// Test that a mixed color block is preserved as-is
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_preserve_mixed_color_block(#[case] color_mode: ColorNormalizationMode) {
        // Create a mixed color block with red and blue
        // - Color0 = Red
        // - Color1 = Blue
        // - Indices = mix of 0 and 1

        let red565 = 0xF800u16.to_le_bytes(); // Red: [0x00, 0xF8]
        let blue565 = 0x001Fu16.to_le_bytes(); // Blue: [0x1F, 0x00]

        let mut block = [0u8; 8];
        block[0] = red565[0]; // Color0 (low byte)
        block[1] = red565[1]; // Color0 (high byte)
        block[2] = blue565[0]; // Color1 (low byte)
        block[3] = blue565[1]; // Color1 (high byte)

        // Mix of indices pointing to both colors
        block[4] = 0b00010001; // 00010001 (alternating indices)
        block[5] = 0b00010001;
        block[6] = 0b00010001;
        block[7] = 0b00010001;

        // Output buffer for normalized block
        let mut output = [0u8; 8];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 8, color_mode);
        }

        // Check that the output is identical to the source (preserved as-is)
        assert_eq!(output, block, "Mixed color block should be preserved as-is");
    }

    /// Test that a solid color block that can't be cleanly round-tripped is preserved as-is
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_preserve_non_roundtrippable_color_block(#[case] color_mode: ColorNormalizationMode) {
        // Create a mix of 2 colours that can't be cleanly round-tripped,
        // this cannot be simplified down
        let red565 = 0xF800u16.to_le_bytes(); // (31, 0, 0) -> 0xF800
        let blue565 = 0x001Fu16.to_le_bytes(); // (0, 0, 31) -> 0x001F

        // Create the source block that would decode to a solid non-roundtrippable color
        let mut source = [0u8; 8];
        source[0] = red565[0]; // Color0 (low byte)
        source[1] = red565[1]; // Color0 (high byte)
        source[2] = blue565[0]; // Color1 (low byte)
        source[3] = blue565[1]; // Color1 (high byte)
                                // All indices pointing at 2/3 color0, 1/3 color1
        source[4] = 0b10101010; // 0b10101010
        source[5] = 0b10101010;
        source[6] = 0b10101010;
        source[7] = 0b10101010;

        // Output buffer for normalized block
        // Decoded 8888: (170, 0, 85, 255)
        // Round Tripped 8888: (173, 0, 82)
        let mut output = [0u8; 8];

        // Normalize the block
        unsafe {
            normalize_blocks(source.as_ptr(), output.as_mut_ptr(), 8, color_mode);
        }

        // Check that the output is identical to the source (preserved as-is)
        assert_eq!(
            output, source,
            "Non-roundtrippable color block should be preserved as-is"
        );
    }

    /// Test normalizing multiple blocks in one call
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_normalize_multiple_blocks(#[case] color_mode: ColorNormalizationMode) {
        // Create data for two blocks: one solid color and one transparent
        let red565 = 0xF800u16.to_le_bytes();

        // Source data for both blocks
        let mut source = [0u8; 16]; // Two blocks (8 bytes each)

        // First block: solid red
        source[0] = red565[0]; // Color0 (low byte)
        source[1] = red565[1]; // Color0 (high byte)
        source[2] = 0x00; // Color1 (low byte)
        source[3] = 0x00; // Color1 (high byte)
                          // All indices pointing to Color0
        source[4] = 0x00;
        source[5] = 0x00;
        source[6] = 0x00;
        source[7] = 0x00;

        // Second block: transparent
        source[8] = 0x00; // Color0 (low byte)
        source[9] = 0x80; // Color0 (high byte) = 0x8000
        source[10] = 0x00; // Color1 (low byte)
        source[11] = 0xF8; // Color1 (high byte) = 0xF800, greater than Color0
                           // All indices set to 3 (0b11)
        source[12] = 0xFF;
        source[13] = 0xFF;
        source[14] = 0xFF;
        source[15] = 0xFF;

        // Expected output
        let mut expected = [0u8; 16];

        // First block: normalized solid color
        expected[0] = red565[0];
        expected[1] = red565[1];

        // Set Color1 based on mode
        if color_mode == ColorNormalizationMode::ReplicateColor {
            expected[2] = red565[0];
            expected[3] = red565[1];
        } else if color_mode == ColorNormalizationMode::Color0Only {
            expected[2] = 0;
            expected[3] = 0;
        }

        // Rest of first block is zeros
        expected[4] = 0;
        expected[5] = 0;
        expected[6] = 0;
        expected[7] = 0;

        // Second block: normalized transparent (all FF)
        #[allow(clippy::needless_range_loop)]
        for x in 8..16 {
            expected[x] = 0xFF;
        }

        // Output buffer
        let mut output = [0u8; 16];

        // Normalize both blocks
        unsafe {
            normalize_blocks(source.as_ptr(), output.as_mut_ptr(), 16, color_mode);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Multiple block normalization failed");
    }

    /// Test normalizing blocks with all modes simultaneously
    #[test]
    #[allow(clippy::needless_range_loop)]
    fn can_normalize_blocks_all_modes() {
        // Create test data for solid color and transparent blocks
        let red565 = 0xF800u16.to_le_bytes();

        // Source data for two blocks: solid red and transparent
        let mut source = [0u8; 16]; // Two blocks (8 bytes each)

        // First block: solid red
        source[0] = red565[0]; // Color0 (low byte)
        source[1] = red565[1]; // Color0 (high byte)
        source[2] = 0x00; // Color1 (low byte)
        source[3] = 0x00; // Color1 (high byte)
                          // All indices pointing to Color0
        source[4] = 0x00;
        source[5] = 0x00;
        source[6] = 0x00;
        source[7] = 0x00;

        // Second block: transparent
        source[8] = 0x00; // Color0 (low byte)
        source[9] = 0x80; // Color0 (high byte) = 0x8000
        source[10] = 0x00; // Color1 (low byte)
        source[11] = 0xF8; // Color1 (high byte) = 0xF800, greater than Color0
                           // All indices set to 3 (0b11)
        source[12] = 0xFF;
        source[13] = 0xFF;
        source[14] = 0xFF;
        source[15] = 0xFF;

        // Create output buffers for each normalization mode
        let num_modes = ColorNormalizationMode::all_values().len();
        let mut output_buffers = vec![[0u8; 16]; num_modes];

        // Create an array of pointers to output buffers
        let mut output_ptrs_array = output_buffers
            .iter_mut()
            .map(|buffer| buffer.as_mut_ptr())
            .collect::<Vec<*mut u8>>()
            .try_into()
            .unwrap_or_else(|_| panic!("Wrong number of output pointers"));

        // Normalize the blocks with all modes simultaneously
        unsafe {
            normalize_blocks_all_modes(source.as_ptr(), &mut output_ptrs_array, 16);
        }

        // Verify each output buffer has the correct normalization applied
        for (x, mode) in ColorNormalizationMode::all_values().iter().enumerate() {
            let output = &output_buffers[x];

            // First block: solid color normalization
            match mode {
                ColorNormalizationMode::None => {
                    // For None mode, the source block should be preserved as-is
                    assert_eq!(output[0], source[0]);
                    assert_eq!(output[1], source[1]);
                    assert_eq!(output[2], source[2]);
                    assert_eq!(output[3], source[3]);
                    assert_eq!(output[4], source[4]);
                    assert_eq!(output[5], source[5]);
                    assert_eq!(output[6], source[6]);
                    assert_eq!(output[7], source[7]);
                }
                ColorNormalizationMode::Color0Only => {
                    // For Color0Only mode, the first block should have Color0 set correctly,
                    // Color1 set to 0, and all indices set to 0
                    assert_eq!(output[0], red565[0]);
                    assert_eq!(output[1], red565[1]);
                    assert_eq!(output[2], 0);
                    assert_eq!(output[3], 0);
                    assert_eq!(output[4], 0);
                    assert_eq!(output[5], 0);
                    assert_eq!(output[6], 0);
                    assert_eq!(output[7], 0);
                }
                ColorNormalizationMode::ReplicateColor => {
                    // For ReplicateColor mode, the first block should have Color0 and Color1
                    // set to the same color, and all indices set to 0
                    assert_eq!(output[0], red565[0]);
                    assert_eq!(output[1], red565[1]);
                    assert_eq!(output[2], red565[0]);
                    assert_eq!(output[3], red565[1]);
                    assert_eq!(output[4], 0);
                    assert_eq!(output[5], 0);
                    assert_eq!(output[6], 0);
                    assert_eq!(output[7], 0);
                }
            }

            // Second block: transparent normalization (should be all 0xFF for all modes)
            for x in 8..16 {
                assert_eq!(
                    output[x], 0xFF,
                    "Transparent block normalization failed for mode {mode:?}"
                );
            }
        }
    }
}
