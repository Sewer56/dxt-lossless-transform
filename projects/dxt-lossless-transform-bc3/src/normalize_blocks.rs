//! # Block Normalization Process
//!
//! This module contains the code used to normalize BC3 blocks to improve compression ratio
//! by making solid color blocks and alpha values have consistent representations.
//!
//! ## BC3 Block Format
//!
//! First, let's recall the BC3 block format:
//!
//! ```text
//! Address: 0       2        8       12      16
//!          +-------+--------+  +-------+--------+
//! Data:    | A0-A1 |AI0-AI15|  | C0-C1 | I0-I15 |
//!          +-------+--------+  +-------+--------+
//! ```
//!
//! Where:
//! - `A0-A1` are two alpha endpoints (8-bit each)
//! - `AI0-AI15` are 6 bytes containing sixteen 3-bit alpha indices
//! - `C0-C1` are 16-bit RGB565 color values (2 bytes each)
//! - `I0-I15` are 4 bytes containing sixteen 2-bit color indices
//!
//! ## Normalization Rules
//!
//! The normalization process applies different rules for alpha and color components:
//!
//! ### Alpha Normalization
//!
//! When an entire block has uniform alpha, several representations are possible:
//!
//! For fully opaque blocks (`0xFF`):
//! - All 8 bytes set to `0xFF` i.e. (`0xFFFFFFFF 0xFFFFFFFF`).
//!   - Because `A0` <= `A1`, index `0xFF` is hardcoded to opaque on the decoder side.
//! - Zero alphas but indices set to `0xFF` i.e. (`0x0000FFFF 0xFFFFFFFF`).
//!
//! For all other values (including `0xFF`):
//! - `A0` set to the alpha value, everything else to `0x00` i.e. (`0xFF000000 0x00000000`).
//!   - Everything uses the alpha value from the first endpoint.
//!
//! ### Color Normalization
//!
//! For solid color blocks with clean RGB565 conversion:
//! - Set color in `C0`, zeroes in `C1` and indices
//!   - This results in a nice repetition of `0x00` across 6 bytes
//! - Or replicate color in both `C0` and `C1`, zeroes in indices
//!   - In some cases, this performs better in compression
//!
//! Note: With BC3, it's important that we put the color in `C0` because the 'alternate alpha mode' of
//! BC1 where `c0 <= c1` is unsupported; it leads to undefined behavior on some GPUs.
//!
//! ## Implementation Details
//!
//! The normalization process:
//!
//! 1. Decodes the block to get all 16 pixels with their colors and alpha values
//! 2. Checks for uniform alpha and solid colors
//! 3. Applies appropriate normalization based on the selected modes
//! 4. Writes the normalized block to the output

use crate::util::decode_bc3_block;
use core::ptr::copy_nonoverlapping;
use derive_enum_all_values::AllValues;
use dxt_lossless_transform_common::color_565::Color565;
use dxt_lossless_transform_common::color_8888::Color8888;
use likely_stable::unlikely;

/// Reads an input of BC3 blocks from `input_ptr` and writes the normalized blocks to `output_ptr`.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks)
/// - `output_ptr`: A pointer to the output data (output BC3 blocks)
/// - `len`: The length of the input data in bytes
/// - `alpha_mode`: How to normalize alpha values
/// - `color_mode`: How to normalize color values
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 16 (BC3 block size)
/// - input_ptr and output_ptr must not overlap
///
/// # Remarks
///
/// This function identifies and normalizes BC3 blocks based on their content:
/// - Blocks with uniform alpha are normalized according to the alpha_mode
/// - Solid color blocks are normalized according to the color_mode
/// - Other blocks are preserved as-is
///
/// Normalization improves compression ratios by ensuring that similar visual blocks
/// have identical binary representations, reducing entropy in the data.
#[inline]
pub unsafe fn normalize_blocks(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    alpha_mode: AlphaNormalizationMode,
    color_mode: ColorNormalizationMode,
) {
    debug_assert!(len % 16 == 0);
    debug_assert!(
        input_ptr.add(len) <= output_ptr || output_ptr.add(len) <= input_ptr as *mut u8,
        "Input and output memory regions must not overlap"
    );

    // Skip normalization if both modes are None
    if alpha_mode == AlphaNormalizationMode::None && color_mode == ColorNormalizationMode::None {
        copy_nonoverlapping(input_ptr, output_ptr, len);
        return;
    }

    // Setup mutable destination pointer
    let mut dst_block_ptr = output_ptr;
    normalize_blocks_impl(
        input_ptr,
        len,
        |src_block_ptr, alpha_case, color_case, color565, alpha_value| {
            // Handle alpha normalization
            match alpha_case {
                AlphaBlockCase::UniformAlpha => {
                    if alpha_mode != AlphaNormalizationMode::None {
                        normalize_alpha(src_block_ptr, dst_block_ptr, alpha_value, alpha_mode);
                    } else {
                        // Copy alpha data as-is (first 8 bytes)
                        copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
                    }
                }
                AlphaBlockCase::CannotNormalize => {
                    // Copy alpha data as-is (first 8 bytes)
                    copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
                }
            }

            // Handle color normalization
            match color_case {
                ColorBlockCase::SolidColorRoundtrippable => {
                    if color_mode != ColorNormalizationMode::None {
                        normalize_color(color565.raw_value(), dst_block_ptr, color_mode);
                    } else {
                        // Copy color data as-is (last 8 bytes)
                        copy_nonoverlapping(src_block_ptr.add(8), dst_block_ptr.add(8), 8);
                    }
                }
                ColorBlockCase::CannotNormalize => {
                    // Copy color data as-is (last 8 bytes)
                    copy_nonoverlapping(src_block_ptr.add(8), dst_block_ptr.add(8), 8);
                }
            }

            // Advance destination pointer
            dst_block_ptr = dst_block_ptr.add(16);
        },
    );
}

