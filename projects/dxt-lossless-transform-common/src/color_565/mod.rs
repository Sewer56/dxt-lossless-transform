//! # RGB565 Color Format Support
//!
//! This module provides comprehensive support for working with 16-bit RGB565 colors,
//! which are commonly used in BC1-BC3 texture compression formats.
//!
//! ## Overview
//!
//! RGB565 is a 16-bit color format that packs red, green, and blue color components
//! into a single 16-bit value:
//!
//! - **Red**: 5 bits (bits 15-11)
//! - **Green**: 6 bits (bits 10-5)
//! - **Blue**: 5 bits (bits 4-0)
//!
//! The green component gets an extra bit because the human eye is more sensitive
//! to green light.
//!
//! ## Core Components
//!
//! - [`Color565`] - The main struct representing a 16-bit RGB565 color
//! - Decorrelation functions for optimizing color transformations
//! - Batch processing utilities for efficient bulk operations
//!
//! ## Color Expansion
//!
//! When converting RGB565 colors back to 8-bit components, this module follows
//! the D3D11 functional specification approach: expanding from 5/6 bits to 8 bits
//! by replicating the top bits. This ensures compatibility with GPU hardware
//! implementations.
//!
//! ## Examples
//!
//! ### Basic Usage
//!
//! ```rust
//! use dxt_lossless_transform_common::color_565::Color565;
//!
//! // Create from RGB components
//! let color = Color565::from_rgb(255, 128, 64);
//!
//! // Access individual components (expanded to 8-bit i.e. 0-255)
//! let r = color.red();
//! let g = color.green();
//! let b = color.blue();
//!
//! // Get raw 16-bit value
//! let raw = color.raw_value();
//!
//! // Convert to RGBA8888
//! let rgba = color.to_color_8888();
//! ```
//!
//! ### Working with Raw Values
//!
//! ```rust
//! use dxt_lossless_transform_common::color_565::Color565;
//!
//! // Create from raw 16-bit value
//! let color = Color565::from_raw(0xF800); // Pure red
//! assert_eq!(color.red(), 255);
//! assert_eq!(color.green(), 0);
//! assert_eq!(color.blue(), 0);
//! ```
//!
//! ## Additional Reading
//!
//! - [etcpak - Fast ETC1/ETC2/EAC encoder](https://github.com/wolfpld/etcpak) -
//!   Source of optimized RGB565 conversion algorithms used in this module
//! - [GPU BCn Decoding by Fabian Giesen](https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/) -
//!   Comprehensive explanation of GPU texture compression and the D3D11 specification
//!   approach to color expansion that this module implements

mod decorrelate;
mod decorrelate_batch_ptr;
mod decorrelate_batch_slice;
mod decorrelate_batch_split_ptr;
mod decorrelate_batch_split_slice;

// File split to multiple chunks to group functionality together.
pub use decorrelate::*;

use crate::color_8888::Color8888;
use multiversion::multiversion;

/// Represents a 16-bit RGB565 color (5 bits red, 6 bits green, 5 bits blue)
/// As encountered in many of the BC1 formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Color565 {
    /// The underlying 16-bit RGB565 value
    value: u16,
}

impl Color565 {
    /// Creates a new [`Color565`] from the raw 16-bit value
    #[inline]
    pub fn from_raw(value: u16) -> Self {
        Self { value }
    }

