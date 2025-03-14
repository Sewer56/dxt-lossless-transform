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
}