/// Defines how alpha values should be normalized for BC3 blocks
///
/// BC3 blocks can represent uniform alpha values in multiple ways. This enum
/// defines the strategies for normalizing these representations to improve compression.
#[derive(Debug, Copy, Clone, PartialEq, Eq, AllValues)]
pub enum AlphaNormalizationMode {
    /// No alpha normalization, preserves original alpha data
    None,

    /// For uniform alpha, set `A0` to the alpha value, `A1` to zero, and indices to zero
    /// This creates a pattern of `alpha,0,0,0,0,0,0,0` for the alpha component
    UniformAlphaZeroIndices,

    /// For fully opaque, use all 0xFF bytes in the alpha component
    /// Creates a pattern of `0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF`.
    ///
    /// If the alpha value is not fully opaque, this mode will use [`AlphaNormalizationMode::UniformAlphaZeroIndices`]
    OpaqueFillAll,

    /// For fully opaque, use zero alphas but 0xFF indices
    /// Creates a pattern of `0,0,0xFF,0xFF,0xFF,0xFF,0xFF,0xFF`
    ///
    /// If the alpha value is not fully opaque, this mode will use [`AlphaNormalizationMode::UniformAlphaZeroIndices`]
    OpaqueZeroAlphaMaxIndices,
}

/// Defines how colors should be normalized for BC3 blocks
///
/// BC3 blocks can represent solid colors in multiple ways. This enum
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

/// Normalizes the alpha component of a BC3 block
///
/// Applies the selected normalization mode to the alpha component of a BC3 block.
/// This is particularly effective for blocks with uniform alpha values.
///
/// # Parameters
///
/// - `src_block_ptr`: Pointer to the source BC3 block
/// - `dst_block_ptr`: Pointer to the destination where the normalized block will be written
/// - `alpha_value`: The uniform alpha value found in the block (0-255)
/// - `mode`: The alpha normalization mode to use
///
/// # Safety
///
/// This is an unsafe function that requires:
/// - Valid pointers to source and destination blocks with at least 8 bytes accessible
/// - Alpha value must be the actual uniform alpha found in the block
#[inline]
unsafe fn normalize_alpha(
    src_block_ptr: *const u8,
    dst_block_ptr: *mut u8,
    alpha_value: u8,
    mode: AlphaNormalizationMode,
) {
    match mode {
        AlphaNormalizationMode::None => {
            // Copy alpha data as-is (first 8 bytes)
            copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
        }
        AlphaNormalizationMode::UniformAlphaZeroIndices => {
            // Set A0 to the alpha value, everything else to 0
            *dst_block_ptr = alpha_value;
            *dst_block_ptr.add(1) = 0;

            // Zero all index bytes
            core::ptr::write_bytes(dst_block_ptr.add(2), 0, 6);
        }
        AlphaNormalizationMode::OpaqueFillAll => {
            if alpha_value == 255 {
                // Fill all alpha bytes with 0xFF
                core::ptr::write_bytes(dst_block_ptr, 0xFF, 8);
            } else {
                // For non-opaque, use the same approach as UniformAlphaZeroIndices
                normalize_alpha(
                    src_block_ptr,
                    dst_block_ptr,
                    alpha_value,
                    AlphaNormalizationMode::UniformAlphaZeroIndices,
                );
            }
        }
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices => {
            if alpha_value == 255 {
                // Set alpha endpoints to 0
                *dst_block_ptr = 0;
                *dst_block_ptr.add(1) = 0;

                // Set all indices to max value (0xFF)
                core::ptr::write_bytes(dst_block_ptr.add(2), 0xFF, 6);
            } else {
                // For non-opaque, use the same approach as UniformAlphaZeroIndices
                normalize_alpha(
                    src_block_ptr,
                    dst_block_ptr,
                    alpha_value,
                    AlphaNormalizationMode::UniformAlphaZeroIndices,
                );
            }
        }
    }
}

