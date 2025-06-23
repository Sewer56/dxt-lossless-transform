//! # YCoCg-R Color Space Decorrelation for Color565
//!
//! This module provides YCoCg-R (reversible YCoCg) color space transformations for [`Color565`] values.
//! YCoCg-R is a lifting-based variation of the YCoCg color space that offers perfect reversibility,
//! making it ideal for lossless compression scenarios.
//!
//! ## Overview
//!
//! The YCoCg-R transformation is designed to decorrelate RGB color components to improve compression
//! efficiency. Unlike standard YCoCg transformations, YCoCg-R maintains perfect reversibility through
//! careful bit manipulation and lifting operations.
//!
//! ## Transformation Process
//!
//! The forward YCoCg-R transformation follows these steps:
//! 1. `Co = R - B` (Chroma Orange component)
//! 2. `t = B + (Co >> 1)` (Temporary value)
//! 3. `Cg = G - t` (Chroma Green component)
//! 4. `Y = t + (Cg >> 1)` (Luma component)
//!
//! The inverse transformation reverses these steps:
//! 1. `t = Y - (Cg >> 1)`
//! 2. `G = Cg + t`
//! 3. `B = t - (Co >> 1)`
//! 4. `R = B + Co`
//!
//! ## Variants
//!
//! This module provides three different variants of the YCoCg-R transformation. The variants differ
//! only in how the transformed bits are arranged within the [`Color565`] format.
//!
//! On real files, the compression differences are negligible. These variants exist primarily for
//! brute-forcing the absolute best possible compression results for people who want to squeeze every
//! last bit of space:
//!
//! - **[`YCoCgVariant::Variant1`]**: Standard arrangement  
//!   `Y(11-15) | Co(6-10) | g_low(5) | Cg(0-4)`
//!
//! - **[`YCoCgVariant::Variant2`]**: Low bit at top  
//!   `g_low(15) | Y(10-14) | Co(5-9) | Cg(0-4)`
//!
//! - **[`YCoCgVariant::Variant3`]**: Low bit at bottom  
//!   `Y(11-15) | Co(6-10) | Cg(1-5) | g_low(0)`
//!
//! - **[`YCoCgVariant::None`]**: No transformation (pass-through)
//!
//! ## Usage
//!
//! ```rust
//! use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
//!
//! let original = Color565::from_rgb(255, 128, 64);
//!
//! // Transform to YCoCg-R space
//! let decorrelated = original.decorrelate_ycocg_r(YCoCgVariant::Variant1);
//!
//! // Transform back to RGB space
//! let recorrelated = decorrelated.recorrelate_ycocg_r(YCoCgVariant::Variant1);
//!
//! // Verify perfect reversibility
//! assert_eq!(original.raw_value(), recorrelated.raw_value());
//! ```

use super::Color565;
use derive_enum_all_values::AllValues;

/// Represents a function variant for decorrelation/recorrelation operations.
///
/// The variants differ only in how the transformed bits are arranged within the [`Color565`] format.
/// On real files, the compression differences are negligible. These variants exist primarily for
/// brute-forcing the absolute best possible compression result in specific scenarios.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, AllValues, Hash)]
pub enum YCoCgVariant {
    /// None: No transformation (original RGB values preserved)
    None = 0,
    /// Variant 1: Standard bit arrangement
    Variant1 = 1,
    /// Variant 2: Alternative bit arrangement with low bit placed at top
    Variant2 = 2,
    /// Variant 3: Alternative bit arrangement with low bit at bottom
    Variant3 = 3,
}

impl Default for YCoCgVariant {
    fn default() -> Self {
        Self::None
    }
}

