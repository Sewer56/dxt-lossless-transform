use crate::util::decode_bc3_block;
use core::ptr::{copy_nonoverlapping, eq, null_mut, read_unaligned, write_bytes};
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
/// - The implementation supports `input_ptr` == `output_ptr` (in-place transformation)
/// - The implementation does NOT support partially overlapping buffers (they must either be completely separate or identical)
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
        input_ptr == output_ptr
            || input_ptr.add(len) <= output_ptr
            || output_ptr.add(len) <= input_ptr as *mut u8,
        "Input and output memory regions must either be the same (in-place) or not overlap"
    );

    // Skip normalization if both modes are None
    if alpha_mode == AlphaNormalizationMode::None && color_mode == ColorNormalizationMode::None {
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
                        (dst_block_ptr as *mut u64)
                            .write_unaligned(read_unaligned(src_block_ptr as *const u64));
                    }
                }
                AlphaBlockCase::CannotNormalize => {
                    // Copy alpha data as-is (first 8 bytes) using write_unaligned
                    (dst_block_ptr as *mut u64)
                        .write_unaligned(read_unaligned(src_block_ptr as *const u64));
                }
            }

            // Handle color normalization
            match color_case {
                ColorBlockCase::SolidColorRoundtrippable => {
                    if color_mode != ColorNormalizationMode::None {
                        normalize_color(color565.raw_value(), dst_block_ptr, color_mode);
                    } else {
                        // Copy color data as-is (last 8 bytes)
                        (dst_block_ptr.add(8) as *mut u64)
                            .write_unaligned(read_unaligned(src_block_ptr.add(8) as *const u64));
                    }
                }
                ColorBlockCase::CannotNormalize => {
                    // Copy color data as-is (last 8 bytes) using write_unaligned
                    (dst_block_ptr.add(8) as *mut u64)
                        .write_unaligned(read_unaligned(src_block_ptr.add(8) as *const u64));
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
            // Copy alpha data as-is (first 8 bytes) using write_unaligned
            (dst_block_ptr as *mut u64)
                .write_unaligned(read_unaligned(src_block_ptr as *const u64));
        }
        AlphaNormalizationMode::UniformAlphaZeroIndices => {
            // Set A0 to the alpha value, everything else to 0
            *dst_block_ptr = alpha_value;
            *dst_block_ptr.add(1) = 0;

            // Zero all index bytes
            write_bytes(dst_block_ptr.add(2), 0, 6);
        }
        AlphaNormalizationMode::OpaqueFillAll => {
            if alpha_value == 255 {
                // Fill all alpha bytes with 0xFF
                write_bytes(dst_block_ptr, 0xFF, 8);
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
                write_bytes(dst_block_ptr.add(2), 0xFF, 6);
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
        let color565 = pixel.to_565_lossy();

        // Determine color case
        let color_case = if has_solid_color {
            // Check if color can be round-tripped cleanly through RGB565
            let color8888 = color565.to_8888_lossy();
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
/// - `output_ptrs`: A 2D array of output pointers, indexed by `[alpha_mode][color_mode]`
/// - `len`: The length of the input data in bytes
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - each pointer in output_ptrs must be valid for writes of len bytes
/// - len must be divisible by 16
/// - The implementation supports in-place transformation (input_ptr can match any output_ptr)
/// - The implementation does NOT support partially overlapping buffers (they must either be completely separate or identical)
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
    let mut dst_block_ptrs = [[null_mut::<u8>(); ColorNormalizationMode::all_values().len()];
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
                                (dst_block_ptr as *mut u64)
                                    .write_unaligned(read_unaligned(src_block_ptr as *const u64));
                            }
                        }
                        AlphaBlockCase::CannotNormalize => {
                            // Copy alpha data as-is (first 8 bytes)
                            (dst_block_ptr as *mut u64)
                                .write_unaligned(read_unaligned(src_block_ptr as *const u64));
                        }
                    }

                    // Handle color normalization
                    match color_case {
                        ColorBlockCase::SolidColorRoundtrippable => {
                            if *color_mode != ColorNormalizationMode::None {
                                normalize_color(color565.raw_value(), dst_block_ptr, *color_mode);
                            } else {
                                // Copy color data as-is (last 8 bytes)
                                (dst_block_ptr.add(8) as *mut u64).write_unaligned(read_unaligned(
                                    src_block_ptr.add(8) as *const u64,
                                ));
                            }
                        }
                        ColorBlockCase::CannotNormalize => {
                            // Copy color data as-is (last 8 bytes)
                            (dst_block_ptr.add(8) as *mut u64).write_unaligned(read_unaligned(
                                src_block_ptr.add(8) as *const u64,
                            ));
                        }
                    }

                    // Update position in this output buffer
                    dst_block_ptrs[a_idx][c_idx] = dst_block_ptrs[a_idx][c_idx].add(16);
                }
            }
        },
    );
}

