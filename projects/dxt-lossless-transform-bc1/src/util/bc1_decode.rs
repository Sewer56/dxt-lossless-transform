//! BC1 (DXT1) decoding implementation; based on etcpak
//! https://github.com/wolfpld/etcpak and MSDN
//! https://learn.microsoft.com/en-us/windows/win32/direct3d9/opaque-and-1-bit-alpha-textures

/// Represents a single RGBA pixel color from a decoded BC1 block
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
}

impl Pixel {
    /// Constructs a new `Pixel` from the specified red, green, blue, and alpha components.
    ///
    /// Each parameter represents the intensity of its corresponding colour channel (0–255).
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1::util::Pixel;
    ///
    /// let pixel = Pixel::new(255, 0, 0, 255);
    /// assert_eq!(pixel.r, 255);
    /// assert_eq!(pixel.g, 0);
    /// assert_eq!(pixel.b, 0);
    /// assert_eq!(pixel.a, 255);
    /// ```
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Represents a decoded 4x4 block of BC1 pixels
#[derive(Debug, Clone, Copy)]
pub struct DecodedBc1Block {
    /// The 16 pixels in the block (row-major order)
    pub pixels: [Pixel; 16],
}

impl DecodedBc1Block {
    /// Constructs a new decoded BC1 block initialised with 16 copies of the provided pixel.
    /// This function creates a 4x4 block where every pixel is set to the specified value.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1::util::{DecodedBc1Block, Pixel};
    ///
    /// let pixel = Pixel::new(255, 0, 0, 255);
    /// let block = DecodedBc1Block::new(pixel);
    /// assert!(block.pixels.iter().all(|&p| p == pixel));
    pub fn new(pixel: Pixel) -> Self {
        Self {
            pixels: [pixel; 16],
        }
    }

    /// Gets a pixel at the specified coordinates (0-3, 0-3) without bounds checking
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < 4` and `y < 4`.
    #[inline]
    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Pixel {
        *self.pixels.get_unchecked(y * 4 + x)
    }

    /// Sets a pixel at the specified coordinates (0-3, 0-3) without bounds checking
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < 4` and `y < 4`.
    #[inline]
    pub unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, pixel: Pixel) {
        *self.pixels.get_unchecked_mut(y * 4 + x) = pixel;
    }
}

/// Decodes a BC1 block into a structured representation of pixels
///
/// # Parameters
///
/// - `src`: Pointer to the source BC1 block (must point to at least 8 bytes of valid memory)
///
/// # Returns
///
/// A [`DecodedBc1Block`] containing all 16 decoded pixels
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
pub unsafe fn decode_bc1_block(src: *const u8) -> DecodedBc1Block {
    // Extract color endpoints and index data
    let c0: u16 = u16::from_le_bytes([*src, *src.add(1)]);
    let c1: u16 = u16::from_le_bytes([*src.add(2), *src.add(3)]);
    let idx: u32 = u32::from_le_bytes([*src.add(4), *src.add(5), *src.add(6), *src.add(7)]);

    // Extract RGB components for the first color
    let r0 = (((c0 & 0xF800) >> 8) | ((c0 & 0xF800) >> 13)) as u8;
    let g0 = (((c0 & 0x07E0) >> 3) | ((c0 & 0x07E0) >> 9)) as u8;
    let b0 = (((c0 & 0x001F) << 3) | ((c0 & 0x001F) >> 2)) as u8;

    // Extract RGB components for the second color
    let r1 = (((c1 & 0xF800) >> 8) | ((c1 & 0xF800) >> 13)) as u8;
    let g1 = (((c1 & 0x07E0) >> 3) | ((c1 & 0x07E0) >> 9)) as u8;
    let b1 = (((c1 & 0x001F) << 3) | ((c1 & 0x001F) >> 2)) as u8;

    // Create color dictionary - no bounds checks needed for fixed-size array
    let mut dict = [Pixel::new(0, 0, 0, 0); 4];
    dict[0] = Pixel::new(r0, g0, b0, 255);
    dict[1] = Pixel::new(r1, g1, b1, 255);

    // Calculate the additional colors based on whether c0 > c1
    if c0 > c1 {
        // Four-color block
        let r = ((2 * r0 as u32) + r1 as u32) / 3;
        let g = ((2 * g0 as u32) + g1 as u32) / 3;
        let b = ((2 * b0 as u32) + b1 as u32) / 3;
        dict[2] = Pixel::new(r as u8, g as u8, b as u8, 255);

        let r = (r0 as u32 + 2 * r1 as u32) / 3;
        let g = (g0 as u32 + 2 * g1 as u32) / 3;
        let b = (b0 as u32 + 2 * b1 as u32) / 3;
        dict[3] = Pixel::new(r as u8, g as u8, b as u8, 255);
    } else {
        // Three-color block, 1 bit alpha.
        let r = (r0 as u32 + r1 as u32) / 2;
        let g = (g0 as u32 + g1 as u32) / 2;
        let b = (b0 as u32 + b1 as u32) / 2;
        dict[2] = Pixel::new(r as u8, g as u8, b as u8, 255);
        dict[3] = Pixel::new(0, 0, 0, 0); // Transparent black
    }

    // Initialize the result block
    let mut result = DecodedBc1Block::new(Pixel::new(0, 0, 0, 0));

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
pub fn decode_bc1_block_from_slice(src: &[u8]) -> Option<DecodedBc1Block> {
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
                assert_eq!(pixel, Pixel::new(255, 0, 0, 255));
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
                assert_eq!(pixel, Pixel::new(0, 0, 0, 0));
            }
        }
    }
}
