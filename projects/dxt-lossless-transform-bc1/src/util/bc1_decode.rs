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
    /// let pixel = Pixel::new(255, 0, 0, 255);
    /// assert_eq!(pixel.r, 255);
    /// assert_eq!(pixel.g, 0);
    /// assert_eq!(pixel.b, 0);
    /// assert_eq!(pixel.a, 255);
    /// ```    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
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
    ///
    /// This function creates a 4x4 block where every pixel is set to the specified value.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::{DecodedBc1Block, Pixel};
    ///
    /// let pixel = Pixel { r: 255, g: 0, b: 0, a: 255 };
    /// let block = DecodedBc1Block::new(pixel);
    /// assert!(block.pixels.iter().all(|&p| p == pixel));
    /// ```    pub fn new(pixel: Pixel) -> Self {
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
    /// Retrieves a pixel at the specified (x, y) coordinate without performing bounds checking.
    ///
    /// # Safety
    ///
    /// Calling this function with coordinates outside the valid range (0 <= x < 4 and 0 <= y < 4)
    /// results in undefined behaviour. The caller must ensure the indices are valid for a 4x4 pixel block.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::bc1_decode::{Pixel, DecodedBc1Block};
    ///
    /// let red = Pixel::new(255, 0, 0, 255);
    /// let block = DecodedBc1Block::new(red);
    ///
    /// unsafe {
    ///     let pixel = block.get_pixel_unchecked(0, 0);
    ///     assert_eq!(pixel, red);
    /// }
    /// ```
    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Pixel {
        *self.pixels.get_unchecked(y * 4 + x)
    }

    /// Sets a pixel at the specified coordinates (0-3, 0-3) without bounds checking
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < 4` and `y < 4`.
    #[inline]
    /// Sets the pixel at the specified `(x, y)` coordinate in a 4x4 block without bounds checking.
    ///
    /// # Safety
    ///
    /// This function does not check that the coordinates `(x, y)` are within the valid range (0 to 3).
    /// The caller must ensure that `x < 4` and `y < 4` to prevent undefined behaviour.
    ///
    /// # Examples
    ///
    /// ```
    /// # use crate::bc1_decode::{DecodedBc1Block, Pixel};
    /// let mut block = DecodedBc1Block::new(Pixel::new(0, 0, 0, 255));
    /// unsafe {
    ///     block.set_pixel_unchecked(1, 2, Pixel::new(255, 0, 0, 255));
    /// }
    /// ```
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
/// A `DecodedBc1Block` containing all 16 decoded pixels
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
#[inline]
/// Decodes a BC1 (DXT1) compressed block into a 4x4 pixel array.
/// 
/// This function reads 8 bytes from the raw pointer `src`, where the first four bytes encode two colour endpoints
/// and the next four bytes encode pixel indices. It then interpolates additional colours based on the BC1 specification,
/// using either a four-colour or a three-colour (with a transparent pixel) mode depending on the ordering of the endpoints.
/// The resulting 4x4 grid of pixels is returned as a `DecodedBc1Block`.
/// 
/// # Safety
/// 
/// This function is unsafe because it dereferences a raw pointer. The caller must ensure that `src` points to at least
/// 8 contiguous bytes of valid memory.
/// 
/// # Examples
/// 
/// ```
/// # use crate::bc1_decode::{decode_bc1_block, DecodedBc1Block, Pixel};
/// // Sample 8-byte BC1 block data (the values here are for demonstration purposes).
/// let data: [u8; 8] = [0x7C, 0x1F, 0x00, 0x00, 0b11001100, 0, 0, 0];
/// let block = unsafe { decode_bc1_block(data.as_ptr()) };
/// 
/// // Verify that the decoded block contains 16 pixels forming a 4x4 grid.
/// assert_eq!(block.pixels.len(), 16);
/// ```
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

/// Safely wraps the unsafe decode_bc1_block function for use with slices
///
/// # Panics
///
/// Decodes a BC1 block from a byte slice.
///
/// This function serves as a safe wrapper around the unsafe `decode_bc1_block` function by
/// interpreting the first 8 bytes of the input slice as a BC1 compressed block and decoding it
/// into a 4x4 block of RGBA pixels.
///
/// # Panics
///
/// Panics if the input slice contains fewer than 8 bytes.
///
/// # Examples
///
/// ```
/// use your_crate::decode_bc1_block_from_slice;
/// use your_crate::DecodedBc1Block;
///
/// // Example BC1 block data (8 bytes). Replace with actual compressed data as needed.
/// let bc1_data: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
/// let block: DecodedBc1Block = decode_bc1_block_from_slice(&bc1_data);
///
/// // Access a pixel from the decoded block (coordinates must be valid).
/// let pixel = block.get_pixel_unchecked(0, 0);
/// // Use the pixel as needed...
/// ```pub fn decode_bc1_block_from_slice(src: &[u8]) -> DecodedBc1Block {
    debug_assert!(src.len() >= 8, "BC1 block must be at least 8 bytes");
    unsafe { decode_bc1_block(src.as_ptr()) }
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

        let decoded = decode_bc1_block_from_slice(&bc1_block);

        // All pixels should be red
        for y in 0..4 {
            for x in 0..4 {
                let pixel = unsafe { decoded.get_pixel_unchecked(x, y) };
                assert_eq!(pixel, Pixel::new(255, 0, 0, 255));
            }
        }
    }
}
