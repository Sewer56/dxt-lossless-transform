use crate::util::decode_bc2_block;
use core::ptr::{copy_nonoverlapping, eq, null_mut, read_unaligned};
use derive_enum_all_values::AllValues;
use dxt_lossless_transform_common::{color_565::Color565, color_8888::Color8888};
use likely_stable::unlikely;

/// Reads an input of blocks from `input_ptr` and writes the normalized blocks to `output_ptr`.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC2 blocks)
/// - `output_ptr`: A pointer to the output data (output BC2 blocks)
/// - `len`: The length of the input data in bytes
/// - `color_mode`: How to normalize color values
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (BC2 block size)
/// - The implementation supports `input_ptr` == `output_ptr` (in-place transformation)
/// - The implementation does NOT support partially overlapping buffers (they must either be completely separate or identical)
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
    color_mode: ColorNormalizationMode,
) {
    debug_assert!(len.is_multiple_of(16));
    // Assert that buffers either don't overlap or are identical (in-place transformation)
    debug_assert!(
        eq(input_ptr, output_ptr as *const u8) ||
        input_ptr.add(len) <= output_ptr ||
        output_ptr.add(len) <= input_ptr as *mut u8,
        "normalize_blocks: overlapping buffers are not supported (must be either completely separate or identical)"
    );

    // Skip normalization if mode is None
    if color_mode == ColorNormalizationMode::None {
        // No need to copy if buffers are identical
        if eq(input_ptr, output_ptr as *const u8) {
            return;
        }

        // This can hit the case where pointers overlap at runtime.
        // That is caught by the copy call.
        copy_nonoverlapping(input_ptr, output_ptr, len);
        return;
    }

    // Setup mutable destination pointer
    let mut dst_block_ptr = output_ptr;
    normalize_blocks_impl(input_ptr, len, |src_block_ptr, block_case, color565| {
        match block_case {
            BlockCase::SolidColorRoundtrippable => {
                // Copy alpha values (first 8 bytes) unchanged
                (dst_block_ptr as *mut u64)
                    .write_unaligned(read_unaligned(src_block_ptr as *const u64));

                // Write normalized color data (bytes 8-15)
                write_normalized_solid_color_block(
                    dst_block_ptr,
                    src_block_ptr,
                    color565,
                    color_mode,
                );
            }
            BlockCase::CannotNormalize => {
                // Cannot normalize, copy source block as-is
                (dst_block_ptr as *mut u64)
                    .write_unaligned(read_unaligned(src_block_ptr as *const u64));
                (dst_block_ptr.add(8) as *mut u64)
                    .write_unaligned(read_unaligned(src_block_ptr.add(8) as *const u64));
            }
        }

        // Advance destination pointer
        dst_block_ptr = dst_block_ptr.add(16);
    });
}

/// Generic implementation for normalizing blocks with customizable output handling.
///
/// This internal function encapsulates the common logic for block analysis
/// and delegates the output writing to a closure.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC2 blocks)
/// - `len`: The length of the input data in bytes
/// - `handle_output`: A closure that handles writing the output. The closure receives:
///   - The source block pointer
///   - A block processing case (solid color w/ roundtrip or cannot normalize)
///   - The color in RGB565 format (valid only for solid color blocks)
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - len must be divisible by 16 (BC2 block size)
/// - The closure must handle memory safety for all output operations
#[inline]
unsafe fn normalize_blocks_impl<F>(input_ptr: *const u8, len: usize, mut handle_output: F)
where
    F: FnMut(*const u8, BlockCase, Color565),
{
    debug_assert!(len.is_multiple_of(16));

    // Calculate pointers to current block
    let mut src_block_ptr = input_ptr;
    let src_end_ptr = input_ptr.add(len);

    // Process each block
    while src_block_ptr < src_end_ptr {
        // Decode the block to analyze its content
        let decoded_block = decode_bc2_block(src_block_ptr);

        // Check if all pixels in the block have identical RGB values (ignoring alpha)
        if decoded_block.has_identical_pixels_ignore_alpha() {
            // Get the first pixel (they all have the same color)
            let pixel = decoded_block.pixels[0];

            // Convert the color to RGB565
            let color565 = pixel.to_565_lossy();

            // Check if color can be round-tripped cleanly through RGB565
            let color8888 = color565.to_8888_lossy();
            let pixel_ignore_alpha = Color8888::new(pixel.r, pixel.g, pixel.b, 255);
            let color8888_ignore_alpha = Color8888::new(color8888.r, color8888.g, color8888.b, 255);

            // Note: As the colour and alpha components are stored separately, we ignore the alpha
            //       when checking if the color can be round-tripped.
            if unlikely(color8888_ignore_alpha == pixel_ignore_alpha) {
                // Can be normalized
                handle_output(src_block_ptr, BlockCase::SolidColorRoundtrippable, color565);
            } else {
                // Cannot normalize
                handle_output(src_block_ptr, BlockCase::CannotNormalize, color565);
            }
        } else {
            // Mixed colors - can't normalize
            handle_output(
                src_block_ptr,
                BlockCase::CannotNormalize,
                Color565::default(),
            );
        }

        // Move to the next block
        src_block_ptr = src_block_ptr.add(16);
    }
}

