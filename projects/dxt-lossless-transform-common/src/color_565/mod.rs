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

    /// Extracts the expanded 8-bit red component
    #[inline]
    pub fn red(&self) -> u8 {
        let r = (self.value & 0b11111000_00000000) >> 11;
        ((r << 3) | (r >> 2)) as u8
    }

    /// Extracts the expanded 8-bit green component
    #[inline]
    pub fn green(&self) -> u8 {
        let g = (self.value & 0b00000111_11100000) >> 5;
        ((g << 2) | (g >> 4)) as u8
    }

    /// Extracts the expanded 8-bit blue component
    #[inline]
    pub fn blue(&self) -> u8 {
        let b = self.value & 0b00000000_00011111;
        ((b << 3) | (b >> 2)) as u8
    }

    /// Compares two [`Color565`] values
    #[inline]
    pub fn greater_than(&self, other: &Self) -> bool {
        self.value > other.value
    }

    /// Converts this [`Color565`] to a [`Color8888`] with full opacity (alpha=255)
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

    /// Tests individual color transformations for each variant
    #[rstest]
    #[case(YCoCgVariant::Variant1)]
    #[case(YCoCgVariant::Variant2)]
    #[case(YCoCgVariant::Variant3)]
    #[case(YCoCgVariant::None)]
    fn can_individual_color_operations(#[case] variant: YCoCgVariant) {
        // Test individual color transformations
        let original = Color565::from_rgb(255, 128, 64);
        let mut transformed = original;

        // Decorrelate
        transformed = transformed.decorrelate_ycocg_r(variant);
        if variant != YCoCgVariant::None {
            assert_ne!(
                transformed.raw_value(),
                original.raw_value(),
                "{variant:?} - Color should change after decorrelation"
            );
        }
        // Recorrelate
        transformed = transformed.recorrelate_ycocg_r(variant);
        assert_eq!(
            transformed.raw_value(),
            original.raw_value(),
            "{variant:?} - Color should be restored after recorrelation"
        );
    }
}
