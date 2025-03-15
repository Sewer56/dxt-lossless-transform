use crate::color_8888::Color8888;

/// Represents a 16-bit RGB565 color (5 bits red, 6 bits green, 5 bits blue)
/// As encountered in many of the BC1 formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    /// Transforms RGB color to YCoCg-R (reversible YCoCg) color space.
    ///
    /// YCoCg-R is a lifting-based variation of YCoCg that offers perfect reversibility.
    /// This implementation treats all channels as 5-bit values for consistency.
    ///
    /// The YCoCg-R transformation follows these steps:
    /// 1. Co = R - B
    /// 2. t = B + (Co >> 1)
    /// 3. Cg = G - t
    /// 4. Y = t + (Cg >> 1)
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let mut color = Color565::from_rgb(255, 128, 64);
    /// color.decorrelate_ycocg_r();
    /// // Color is now in YCoCg-R form
    /// ```
    #[no_mangle]
    pub fn decorrelate_ycocg_r(&mut self) {
        // Extract RGB components
        let r = (self.value >> 11) & 0b11111; // 5 bits for red
        let g = (self.value >> 6) & 0b11111; // 5 top bits for green (ignoring bottom 1 bit)
        let g_low = (self.value >> 5) & 0b1; // leftover bit for green
        let b = self.value & 0b11111; // 5 bits for blue

        // Apply YCoCg-R forward transform (all operations in 5-bit space)
        // Step 1: Co = R - B
        let co = (r as i16 - b as i16) & 0b11111;

        // Step 2: t = B + (Co >> 1)
        let t = (b as i16 + (co >> 1)) & 0b11111;

        // Step 3: Cg = G - t
        let cg = (g as i16 - t) & 0b11111;

        // Step 4: Y = t + (Cg >> 1)
        let y = (t + (cg >> 1)) & 0b11111;

        // Pack into Color565 format:
        // - Y (5 bits) in red position
        // - Co (5 bits) in green position (shifted to use upper 5 bits of the 6-bit field)
        // - Cg (5 bits) in blue position
        self.value = ((y as u16) << 11) | ((co as u16) << 6) | (g_low << 5) | (cg as u16);
    }

    /// Transforms color from YCoCg-R back to RGB color space.
    ///
    /// This is the inverse of the decorrelation operation, following these steps:
    /// 1. t = Y - (Cg >> 1)
    /// 2. G = Cg + t
    /// 3. B = t - (Co >> 1)
    /// 4. R = B + Co
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_common::color_565::Color565;
    ///
    /// let original = Color565::from_rgb(255, 128, 64);
    /// let mut decorrelated = original;
    /// decorrelated.decorrelate_ycocg_r();
    ///
    /// // Now recorrelate it back to original
    /// decorrelated.recorrelate_ycocg_r();
    /// assert_eq!(decorrelated.raw_value(), original.raw_value());
    /// ```
    #[no_mangle]
    pub fn recorrelate_ycocg_r(&mut self) {
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
        self.value = ((r as u16) << 11) | ((g as u16) << 6) | (g_low << 5) | (b as u16);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that colors can be properly decorrelated and recorrelated
    /// back to their original values, testing each color individually.
    #[test]
    fn can_decorrelate_recorrelate() {
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
            // Create a copy to transform
            let mut color = *original_color;

            // Step 1: Decorrelate
            color.decorrelate_ycocg_r();

            // Step 2: Recorrelate
            color.recorrelate_ycocg_r();

            // Verify the color is restored to its original value
            assert_eq!(
                color.raw_value(),
                original_color.raw_value(),
                "Color at index {} failed to restore.",
                x
            );
        }
    }

    #[test]
    fn can_individual_color_operations() {
        // Test individual color transformations
        let original = Color565::from_rgb(255, 128, 64);
        let mut transformed = original;

        // Decorrelate
        transformed.decorrelate_ycocg_r();
        assert_ne!(
            transformed.raw_value(),
            original.raw_value(),
            "Color should change after decorrelation"
        );

        // Recorrelate
        transformed.recorrelate_ycocg_r();
        assert_eq!(
            transformed.raw_value(),
            original.raw_value(),
            "Color should be restored after recorrelation"
        );
    }
}