    /// Creates a new [`Color565`] from separate RGB components
    ///
    /// # Parameters
    ///
    /// - `r`: The red component (0-255)
    /// - `g`: The green component (0-255)
    /// - `b`: The blue component (0-255)
    #[inline]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        // Implementation matches etcpak's optimized to565 function
        // Source: https://github.com/wolfpld/etcpak/blob/master/ProcessDxtc.cpp
        // This approach calculates the entire value in one expression, potentially allowing
        // better compiler optimizations.
        Self {
            value: ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3),
        }

        // Original implementation (me):
        //   Computes to same thing, but just for compiler's sake, using the above.
        //
        // let r = (r as u16 >> 3) & 0b11111;
        // let g = (g as u16 >> 2) & 0b111111;
        // let b = b as u16 >> 3;
        //
        // Self {
        //     value: (r << 11) | (g << 5) | b,
        // }
        //
    }

    /// Returns the raw 16-bit value
    #[inline]
    pub fn raw_value(&self) -> u16 {
        self.value
    }

    // NOTE: https://fgiesen.wordpress.com/2021/10/04/gpu-bcn-decoding/
    // BC1 as written in the D3D11 functional spec first expands the endpoint values from 5 or 6 bits
    // to 8 bits by replicating the top bits; all three vendors appear to do this or something equivalent,
    // and then convert the result from 8-bit UNorm to float exactly.
    // Thanks ryg!! I know am decoding the colour endpoints right!!

    /// Extracts the expanded 8-bit red component (0-255)
    ///
    /// This function expands the 5-bit red value to 8 bits (0-255) by replicating the top bits,
    /// following the D3D11 functional specification. This ensures hardware compatibility
    /// and matches GPU implementations.
    ///
    /// # Note
    ///
    /// The expansion from 5 bits to 8 bits (0-255) means that not all 8-bit values can be
    /// represented exactly in RGB565 format. The expansion process follows the formula:
    /// `result = (value << 3) | (value >> 2)` for 5-bit components.
    #[inline]
    pub fn red(&self) -> u8 {
        let r = (self.value & 0b11111000_00000000) >> 11;
        ((r << 3) | (r >> 2)) as u8
    }

    /// Extracts the expanded 8-bit green component (0-255)
    ///
    /// This function expands the 6-bit green value to 8 bits (0-255) by replicating the top bits,
    /// following the D3D11 functional specification. This ensures hardware compatibility
    /// and matches GPU implementations.
    ///
    /// # Note
    ///
    /// The expansion from 6 bits to 8 bits (0-255) means that not all 8-bit values can be
    /// represented exactly in RGB565 format. The expansion process follows the formula:
    /// `result = (value << 2) | (value >> 4)` for 6-bit components.
    #[inline]
    pub fn green(&self) -> u8 {
        let g = (self.value & 0b00000111_11100000) >> 5;
        ((g << 2) | (g >> 4)) as u8
    }

    /// Extracts the expanded 8-bit blue component (0-255)
    ///
    /// This function expands the 5-bit blue value to 8 bits (0-255) by replicating the top bits,
    /// following the D3D11 functional specification. This ensures hardware compatibility
    /// and matches GPU implementations.
    ///
    /// # Note
    ///
    /// The expansion from 5 bits to 8 bits (0-255) means that not all 8-bit values can be
    /// represented exactly in RGB565 format. The expansion process follows the formula:
    /// `result = (value << 3) | (value >> 2)` for 5-bit components.
    #[inline]
    pub fn blue(&self) -> u8 {
        let b = self.value & 0b00000000_00011111;
        ((b << 3) | (b >> 2)) as u8
    }

    /// Compares two [`Color565`] values
    ///
    /// Returns if this value is greater than the other.
    /// This is a useful comparison when decoding BC1-BC3 blocks.
    #[inline]
    pub fn greater_than(&self, other: &Self) -> bool {
        self.value > other.value
    }

    /// Converts this [`Color565`] to a [`Color8888`] with full opacity (alpha=255)
    ///
    /// # Note on Precision
    ///
    /// This conversion is **lossy** due to the precision difference between RGB565 and RGBA8888.
    /// The RGB565 format uses fewer bits per channel (5/6/5) compared to the 8 bits per channel
    /// in RGBA8888. The conversion expands the color channels using bit replication following
    /// the D3D11 specification, but not all possible 8-bit color values can be represented
    /// exactly in RGB565 format.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let rgb565 = Color565::from_rgb(255, 0, 0);
    /// let rgba8888 = rgb565.to_color_8888();
    /// assert_eq!(rgba8888.r, 255);
    /// assert_eq!(rgba8888.g, 0);
    /// assert_eq!(rgba8888.b, 0);
    /// assert_eq!(rgba8888.a, 255);
    /// ```
    pub fn to_color_8888(&self) -> Color8888 {
        Color8888::new(self.red(), self.green(), self.blue(), 255)
    }

    /// Converts this RGB565 color to a RGBA8888 color with the specified alpha value
    ///
    /// # Note on Precision
    ///
    /// This conversion is **lossy** due to the precision difference between RGB565 and RGBA8888.
    /// The RGB565 format uses fewer bits per channel (5/6/5) compared to the 8 bits per channel
    /// in RGBA8888. The conversion expands the color channels using bit replication following
    /// the D3D11 specification, but not all possible 8-bit color values can be represented
    /// exactly in RGB565 format.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let rgb565 = Color565::from_rgb(255, 0, 0);
    /// let rgba8888 = rgb565.to_color_8888_with_alpha(128);
    /// assert_eq!(rgba8888.r, 255);
    /// assert_eq!(rgba8888.g, 0);
    /// assert_eq!(rgba8888.b, 0);
    /// assert_eq!(rgba8888.a, 128);
    /// ```
    pub fn to_color_8888_with_alpha(&self, alpha: u8) -> Color8888 {
        Color8888::new(self.red(), self.green(), self.blue(), alpha)
    }
}