/// Normalizes BC3 blocks that are already split into separate alpha endpoint, alpha indices, color and index sections.
///
/// # Parameters
///
/// - `alpha_endpoints_ptr`: A pointer to the section containing the alpha endpoints (2 bytes per block)
/// - `alpha_indices_ptr`: A pointer to the section containing the alpha indices (6 bytes per block)
/// - `color_endpoints_ptr`: A pointer to the section containing the color endpoints (4 bytes per block)
/// - `color_indices_ptr`: A pointer to the section containing the color indices (4 bytes per block)
/// - `num_blocks`: The number of blocks to process (1 block = 16 bytes total across all sections)
/// - `alpha_mode`: How to normalize alpha values
/// - `color_mode`: How to normalize color values
///
/// # Safety
///
/// - alpha_endpoints_ptr must be valid for reads and writes of num_blocks * 2 bytes
/// - alpha_indices_ptr must be valid for reads and writes of num_blocks * 6 bytes
/// - color_endpoints_ptr must be valid for reads and writes of num_blocks * 4 bytes
/// - color_indices_ptr must be valid for reads and writes of num_blocks * 4 bytes
/// - This function works in-place, modifying all buffers directly
///
/// # Remarks
///
/// This function normalizes blocks that have already been split into 4 separate sections:
/// - Alpha endpoints (A0-A1): 2 bytes per block
/// - Alpha indices (16x 3-bit): 6 bytes per block  
/// - Color endpoints (C0-C1): 4 bytes per block
/// - Color indices (16x 2-bit): 4 bytes per block
///
/// It applies the same normalization rules as [`normalize_blocks`]:
/// - Blocks with uniform alpha are normalized according to the alpha_mode
/// - Solid color blocks are normalized according to the color_mode
/// - Other blocks are preserved as-is
///
/// See the module-level documentation for more details on the normalization process.
#[inline]
pub unsafe fn normalize_split_blocks_in_place(
    alpha_endpoints_ptr: *mut u8,
    alpha_indices_ptr: *mut u8,
    color_endpoints_ptr: *mut u8,
    color_indices_ptr: *mut u8,
    num_blocks: usize,
    alpha_mode: AlphaNormalizationMode,
    color_mode: ColorNormalizationMode,
) {
    // Skip normalization if both modes are None
    if alpha_mode == AlphaNormalizationMode::None && color_mode == ColorNormalizationMode::None {
        return;
    }

    // Process each block
    for block_idx in 0..num_blocks {
        // Calculate current block pointers for each section
        let curr_alpha_endpoints_ptr = alpha_endpoints_ptr.add(block_idx * 2);
        let curr_alpha_indices_ptr = alpha_indices_ptr.add(block_idx * 6);
        let curr_color_endpoints_ptr = color_endpoints_ptr.add(block_idx * 4);
        let curr_color_indices_ptr = color_indices_ptr.add(block_idx * 4);

        // Reconstruct a temporary block for analysis
        // BC3 layout: [alpha endpoints: 2 bytes][alpha indices: 6 bytes][color endpoints: 4 bytes][color indices: 4 bytes]
        let mut temp_block = [0u8; 16];
        copy_nonoverlapping(curr_alpha_endpoints_ptr, temp_block.as_mut_ptr(), 2);
        copy_nonoverlapping(curr_alpha_indices_ptr, temp_block.as_mut_ptr().add(2), 6);
        copy_nonoverlapping(curr_color_endpoints_ptr, temp_block.as_mut_ptr().add(8), 4);
        copy_nonoverlapping(curr_color_indices_ptr, temp_block.as_mut_ptr().add(12), 4);

        // Decode the block to analyze its content
        let decoded_block = decode_bc3_block(temp_block.as_ptr());

        // Check alpha normalization is enabled and alpha can be normalized
        // (all pixels have the same value)
        if alpha_mode != AlphaNormalizationMode::None && decoded_block.has_identical_alpha() {
            normalize_alpha_in_place(
                alpha_mode,
                curr_alpha_endpoints_ptr,
                curr_alpha_indices_ptr,
                decoded_block.pixels[0].a, // same in entire block, so can use first pixel
            );
        }

        // Check color normalization is enabled and colour can be normalized
        // (all pixels have the same value, excluding alpha)
        if color_mode != ColorNormalizationMode::None
            && decoded_block.has_identical_pixels_ignore_alpha()
        {
            // Check if all pixels have the same color (ignoring alpha)
            let pixel = decoded_block.pixels[0];
            let pixel_ignore_alpha = Color8888::new(pixel.r, pixel.g, pixel.b, 255);

            // Solid color - check if it can be normalized
            let color565 = pixel_ignore_alpha.to_565_lossy();
            let color8888_roundtrip = color565.to_8888_lossy();

            if unlikely(color8888_roundtrip == pixel_ignore_alpha) {
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
                        *curr_color_endpoints_ptr = color_bytes[0];
                        *curr_color_endpoints_ptr.add(1) = color_bytes[1];

                        // Write Color1 = 0
                        *curr_color_endpoints_ptr.add(2) = 0;
                        *curr_color_endpoints_ptr.add(3) = 0;

                        // Write indices = 0
                        *curr_color_indices_ptr = 0;
                        *curr_color_indices_ptr.add(1) = 0;
                        *curr_color_indices_ptr.add(2) = 0;
                        *curr_color_indices_ptr.add(3) = 0;
                    }
                    ColorNormalizationMode::ReplicateColor => {
                        // Write Color0 (the solid color)
                        *curr_color_endpoints_ptr = color_bytes[0];
                        *curr_color_endpoints_ptr.add(1) = color_bytes[1];

                        // Write Color1 = same as Color0
                        *curr_color_endpoints_ptr.add(2) = color_bytes[0];
                        *curr_color_endpoints_ptr.add(3) = color_bytes[1];

                        // Write indices = 0
                        *curr_color_indices_ptr = 0;
                        *curr_color_indices_ptr.add(1) = 0;
                        *curr_color_indices_ptr.add(2) = 0;
                        *curr_color_indices_ptr.add(3) = 0;
                    }
                }
            }
        }
    }

    #[inline]
    unsafe fn normalize_alpha_in_place(
        alpha_mode: AlphaNormalizationMode,
        curr_alpha_endpoints_ptr: *mut u8,
        curr_alpha_indices_ptr: *mut u8,
        alpha_value: u8,
    ) {
        match alpha_mode {
            AlphaNormalizationMode::UniformAlphaZeroIndices => {
                // Set A0 to the alpha value, everything else to 0
                *curr_alpha_endpoints_ptr = alpha_value;
                *curr_alpha_endpoints_ptr.add(1) = 0;

                // Zero all index bytes
                write_bytes(curr_alpha_indices_ptr, 0, 6);
            }
            AlphaNormalizationMode::OpaqueFillAll => {
                if alpha_value == 255 {
                    // Fill all alpha bytes with 0xFF (both endpoints and values)
                    write_bytes(curr_alpha_endpoints_ptr, 0xFF, 2);
                    write_bytes(curr_alpha_indices_ptr, 0xFF, 6);
                } else {
                    // For non-opaque, use the same approach as UniformAlphaZeroIndices
                    normalize_alpha_in_place(
                        AlphaNormalizationMode::UniformAlphaZeroIndices,
                        curr_alpha_endpoints_ptr,
                        curr_alpha_indices_ptr,
                        alpha_value,
                    );
                }
            }
            AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices => {
                if alpha_value == 255 {
                    // Set alpha endpoints to 0
                    *curr_alpha_endpoints_ptr = 0;
                    *curr_alpha_endpoints_ptr.add(1) = 0;

                    // Set all indices to max value (0xFF)
                    write_bytes(curr_alpha_indices_ptr, 0xFF, 6);
                } else {
                    // For non-opaque, use the same approach as UniformAlphaZeroIndices
                    normalize_alpha_in_place(
                        AlphaNormalizationMode::UniformAlphaZeroIndices,
                        curr_alpha_endpoints_ptr,
                        curr_alpha_indices_ptr,
                        alpha_value,
                    );
                }
            }
            AlphaNormalizationMode::None => { /* no-op */ }
        }
    }
}

