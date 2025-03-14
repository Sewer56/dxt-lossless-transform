//! BC1 (DXT1) decoding implementation; based on etcpak
//! https://github.com/wolfpld/etcpak and MSDN
//! https://learn.microsoft.com/en-us/windows/win32/direct3d9/opaque-and-1-bit-alpha-textures

/// Decodes a BC1 block into RGBA8 pixels
///
/// # Parameters
///
/// - `src`: The source BC1 block (8 bytes)
/// - `dst`: Destination buffer for the decoded pixels (4x4 RGBA pixels = 64 bytes)
/// - `stride`: Number of pixels in a row of the destination image
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_bc1::util::decode_bc1_block;
///
/// let mut pixels = [0u32; 16]; // 4x4 block of pixels
/// let bc1_block = [0u8; 8]; // Compressed BC1 block
///
/// // Decode the BC1 block into RGBA pixels
/// decode_bc1_block(&bc1_block, &mut pixels, 4);
/// ```
#[inline]
pub fn decode_bc1_block(src: &[u8], dst: &mut [u32], stride: usize) {
    // Extract color endpoints and index data
    let c0: u16 = u16::from_le_bytes([src[0], src[1]]);
    let c1: u16 = u16::from_le_bytes([src[2], src[3]]);
    let idx: u32 = u32::from_le_bytes([src[4], src[5], src[6], src[7]]);

    // Extract RGB components for the first color
    let r0 = (((c0 & 0xF800) >> 8) | ((c0 & 0xF800) >> 13)) as u8;
    let g0 = (((c0 & 0x07E0) >> 3) | ((c0 & 0x07E0) >> 9)) as u8;
    let b0 = (((c0 & 0x001F) << 3) | ((c0 & 0x001F) >> 2)) as u8;

    // Extract RGB components for the second color
    let r1 = (((c1 & 0xF800) >> 8) | ((c1 & 0xF800) >> 13)) as u8;
    let g1 = (((c1 & 0x07E0) >> 3) | ((c1 & 0x07E0) >> 9)) as u8;
    let b1 = (((c1 & 0x001F) << 3) | ((c1 & 0x001F) >> 2)) as u8;

    // Create color dictionary
    let mut dict = [0u32; 4];
    dict[0] = 0xFF000000 | (b0 as u32) << 16 | (g0 as u32) << 8 | r0 as u32;
    dict[1] = 0xFF000000 | (b1 as u32) << 16 | (g1 as u32) << 8 | r1 as u32;

    // Calculate the additional colors based on whether c0 > c1
    if c0 > c1 {
        // Four-color block
        let r = (2 * r0 as u32 + r1 as u32) / 3;
        let g = (2 * g0 as u32 + g1 as u32) / 3;
        let b = (2 * b0 as u32 + b1 as u32) / 3;
        dict[2] = 0xFF000000 | (b << 16) | (g << 8) | r;

        let r = (r0 as u32 + 2 * r1 as u32) / 3;
        let g = (g0 as u32 + 2 * g1 as u32) / 3;
        let b = (b0 as u32 + 2 * b1 as u32) / 3;
        dict[3] = 0xFF000000 | (b << 16) | (g << 8) | r;
    } else {
        // Three-color block
        let r = (r0 as u32 + r1 as u32) / 2;
        let g = (g0 as u32 + g1 as u32) / 2;
        let b = (b0 as u32 + b1 as u32) / 2;
        dict[2] = 0xFF000000 | (b << 16) | (g << 8) | r;
        dict[3] = 0xFF000000; // Transparent black
    }

    // Decode indices and write to destination buffer
    // First row
    let mut index_pos = 0;
    for y in 0..4 {
        for x in 0..4 {
            let pixel_idx = (idx >> index_pos) & 0x3;
            dst[y * stride + x] = dict[pixel_idx as usize];
            index_pos += 2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_bc1_block() {
        // Test case: Simple red color
        let bc1_block = [
            0x00, 0xF8, // c0 = R:31 G:0 B:0
            0x00, 0xF8, // c1 = R:31 G:0 B:0 (identical to create solid color)
            0x00, 0x00, 0x00, 0x00, // All pixels use index 0
        ];

        let mut pixels = [0u32; 16];
        decode_bc1_block(&bc1_block, &mut pixels, 4);

        // All pixels should be red (0xFFFF0000 in RGBA8 format)
        for pixel in pixels.iter() {
            assert_eq!(*pixel, 0xFF0000FF); // RGBA format with red (FF) in the lowest byte
        }
    }
}
