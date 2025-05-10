//! BC3 (DXT4/DXT5) decoding implementation; based on etcpak
//! <https://github.com/wolfpld/etcpak> and MSDN
//! <https://learn.microsoft.com/en-us/windows/win32/direct3d10/d3d10-graphics-programming-guide-resources-block-compression#bc3>
//!
//! Uses the 'ideal' rounding/computing method described in the DX9 docs, as opposed to DX10, AMD or Nvidia
//! method.

use dxt_lossless_transform_common::{
    color_565::Color565, color_8888::Color8888, decoded_4x4_block::Decoded4x4Block,
};

/// Decodes a BC3 block into a structured representation of pixels
///
/// # Parameters
///
/// - `src`: Pointer to the source BC3 block (must point to at least 16 bytes of valid memory)
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
/// use dxt_lossless_transform_bc3::util::decode_bc3_block;
///
/// let bc3_block = [0u8; 16]; // Compressed BC3 block
///
/// // Decode the BC3 block into a structured representation
/// unsafe {
///     let decoded = decode_bc3_block(bc3_block.as_ptr());
///     
///     // Access individual pixels
///     let pixel_at_0_0 = decoded.get_pixel_unchecked(0, 0);
/// }
/// ```
#[inline(always)]
#[allow(clippy::identity_op)]
pub unsafe fn decode_bc3_block(src: *const u8) -> Decoded4x4Block {
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
    // BC3 always uses the 4-color mode (no transparency from color section)
    let mut dict = [Color8888::new(0, 0, 0, 0); 4];
    dict[0] = Color8888::new(r0, g0, b0, 255);
    dict[1] = Color8888::new(r1, g1, b1, 255);

    // Four-color block (BC3 always uses 4-color mode regardless of c0/c1 comparison)
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

    // First 8 bytes contain the BC4 compressed alpha data
    let alpha_src = src;

    // Extract alpha endpoints
    let alpha0 = *alpha_src;
    let alpha1 = *alpha_src.add(1);

    // Create alpha lookup table
    let mut alpha_values = [0u8; 8];
    alpha_values[0] = alpha0;
    alpha_values[1] = alpha1;

    // In BC4/BC3, if alpha0 > alpha1, we have 8 interpolated values
    // Otherwise we have 6 interpolated values plus transparent and opaque
    if alpha0 > alpha1 {
        // 8 interpolated alpha values
        alpha_values[2] = ((6 * alpha0 as u16 + 1 * alpha1 as u16) / 7) as u8; // bit code 010
        alpha_values[3] = ((5 * alpha0 as u16 + 2 * alpha1 as u16) / 7) as u8; // bit code 011
        alpha_values[4] = ((4 * alpha0 as u16 + 3 * alpha1 as u16) / 7) as u8; // bit code 100
        alpha_values[5] = ((3 * alpha0 as u16 + 4 * alpha1 as u16) / 7) as u8; // bit code 101
        alpha_values[6] = ((2 * alpha0 as u16 + 5 * alpha1 as u16) / 7) as u8; // bit code 110
        alpha_values[7] = ((1 * alpha0 as u16 + 6 * alpha1 as u16) / 7) as u8; // bit code 111
    } else {
        // 6 interpolated alpha values + transparent and opaque
        alpha_values[2] = ((4 * alpha0 as u16 + 1 * alpha1 as u16) / 5) as u8; // bit code 010
        alpha_values[3] = ((3 * alpha0 as u16 + 2 * alpha1 as u16) / 5) as u8; // bit code 011
        alpha_values[4] = ((2 * alpha0 as u16 + 3 * alpha1 as u16) / 5) as u8; // bit code 100
        alpha_values[5] = ((1 * alpha0 as u16 + 4 * alpha1 as u16) / 5) as u8; // bit code 101
        alpha_values[6] = 0; // Transparent (bit code 110)
        alpha_values[7] = 255; // Opaque (bit code 111)
    }

    // Extract alpha indices (3 bits per index, 48 bits total for 16 pixels)
    // Read 6 bytes of indices (starting from byte 2)
    let alpha_indices = [
        *alpha_src.add(2),
        *alpha_src.add(3),
        *alpha_src.add(4),
        *alpha_src.add(5),
        *alpha_src.add(6),
        *alpha_src.add(7),
    ];

    // Decode color indices and set pixels with alpha from BC4 compression
    let mut index_pos = 0;
    let mut alpha_bit_pos = 0;

    // Compiler unrolls this!
    // And also undoes the `if` branches
    for y in 0..4 {
        for x in 0..4 {
            // Get color index and fetch color
            let pixel_idx = (idx >> index_pos) & 0x3;
            let mut pixel = *dict.get_unchecked(pixel_idx as usize);

            // Get alpha index (3 bits) and corresponding alpha value
            // Calculate byte position and bit position within the byte
            let byte_pos = alpha_bit_pos / 8;
            let bit_shift = alpha_bit_pos % 8;

            let alpha_idx = if bit_shift <= 5 {
                // Index contained within one byte
                (alpha_indices[byte_pos] >> bit_shift) & 0b111
            } else {
                // Index spans two bytes (part from current byte, part from next byte)
                let bits_from_current = alpha_indices[byte_pos] >> bit_shift;
                let bits_from_next = alpha_indices[byte_pos + 1] << (8 - bit_shift);
                (bits_from_current | bits_from_next) & 0b111
            };

            // Set alpha value
            pixel.a = alpha_values[alpha_idx as usize];

            // Set pixel with color and alpha
            result.set_pixel_unchecked(x, y, pixel);

            // Move to next indices
            index_pos += 2;
            alpha_bit_pos += 3; // BC3/BC4 uses 3 bits per alpha index
        }
    }

    result
}

/// Safely wraps the unsafe [`decode_bc3_block`] function for use with slices
///
/// # Returns
///
/// A decoded block, else [`None`] if the slice is too short.
#[inline(always)]
pub fn decode_bc3_block_from_slice(src: &[u8]) -> Option<Decoded4x4Block> {
    if src.len() < 16 {
        return None;
    }
    unsafe { Some(decode_bc3_block(src.as_ptr())) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // There is also a fuzz test against a good known implementation in bc7enc, so this is minimal.
}