/// Normalizes the color component of a BC3 block
///
/// Applies the selected normalization mode to the color component of a BC3 block.
/// This is particularly effective for blocks with solid colors that have a clean
/// conversion between 8888 and 565 color formats.
///
/// # Parameters
///
/// - `color565`: The RGB565 color value to use for normalization
/// - `dst_block_ptr`: Pointer to the destination block where normalized color will be written
/// - `mode`: The color normalization mode to use
///
/// # Safety
///
/// This is an unsafe function that requires:
/// - Valid pointer to destination block with at least 8 bytes accessible at offset 8
/// - color565 must be the actual RGB565 value corresponding to the solid color
#[inline]
unsafe fn normalize_color(color565: u16, dst_block_ptr: *mut u8, mode: ColorNormalizationMode) {
    let color_ptr = dst_block_ptr.add(8);
    let color_bytes = color565.to_le_bytes();

    match mode {
        ColorNormalizationMode::None => {
            // Do nothing - color component was already copied elsewhere
        }
        ColorNormalizationMode::Color0Only => {
            // Write Color0 (the solid color)
            *color_ptr = color_bytes[0];
            *color_ptr.add(1) = color_bytes[1];

            // Write Color1 = 0
            *color_ptr.add(2) = 0;
            *color_ptr.add(3) = 0;

            // Write indices = 0
            *color_ptr.add(4) = 0;
            *color_ptr.add(5) = 0;
            *color_ptr.add(6) = 0;
            *color_ptr.add(7) = 0;
        }
        ColorNormalizationMode::ReplicateColor => {
            // Write Color0 (the solid color)
            *color_ptr = color_bytes[0];
            *color_ptr.add(1) = color_bytes[1];

            // Write Color1 = same color
            *color_ptr.add(2) = color_bytes[0];
            *color_ptr.add(3) = color_bytes[1];

            // Write indices = 0
            *color_ptr.add(4) = 0;
            *color_ptr.add(5) = 0;
            *color_ptr.add(6) = 0;
            *color_ptr.add(7) = 0;
        }
    }
}

/// Alpha block processing case for the normalization functions
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum AlphaBlockCase {
    /// Block with uniform (all equal) alpha
    UniformAlpha,
    /// Block with non-uniform alpha (cannot normalize)
    CannotNormalize,
}

/// Color block processing case for the normalization functions
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ColorBlockCase {
    /// Block with solid color that can be round-tripped
    SolidColorRoundtrippable,
    /// Block with non-uniform color or cannot be round-tripped
    CannotNormalize,
}

/// Generic implementation for normalizing blocks with customizable output handling.
///
/// This internal function encapsulates the common logic for block analysis
/// and delegates the output writing to a closure.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks)
/// - `len`: The length of the input data in bytes
/// - `handle_output`: A closure that handles writing the output. The closure receives:
///   - The source block pointer
///   - The alpha block case (uniform or cannot normalize)
///   - The color block case (solid color or cannot normalize)
///   - The color in RGB565 format (valid only for solid color blocks)
///   - The alpha value (valid only for uniform alpha blocks)
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - len must be divisible by 16
/// - The closure must handle memory safety for all output operations
#[inline]
unsafe fn normalize_blocks_impl<F>(input_ptr: *const u8, len: usize, mut handle_output: F)
where
    F: FnMut(*const u8, AlphaBlockCase, ColorBlockCase, Color565, u8),
{
    debug_assert!(len % 16 == 0);

    // Calculate pointers to current block
    let mut src_block_ptr = input_ptr;
    let src_end_ptr = input_ptr.add(len);

    // Process each block
    while src_block_ptr < src_end_ptr {
        // Decode the block to analyze its content
        let decoded_block = decode_bc3_block(src_block_ptr);

        // Check for uniform alpha
        let has_uniform_alpha = decoded_block.has_identical_alpha();

        // Determine alpha case
        let alpha_case = if has_uniform_alpha {
            AlphaBlockCase::UniformAlpha
        } else {
            AlphaBlockCase::CannotNormalize
        };

        // Check for solid color (ignoring alpha)
        let has_solid_color = decoded_block.has_identical_pixels_ignore_alpha();

        // Get the first pixel (will be used if solid color)
        let pixel = decoded_block.pixels[0];
        let color565 = pixel.to_color_565();

        // Determine color case
        let color_case = if has_solid_color {
            // Check if color can be round-tripped cleanly through RGB565
            let color8888 = color565.to_color_8888();
            let pixel_ignore_alpha = Color8888::new(pixel.r, pixel.g, pixel.b, 255);
            let color8888_ignore_alpha = Color8888::new(color8888.r, color8888.g, color8888.b, 255);

            if unlikely(color8888_ignore_alpha == pixel_ignore_alpha) {
                ColorBlockCase::SolidColorRoundtrippable
            } else {
                ColorBlockCase::CannotNormalize
            }
        } else {
            ColorBlockCase::CannotNormalize
        };

        // Call the output handler with the determined cases
        handle_output(
            src_block_ptr,
            alpha_case,
            color_case,
            color565,
            decoded_block.pixels[0].a,
        );

        // Move to the next block
        src_block_ptr = src_block_ptr.add(16);
    }
}

