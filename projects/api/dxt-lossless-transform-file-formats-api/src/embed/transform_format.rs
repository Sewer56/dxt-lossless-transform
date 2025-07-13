//! Transform format enumeration for different DXT compression formats.

/// Represents the different transform formats that can be embedded.
///
/// This enum uses 4 bits in the header, allowing for up to 16 different formats.
/// Additional formats may be added in future versions.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TransformFormat {
    /// BC1 format transform
    Bc1 = 0x00,
    /// BC2 format transform  
    Bc2 = 0x01,
    /// BC3 format transform
    Bc3 = 0x02,
    /// BC7 format transform
    Bc7 = 0x03,
    /// BC6H format transform
    Bc6H = 0x04,
    /// RGBA8888 format transform
    Rgba8888 = 0x05,
    /// BGRA8888 format transform
    Bgra8888 = 0x06,
    /// BGR888 format transform
    Bgr888 = 0x07,
    /// BC4 format transform
    Bc4 = 0x08,
    /// BC5 format transform
    Bc5 = 0x09,
}

impl TransformFormat {
    /// Convert from u8 value
    ///
    /// Returns [`None`] if the value does not correspond to a known transform format.
    /// This allows for forward compatibility when encountering newer format versions.
    pub(super) fn from_u8(value: u8) -> Option<Self> {
        match value & 0x0F {
            // Mask to 4 bits
            0x00 => Some(Self::Bc1),
            0x01 => Some(Self::Bc2),
            0x02 => Some(Self::Bc3),
            0x03 => Some(Self::Bc7),
            0x04 => Some(Self::Bc6H),
            0x05 => Some(Self::Rgba8888),
            0x06 => Some(Self::Bgra8888),
            0x07 => Some(Self::Bgr888),
            0x08 => Some(Self::Bc4),
            0x09 => Some(Self::Bc5),
            _ => None,
        }
    }

    /// Convert to u8 value
    pub(super) fn to_u8(self) -> u8 {
        match self {
            Self::Bc1 => 0x00,
            Self::Bc2 => 0x01,
            Self::Bc3 => 0x02,
            Self::Bc7 => 0x03,
            Self::Bc6H => 0x04,
            Self::Rgba8888 => 0x05,
            Self::Bgra8888 => 0x06,
            Self::Bgr888 => 0x07,
            Self::Bc4 => 0x08,
            Self::Bc5 => 0x09,
        }
    }
}
