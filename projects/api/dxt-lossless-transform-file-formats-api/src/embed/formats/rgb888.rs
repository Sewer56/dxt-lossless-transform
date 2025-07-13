//! RGB888 format file format support.
//!
//! This module provides RGB888-specific implementations of the file format traits.
//! Since RGB888 is an uncompressed format, no actual transformation is performed,
//! but decorrelation can still be applied.

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat, TransformHeader};
use bitfield::bitfield;

/// Header version for RGB888 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Rgb888HeaderVersion {
    /// Initial version - supports decorrelation
    InitialVersion = 0,
}

impl Rgb888HeaderVersion {
    /// Convert from u32 value
    fn from_u32(value: u32) -> Result<Self, EmbedError> {
        match value {
            0 => Ok(Self::InitialVersion),
            _ => Err(EmbedError::CorruptedEmbeddedData),
        }
    }

    /// Convert to u32 value  
    fn to_u32(self) -> u32 {
        self as u32
    }
}

bitfield! {
    /// Packed RGB888 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bit 2: Decorrelation flag (1 bit)
    /// - Bits 3-27: Reserved for future use (25 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    struct Rgb888TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Whether to apply decorrelation (1 bit)
    decorrelation, set_decorrelation: 2;
    /// Reserved for future use (25 bits)
    reserved, set_reserved: 27, 3;
}

/// RGB888 transform details for embedding in headers.
///
/// Contains settings for RGB888 pixel processing, primarily decorrelation options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmbeddableRgb888Details(Rgb888TransformHeaderData);

impl EmbeddableRgb888Details {
    /// Create new RGB888 details with default settings (no decorrelation)
    pub fn new() -> Self {
        Self::with_decorrelation(false)
    }

    /// Create new RGB888 details with specified decorrelation setting
    pub fn with_decorrelation(decorrelation: bool) -> Self {
        let mut data = Rgb888TransformHeaderData::default();
        data.set_header_version(Rgb888HeaderVersion::InitialVersion.to_u32());
        data.set_decorrelation(decorrelation);
        data.set_reserved(0);
        Self(data)
    }

    /// Convert to a [`TransformHeader`]
    pub fn to_header(self) -> TransformHeader {
        crate::embed::TransformHeader::new(Self::FORMAT, self.pack())
    }
}

impl Default for EmbeddableRgb888Details {
    fn default() -> Self {
        Self::new()
    }
}

impl EmbeddableTransformDetails for EmbeddableRgb888Details {
    const FORMAT: TransformFormat = TransformFormat::Rgb888;

    fn pack(&self) -> u32 {
        self.0 .0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        let header_data = Rgb888TransformHeaderData(data);

        // Validate header version
        Rgb888HeaderVersion::from_u32(header_data.header_version())?;

        // Reserved bits should be zero for forward compatibility
        if header_data.reserved() != 0 {
            return Err(EmbedError::CorruptedEmbeddedData);
        }

        Ok(Self(header_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb888_details_default() {
        let details = EmbeddableRgb888Details::new();
        // Test that default details can be created
        assert_eq!(details, EmbeddableRgb888Details::with_decorrelation(false));
    }

    #[test]
    fn test_rgb888_details_with_decorrelation() {
        let details_true = EmbeddableRgb888Details::with_decorrelation(true);
        let details_false = EmbeddableRgb888Details::with_decorrelation(false);

        // Test that different decorrelation settings create different details
        assert_ne!(details_true, details_false);
    }

    #[test]
    fn test_rgb888_pack_unpack_roundtrip() {
        let original = EmbeddableRgb888Details::with_decorrelation(true);
        let packed = original.pack();
        let unpacked = EmbeddableRgb888Details::unpack(packed).unwrap();

        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_rgb888_header_roundtrip() {
        let details = EmbeddableRgb888Details::with_decorrelation(true);
        let header = details.to_header();

        assert_eq!(header.format(), Some(TransformFormat::Rgb888));

        let recovered = EmbeddableRgb888Details::from_header(header).unwrap();
        assert_eq!(details, recovered);
    }
}
