//! BC2 (DXT2/DXT3) decoding implementation; based on etcpak
//! <https://github.com/wolfpld/etcpak> and MSDN
//! <https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression#bc2>
//!
//! Uses the 'ideal' rounding/computing method described in the DX9 docs, as opposed to DX10, AMD or Nvidia
//! method.

use core::slice;

use dxt_lossless_transform_common::{
    color_565::Color565, color_8888::Color8888, decoded_4x4_block::Decoded4x4Block,
};

/// Decodes a BC2 block into a structured representation of pixels
///
/// # Parameters
///
/// - `src`: Pointer to the source BC2 block (must point to at least 16 bytes of valid memory)
///
/// # Returns
///
/// A [`Decoded4x4Block`] containing all 16 decoded pixels with alpha
///
/// # Safety
///
/// The caller must ensure that `src` points to at least 16 bytes of valid memory.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_bc2::util::decode_bc2_block;
///
/// let bc2_block = [0u8; 16]; // Compressed BC2 block
///
/// // Decode the BC2 block into a structured representation
/// unsafe {
///     let decoded = decode_bc2_block(bc2_block.as_ptr());
///     
///     // Access individual pixels
///     let pixel_at_0_0 = decoded.get_pixel_unchecked(0, 0);
/// }
/// ```
#[inline(always)]
pub unsafe fn decode_bc2_block(src: *const u8) -> Decoded4x4Block {
    // Last 8 bytes contain the color data (same format as BC1)
    let color_src = src.add(8);

    // Extract color endpoints and index data
    let c0_raw: u16 = u16::from_le_bytes([*color_src, *color_src.add(1)]);
    let c1_raw: u16 = u16::from_le_bytes([*color_src.add(2), *color_src.add(3)]);
    let idx: u32 = u32::from_le_bytes([
        *color_src.add(4),
        *color_src.add(5),
        *color_src.add(6),
        *color_src.add(7),
    ]);

    // Create Color565 wrappers
    let c0 = Color565::from_raw(c0_raw);
    let c1 = Color565::from_raw(c1_raw);

    // Extract RGB components for the colors
    let r0 = c0.red();
    let g0 = c0.green();
    let b0 = c0.blue();

    let r1 = c1.red();
    let g1 = c1.green();
    let b1 = c1.blue();

    // Create color dictionary - no bounds checks needed for fixed-size array
    // BC2 always uses the 4-color mode (no transparency from color section)
    let mut dict = [Color8888::new(0, 0, 0, 0); 4];
    dict[0] = Color8888::new(r0, g0, b0, 255);
    dict[1] = Color8888::new(r1, g1, b1, 255);

    // Four-color block (BC2 always uses 4-color mode regardless of c0/c1 comparison)
    let r = ((2 * r0 as u32) + r1 as u32) / 3;
    let g = ((2 * g0 as u32) + g1 as u32) / 3;
    let b = ((2 * b0 as u32) + b1 as u32) / 3;
    dict[2] = Color8888::new(r as u8, g as u8, b as u8, 255);

    let r = (r0 as u32 + 2 * r1 as u32) / 3;
    let g = (g0 as u32 + 2 * g1 as u32) / 3;
    let b = (b0 as u32 + 2 * b1 as u32) / 3;
    dict[3] = Color8888::new(r as u8, g as u8, b as u8, 255);

    // Initialize the result block
    let mut result = Decoded4x4Block::new(Color8888::new(0, 0, 0, 0));

    // First 8 bytes contain the explicit alpha values (4 bits per pixel)
    let alpha_bytes = slice::from_raw_parts(src, 8);

    // Decode indices and set pixels with explicit alpha
    let mut index_pos = 0;
    let mut alpha_bit_pos = 0;

    // Compiler unrolls this!
    for y in 0..4 {
        for x in 0..4 {
            // Get color index and fetch color
            let pixel_idx = (idx >> index_pos) & 0x3;
            let mut pixel = *dict.get_unchecked(pixel_idx as usize);

            // Get 4-bit alpha value (0-15)
            // Branchless approach: multiply shift by (alpha_bit_pos & 0x1) * 4 which gives 0 or 4
            let shift_amount = (alpha_bit_pos & 0x1) * 4;
            let alpha_value = (alpha_bytes[alpha_bit_pos >> 1] >> shift_amount) & 0x0F;

            // Scale 4-bit alpha (0-15) to 8-bit (0-255): multiply by 17 (255/15 ≈ 17)
            pixel.a = alpha_value * 17;

            // Set pixel with color and alpha
            result.set_pixel_unchecked(x, y, pixel);

            index_pos += 2;
            alpha_bit_pos += 1;
        }
    }

    result
}