/// Reads an input of blocks from `input_ptr` and writes the normalized blocks to multiple output pointers,
/// one for each combination of [`AlphaNormalizationMode`] and [`ColorNormalizationMode`].
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks)
/// - `output_ptrs`: A 2D array of output pointers, indexed by [alpha_mode][color_mode]
/// - `len`: The length of the input data in bytes
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - each pointer in output_ptrs must be valid for writes of len bytes
/// - len must be divisible by 16
/// - input_ptr and output_ptrs must not overlap
///
/// # Remarks
///
/// This function processes each block once and writes it to multiple output buffers,
/// applying different combinations of normalization modes to each output.
///
/// The output_ptrs array must be a 2D array with dimensions `[AlphaNormalizationMode::all_values().len()][ColorNormalizationMode::all_values().len()]`,
/// with pointers organized in the same order as the modes are defined in their respective enums.
///
/// See the module-level documentation for more details on the normalization process.
#[inline]
pub unsafe fn normalize_blocks_all_modes(
    input_ptr: *const u8,
    output_ptrs: &[[*mut u8; ColorNormalizationMode::all_values().len()];
         AlphaNormalizationMode::all_values().len()],
    len: usize,
) {
    debug_assert!(len % 16 == 0);
    debug_assert!(
        output_ptrs.iter().flatten().all(|&out_ptr| {
            input_ptr.add(len) <= out_ptr || out_ptr.add(len) <= input_ptr as *mut _
        }),
        "Input and output memory regions must not overlap"
    );

    // Setup arrays to track current position in each output buffer
    let mut dst_block_ptrs = [[core::ptr::null_mut::<u8>();
        ColorNormalizationMode::all_values().len()];
        AlphaNormalizationMode::all_values().len()];

    // Initialize destination pointers
    for (a_idx, a_ptrs) in dst_block_ptrs.iter_mut().enumerate() {
        for (c_idx, dst_ptr) in a_ptrs.iter_mut().enumerate() {
            *dst_ptr = output_ptrs[a_idx][c_idx];
        }
    }

    normalize_blocks_impl(
        input_ptr,
        len,
        |src_block_ptr, alpha_case, color_case, color565, alpha_value| {
            // Process for each combination of modes
            for (a_idx, alpha_mode) in AlphaNormalizationMode::all_values().iter().enumerate() {
                for (c_idx, color_mode) in ColorNormalizationMode::all_values().iter().enumerate() {
                    let dst_block_ptr = dst_block_ptrs[a_idx][c_idx];

                    // Handle alpha normalization
                    match alpha_case {
                        AlphaBlockCase::UniformAlpha => {
                            if *alpha_mode != AlphaNormalizationMode::None {
                                normalize_alpha(
                                    src_block_ptr,
                                    dst_block_ptr,
                                    alpha_value,
                                    *alpha_mode,
                                );
                            } else {
                                // Copy alpha data as-is (first 8 bytes)
                                copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
                            }
                        }
                        AlphaBlockCase::CannotNormalize => {
                            // Copy alpha data as-is (first 8 bytes)
                            copy_nonoverlapping(src_block_ptr, dst_block_ptr, 8);
                        }
                    }

                    // Handle color normalization
                    match color_case {
                        ColorBlockCase::SolidColorRoundtrippable => {
                            if *color_mode != ColorNormalizationMode::None {
                                normalize_color(color565.raw_value(), dst_block_ptr, *color_mode);
                            } else {
                                // Copy color data as-is (last 8 bytes)
                                copy_nonoverlapping(src_block_ptr.add(8), dst_block_ptr.add(8), 8);
                            }
                        }
                        ColorBlockCase::CannotNormalize => {
                            // Copy color data as-is (last 8 bytes)
                            copy_nonoverlapping(src_block_ptr.add(8), dst_block_ptr.add(8), 8);
                        }
                    }

                    // Update position in this output buffer
                    dst_block_ptrs[a_idx][c_idx] = dst_block_ptrs[a_idx][c_idx].add(16);
                }
            }
        },
    );
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use core::ptr;

    use rstest::rstest;

    use super::*;

    /// Test normalizing a solid color block with uniform alpha
    #[rstest]
    #[case(AlphaNormalizationMode::None, ColorNormalizationMode::Color0Only)]
    #[case(AlphaNormalizationMode::None, ColorNormalizationMode::ReplicateColor)]
    #[case(
        AlphaNormalizationMode::UniformAlphaZeroIndices,
        ColorNormalizationMode::Color0Only
    )]
    #[case(AlphaNormalizationMode::OpaqueFillAll, ColorNormalizationMode::None)]
    #[case(
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ColorNormalizationMode::None
    )]
    #[case(
        AlphaNormalizationMode::OpaqueFillAll,
        ColorNormalizationMode::Color0Only
    )]
    #[case(
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ColorNormalizationMode::ReplicateColor
    )]
    fn can_normalize_block_opaque_alpha_single_colour(
        #[case] alpha_mode: AlphaNormalizationMode,
        #[case] color_mode: ColorNormalizationMode,
    ) {
        // Red in RGB565: (31, 0, 0) -> 0xF800
        // This cleanly round trips into (255, 0, 0) and back to (31, 0, 0).
        let red565 = 0xF800u16.to_le_bytes(); // Little endian: [0x00, 0xF8]

        // This creates a BC3 block with the following characteristics:
        // - Alpha endpoints (A0=A1=0xFF)
        // - Alpha indices all point to A0
        // - Color0 = Red (RGB565)
        // - Color1 = Another color (doesn't matter)
        // - Indices = All pointing to Color0 (all 0b00)
        let mut block = [0u8; 16];

        // Set alpha endpoints to fully opaque
        block[0] = 0xFF; // A0
        block[1] = 0xFF; // A1

        // Set alpha indices to point to A0
        for x in 2..8 {
            block[x] = 0; // All indices point to A0
        }

        // Set Color0 to red
        block[8] = red565[0];
        block[9] = red565[1];

        // Set Color1 to another color (doesn't matter)
        block[10] = 0x12;
        block[11] = 0x34;

        // Set indices to all point to Color0
        for x in 12..16 {
            block[x] = 0; // All indices point to Color0
        }

        // Create output buffer
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(
                block.as_ptr(),
                output.as_mut_ptr(),
                16,
                alpha_mode,
                color_mode,
            );
        }

        // Verify normalization
        // For alpha
        match alpha_mode {
            AlphaNormalizationMode::None => {
                // Alpha part should be unchanged
                assert_eq!(output[0], 0xFF);
                assert_eq!(output[1], 0xFF);
                for x in 2..8 {
                    assert_eq!(output[x], 0);
                }
            }
            AlphaNormalizationMode::UniformAlphaZeroIndices => {
                // A0 set to alpha value, rest zero
                assert_eq!(output[0], 0xFF);
                assert_eq!(output[1], 0);
                for x in 2..8 {
                    assert_eq!(output[x], 0);
                }
            }
            AlphaNormalizationMode::OpaqueFillAll => {
                // For fully opaque, all alpha bytes should be 0xFF
                for x in 0..8 {
                    assert_eq!(output[x], 0xFF);
                }
            }
            AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices => {
                // For fully opaque, alpha endpoints should be 0, indices should be 0xFF
                assert_eq!(output[0], 0);
                assert_eq!(output[1], 0);
                for x in 2..8 {
                    assert_eq!(output[x], 0xFF);
                }
            }
        }

        // For color
        match color_mode {
            ColorNormalizationMode::Color0Only => {
                // Color0 = red, Color1 = 0, indices = 0
                assert_eq!(output[8], red565[0]);
                assert_eq!(output[9], red565[1]);
                assert_eq!(output[10], 0);
                assert_eq!(output[11], 0);
                for x in 12..16 {
                    assert_eq!(output[x], 0);
                }
            }
            ColorNormalizationMode::ReplicateColor => {
                // Color0 = Color1 = red, indices = 0
                assert_eq!(output[8], red565[0]);
                assert_eq!(output[9], red565[1]);
                assert_eq!(output[10], red565[0]);
                assert_eq!(output[11], red565[1]);
                for x in 12..16 {
                    assert_eq!(output[x], 0);
                }
            }
            _ => {}
        }

        // Decode the normalized block to verify it still represents the same pixels
        let decoded = unsafe { decode_bc3_block(output.as_ptr()) };

        // Check all pixels have the correct color and alpha
        for x in 0..16 {
            assert_eq!(decoded.pixels[x].r, 255); // Red
            assert_eq!(decoded.pixels[x].g, 0);
            assert_eq!(decoded.pixels[x].b, 0);
            assert_eq!(decoded.pixels[x].a, 255); // Fully opaque
        }
    }

    /// Test normalizing a block with uniform alpha but mixed colors
    #[rstest]
    #[case(AlphaNormalizationMode::UniformAlphaZeroIndices)]
    #[case(AlphaNormalizationMode::OpaqueFillAll)]
    fn can_normalize_uniform_alpha_mixed_colors(#[case] alpha_mode: AlphaNormalizationMode) {
        // Create a BC3 block with uniform alpha but mixed colors
        let mut block = [0u8; 16];

        // Set alpha endpoints to fully opaque
        block[0] = 0xFF; // A0
        block[1] = 0xFF; // A1

        // Set alpha indices to point to A0
        for x in 2..8 {
            block[x] = 0; // All indices point to A0
        }

        // Set Color0 and Color1 to different colors
        block[8] = 0x00;
        block[9] = 0xF8; // Red
        block[10] = 0xE0;
        block[11] = 0x07; // Green

        // Set mixed indices (use both colors)
        block[12] = 0x55; // Alternating 01 01 01 01
        block[13] = 0x55;
        block[14] = 0x55;
        block[15] = 0x55;

        // Create output buffer
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(
                block.as_ptr(),
                output.as_mut_ptr(),
                16,
                alpha_mode,
                ColorNormalizationMode::None,
            );
        }

        // Verify alpha normalization
        match alpha_mode {
            AlphaNormalizationMode::UniformAlphaZeroIndices => {
                // A0 set to alpha value, rest zero
                assert_eq!(output[0], 0xFF);
                assert_eq!(output[1], 0);
                for x in 2..8 {
                    assert_eq!(output[x], 0);
                }
            }
            AlphaNormalizationMode::OpaqueFillAll => {
                // All alpha bytes set to 0xFF
                for x in 0..8 {
                    assert_eq!(output[x], 0xFF);
                }
            }
            _ => {}
        }

        // Color part should be unchanged
        assert_eq!(output[8], block[8]);
        assert_eq!(output[9], block[9]);
        assert_eq!(output[10], block[10]);
        assert_eq!(output[11], block[11]);
        assert_eq!(output[12], block[12]);
        assert_eq!(output[13], block[13]);
        assert_eq!(output[14], block[14]);
        assert_eq!(output[15], block[15]);

        // Decode the normalized block to verify it still represents the same visual data
        let decoded = unsafe { decode_bc3_block(output.as_ptr()) };

        // All pixels should have alpha = 255
        for x in 0..16 {
            assert_eq!(decoded.pixels[x].a, 255);
        }
    }

    /// Test normalizing a block with mixed alpha and solid color
    #[rstest]
    #[case(ColorNormalizationMode::Color0Only)]
    #[case(ColorNormalizationMode::ReplicateColor)]
    fn can_normalize_mixed_alpha_solid_color(#[case] color_mode: ColorNormalizationMode) {
        // Create a BC3 block with varying alpha but solid color
        let mut block = [0u8; 16];

        // Set different alpha endpoints
        block[0] = 0xFF; // A0
        block[1] = 0x80; // A1

        // Set mixed alpha indices
        for x in 2..8 {
            block[x] = 0x55; // Mixed indices
        }

        // Set Color0 and Color1 to the same color (red)
        let red565 = 0xF800u16.to_le_bytes();
        block[8] = red565[0];
        block[9] = red565[1];
        block[10] = 0x12; // Different value for color1
        block[11] = 0x34;

        // Set all indices to point to Color0
        for x in 12..16 {
            block[x] = 0;
        }

        // Create output buffer
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(
                block.as_ptr(),
                output.as_mut_ptr(),
                16,
                AlphaNormalizationMode::OpaqueFillAll,
                color_mode,
            );
        }

        // Alpha part should be unchanged regardless of option
        for x in 0..8 {
            assert_eq!(output[x], block[x]);
        }

        // Verify color normalization
        match color_mode {
            ColorNormalizationMode::Color0Only => {
                // Color0 = red, Color1 = 0, indices = 0
                assert_eq!(output[8], red565[0]);
                assert_eq!(output[9], red565[1]);
                assert_eq!(output[10], 0);
                assert_eq!(output[11], 0);
                for x in 12..16 {
                    assert_eq!(output[x], 0);
                }
            }
            ColorNormalizationMode::ReplicateColor => {
                // Color0 = Color1 = red, indices = 0
                assert_eq!(output[8], red565[0]);
                assert_eq!(output[9], red565[1]);
                assert_eq!(output[10], red565[0]);
                assert_eq!(output[11], red565[1]);
                for x in 12..16 {
                    assert_eq!(output[x], 0);
                }
            }
            _ => {}
        }
    }

    /// Test the OpaqueZeroAlphaMaxIndices alpha mode
    #[test]
    fn can_normalize_with_opaque_zero_alpha_max_indices() {
        // Create a BC3 block with fully opaque alpha
        let mut block = [0u8; 16];

        // Set alpha endpoints to fully opaque
        block[0] = 0xFF; // A0
        block[1] = 0xFF; // A1

        // Set some alpha indices
        for x in 2..8 {
            block[x] = 0; // All indices point to A0
        }

        // Set some color values (doesn't matter for this test)
        for x in 8..16 {
            block[x] = x as u8;
        }

        // Create output buffer
        let mut output = [0u8; 16];

        // Normalize the block
        unsafe {
            normalize_blocks(
                block.as_ptr(),
                output.as_mut_ptr(),
                16,
                AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
                ColorNormalizationMode::None,
            );
        }

        // Verify alpha normalization
        // First two bytes should be zero
        assert_eq!(output[0], 0);
        assert_eq!(output[1], 0);

        // All index bytes should be 0xFF
        for x in 2..8 {
            assert_eq!(output[x], 0xFF);
        }

        // Color bytes should be unchanged
        for x in 8..16 {
            assert_eq!(output[x], block[x]);
        }

        // Decode the normalized block to verify alphas are still 255
        let decoded = unsafe { decode_bc3_block(output.as_ptr()) };
        for x in 0..16 {
            assert_eq!(decoded.pixels[x].a, 255);
        }
    }

    /// Test that non-opaque uniform alpha is properly normalized
    #[test]
    fn can_normalize_non_opaque_uniform_alpha() {
        // Create a BC3 block with semi-transparent alpha
        let mut block = [0u8; 16];

        // Set alpha endpoints to 128 (semi-transparent)
        block[0] = 128; // A0
        block[1] = 128; // A1

        // Set all alpha indices to point to A0
        for x in 2..8 {
            block[x] = 0;
        }

        // Set some color values (doesn't matter for this test)
        for x in 8..16 {
            block[x] = x as u8;
        }

        // Create output buffer
        let mut output = [0u8; 16];

        // Test non-None alpha modes
        let normalization_modes = [
            AlphaNormalizationMode::UniformAlphaZeroIndices,
            AlphaNormalizationMode::OpaqueFillAll,
            AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ];

        for mode in normalization_modes {
            // Normalize the block
            unsafe {
                normalize_blocks(
                    block.as_ptr(),
                    output.as_mut_ptr(),
                    16,
                    mode,
                    ColorNormalizationMode::None,
                );
            }

            // For non-opaque alpha, all normalization modes should behave like UniformAlphaZeroIndices
            assert_eq!(output[0], 128, "Alpha value incorrect for mode {mode:?}"); // A0 = alpha value
            assert_eq!(output[1], 0, "A1 value incorrect for mode {mode:?}"); // A1 = 0

            // All index bytes should be 0
            for x in 2..8 {
                assert_eq!(output[x], 0, "Index byte {x} incorrect for mode {mode:?}");
            }

            // Decode the normalized block to verify alphas are still 128
            let decoded = unsafe { decode_bc3_block(output.as_ptr()) };
            for x in 0..16 {
                assert_eq!(
                    decoded.pixels[x].a, 128,
                    "Decoded alpha incorrect for mode {mode:?}",
                );
            }
        }

        // Test None mode separately - it should preserve the original data
        unsafe {
            normalize_blocks(
                block.as_ptr(),
                output.as_mut_ptr(),
                16,
                AlphaNormalizationMode::None,
                ColorNormalizationMode::None,
            );
        }

        // With None mode, the alpha data should be unchanged
        assert_eq!(output[0], 128); // A0 = original value
        assert_eq!(output[1], 128); // A1 = original value

        // Indices should also be unchanged
        for x in 2..8 {
            assert_eq!(output[x], 0);
        }

        // Decoded alpha should still be 128
        let decoded = unsafe { decode_bc3_block(output.as_ptr()) };
        for x in 0..16 {
            assert_eq!(decoded.pixels[x].a, 128);
        }
    }

    /// Test normalizing multiple blocks in one call
    #[test]
    fn can_normalize_multiple_blocks() {
        // Create two BC3 blocks
        let mut blocks = [0u8; 32]; // 2 blocks, 16 bytes each

        // Block 1: Solid red with uniform alpha
        // Set alpha endpoints to fully opaque
        blocks[0] = 0xFF;
        blocks[1] = 0xFF;

        // Set alpha indices
        for x in 2..8 {
            blocks[x] = 0;
        }

        // Set Color0 to red, Color1 to something else
        let red565 = 0xF800u16.to_le_bytes();
        blocks[8] = red565[0];
        blocks[9] = red565[1];
        blocks[10] = 0x12;
        blocks[11] = 0x34;

        // Set indices to all point to Color0
        for x in 12..16 {
            blocks[x] = 0;
        }

        // Block 2: Explicitly set to have mixed alpha and colors
        // This should not get normalized

        // Set different alpha endpoints
        blocks[16] = 0x80; // A0
        blocks[17] = 0x40; // A1

        // Important: Need different alpha indices to create truly mixed alpha values
        blocks[18] = 0xAA; // Mixed pattern of indices
        blocks[19] = 0x55; // Different pattern
        blocks[20] = 0x33; // Different pattern
        blocks[21] = 0xCC; // Different pattern
        blocks[22] = 0x0F; // Different pattern
        blocks[23] = 0xF0; // Different pattern

        // Set different colors in Color0 and Color1
        let green565 = 0x07E0u16.to_le_bytes();
        blocks[24] = green565[0]; // Green in Color0
        blocks[25] = green565[1];

        let blue565 = 0x001Fu16.to_le_bytes();
        blocks[26] = blue565[0]; // Blue in Color1
        blocks[27] = blue565[1];

        // Set truly mixed color indices
        blocks[28] = 0x55; // Mixed pattern 01 01 01 01
        blocks[29] = 0xAA; // Mixed pattern 10 10 10 10
        blocks[30] = 0x3C; // Mixed pattern 00 11 11 00
        blocks[31] = 0x69; // Mixed pattern 01 10 01 00

        // Create a copy of the blocks for later comparison
        let blocks_copy = blocks;

        // Create output buffer
        let mut output = [0u8; 32];

        // Normalize the blocks
        unsafe {
            normalize_blocks(
                blocks.as_ptr(),
                output.as_mut_ptr(),
                32,
                AlphaNormalizationMode::UniformAlphaZeroIndices,
                ColorNormalizationMode::Color0Only,
            );
        }

        // Block 1 should be normalized for both alpha and color
        assert_eq!(output[0], 0xFF); // A0 = alpha value
        assert_eq!(output[1], 0); // A1 = 0

        // Alpha indices should be 0 (UniformAlphaZeroIndices)
        for x in 2..8 {
            assert_eq!(output[x], 0);
        }

        // Color0 should be red, Color1 and indices should be 0 (Color0Only mode)
        assert_eq!(output[8], red565[0]);
        assert_eq!(output[9], red565[1]);
        assert_eq!(output[10], 0);
        assert_eq!(output[11], 0);

        for x in 12..16 {
            assert_eq!(output[x], 0);
        }

        // Block 2 should be completely unchanged as it has mixed colors and mixed alpha
        for x in 16..32 {
            assert_eq!(
                output[x], blocks_copy[x],
                "Block 2 byte {x} was modified when it should be unchanged",
            );
        }
    }

    /// Test normalizing blocks using all combinations of modes
    #[test]
    fn can_normalize_blocks_all_modes() {
        // Create a single BC3 block with solid red color and uniform alpha
        let mut block = [0u8; 16]; // 1 block, 16 bytes

        // Set alpha endpoints to fully opaque
        block[0] = 0xFF;
        block[1] = 0xFF;

        // Set alpha indices
        for x in 2..8 {
            block[x] = 0;
        }

        // Set Color0 to red, Color1 to something else
        let red565 = 0xF800u16.to_le_bytes();
        block[8] = red565[0];
        block[9] = red565[1];
        block[10] = 0x12;
        block[11] = 0x34;

        // Set indices to all point to Color0
        for x in 12..16 {
            block[x] = 0;
        }

        // Create output buffers for all mode combinations
        const ALPHA_MODE_COUNT: usize = AlphaNormalizationMode::all_values().len();
        const COLOR_MODE_COUNT: usize = ColorNormalizationMode::all_values().len();

        let mut output_buffers = vec![vec![0u8; 16]; ALPHA_MODE_COUNT * COLOR_MODE_COUNT];
        let mut output_ptrs = [[ptr::null_mut::<u8>(); COLOR_MODE_COUNT]; ALPHA_MODE_COUNT];

        // Set up output pointers
        for a_idx in 0..ALPHA_MODE_COUNT {
            for c_idx in 0..COLOR_MODE_COUNT {
                let buffer_idx = (a_idx * COLOR_MODE_COUNT) + c_idx;
                output_ptrs[a_idx][c_idx] = output_buffers[buffer_idx].as_mut_ptr();
            }
        }

        // Normalize the block using all mode combinations
        unsafe {
            normalize_blocks_all_modes(block.as_ptr(), &output_ptrs, 16);
        }

        // Verify each output buffer has been normalized according to its mode combination
        for (a_idx, a_mode) in AlphaNormalizationMode::all_values().iter().enumerate() {
            for (c_idx, c_mode) in ColorNormalizationMode::all_values().iter().enumerate() {
                let buffer_idx = a_idx * COLOR_MODE_COUNT + c_idx;
                let output = &output_buffers[buffer_idx];

                // Create a reference output by normalizing with the same modes individually
                let mut reference_output = [0u8; 16];
                unsafe {
                    normalize_blocks(
                        block.as_ptr(),
                        reference_output.as_mut_ptr(),
                        16,
                        *a_mode,
                        *c_mode,
                    );
                }

                // Compare the output with the reference
                for x in 0..16 {
                    assert_eq!(
                        output[x], reference_output[x],
                        "Output for mode combination [{a_mode:?}][{c_mode:?}] at byte {x} does not match expected value"
                    );
                }
            }
        }
    }
}
