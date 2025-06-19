//! RGBA8888 Color Representation
//!
//! This module provides the [`Color8888`] type for representing 32-bit RGBA pixels
//! with 8 bits per color channel. This is commonly used as an intermediate format
//! when working with decompressed DXT/BC texture data.
//!
//! The [`Color8888`] type supports lossy conversion to the more compact [`Color565`] format
//! and various utility operations for texture processing workflows.
//!
//! # Examples
//!
//! ```
//! use dxt_lossless_transform_common::color_8888::Color8888;
//!
//! // Create a red pixel with full opacity
//! let red_pixel = Color8888::new(255, 0, 0, 255);
//!
//! // Convert to RGB565 format
//! let rgb565 = red_pixel.to_565_lossy();
//!
//! // Create a transparent version
//! let transparent = red_pixel.without_alpha();
//! assert_eq!(transparent.a, 0);
//! ```

use crate::color_565::Color565;

/// Represents a single RGBA8888 pixel color from a decoded BC1 block
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color8888 {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
}

impl Color8888 {
    /// Constructs a new [`Color8888`] from the specified red, green, blue, and alpha components.
    ///
    /// Each parameter represents the intensity of its corresponding colour channel (0–255).
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_8888::Color8888;
    ///
    /// let pixel = Color8888::new(255, 0, 0, 255);
    /// assert_eq!(pixel.r, 255);
    /// assert_eq!(pixel.g, 0);
    /// assert_eq!(pixel.b, 0);
    /// assert_eq!(pixel.a, 255);
    /// ```
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Converts this [`Color8888`] to a [`Color565`] with lossy precision reduction.
    ///
    /// Note that this conversion is lossy as it reduces the color precision from 8 bits
    /// per channel (RGB) to 5-6-5 bits respectively. The alpha channel is discarded.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_8888::Color8888;
    ///
    /// let pixel = Color8888::new(255, 0, 0, 255);
    /// let rgb565 = pixel.to_565_lossy();
    /// assert_eq!(rgb565.red(), 255);
    /// assert_eq!(rgb565.green(), 0);
    /// assert_eq!(rgb565.blue(), 0);
    /// ```
    pub fn to_565_lossy(&self) -> Color565 {
        Color565::from_rgb(self.r, self.g, self.b)
    }

    /// Returns a new [`Color8888`] with the alpha component set to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_8888::Color8888;
    ///
    /// let pixel = Color8888::new(255, 0, 0, 255);
    /// let pixel_without_alpha = pixel.without_alpha();
    /// assert_eq!(pixel_without_alpha.a, 0);
    /// ```
    pub fn without_alpha(&self) -> Color8888 {
        Color8888::new(self.r, self.g, self.b, 0)
    }
}