/// Safely wraps the unsafe [`decode_bc2_block`] function for use with slices
///
/// # Returns
///
/// A decoded block, else [`None`] if the slice is too short.
#[inline(always)]
pub fn decode_bc2_block_from_slice(src: &[u8]) -> Option<Decoded4x4Block> {
    if src.len() < 16 {
        return None;
    }
    unsafe { Some(decode_bc2_block(src.as_ptr())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // There is also a fuzz test against a good known implementation in bc7enc, so this is minimal.

    #[test]
    fn can_decode_bc2_block() {
        // Test case: Simple red color with varying alpha
        let bc2_block = [
            // Alpha data (4 bits per pixel): 0x0 to 0xF across the 16 pixels
            0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE,
            // Color data (identical to BC1)
            0x00, 0xF8, // c0 = R:31 G:0 B:0
            0x00, 0xF8, // c1 = R:31 G:0 B:0 (identical for solid color)
            0x00, 0x00, 0x00, 0x00, // All pixels use index 0
        ];

        let decoded = decode_bc2_block_from_slice(&bc2_block).unwrap();

        // Expected alpha values after scaling from 4-bit to 8-bit
        let expected_alphas = [
            0, 17, 34, 51, 68, 85, 102, 119, 136, 153, 170, 187, 204, 221, 238, 255,
        ];

        // All pixels should be red with varying alpha
        let mut pixel_idx = 0;
        for y in 0..4 {
            for x in 0..4 {
                let pixel = unsafe { decoded.get_pixel_unchecked(x, y) };
                assert_eq!(pixel.r, 255, "Red component should be 255");
                assert_eq!(pixel.g, 0, "Green component should be 0");
                assert_eq!(pixel.b, 0, "Blue component should be 0");
                assert_eq!(
                    pixel.a, expected_alphas[pixel_idx],
                    "Alpha value incorrect at pixel ({x}, {y})",
                );
                pixel_idx += 1;
            }
        }
    }

    #[test]
    fn can_decode_bc2_block_with_zero_alpha() {
        // Test case: Zero alpha with color data
        let bc2_block = [
            // Alpha data: all zeros (fully transparent)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Color data
            0x00, 0xF8, // c0 = R:31 G:0 B:0 (Red)
            0x00, 0xF8, // c1 = R:31 G:0 B:0 (Red)
            0x00, 0x00, 0x00, 0x00, // All pixels use index 0
        ];

        let decoded = decode_bc2_block_from_slice(&bc2_block).unwrap();

        // All pixels should have zero alpha but red color
        for y in 0..4 {
            for x in 0..4 {
                let pixel = unsafe { decoded.get_pixel_unchecked(x, y) };
                assert_eq!(pixel.r, 255, "Red component should be 255");
                assert_eq!(pixel.g, 0, "Green component should be 0");
                assert_eq!(pixel.b, 0, "Blue component should be 0");
                assert_eq!(pixel.a, 0, "Alpha should be 0 (transparent)");
            }
        }
    }

    #[test]
    fn test_slice_too_small() {
        // Test with a slice that's too small
        let bc2_block = [0u8; 15]; // BC2 requires 16 bytes
        let result = decode_bc2_block_from_slice(&bc2_block);
        assert!(
            result.is_none(),
            "Decoding should fail with too small slice"
        );
    }
}