/// Reads an input of blocks from `input_ptr` and writes the normalized blocks to multiple output pointers,
/// one for each available [`ColorNormalizationMode`].
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC2 blocks)
/// - `output_ptrs`: An array of output pointers, one for each [`ColorNormalizationMode`]
/// - `len`: The length of the input data in bytes
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - each pointer in output_ptrs must be valid for writes of len bytes
/// - len must be divisible by 16 (BC2 block size)
/// - The implementation supports in-place transformation (input_ptr == output_ptr)
/// - The implementation does NOT support partially overlapping buffers (they must either be completely separate or identical)
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
    output_ptrs: &[*mut u8; ColorNormalizationMode::all_values().len()],
    len: usize,
) {
    debug_assert!(len.is_multiple_of(16));
    debug_assert!(output_ptrs.len() == ColorNormalizationMode::all_values().len());
    debug_assert!(
        output_ptrs.iter().all(|&out_ptr| {
            // Allow case where input_ptr == out_ptr (in-place transform)
            eq(input_ptr, out_ptr as *const u8) ||
            // Otherwise no partial overlap
            input_ptr.add(len) <= out_ptr ||
            out_ptr.add(len) <= input_ptr as *mut _
        }),
        "normalize_blocks_all_modes: overlapping buffers are not supported (must be either completely separate or identical)"
    );

    let mut dst_block_ptrs = [null_mut::<u8>(); ColorNormalizationMode::all_values().len()];

    // Initialize destination pointers
    for (c_idx, dst_ptr) in dst_block_ptrs.iter_mut().enumerate() {
        *dst_ptr = output_ptrs[c_idx];
    }

    // Process all blocks once, writing to multiple output buffers
    normalize_blocks_impl(input_ptr, len, |src_block_ptr, block_case, color565| {
        match block_case {
            BlockCase::SolidColorRoundtrippable => {
                // Process each mode
                for (x, mode) in ColorNormalizationMode::all_values().iter().enumerate() {
                    let dst_block_ptr = dst_block_ptrs[x];

                    // Copy alpha values (first 8 bytes) unchanged
                    (dst_block_ptr as *mut u64)
                        .write_unaligned(read_unaligned(src_block_ptr as *const u64));

                    // Write normalized color data (bytes 8-15)
                    write_normalized_solid_color_block(
                        dst_block_ptr,
                        src_block_ptr,
                        color565,
                        *mode,
                    );

                    // Advance this mode's destination pointer
                    dst_block_ptrs[x] = dst_block_ptr.add(16);
                }
            }
            BlockCase::CannotNormalize => {
                // For blocks that can't be normalized, just copy the original for all modes
                for (x, _) in ColorNormalizationMode::all_values().iter().enumerate() {
                    let dst_block_ptr = dst_block_ptrs[x];

                    // Cannot normalize, copy source block as-is for all modes
                    (dst_block_ptr as *mut u64)
                        .write_unaligned(read_unaligned(src_block_ptr as *const u64));
                    (dst_block_ptr.add(8) as *mut u64)
                        .write_unaligned(read_unaligned(src_block_ptr.add(8) as *const u64));

                    // Advance this mode's destination pointer
                    dst_block_ptrs[x] = dst_block_ptr.add(16);
                }
            }
        }
    });
}

/// Block processing case for the normalization functions
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum BlockCase {
    /// Solid color block that can be round-tripped cleanly through RGB565
    SolidColorRoundtrippable,
    /// Block that cannot be normalized (mixed colors or non-roundtrippable color)
    CannotNormalize,
}

