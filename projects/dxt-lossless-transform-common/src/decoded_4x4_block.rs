//! # Decoded 4x4 Block
//!
//! This module provides the [`Decoded4x4Block`] structure for representing decompressed
//! 4x4 pixel blocks used in DXT/BC texture compression formats.
//!
//! ## Overview
//!
//! DXT (DirectX Texture Compression) and BC (Block Compression) formats compress textures
//! by dividing them into 4x4 pixel blocks. This module provides a convenient representation
//! for working with these blocks after they have been decompressed to RGBA format.
//!
//! ## Features
//!
//! - Storage of 16 pixels in row-major order using [`Color8888`] format
//! - Fast pixel access methods with bounds checking options
//! - Utility methods for analyzing block properties:
//!   - Checking for identical pixels (with or without alpha)
//!   - Checking for identical alpha values
//! - Optimized implementations using unsafe methods for performance-critical code
//!
//! ## Usage
//!
//! ```
//! use dxt_lossless_transform_common::color_8888::Color8888;
//! use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;
//!
//! // Create a block filled with red pixels
//! let red_pixel = Color8888::new(255, 0, 0, 255);
//! let block = Decoded4x4Block::new(red_pixel);
//!
//! // Check if all pixels are identical
//! assert!(block.has_identical_pixels());
//! ```
//!
//! ## Memory Layout
//!
//! The pixels are stored in row-major order:
//! ```text
//! [ 0] [ 1] [ 2] [ 3]
//! [ 4] [ 5] [ 6] [ 7]
//! [ 8] [ 9] [10] [11]
//! [12] [13] [14] [15]
//! ```
//!
//! ## Safety
//!
//! This module provides both safe and unsafe methods for pixel access. The unsafe methods
//! (`get_pixel_unchecked`, `set_pixel_unchecked`) bypass bounds checking for performance
//! in hot code paths, but require the caller to ensure coordinates are within bounds (0-3).

use crate::color_8888::Color8888;
use core::mem::transmute;

/// Represents a decoded 4x4 block of BC pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Decoded4x4Block {
    /// The 16 pixels in the block (row-major order)
    /// (i.e. `pixels[0]` is top-left, `pixels[3]` is top-right, etc.)
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

    /// Checks if all pixels in the block have the same color values
    /// Ignoring the alpha values.
    ///
    /// # Returns
    /// `true` if all pixels in the block are identical, `false` otherwise
    #[inline]
    pub fn has_identical_pixels_ignore_alpha(&self) -> bool {
        // Get first pixel without alpha
        let first_pixel_u32_no_alpha = self.pixels[0].without_alpha();

        // Compare all other pixels to the first one
        self.pixels
            .iter()
            .all(|pixel| pixel.without_alpha() == first_pixel_u32_no_alpha)
    }

    /// Checks if all pixels in the block have the same alpha values
    #[inline]
    pub fn has_identical_alpha(&self) -> bool {
        let first_pixel_alpha = self.pixels[0].a;
        self.pixels.iter().all(|pixel| pixel.a == first_pixel_alpha)
    }
}