impl Color565 {
    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// See the [module documentation](self) for details on the YCoCg-R transformation process.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r_var1();
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r_var1(&self) -> Self {
        // 0x1F == 0b11111
        // Extract RGB components
        let r = (self.value >> 11) & 0x1F; // 5 bits for red
        let g = (self.value >> 6) & 0x1F; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0x1; // leftover bit for green
        let b = self.value & 0x1F; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0x1F;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0x1F;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0x1F;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0x1F;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        Color565::from_raw(((y as u16) << 11) | ((co as u16) << 6) | (g_low << 5) | (cg as u16))
    }

    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of [`decorrelate_ycocg_r_var1`](Self::decorrelate_ycocg_r_var1).
    /// See the [module documentation](self) for details on the transformation process.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r_var1();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r_var1();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r_var1(&self) -> Self {
        // 0x1F == 0b11111
        // Extract YCoCg-R components
        let y = (self.value >> 11) & 0x1F; // 5 bits (Y in red position)
        let co = (self.value >> 6) & 0x1F; // 5 bits (Co in upper 5 bits of green position)
        let g_low = (self.value >> 5) & 0x1; // Extract the preserved low bit of green
        let cg = self.value & 0x1F; // 5 bits (Cg in blue position)

        // Apply YCoCg-R inverse transform (all operations in 5-bit space)
        // Step 1: t = Y - (Cg >> 1)
        let t = (y as i16 - ((cg as i16) >> 1)) & 0x1F;

        // Step 2: G = Cg + t
        let g = (cg as i16 + t) & 0x1F;

        // Step 3: B = t - (Co >> 1)
        let b = (t - ((co as i16) >> 1)) & 0x1F;

        // Step 4: R = B + Co
        let r = (b + co as i16) & 0x1F;

        // Pack back into RGB565 format, preserving the original g_low bit
        Color565::from_raw(((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16))
    }

    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// See the [module documentation](self) for details on the YCoCg-R transformation process.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r_var2();
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r_var2(&self) -> Self {
        // 0x1F == 0b11111
        // Extract RGB components
        let r = (self.value >> 11) & 0x1F; // 5 bits for red
        let g = (self.value >> 6) & 0x1F; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0x1; // leftover bit for green
        let b = self.value & 0x1F; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0x1F;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0x1F;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0x1F;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0x1F;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        Color565::from_raw((g_low << 15) | ((y as u16) << 10) | ((co as u16) << 5) | (cg as u16))
        // Note: Marginal speed improvement on recorrelate by placing low bit in the top.
    }

    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of [`decorrelate_ycocg_r_var2`](Self::decorrelate_ycocg_r_var2).
    /// See the [module documentation](self) for details on the transformation process.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r_var2();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r_var2();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r_var2(&self) -> Self {
        // 0x1F == 0b11111
        // Extract YCoCg-R components
        let g_low = self.value >> 15; // Extract the preserved low bit of green
        let y = (self.value >> 10) & 0x1F; // 5 bits (Y in red position)
        let co = (self.value >> 5) & 0x1F; // 5 bits (Co in upper 5 bits of green position)
        let cg = self.value & 0x1F; // 5 bits (Cg in blue position)

        // Apply YCoCg-R inverse transform (all operations in 5-bit space)
        // Step 1: t = Y - (Cg >> 1)
        let t = (y as i16 - ((cg as i16) >> 1)) & 0x1F;

        // Step 2: G = Cg + t
        let g = (cg as i16 + t) & 0x1F;

        // Step 3: B = t - (Co >> 1)
        let b = (t - ((co as i16) >> 1)) & 0x1F;

        // Step 4: R = B + Co
        let r = (b + co as i16) & 0x1F;

        // Pack back into RGB565 format, preserving the original g_low bit
        Color565::from_raw(((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16))
    }

    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// See the [module documentation](self) for details on the YCoCg-R transformation process.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r_var3();
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r_var3(&self) -> Self {
        // 0x1F == 0b11111
        // Extract RGB components
        let r = (self.value >> 11) & 0x1F; // 5 bits for red
        let g = (self.value >> 6) & 0x1F; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0x1; // leftover bit for green
        let b = self.value & 0x1F; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0x1F;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0x1F;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0x1F;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0x1F;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        Color565::from_raw(((y as u16) << 11) | ((co as u16) << 6) | ((cg as u16) << 1) | g_low)
    }

    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of [`decorrelate_ycocg_r_var3`](Self::decorrelate_ycocg_r_var3).
    /// See the [module documentation](self) for details on the transformation process.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r_var3();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r_var3();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r_var3(&self) -> Self {
        // 0x1F == 0b11111
        // Extract YCoCg-R components
        let y = (self.value >> 11) & 0x1F; // 5 bits (Y in red position)
        let co = (self.value >> 6) & 0x1F; // 5 bits (Co in upper 5 bits of green position)
        let cg = (self.value >> 1) & 0x1F; // 5 bits (Cg in bits 1-5)
        let g_low = self.value & 0x1; // Extract the preserved low bit of green (lowermost bit)