/// Helper function to write normalized color data for solid color blocks with the
/// specified normalization mode.
///
/// # Parameters
///
/// - `dst_block_ptr`: Pointer to the **start** of the destination BC2 block
/// - `src_block_ptr`: Pointer to the **start** of the source BC2 block (used for None mode)
/// - `color565`: The RGB565 color to write
/// - `color_mode`: The normalization mode to use
///
/// # Safety
///
/// - dst_block_ptr must be valid for writes from offset 8 to 15 (color data in BC2 block)
/// - src_block_ptr must be valid for reads of 16 bytes if color_mode is None
#[inline]
unsafe fn write_normalized_solid_color_block(
    dst_block_ptr: *mut u8,
    src_block_ptr: *const u8,
    color565: Color565,
    color_mode: ColorNormalizationMode,
) {
    // Can be normalized - write the standard pattern for the color part:
    // Color0 = the color, Color1 = 0 or repeat, indices = 0
    let color_bytes = color565.raw_value().to_le_bytes();

    // Write Color1 = 0 or repeat
    match color_mode {
        ColorNormalizationMode::None => {
            (dst_block_ptr.add(8) as *mut u64)
                .write_unaligned(read_unaligned(src_block_ptr.add(8) as *const u64));
        }
        ColorNormalizationMode::Color0Only => {
            // Write Color0 (the solid color)
            *dst_block_ptr.add(8) = color_bytes[0];
            *dst_block_ptr.add(9) = color_bytes[1];

            // Write Color1 = 0
            *dst_block_ptr.add(10) = 0;
            *dst_block_ptr.add(11) = 0;

            // Write indices = 0
            *dst_block_ptr.add(12) = 0;
            *dst_block_ptr.add(13) = 0;
            *dst_block_ptr.add(14) = 0;
            *dst_block_ptr.add(15) = 0;
        }
        ColorNormalizationMode::ReplicateColor => {
            // Write Color0 (the solid color)
            *dst_block_ptr.add(8) = color_bytes[0];
            *dst_block_ptr.add(9) = color_bytes[1];

            // Write Color1 = same as Color0
            *dst_block_ptr.add(10) = color_bytes[0];
            *dst_block_ptr.add(11) = color_bytes[1];

            // Write indices = 0
            *dst_block_ptr.add(12) = 0;
            *dst_block_ptr.add(13) = 0;
            *dst_block_ptr.add(14) = 0;
            *dst_block_ptr.add(15) = 0;
        }
    }
}

/// Defines how colors should be normalized for BC2 blocks
///
/// BC2 blocks can represent solid colors in multiple ways. This enum
/// defines the strategies for normalizing these representations to improve compression.
#[derive(Debug, Copy, Clone, PartialEq, Eq, AllValues)]
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