#[cfg(test)]
#[allow(clippy::needless_range_loop)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use core::ptr::null_mut;

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
        let mut output_ptrs = [[null_mut::<u8>(); COLOR_MODE_COUNT]; ALPHA_MODE_COUNT];

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

    #[rstest]
    #[case(
        AlphaNormalizationMode::OpaqueFillAll,
        ColorNormalizationMode::ReplicateColor
    )]
    #[case(
        AlphaNormalizationMode::UniformAlphaZeroIndices,
        ColorNormalizationMode::Color0Only
    )]
    #[case(
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ColorNormalizationMode::None
    )]
    #[test]
    fn can_normalize_blocks_inplace(
        #[case] alpha_mode: AlphaNormalizationMode,
        #[case] color_mode: ColorNormalizationMode,
    ) {
        // Create sample blocks - we'll use solid color with opaque alpha for simplicity
        let mut input_blocks = [0u8; 32]; // Two BC3 blocks

        // First block: solid red
        input_blocks[0] = 0xFF; // A0 = 255 (opaque)
        input_blocks[1] = 0xFF; // A1 = 255
                                // alpha indices set to pattern that's not all zeros
        for x in 2..8 {
            input_blocks[x] = 0xAA;
        }

        input_blocks[8] = 0x00; // C0 low byte (RGB565: 0xF800 = pure red)
        input_blocks[9] = 0xF8; // C0 high byte
        input_blocks[10] = 0x00; // C1 low byte (zero)
        input_blocks[11] = 0x00; // C1 high byte
                                 // No need to set color indices as they'll be set to zero (default)

        // Second block: solid green
        input_blocks[16] = 0xFF; // A0 = 255 (opaque)
        input_blocks[17] = 0xFF; // A1 = 255
                                 // alpha indices set to pattern that's not all zeros
        for x in 18..24 {
            input_blocks[x] = 0x55;
        }

        input_blocks[24] = 0xE0; // C0 low byte (RGB565: 0x07E0 = pure green)
        input_blocks[25] = 0x07; // C0 high byte
        input_blocks[26] = 0x00; // C1 low byte (zero)
        input_blocks[27] = 0x00; // C1 high byte
                                 // No need to set color indices as they'll be set to zero (default)

        // Create a copy for reference output
        let mut reference_output = input_blocks;

        // Normalize using separate buffers for reference
        unsafe {
            normalize_blocks(
                input_blocks.as_ptr(),
                reference_output.as_mut_ptr(),
                32,
                alpha_mode,
                color_mode,
            );
        }

        // Now normalize in-place
        unsafe {
            normalize_blocks(
                input_blocks.as_ptr(),
                input_blocks.as_mut_ptr(), // Same pointer for in-place operation
                32,
                alpha_mode,
                color_mode,
            );
        }

        // Verify the in-place normalization matches the reference
        assert_eq!(
            reference_output,
            input_blocks,
            "In-place normalization result doesn't match reference output for alpha_mode={alpha_mode:?}, color_mode={color_mode:?}"
        );
    }

    /// Test normalizing split BC3 blocks in place with all combinations of alpha and color modes
    #[rstest]
    #[case::none_none(AlphaNormalizationMode::None, ColorNormalizationMode::None)]
    #[case::none_color0(AlphaNormalizationMode::None, ColorNormalizationMode::Color0Only)]
    #[case::none_replicate(AlphaNormalizationMode::None, ColorNormalizationMode::ReplicateColor)]
    #[case::uniform_zero_none(
        AlphaNormalizationMode::UniformAlphaZeroIndices,
        ColorNormalizationMode::None
    )]
    #[case::uniform_zero_color0(
        AlphaNormalizationMode::UniformAlphaZeroIndices,
        ColorNormalizationMode::Color0Only
    )]
    #[case::uniform_zero_replicate(
        AlphaNormalizationMode::UniformAlphaZeroIndices,
        ColorNormalizationMode::ReplicateColor
    )]
    #[case::opaque_fill_none(AlphaNormalizationMode::OpaqueFillAll, ColorNormalizationMode::None)]
    #[case::opaque_fill_color0(
        AlphaNormalizationMode::OpaqueFillAll,
        ColorNormalizationMode::Color0Only
    )]
    #[case::opaque_fill_replicate(
        AlphaNormalizationMode::OpaqueFillAll,
        ColorNormalizationMode::ReplicateColor
    )]
    #[case::opaque_zero_none(
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ColorNormalizationMode::None
    )]
    #[case::opaque_zero_color0(
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ColorNormalizationMode::Color0Only
    )]
    #[case::opaque_zero_replicate(
        AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices,
        ColorNormalizationMode::ReplicateColor
    )]
    fn can_normalize_split_blocks_in_place(
        #[case] alpha_mode: AlphaNormalizationMode,
        #[case] color_mode: ColorNormalizationMode,
    ) {
        // Create test data with BC3 blocks - now using the split layout
        // BC3 block is 16 bytes split into 4 sections:
        // - Alpha endpoints: 2 bytes per block
        // - Alpha indices: 6 bytes per block
        // - Color endpoints: 4 bytes per block
        // - Color indices: 4 bytes per block

        // Test with uniform alpha (255) and solid color for 2 blocks
        let mut test_alpha_endpoints = [0u8; 4]; // 2 blocks * 2 bytes each
        let mut test_alpha_indices = [0u8; 12]; // 2 blocks * 6 bytes each
        let mut test_color_endpoints = [0u8; 8]; // 2 blocks * 4 bytes each
        let mut test_color_indices = [0u8; 8]; // 2 blocks * 4 bytes each

        // Set up alpha endpoints for uniform alpha = 255 (fully opaque)
        // Block #0: A0=255, A1=255
        test_alpha_endpoints[0] = 0xFF; // A0
        test_alpha_endpoints[1] = 0xFF; // A1
                                        // Block #1: A0=255, A1=255
        test_alpha_endpoints[2] = 0xFF; // A0
        test_alpha_endpoints[3] = 0xFF; // A1

        // Set up alpha indices (all 0 means use A0)
        test_alpha_indices.fill(0x00);

        // Set up color endpoints for solid red color (0xF800 = bright red in RGB565)
        // Block #0: C0=red, C1=red
        test_color_endpoints[0] = 0x00; // C0 low byte
        test_color_endpoints[1] = 0xF8; // C0 high byte
        test_color_endpoints[2] = 0x00; // C1 low byte
        test_color_endpoints[3] = 0xF8; // C1 high byte
                                        // Block #1: C0=red, C1=red
        test_color_endpoints[4] = 0x00; // C0 low byte
        test_color_endpoints[5] = 0xF8; // C0 high byte
        test_color_endpoints[6] = 0x00; // C1 low byte
        test_color_endpoints[7] = 0xF8; // C1 high byte

        // Set up color indices (all 0 means use C0)
        test_color_indices.fill(0x00);

        // Clone the original data for later comparison when mode is None
        let original_alpha_endpoints = test_alpha_endpoints;
        let original_alpha_indices = test_alpha_indices;
        let original_color_endpoints = test_color_endpoints;
        let original_color_indices = test_color_indices;

        // Get pointers to the test data
        let alpha_endpoints_ptr = test_alpha_endpoints.as_mut_ptr();
        let alpha_indices_ptr = test_alpha_indices.as_mut_ptr();
        let color_endpoints_ptr = test_color_endpoints.as_mut_ptr();
        let color_indices_ptr = test_color_indices.as_mut_ptr();

        // Call normalize_split_blocks_in_place with the test case's modes
        unsafe {
            normalize_split_blocks_in_place(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                color_endpoints_ptr,
                color_indices_ptr,
                2, // 2 blocks
                alpha_mode,
                color_mode,
            );
        }

        // Check normalized data based on the alpha mode
        match alpha_mode {
            AlphaNormalizationMode::None => {
                // No alpha normalization, data should be unchanged
                assert_eq!(test_alpha_endpoints, original_alpha_endpoints);
                assert_eq!(test_alpha_indices, original_alpha_indices);
            }
            AlphaNormalizationMode::UniformAlphaZeroIndices => {
                // Alpha endpoints should be A0=0xFF, A1=0, and indices 0
                for x in 0..2 {
                    assert_eq!(
                        test_alpha_endpoints[x * 2],
                        0xFF,
                        "A0 for block {x} should be 0xFF"
                    );
                    assert_eq!(
                        test_alpha_endpoints[x * 2 + 1],
                        0,
                        "A1 for block {x} should be 0"
                    );
                }
                for x in 0..12 {
                    assert_eq!(test_alpha_indices[x], 0, "Alpha index byte {x} should be 0");
                }
            }
            AlphaNormalizationMode::OpaqueFillAll => {
                // For fully opaque (0xFF), all alpha bytes should be 0xFF
                for x in 0..4 {
                    assert_eq!(
                        test_alpha_endpoints[x], 0xFF,
                        "Alpha endpoint byte {x} should be 0xFF"
                    );
                }
                for x in 0..12 {
                    assert_eq!(
                        test_alpha_indices[x], 0xFF,
                        "Alpha index byte {x} should be 0xFF"
                    );
                }
            }
            AlphaNormalizationMode::OpaqueZeroAlphaMaxIndices => {
                // Alpha endpoints should be 0 and indices 0xFF
                for x in 0..4 {
                    assert_eq!(
                        test_alpha_endpoints[x], 0,
                        "Alpha endpoint byte {x} should be 0"
                    );
                }
                for x in 0..12 {
                    assert_eq!(
                        test_alpha_indices[x], 0xFF,
                        "Alpha index byte {x} should be 0xFF"
                    );
                }
            }
        }

        // Check normalized data based on the color mode
        match color_mode {
            ColorNormalizationMode::None => {
                // No color normalization, data should be unchanged
                assert_eq!(test_color_endpoints, original_color_endpoints);
                assert_eq!(test_color_indices, original_color_indices);
            }
            ColorNormalizationMode::Color0Only => {
                // Block 0: C0=red, C1=0
                assert_eq!(test_color_endpoints[0], 0x00); // C0 low
                assert_eq!(test_color_endpoints[1], 0xF8); // C0 high
                assert_eq!(test_color_endpoints[2], 0x00); // C1 low (should be 0)
                assert_eq!(test_color_endpoints[3], 0x00); // C1 high (should be 0)
                                                           // Block 1: C0=red, C1=0
                assert_eq!(test_color_endpoints[4], 0x00); // C0 low
                assert_eq!(test_color_endpoints[5], 0xF8); // C0 high
                assert_eq!(test_color_endpoints[6], 0x00); // C1 low (should be 0)
                assert_eq!(test_color_endpoints[7], 0x00); // C1 high (should be 0)

                // Color indices should be 0
                for x in 0..8 {
                    assert_eq!(
                        test_color_indices[x], 0x00,
                        "Color index byte {x} should be 0x00"
                    );
                }
            }
            ColorNormalizationMode::ReplicateColor => {
                // Block 0: C0=red, C1=red
                assert_eq!(test_color_endpoints[0], 0x00); // C0 low
                assert_eq!(test_color_endpoints[1], 0xF8); // C0 high
                assert_eq!(test_color_endpoints[2], 0x00); // C1 low (same as C0)
                assert_eq!(test_color_endpoints[3], 0xF8); // C1 high (same as C0)
                                                           // Block 1: C0=red, C1=red
                assert_eq!(test_color_endpoints[4], 0x00); // C0 low
                assert_eq!(test_color_endpoints[5], 0xF8); // C0 high
                assert_eq!(test_color_endpoints[6], 0x00); // C1 low (same as C0)
                assert_eq!(test_color_endpoints[7], 0xF8); // C1 high (same as C0)

                // Color indices should be 0
                for x in 0..8 {
                    assert_eq!(
                        test_color_indices[x], 0x00,
                        "Color index byte {x} should be 0x00"
                    );
                }
            }
        }
    }

    #[test]
    fn can_normalize_split_blocks_in_place_with_replicate_color() {
        // Create test data with BC3 blocks using the split layout
        let mut test_alpha_endpoints = [0u8; 2]; // 1 block * 2 bytes
        let mut test_alpha_indices = [0u8; 6]; // 1 block * 6 bytes
        let mut test_color_endpoints = [0u8; 4]; // 1 block * 4 bytes
        let mut test_color_indices = [0u8; 4]; // 1 block * 4 bytes

        // Set up alpha endpoints for uniform alpha = 128 (half transparent)
        test_alpha_endpoints[0] = 128; // A0
        test_alpha_endpoints[1] = 128; // A1

        // Set up alpha indices (all 0 means use A0)
        test_alpha_indices.fill(0x00);

        // Set up color endpoints for solid red color
        test_color_endpoints[0] = 0x00; // C0 low byte
        test_color_endpoints[1] = 0xF8; // C0 high byte
        test_color_endpoints[2] = 0x00; // C1 low byte
        test_color_endpoints[3] = 0xF8; // C1 high byte

        // Set up color indices with non-zero values
        test_color_indices.fill(0x55);

        // Get pointers to the test data
        let alpha_endpoints_ptr = test_alpha_endpoints.as_mut_ptr();
        let alpha_indices_ptr = test_alpha_indices.as_mut_ptr();
        let color_endpoints_ptr = test_color_endpoints.as_mut_ptr();
        let color_indices_ptr = test_color_indices.as_mut_ptr();

        // Call normalize_split_blocks_in_place with ReplicateColor mode
        unsafe {
            normalize_split_blocks_in_place(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                color_endpoints_ptr,
                color_indices_ptr,
                1, // 1 block
                AlphaNormalizationMode::UniformAlphaZeroIndices,
                ColorNormalizationMode::ReplicateColor,
            );
        }

        // Check that alpha was normalized (A0 = 128, A1 should be 0, indices should be 0)
        assert_eq!(test_alpha_endpoints[0], 128); // A0 should be the alpha value
        assert_eq!(test_alpha_endpoints[1], 0); // A1 should be 0
        for x in 0..6 {
            assert_eq!(test_alpha_indices[x], 0, "Alpha index byte {x} should be 0",);
        }

        // Check that color was normalized with ReplicateColor (both C0 and C1 = red, indices = 0)
        assert_eq!(test_color_endpoints[0], 0x00); // C0 low
        assert_eq!(test_color_endpoints[1], 0xF8); // C0 high
        assert_eq!(test_color_endpoints[2], 0x00); // C1 low (replicated)
        assert_eq!(test_color_endpoints[3], 0xF8); // C1 high (replicated)
        for x in 0..4 {
            assert_eq!(
                test_color_indices[x], 0x00,
                "Color index byte {x} should be 0x00",
            );
        }
    }

    #[test]
    fn can_normalize_split_blocks_no_op_when_modes_are_none() {
        // Create test data using the split layout
        let mut test_alpha_endpoints = [0xAA; 2]; // Fill with pattern
        let mut test_alpha_indices = [0xBB; 6]; // Fill with different pattern
        let mut test_color_endpoints = [0xCC; 4]; // Fill with different pattern
        let mut test_color_indices = [0xDD; 4]; // Fill with different pattern

        // Store original data for comparison
        let original_alpha_endpoints = test_alpha_endpoints;
        let original_alpha_indices = test_alpha_indices;
        let original_color_endpoints = test_color_endpoints;
        let original_color_indices = test_color_indices;

        // Get pointers to the test data
        let alpha_endpoints_ptr = test_alpha_endpoints.as_mut_ptr();
        let alpha_indices_ptr = test_alpha_indices.as_mut_ptr();
        let color_endpoints_ptr = test_color_endpoints.as_mut_ptr();
        let color_indices_ptr = test_color_indices.as_mut_ptr();

        // Call normalize_split_blocks_in_place with None modes
        unsafe {
            normalize_split_blocks_in_place(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                color_endpoints_ptr,
                color_indices_ptr,
                1, // 1 block
                AlphaNormalizationMode::None,
                ColorNormalizationMode::None,
            );
        }

        // Check that data was not modified
        assert_eq!(test_alpha_endpoints, original_alpha_endpoints);
        assert_eq!(test_alpha_indices, original_alpha_indices);
        assert_eq!(test_color_endpoints, original_color_endpoints);
        assert_eq!(test_color_indices, original_color_indices);
    }
}