        // Apply YCoCg-R inverse transform (all operations in 5-bit space)
        // Step 1: t = Y - (Cg >> 1)
        let t = (y as i16 - ((cg as i16) >> 1)) & 0x1F;

        // Step 2: G = Cg + t
        let g = (cg as i16 + t) & 0x1F;

        // Step 3: B = t - (Co >> 1)
        let b = (t - ((co as i16) >> 1)) & 0x1F;

        // Step 4: R = B + Co
        let r = (b + co as i16) & 0x1F;

        // Pack back into RGB565 format, preserving the original g_low bit
        Color565::from_raw(((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16))
    }

    /// Applies the specified decorrelation variant to a color
    ///
    /// Wrapper around the variant-specific decorrelate methods.
    ///
    /// # Parameters
    ///
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r(YCoCgVariant::Variant1);
    /// // Color is now in YCoCg-R form
    /// ```
    #[inline]
    pub fn decorrelate_ycocg_r(&self, variant: YCoCgVariant) -> Self {
        match variant {
            YCoCgVariant::Variant1 => self.decorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => self.decorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => self.decorrelate_ycocg_r_var3(),
            YCoCgVariant::None => *self, // No transformation
        }
    }

    /// Applies the specified recorrelation variant to a color
    ///
    /// Wrapper around the variant-specific recorrelate methods.
    ///
    /// # Parameters
    ///
    /// - `variant`: The [`YCoCgVariant`] to use
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::{Color565, YCoCgVariant};
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r(YCoCgVariant::Variant1);
    /// color.recorrelate_ycocg_r(YCoCgVariant::Variant1);
    /// ```
    #[inline]
    pub fn recorrelate_ycocg_r(&self, variant: YCoCgVariant) -> Self {
        match variant {
            YCoCgVariant::Variant1 => self.recorrelate_ycocg_r_var1(),
            YCoCgVariant::Variant2 => self.recorrelate_ycocg_r_var2(),
            YCoCgVariant::Variant3 => self.recorrelate_ycocg_r_var3(),
            YCoCgVariant::None => *self, // No transformation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    /// Tests that colors can be properly decorrelated and recorrelated
    /// back to their original values for all three variants.
    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    #[case(YCoCgVariant::None)]
    fn can_decorrelate_recorrelate(#[case] variant: YCoCgVariant) {
        // Create a variety of test colors to ensure coverage across the color space
        let test_colors = [
            Color565::from_rgb(255, 0, 0),     // Red
            Color565::from_rgb(0, 255, 0),     // Green
            Color565::from_rgb(0, 0, 255),     // Blue
            Color565::from_rgb(255, 255, 0),   // Yellow
            Color565::from_rgb(0, 255, 255),   // Cyan
            Color565::from_rgb(255, 0, 255),   // Magenta
            Color565::from_rgb(128, 128, 128), // Gray
            Color565::from_rgb(255, 255, 255), // White
            Color565::from_rgb(0, 0, 0),       // Black
            Color565::from_rgb(255, 128, 64),  // Orange
            Color565::from_rgb(128, 0, 255),   // Purple
            Color565::from_rgb(0, 128, 64),    // Teal
            Color565::from_rgb(31, 79, 83),    // Prime Numbers
        ];

        // Test each color individually in a loop for easier debugging
        for (x, original_color) in test_colors.iter().enumerate() {
            // Step 1: Decorrelate
            let decorr = original_color.decorrelate_ycocg_r(variant);

            // Step 2: Recorrelate
            let recorr = decorr.recorrelate_ycocg_r(variant);

            // Verify the color is restored to its original value
            assert_eq!(
                recorr.raw_value(),
                original_color.raw_value(),
                "{variant:?} - Color at index {x} failed to restore."
            );
        }
    }
}