/// Normalizes BC2 blocks that are already split into separate alpha, color and indices sections.
///
/// # Parameters
///
/// - `alpha_ptr`: A pointer to the section containing the alpha values (8 bytes per block)
/// - `colors_ptr`: A pointer to the section containing the colors (4 bytes per block)
/// - `indices_ptr`: A pointer to the section containing the indices (4 bytes per block)
/// - `num_blocks`: The number of blocks to process (1 block = 16 bytes)
/// - `color_mode`: How to normalize color values
///
/// # Safety
///
/// - alpha_ptr must be valid for reads and writes of num_blocks * 8 bytes
/// - colors_ptr must be valid for reads and writes of num_blocks * 4 bytes
/// - indices_ptr must be valid for reads and writes of num_blocks * 4 bytes
/// - This function works in-place, modifying the color and indices buffers directly
/// - The alpha buffer is preserved unchanged
///
/// # Remarks
///
/// This function normalizes blocks that have already been split, with alpha, colors and indices
/// in separate memory locations. It applies the same normalization rules as [`normalize_blocks`]
/// - Solid color blocks are normalized to a standard format in the color section
/// - Alpha values are preserved as they are
/// - Mixed color blocks are preserved as-is
///
/// See the module-level documentation for more details on the normalization process.
#[inline]
pub unsafe fn normalize_split_blocks_in_place(
    alpha_ptr: *const u8,
    colors_ptr: *mut u8,
    indices_ptr: *mut u8,
    num_blocks: usize,
    color_mode: ColorNormalizationMode,
) {
    // Skip normalization if mode is None
    if color_mode == ColorNormalizationMode::None {
        return;
    }

    // TODO: This can be optimized by using the BC1 decoder, with disabled
    //       alpha mode.
    // Process each block
    for block_idx in 0..num_blocks {
        // Calculate current block pointers
        let curr_alpha_ptr = alpha_ptr.add(block_idx * 8);
        let curr_colors_ptr = colors_ptr.add(block_idx * 4);
        let curr_indices_ptr = indices_ptr.add(block_idx * 4);

        // Reconstruct a temporary block for analysis
        let mut temp_block = [0u8; 16];
        copy_nonoverlapping(curr_alpha_ptr, temp_block.as_mut_ptr(), 8);
        copy_nonoverlapping(curr_colors_ptr, temp_block.as_mut_ptr().add(8), 4);
        copy_nonoverlapping(curr_indices_ptr, temp_block.as_mut_ptr().add(12), 4);

        // Decode the block to analyze its content
        let decoded_block = decode_bc2_block(temp_block.as_ptr());

        // Check if all pixels in the block have identical RGB values (ignoring alpha)
        if decoded_block.has_identical_pixels_ignore_alpha() {
            // Get the first pixel (they all have the same color)
            let pixel = decoded_block.pixels[0];

            // Convert the color to RGB565
            let color565 = pixel.to_565_lossy();

            // Check if color can be round-tripped cleanly through RGB565
            let color8888 = color565.to_8888_lossy();
            let pixel_ignore_alpha = Color8888::new(pixel.r, pixel.g, pixel.b, 255);
            let color8888_ignore_alpha = Color8888::new(color8888.r, color8888.g, color8888.b, 255);

            if unlikely(color8888_ignore_alpha == pixel_ignore_alpha) {
                // Since this is an 'in-place' operation, we don't need
                // to overwrite the alpha, we can just skip it.

                // Can be normalized, write the standard pattern
                let color_bytes = color565.raw_value().to_le_bytes();

                // Write Color0 and Color1 based on the mode
                match color_mode {
                    ColorNormalizationMode::None => {
                        // For None mode, the operation is a no-op.
                        // Since this is a transform in place, we do nothing.
                    }
                    ColorNormalizationMode::Color0Only => {
                        // Write Color0 (the solid color)
                        *curr_colors_ptr = color_bytes[0];
                        *curr_colors_ptr.add(1) = color_bytes[1];

                        // Write Color1 = 0
                        *curr_colors_ptr.add(2) = 0;
                        *curr_colors_ptr.add(3) = 0;

                        // Write indices = 0
                        *curr_indices_ptr = 0;
                        *curr_indices_ptr.add(1) = 0;
                        *curr_indices_ptr.add(2) = 0;
                        *curr_indices_ptr.add(3) = 0;
                    }
                    ColorNormalizationMode::ReplicateColor => {
                        // Write Color0 (the solid color)
                        *curr_colors_ptr = color_bytes[0];
                        *curr_colors_ptr.add(1) = color_bytes[1];

                        // Write Color1 = same as Color0
                        *curr_colors_ptr.add(2) = color_bytes[0];
                        *curr_colors_ptr.add(3) = color_bytes[1];

                        // Write indices = 0
                        *curr_indices_ptr = 0;
                        *curr_indices_ptr.add(1) = 0;
                        *curr_indices_ptr.add(2) = 0;
                        *curr_indices_ptr.add(3) = 0;
                    }
                }
            }
            // else: Case: Cannot normalize
            // This is a no-op, since this is an 'in-place' operation.
        }
        // else: Case: Mixed colors
        // Cannot normalize, so this is a no-op as this is an 'in-place' operation.
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    /// Test normalizing a solid color block with uniform alpha
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_normalize_solid_color_block(#[case] color_mode: ColorNormalizationMode) {
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

        // Check that the normalization worked correctly
        if color_mode == ColorNormalizationMode::ReplicateColor {
            expected[10] = red565[0];
            expected[11] = red565[1];
        } else if color_mode == ColorNormalizationMode::Color0Only {
            expected[10] = 0;
            expected[11] = 0;
        }
        // All other bytes remain 0

        // Output buffer for normalized block
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 16, color_mode);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Solid color block normalization failed");
    }

    /// Test that a mixed color block is preserved as-is
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_preserve_mixed_color_block(#[case] color_mode: ColorNormalizationMode) {
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
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 16, color_mode);
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
            normalize_blocks(source.as_ptr(), output.as_mut_ptr(), 16, color_mode);
        }

        // Check that the output is identical to the source (preserved as-is)
        assert_eq!(
            output, source,
            "Non-roundtrippable color block should be preserved as-is"
        );
    }

    /// Test that varying alpha values are preserved as-is
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_preserve_varying_alpha_block(#[case] color_mode: ColorNormalizationMode) {
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
        if color_mode == ColorNormalizationMode::ReplicateColor {
            expected[10] = red565[0];
            expected[11] = red565[1];
        } else if color_mode == ColorNormalizationMode::Color0Only {
            expected[10] = 0;
            expected[11] = 0;
        }

        // Output buffer for normalized block
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(block.as_ptr(), output.as_mut_ptr(), 16, color_mode);
        }

        // Check that the output matches what we expect
        assert_eq!(output, expected, "Varying alpha block normalization failed");
    }

    /// Test normalizing multiple blocks in one call
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_normalize_multiple_blocks(#[case] color_mode: ColorNormalizationMode) {
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

        // All indices = 0, pointing to Color0
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

        // First block: normalized (alpha preserved, color0 red, color1 0 or replicated, indices 0)
        for x in 0..8 {
            expected[x] = 0xFF; // Copy alpha values
        }
        expected[8] = red565[0];
        expected[9] = red565[1];

        // Set Color1 based on mode
        if color_mode == ColorNormalizationMode::ReplicateColor {
            expected[10] = red565[0];
            expected[11] = red565[1];
        } else if color_mode == ColorNormalizationMode::Color0Only {
            expected[10] = 0;
            expected[11] = 0;
        }

        // Rest of first block's indices are zeros
        expected[12] = 0;
        expected[13] = 0;
        expected[14] = 0;
        expected[15] = 0;

        // Second block: preserved as-is
        expected[16..32].copy_from_slice(&source[16..32]);

        // Output buffer
        let mut output = [0u8; 32];

        // Normalize both blocks
        unsafe {
            normalize_blocks(source.as_ptr(), output.as_mut_ptr(), 32, color_mode);
        }

        // Check that the output matches expected
        assert_eq!(output, expected, "Multiple block normalization failed");
    }

    /// Test normalizing blocks with all normalization modes at once
    #[test]
    fn can_normalize_blocks_all_modes() {
        // Red in RGB565: (31, 0, 0) -> 0xF800
        // This cleanly round trips into (255, 0, 0) and back to (31, 0, 0).
        let red565 = 0xF800u16.to_le_bytes(); // Little endian: [0x00, 0xF8]

        // Create a BC2 block with solid red color and uniform alpha
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

        // Create multiple blocks to test processing more than one block at once
        let mut input = [0u8; 32]; // 2 blocks
        input[0..16].copy_from_slice(&block);
        input[16..32].copy_from_slice(&block);

        // Create output buffers, one for each normalization mode
        let num_modes = ColorNormalizationMode::all_values().len();
        let mut outputs = vec![[0u8; 32]; num_modes];

        // Create array of pointers to output buffers
        let output_ptrs = core::array::from_fn(|x| outputs[x].as_mut_ptr());

        // Normalize the blocks using all modes at once
        unsafe {
            normalize_blocks_all_modes(input.as_ptr(), &output_ptrs, 32);
        }

        // Verify the results for each mode
        for (mode_idx, mode) in ColorNormalizationMode::all_values().iter().enumerate() {
            // Create expected output for this mode by using regular normalize_blocks with the same mode.
            let mut expected = [0u8; 32];

            unsafe {
                normalize_blocks(
                    input[0..32].as_ptr(),
                    expected[0..32].as_mut_ptr(),
                    32,
                    *mode,
                );
            }

            // Check that the output from normalize_blocks_all_modes matches the expected output
            assert_eq!(
                outputs[mode_idx], expected,
                "normalize_blocks_all_modes failed for mode {mode:?}",
            );
        }
    }

    /// Test that in-place transformation works correctly
    #[rstest]
    #[case::color0_only(ColorNormalizationMode::Color0Only)]
    #[case::replicate_color(ColorNormalizationMode::ReplicateColor)]
    #[case::none(ColorNormalizationMode::None)]
    fn can_perform_inplace_transformation(#[case] color_mode: ColorNormalizationMode) {
        // Red in RGB565: (31, 0, 0) -> 0xF800
        // This cleanly round trips into (255, 0, 0) and back to (31, 0, 0).
        let red565 = 0xF800u16.to_le_bytes(); // Little endian: [0x00, 0xF8]

        // Create a BC2 block with solid red color and uniform alpha
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

        // Create a copy for the expected result
        let mut expected = [0u8; 16];

        // Normalize to a separate buffer to get the expected result
        unsafe {
            normalize_blocks(
                block.as_ptr(),
                expected.as_mut_ptr(),
                block.len(),
                color_mode,
            );
        }

        // Now perform in-place transformation
        unsafe {
            normalize_blocks(block.as_ptr(), block.as_mut_ptr(), block.len(), color_mode);
        }

        // The in-place transformation should produce the same result as the separate buffer transformation
        assert_eq!(
            block, expected,
            "In-place transformation result does not match expected result"
        );
    }

    #[test]
    fn can_normalize_split_blocks_in_place() {
        // Create test data with three solid color blocks
        let mut test_alpha = [0u8; 24]; // 3 blocks * 8 bytes per block for alpha
        let mut test_colors = [0u8; 12]; // 3 blocks * 4 bytes per block for colors
        let mut test_indices = [0u8; 12]; // 3 blocks * 4 bytes per block for indices

        // Set up alpha values (uniform alpha for all pixels in each block)
        test_alpha.fill(0xFF); // Full opacity for all blocks

        // Set up color endpoints (0xF800 = bright red in RGB565)
        // Block #0
        test_colors[0] = 0x00; // color0 (low byte)
        test_colors[1] = 0xF8; // color0 (high byte)
        test_colors[2] = 0x00; // color1 (low byte)
        test_colors[3] = 0xF8; // color1 (high byte)

        // Block #1
        test_colors[4] = 0x00; // color0 (low byte)
        test_colors[5] = 0xF8; // color0 (high byte)
        test_colors[6] = 0x00; // color1 (low byte)
        test_colors[7] = 0xF8; // color1 (high byte)

        // Block #2 (should remain untouched)
        test_colors[8] = 0x00; // color0 (low byte)
        test_colors[9] = 0xF8; // color0 (high byte)
        test_colors[10] = 0x00; // color1 (low byte)
        test_colors[11] = 0xF8; // color1 (high byte)

        // Set indices to non-zero values
        test_indices.fill(0xAA);

        // Get pointers to the test data
        let alpha_ptr = test_alpha.as_ptr();
        let colors_ptr = test_colors.as_mut_ptr();
        let indices_ptr = test_indices.as_mut_ptr();

        // Call normalize_split_blocks_in_place for the first 2 blocks only
        unsafe {
            normalize_split_blocks_in_place(
                alpha_ptr,
                colors_ptr,
                indices_ptr,
                2,
                ColorNormalizationMode::Color0Only,
            );
        }

        // First block should be normalized (Color0 = red, Color1 = 0, indices = 0)
        assert_eq!(test_colors[0], 0x00);
        assert_eq!(test_colors[1], 0xF8);
        assert_eq!(test_colors[2], 0x00);
        assert_eq!(test_colors[3], 0x00);

        // Second block should also be normalized
        assert_eq!(test_colors[4], 0x00);
        assert_eq!(test_colors[5], 0xF8);
        assert_eq!(test_colors[6], 0x00);
        assert_eq!(test_colors[7], 0x00);

        // Third block should remain untouched
        assert_eq!(test_colors[8], 0x00);
        assert_eq!(test_colors[9], 0xF8);
        assert_eq!(test_colors[10], 0x00);
        assert_eq!(test_colors[11], 0xF8);

        // First block indices should be zeros
        assert_eq!(test_indices[0], 0x00);
        assert_eq!(test_indices[1], 0x00);
        assert_eq!(test_indices[2], 0x00);
        assert_eq!(test_indices[3], 0x00);

        // Second block indices should also be zeros
        assert_eq!(test_indices[4], 0x00);
        assert_eq!(test_indices[5], 0x00);
        assert_eq!(test_indices[6], 0x00);
        assert_eq!(test_indices[7], 0x00);

        // Third block indices should remain untouched
        assert_eq!(test_indices[8], 0xAA);
        assert_eq!(test_indices[9], 0xAA);
        assert_eq!(test_indices[10], 0xAA);
        assert_eq!(test_indices[11], 0xAA);

        // Alpha values should remain unchanged
        for x in 0..24 {
            assert_eq!(test_alpha[x], 0xFF);
        }
    }

    #[test]
    fn can_normalize_split_blocks_in_place_with_replicate_color() {
        // Create test data with three solid color blocks
        let mut test_alpha = [0u8; 24]; // 3 blocks * 8 bytes per block for alpha
        let mut test_colors = [0u8; 12]; // 3 blocks * 4 bytes per block for colors
        let mut test_indices = [0u8; 12]; // 3 blocks * 4 bytes per block for indices

        // Set up alpha values (uniform alpha for all pixels in each block)
        test_alpha.fill(0xFF); // Full opacity for all blocks

        // Set up color endpoints (0xF800 = bright red in RGB565)
        // Block #0
        test_colors[0] = 0x00; // color0 (low byte)
        test_colors[1] = 0xF8; // color0 (high byte)
        test_colors[2] = 0x00; // color1 (low byte)
        test_colors[3] = 0xF8; // color1 (high byte)

        // Block #1
        test_colors[4] = 0x00; // color0 (low byte)
        test_colors[5] = 0xF8; // color0 (high byte)
        test_colors[6] = 0x00; // color1 (low byte)
        test_colors[7] = 0xF8; // color1 (high byte)

        // Block #2 (should remain untouched)
        test_colors[8] = 0x00; // color0 (low byte)
        test_colors[9] = 0xF8; // color0 (high byte)
        test_colors[10] = 0x00; // color1 (low byte)
        test_colors[11] = 0xF8; // color1 (high byte)

        // Set indices to non-zero values
        test_indices.fill(0x55);

        // Get pointers to the test data
        let alpha_ptr = test_alpha.as_ptr();
        let colors_ptr = test_colors.as_mut_ptr();
        let indices_ptr = test_indices.as_mut_ptr();

        // Call normalize_split_blocks_in_place using ReplicateColor mode for the first 2 blocks only
        unsafe {
            normalize_split_blocks_in_place(
                alpha_ptr,
                colors_ptr,
                indices_ptr,
                2,
                ColorNormalizationMode::ReplicateColor,
            );
        }

        // First block should be normalized (Color0 = red, Color1 = red (replicated), indices = 0)
        assert_eq!(test_colors[0], 0x00);
        assert_eq!(test_colors[1], 0xF8);
        assert_eq!(test_colors[2], 0x00); // Color1 should be the same as Color0 for ReplicateColor
        assert_eq!(test_colors[3], 0xF8); // Color1 should be the same as Color0 for ReplicateColor

        // Second block should also be normalized
        assert_eq!(test_colors[4], 0x00);
        assert_eq!(test_colors[5], 0xF8);
        assert_eq!(test_colors[6], 0x00); // Color1 should be the same as Color0 for ReplicateColor
        assert_eq!(test_colors[7], 0xF8); // Color1 should be the same as Color0 for ReplicateColor

        // Third block should remain untouched
        assert_eq!(test_colors[8], 0x00);
        assert_eq!(test_colors[9], 0xF8);
        assert_eq!(test_colors[10], 0x00);
        assert_eq!(test_colors[11], 0xF8);

        // First block indices should be zeros
        assert_eq!(test_indices[0], 0x00);
        assert_eq!(test_indices[1], 0x00);
        assert_eq!(test_indices[2], 0x00);
        assert_eq!(test_indices[3], 0x00);

        // Second block indices should also be zeros
        assert_eq!(test_indices[4], 0x00);
        assert_eq!(test_indices[5], 0x00);
        assert_eq!(test_indices[6], 0x00);
        assert_eq!(test_indices[7], 0x00);

        // Third block indices should remain untouched
        assert_eq!(test_indices[8], 0x55);
        assert_eq!(test_indices[9], 0x55);
        assert_eq!(test_indices[10], 0x55);
        assert_eq!(test_indices[11], 0x55);

        // Alpha values should remain unchanged
        for x in 0..24 {
            assert_eq!(test_alpha[x], 0xFF);
        }
    }
}
