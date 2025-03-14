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
    #[inline]
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r = (r as u16 >> 3) & 0x1F;
        let g = (g as u16 >> 2) & 0x3F;
        let b = (b as u16 >> 3) & 0x1F;

        Self {
            value: (r << 11) | (g << 5) | b,
        }
    }

    /// Returns the raw 16-bit value
    #[inline]
    pub fn raw_value(&self) -> u16 {
        self.value
    }

    /// Extracts the expanded 8-bit red component
    #[inline]
    pub fn red(&self) -> u8 {
        let r = (self.value & 0xF800) >> 11;
        ((r << 3) | (r >> 2)) as u8
    }

    /// Extracts the expanded 8-bit green component
    #[inline]
    pub fn green(&self) -> u8 {
        let g = (self.value & 0x07E0) >> 5;
        ((g << 2) | (g >> 4)) as u8
    }

    /// Extracts the expanded 8-bit blue component
    #[inline]
    pub fn blue(&self) -> u8 {
        let b = self.value & 0x001F;
        ((b << 3) | (b >> 2)) as u8
    }

    /// Compares two `Color565` values
    #[inline]
    pub fn greater_than(&self, other: &Self) -> bool {
        self.value > other.value
    }
}
