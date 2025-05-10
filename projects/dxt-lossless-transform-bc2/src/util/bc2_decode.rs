//! BC2 (DXT2/DXT3) decoding implementation; based on etcpak
//! <https://github.com/wolfpld/etcpak> and MSDN
//! <https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression#bc2>

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
