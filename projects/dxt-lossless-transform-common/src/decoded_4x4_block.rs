use crate::color_8888::Color8888;
use core::mem::transmute;

/// Represents a decoded 4x4 block of BC pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decoded4x4Block {
    /// The 16 pixels in the block (row-major order)
    pub pixels: [Color8888; 16],
}

impl Decoded4x4Block {
    /// Constructs a new decoded block initialised with 16 copies of the provided pixel.
    /// This function creates a 4x4 block where every pixel is set to the specified value.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_8888::Color8888;
    /// use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
    ///
    /// let pixel = Color8888::new(255, 0, 0, 255);
    /// let block = Decoded4x4Block::new(pixel);
    /// assert!(block.pixels.iter().all(|&p| p == pixel));
    /// ```
    pub fn new(pixel: Color8888) -> Self {
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
    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Color8888 {
        *self.pixels.get_unchecked(y * 4 + x)
    }

    /// Sets a pixel at the specified coordinates (0-3, 0-3) without bounds checking
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < 4` and `y < 4`.
    #[inline]
    pub unsafe fn set_pixel_unchecked(&mut self, x: usize, y: usize, pixel: Color8888) {
        *self.pixels.get_unchecked_mut(y * 4 + x) = pixel;
    }

    /// Checks if all pixels in the block have the same color values
    ///
    /// # Returns
    /// `true` if all pixels in the block are identical, `false` otherwise
    #[inline]
    pub fn has_identical_pixels(&self) -> bool {
        // Assert at compile time that Color8888 is the same size as u32
        const _: () = assert!(size_of::<Color8888>() == size_of::<u32>());

        // Cast first pixel to u32
        let first_pixel_u32: u32 = unsafe { transmute(self.pixels[0]) };

        // Compare all other pixels to the first one after casting to u32
        self.pixels.iter().all(|pixel| {
            let pixel_u32: u32 = unsafe { transmute(*pixel) };
            pixel_u32 == first_pixel_u32
        })
    }
}
