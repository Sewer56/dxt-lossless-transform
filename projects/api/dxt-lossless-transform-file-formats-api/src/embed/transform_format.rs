//! Transform format enumeration for different DXT compression formats.

use core::hint::unreachable_unchecked;

/// Represents the different transform formats that can be embedded.
///
/// This enum uses 4 bits in the header, allowing for up to 16 different formats.
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
    /// Reserved for future use
    Reserved4 = 0x04,
    /// Reserved for future use
    Reserved5 = 0x05,
    /// Reserved for future use
    Reserved6 = 0x06,
    /// Reserved for future use
    Reserved7 = 0x07,
    /// Reserved for future use
    Reserved8 = 0x08,
    /// Reserved for future use
    Reserved9 = 0x09,
    /// Reserved for future use
    Reserved10 = 0x0A,
    /// Reserved for future use
    Reserved11 = 0x0B,
    /// Reserved for future use
    Reserved12 = 0x0C,
    /// Reserved for future use
    Reserved13 = 0x0D,
    /// Reserved for future use
    Reserved14 = 0x0E,
    /// Reserved for future use
    Reserved15 = 0x0F,
}

impl TransformFormat {
    /// Convert from u8 value
    pub fn from_u8(value: u8) -> Self {
        match value & 0x0F {
            // Mask to 4 bits
            0x00 => Self::Bc1,
            0x01 => Self::Bc2,
            0x02 => Self::Bc3,
            0x03 => Self::Bc7,
            0x04 => Self::Reserved4,
            0x05 => Self::Reserved5,
            0x06 => Self::Reserved6,
            0x07 => Self::Reserved7,
            0x08 => Self::Reserved8,
            0x09 => Self::Reserved9,
            0x0A => Self::Reserved10,
            0x0B => Self::Reserved11,
            0x0C => Self::Reserved12,
            0x0D => Self::Reserved13,
            0x0E => Self::Reserved14,
            0x0F => Self::Reserved15,
            _ => unsafe { unreachable_unchecked() }, // "Value is masked to 4 bits"
        }
    }

    /// Convert to u8 value
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Bc1 => 0x00,
            Self::Bc2 => 0x01,
            Self::Bc3 => 0x02,
            Self::Bc7 => 0x03,
            Self::Reserved4 => 0x04,
            Self::Reserved5 => 0x05,
            Self::Reserved6 => 0x06,
            Self::Reserved7 => 0x07,
            Self::Reserved8 => 0x08,
            Self::Reserved9 => 0x09,
            Self::Reserved10 => 0x0A,
            Self::Reserved11 => 0x0B,
            Self::Reserved12 => 0x0C,
            Self::Reserved13 => 0x0D,
            Self::Reserved14 => 0x0E,
            Self::Reserved15 => 0x0F,
        }
    }
}
