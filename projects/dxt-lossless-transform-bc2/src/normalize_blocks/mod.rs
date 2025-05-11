//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC2 blocks to improve compression ratio
//! by making solid color blocks have consistent representations.
//!
//! ## BC2 Block Format
//!
//! First, let's recall the BC2 block format:
//!
//! ```text
//! Address: 0        8       12      16
//!          +--------+-------+---------+
//! Data:    | A00-A15| C0-C1 | Indices |
//!          +--------+-------+---------+
//! ```
//!
//! Where:
//! - `A00-A15` are 8 bytes containing sixteen 4-bit alpha values (explicit alpha)
//! - `C0-C1` are 16-bit RGB565 color values (2 bytes each)
//! - `Indices` are 4 bytes containing sixteen 2-bit indices (one for each pixel in the 4x4 block)
//!
//! ## Normalization Rules
//!
//! The normalization process applies the following rules to improve compression:
//!
//! ### 1. Solid Color Blocks with Uniform Alpha
//!
//! When an entire block represents a single solid color with a clean conversion between RGBA8888
//! and RGB565, we standardize the representation:
//!
//! ```text
//! +--------+--------+--------+--------+
//! | Alpha  | Color  | 0x0000 |  0x00  |
//! +--------+--------+--------+--------+
//! ```
//!
//! We preserve the alpha values as they are, place the block's color in `Color0`, set `Color1` to zero,
//! and set all indices to zero (represented by all-zero bytes in the indices section).
//! In some cases, it's beneficial to replicate the color across `C0` and `C1` instead.
//!
//! The implementation checks for this case by:
//! 1. Decoding the block to get all 16 pixels
//! 2. Checking that all pixels have the same color (ignoring alpha)
//! 3. Verifying the color can be cleanly round-tripped through RGB565 encoding
//! 4. Constructing a new normalized block with the pattern above
//!
//! ### 2. Other Blocks
//!
//! In BC2, the explicit alpha values in the first 8 bytes already handle transparency, so there's
//! no special handling needed for transparent blocks based on color indices like in BC1.
//!
//! Unlike BC1, BC2 doesn't support the "punch-through alpha" mode (where `Color0 <= Color1`),
//! as this leads to undefined behavior on some GPUs. BC2 always uses the 4-color mode.
//!
//! ## Implementation Details
//!
//! The normalization process uses the BC2 decoder to analyze the block content, then rebuilds
//! blocks according to the rules above.
//!
//! When normalizing blocks, we:
//!
//! 1. Decode the block to get all 16 pixels with their colors and alpha values
//! 2. Check if the block contains a solid color (ignoring alpha variations)
//! 3. If it's a solid color that can be cleanly round-tripped, normalize the color part of the block
//! 4. Leave the alpha values unchanged
//! 5. Write the normalized block to the output

use crate::util::decode_bc2_block;
use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform_common::color_8888::Color8888;
use likely_stable::unlikely;

