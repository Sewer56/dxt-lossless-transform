//! BC1 (DXT1) decoding implementation; based on etcpak
//! <https://github.com/wolfpld/etcpak> and MSDN
//! <https://learn.microsoft.com/en-us/windows/win32/direct3d9/opaque-and-1-bit-alpha-textures>
//!
//! Uses the 'ideal' rounding/computing method described in the DX9 docs, as opposed to DX10, AMD or Nvidia
//! method.

use dxt_lossless_transform_common::{
    color_565::Color565, color_8888::Color8888, decoded_4x4_block::Decoded4x4Block,
};

/// Decodes a BC1 block into a structured representation of pixels
///
/// # Parameters
///
/// - `src`: Pointer to the source BC1 block (must point to at least 8 bytes of valid memory)
///
/// # Returns
///
/// A [`Decoded4x4Block`] containing all 16 decoded pixels
///
/// # Safety
///
/// The caller must ensure that `src` points to at least 8 bytes of valid memory.
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_bc1::util::decode_bc1_block;
///
/// let bc1_block = [0u8; 8]; // Compressed BC1 block
///
/// // Decode the BC1 block into a structured representation
/// unsafe {
///     let decoded = decode_bc1_block(bc1_block.as_ptr());
///     
///     // Access individual pixels
///     let pixel_at_0_0 = decoded.get_pixel_unchecked(0, 0);
/// }
/// ```
#[inline(always)]
pub unsafe fn decode_bc1_block(src: *const u8) -> Decoded4x4Block {
    // Extract color endpoints and index data
    let c0_raw: u16 = u16::from_le_bytes([*src, *src.add(1)]);
    let c1_raw: u16 = u16::from_le_bytes([*src.add(2), *src.add(3)]);
    let idx: u32 = u32::from_le_bytes([*src.add(4), *src.add(5), *src.add(6), *src.add(7)]);

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
    let mut dict = [Color8888::new(0, 0, 0, 0); 4];
    dict[0] = Color8888::new(r0, g0, b0, 255);
    dict[1] = Color8888::new(r1, g1, b1, 255);

    // Calculate the additional colors based on whether c0 > c1
    if c0.greater_than(&c1) {
        // Four-color block
        let r = ((2 * r0 as u32) + r1 as u32) / 3;
        let g = ((2 * g0 as u32) + g1 as u32) / 3;
        let b = ((2 * b0 as u32) + b1 as u32) / 3;
        dict[2] = Color8888::new(r as u8, g as u8, b as u8, 255);

        let r = (r0 as u32 + 2 * r1 as u32) / 3;
        let g = (g0 as u32 + 2 * g1 as u32) / 3;
        let b = (b0 as u32 + 2 * b1 as u32) / 3;
        dict[3] = Color8888::new(r as u8, g as u8, b as u8, 255);
    } else {
        // Three-color block, 1 bit alpha.
        let r = (r0 as u32 + r1 as u32) / 2;
        let g = (g0 as u32 + g1 as u32) / 2;
        let b = (b0 as u32 + b1 as u32) / 2;
        dict[2] = Color8888::new(r as u8, g as u8, b as u8, 255);
        dict[3] = Color8888::new(0, 0, 0, 0); // Transparent black
    }

    // Initialize the result block
    let mut result = Decoded4x4Block::new(Color8888::new(0, 0, 0, 0));

    // Decode indices and set pixels
    let mut index_pos = 0;
    for y in 0..4 {
        for x in 0..4 {
            let pixel_idx = (idx >> index_pos) & 0x3;
            result.set_pixel_unchecked(x, y, *dict.get_unchecked(pixel_idx as usize));
            index_pos += 2;
        }
    }

    result
}

/// Safely wraps the unsafe [`decode_bc1_block`] function for use with slices
///
/// # Returns
///
/// A decoded block, else [`None`] if the slice is too short.
#[inline(always)]
pub fn decode_bc1_block_from_slice(src: &[u8]) -> Option<Decoded4x4Block> {
    if src.len() < 8 {
        return None;
    }
    unsafe { Some(decode_bc1_block(src.as_ptr())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_decode_bc1_block() {
        // Test case: Simple red color
        let bc1_block = [
            0x00, 0xF8, // c0 = R:31 G:0 B:0
            0x00, 0xF8, // c1 = R:31 G:0 B:0 (identical to create solid color)
            0x00, 0x00, 0x00, 0x00, // All pixels use index 0
        ];

        let decoded = decode_bc1_block_from_slice(&bc1_block).unwrap();

        // All pixels should be red
        for y in 0..4 {
            for x in 0..4 {
                let pixel = unsafe { decoded.get_pixel_unchecked(x, y) };
                assert_eq!(pixel, Color8888::new(255, 0, 0, 255));
            }
        }
    }

    #[test]
    fn can_decode_bc1_block_with_transparency() {
        // Test case with transparency (c1 > c0 for 3-color mode)
        let bc1_block = [
            0x00, 0xF0, // c0 = R:30 G:0 B:0 (intentionally less than c1)
            0x00, 0xF8, // c1 = R:31 G:0 B:0
            0xFF, 0xFF, 0xFF, 0xFF, // All pixels use index 3 (transparent)
        ];

        let decoded = decode_bc1_block_from_slice(&bc1_block).unwrap();

        // All pixels should be transparent, because color1 is greater than color0
        for y in 0..4 {
            for x in 0..4 {
                let pixel = unsafe { decoded.get_pixel_unchecked(x, y) };
                assert_eq!(pixel, Color8888::new(0, 0, 0, 0));
            }
        }
    }

    #[test]
    fn has_identical_pixels() {
        // Create a block where all pixels are the same
        let identical_block = Decoded4x4Block::new(Color8888::new(100, 150, 200, 255));

        // Create a block where one pixel is different
        let mut different_block = Decoded4x4Block::new(Color8888::new(100, 150, 200, 255));
        different_block.pixels[10] = Color8888::new(101, 150, 200, 255);

        // Test the function
        assert!(
            identical_block.has_identical_pixels(),
            "Block with identical pixels should return true"
        );
        assert!(
            !different_block.has_identical_pixels(),
            "Block with different pixels should return false"
        );
    }
}