/// Reads an input of blocks from `input_ptr` and writes the normalized blocks to `output_ptr`.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC2 blocks)
/// - `output_ptr`: A pointer to the output data (output BC2 blocks)
/// - `len`: The length of the input data in bytes
/// - `repeat_colour`: Whether to repeat the solid color in both color slots when normalizing.
///   - When `true`: For solid color blocks, the same color value is written to both color0 and color1
///     Example: A red block might become `[0xF800, 0xF800]`
///   - When `false`: For solid color blocks, color0 contains the color and color1 is set to zero
///     Example: A red block might become `[0xF800, 0x0000]`
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (BC2 block size)
///
/// # Remarks
///
/// This function identifies and normalizes BC2 blocks based on their content:
/// - Solid color blocks are normalized to a standard format with the color in color0
/// - Alpha values are preserved as they are (in the first 8 bytes of each block)
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
    repeat_colour: bool,
) {
    debug_assert!(len % 16 == 0);

    // Calculate pointers to current block
    let mut src_block_ptr = input_ptr;
    let mut dst_block_ptr = output_ptr;
    let src_end_ptr = input_ptr.add(len);

    // Process each block
    while src_block_ptr < src_end_ptr {
        // Decode the block to analyze its content
        let decoded_block = decode_bc2_block(src_block_ptr);

        // Check if all pixels in the block have identical RGB values
        if decoded_block.has_identical_pixels_ignore_alpha() {
            // Get the first pixel (they all have the same color)
            let pixel = decoded_block.pixels[0];

            // Convert the color to RGB565
            let color565 = pixel.to_color_565();

            // Check if color can be round-tripped cleanly through RGB565
            let color8888 = color565.to_color_8888();
            let pixel_ignore_alpha = Color8888::new(pixel.r, pixel.g, pixel.b, 255);
            let color8888_ignore_alpha = Color8888::new(color8888.r, color8888.g, color8888.b, 255);

            // Note: As the colour and alpha components are stored separately, we ignore the alpha
            //       when checking if the color can be round-tripped.
            if unlikely(color8888_ignore_alpha == pixel_ignore_alpha) {
                // Copy alpha values (first 8 bytes) unchanged
                copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);

                // Can be normalized - write the standard pattern for the color part:
                // Color0 = the color, Color1 = 0 or repeat, indices = 0
                let color_bytes = color565.raw_value().to_le_bytes();

                // Write Color0 (the solid color)
                *dst_block_ptr.add(8) = color_bytes[0];
                *dst_block_ptr.add(9) = color_bytes[1];

                // Write Color1 = 0 or repeat
                if repeat_colour {
                    *dst_block_ptr.add(10) = color_bytes[0];
                    *dst_block_ptr.add(11) = color_bytes[1];
                } else {
                    *dst_block_ptr.add(10) = 0;
                    *dst_block_ptr.add(11) = 0;
                }

                // Write indices = 0
                *dst_block_ptr.add(12) = 0;
                *dst_block_ptr.add(13) = 0;
                *dst_block_ptr.add(14) = 0;
                *dst_block_ptr.add(15) = 0;
            } else {
                // Cannot normalize, copy source block as-is
                copy_nonoverlapping(src_block_ptr, dst_block_ptr, 16);
            }
        } else {
            // Mixed colors - copy source block as-is
            copy_nonoverlapping(src_block_ptr, dst_block_ptr, 16);
        }

        // Move to the next block
        src_block_ptr = src_block_ptr.add(16);
        dst_block_ptr = dst_block_ptr.add(16);
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use rstest::rstest;

    use super::*;

    /// Test normalizing a solid color block with uniform alpha
    #[rstest]
    #[case(false)]
    #[case(true)]
    fn can_normalize_solid_color_block(#[case] repeat_colour: bool) {
        // Red in RGB565: (31, 0, 0) -> 0xF800
        // This cleanly round trips into (255, 0, 0) and back to (31, 0, 0).
        let red565 = 0xF800u16.to_le_bytes(); // Little endian: [0x00, 0xF8]

        // This creates a BC2 block with the following characteristics:
        // - Uniform alpha (all 0xFF)
        // - Color0 = Red (RGB565)
        // - Color1 = Another color (doesn't matter)
        // - Indices = All pointing to Color0 (all 0b00)
        let mut block = [0u8; 16];

        // Fill alpha values with 0xFF (fully opaque)
        for x in 0..8 {
            block[x] = 0xFF;
        }

        // Color part
        block[8] = red565[0]; // Color0 (low byte)
        block[9] = red565[1]; // Color0 (high byte)
        block[10] = 0x01; // Color1 (low byte)
        block[11] = 0x01; // Color1 (high byte)

        // All indices = 0, pointing to Color0
        block[12] = 0x00;
        block[13] = 0x00;
        block[14] = 0x00;
        block[15] = 0x00;

        // Expected normalized block:
        // - Alpha values remain the same
        // - Color0 = the color (0xF800)
        // - Color1 = 0 or repeated Color0
        // - All indices = 0
        let mut expected = [0u8; 16];

        // Copy alpha values
        for x in 0..8 {
            expected[x] = 0xFF;
        }

        // Set Color0
        expected[8] = red565[0];
        expected[9] = red565[1];

        // Set Color1 based on repeat_colour
        if repeat_colour {
            expected[10] = red565[0];
            expected[11] = red565[1];
        }
        // All other bytes remain 0

        // Output buffer for normalized block
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 16, repeat_colour);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Solid color block normalization failed");
    }

    /// Test that a mixed color block is preserved as-is
    #[rstest]
    #[case(false)]
    #[case(true)]
    fn can_preserve_mixed_color_block(#[case] repeat_colour: bool) {
        // Create a mixed color block with red and blue
        // - Uniform alpha
        // - Color0 = Red
        // - Color1 = Blue
        // - Indices = mix of 0 and 1

        let red565 = 0xF800u16.to_le_bytes(); // Red: [0x00, 0xF8]
        let blue565 = 0x001Fu16.to_le_bytes(); // Blue: [0x1F, 0x00]

        let mut block = [0u8; 16];

        // Fill alpha values with 0xFF (fully opaque)
        for x in 0..8 {
            block[x] = 0xFF;
        }

        // Color part
        block[8] = red565[0]; // Color0 (low byte)
        block[9] = red565[1]; // Color0 (high byte)
        block[10] = blue565[0]; // Color1 (low byte)
        block[11] = blue565[1]; // Color1 (high byte)

        // Mix of indices pointing to both colors
        block[12] = 0b00010001; // 00010001 (alternating indices)
        block[13] = 0b00010001;
        block[14] = 0b00010001;
        block[15] = 0b00010001;

        // Output buffer for normalized block
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 16, repeat_colour);
        }

        // Check that the output is identical to the source (preserved as-is)
        assert_eq!(output, block, "Mixed color block should be preserved as-is");
    }

    /// Test that a solid color block that can't be cleanly round-tripped is preserved as-is
    #[rstest]
    #[case(false)]
    #[case(true)]
    fn can_preserve_non_roundtrippable_color_block(#[case] repeat_colour: bool) {
        // Create a mix of 2 colours that can't be cleanly round-tripped,
        // this cannot be simplified down
        let red565 = 0xF800u16.to_le_bytes(); // (31, 0, 0) -> 0xF800
        let blue565 = 0x001Fu16.to_le_bytes(); // (0, 0, 31) -> 0x001F

        // Create the source block that would decode to a solid non-roundtrippable color
        let mut source = [0u8; 16];

        // Fill alpha values with 0xFF (fully opaque)
        for x in 0..8 {
            source[x] = 0xFF;
        }

        source[8] = red565[0]; // Color0 (low byte)
        source[9] = red565[1]; // Color0 (high byte)
        source[10] = blue565[0]; // Color1 (low byte)
        source[11] = blue565[1]; // Color1 (high byte)
                                 // All indices pointing at 2/3 color0, 1/3 color1
        source[12] = 0b10101010; // 0b10101010
        source[13] = 0b10101010;
        source[14] = 0b10101010;
        source[15] = 0b10101010;

        // Output buffer for normalized block
        // Decoded 8888: (170, 0, 85, 255)
        // Round Tripped 8888: (173, 0, 82, 255)
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(source.as_ptr(), output.as_mut_ptr(), 16, repeat_colour);
        }

        // Check that the output is identical to the source (preserved as-is)
        assert_eq!(
            output, source,
            "Non-roundtrippable color block should be preserved as-is"
        );
    }

    /// Test that varying alpha values are preserved as-is
    #[rstest]
    #[case(false)]
    #[case(true)]
    fn can_preserve_varying_alpha_block(#[case] repeat_colour: bool) {
        // Create a block with uniform color but varying alpha values
        let red565 = 0xF800u16.to_le_bytes(); // Red: [0x00, 0xF8]

        let mut block = [0u8; 16];

        // Fill alpha values with varying pattern
        for x in 0..8 {
            block[x] = (x * 32) as u8; // Creates a pattern of alpha values
        }

        // Color part - all red
        block[8] = red565[0]; // Color0 (low byte)
        block[9] = red565[1]; // Color0 (high byte)
        block[10] = 0x00; // Color1 (low byte)
        block[11] = 0x00; // Color1 (high byte)

        // All indices pointing to Color0
        block[12] = 0x00;
        block[13] = 0x00;
        block[14] = 0x00;
        block[15] = 0x00;

        // Create a copy for comparison
        let mut expected = [0u8; 16];
        expected.copy_from_slice(&block);

        // If repeating colors, Color1 should be the same as Color0
        if repeat_colour {
            expected[10] = red565[0];
            expected[11] = red565[1];
        }

        // Output buffer for normalized block
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 16, repeat_colour);
        }

        // Check that the output matches what we expect
        assert_eq!(output, expected, "Varying alpha block normalization failed");
    }

    /// Test normalizing multiple blocks in one call
    #[test]
    fn can_normalize_multiple_blocks() {
        // Create data for two blocks: one solid color and one mixed color
        let red565 = 0xF800u16.to_le_bytes(); // Red: [0x00, 0xF8]
        let blue565 = 0x001Fu16.to_le_bytes(); // Blue: [0x1F, 0x00]

        // Source data for both blocks
        let mut source = [0u8; 32]; // Two blocks (16 bytes each)

        // First block: solid red with uniform alpha
        for x in 0..8 {
            source[x] = 0xFF; // Fully opaque alpha
        }
        source[8] = red565[0]; // Color0 (low byte)
        source[9] = red565[1]; // Color0 (high byte)
        source[10] = 0x00; // Color1 (low byte)
        source[11] = 0x00; // Color1 (high byte)
                           // All indices pointing to Color0
        source[12] = 0x00;
        source[13] = 0x00;
        source[14] = 0x00;
        source[15] = 0x00;

        // Second block: mixed colors that won't be normalized
        for x in 16..24 {
            source[x] = 0xFF; // Fully opaque alpha
        }
        source[24] = red565[0]; // Color0 (low byte)
        source[25] = red565[1]; // Color0 (high byte)
        source[26] = blue565[0]; // Color1 (low byte)
        source[27] = blue565[1]; // Color1 (high byte)
                                 // Mix of indices pointing to both colors
        source[28] = 0b00010001; // 00010001 (alternating indices)
        source[29] = 0b00010001;
        source[30] = 0b00010001;
        source[31] = 0b00010001;

        // Expected output
        let mut expected = [0u8; 32];

        // First block: normalized (alpha preserved, color0 red, color1 0, indices 0)
        for x in 0..8 {
            expected[x] = 0xFF; // Copy alpha values
        }
        expected[8] = red565[0];
        expected[9] = red565[1];
        // Rest of first block's color part is zeros

        // Second block: preserved as-is
        expected[16..32].copy_from_slice(&source[16..32]);

        // Output buffer
        let mut output = [0u8; 32];

        // Normalize both blocks
        unsafe {
            normalize_blocks(source.as_ptr(), output.as_mut_ptr(), 32, false);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Multiple block normalization failed");
    }
}
