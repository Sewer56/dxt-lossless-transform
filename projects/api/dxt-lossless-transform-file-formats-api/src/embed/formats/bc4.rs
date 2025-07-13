//! BC4 format file format support.
//!
//! This module provides BC4-specific implementations of the file format traits.
//! BC4 is a single-channel compressed format, typically used for alpha or grayscale data.

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat, TransformHeader};
use bitfield::bitfield;

/// BC4 transform settings for single-channel compression.
///
/// BC4 is a single-channel format so it has simpler settings compared to color formats.
/// This is a placeholder structure until the BC4 core crate is implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc4TransformSettings {
    /// Whether to split alpha_0 and alpha_1 into separate arrays
    pub split_endpoints: bool,
}

impl Bc4TransformSettings {
    /// Create default BC4 transform settings
    pub fn new() -> Self {
        Self {
            split_endpoints: false,
        }
    }

    /// Create BC4 settings with specified endpoint splitting
    pub fn with_split_endpoints(split: bool) -> Self {
        Self {
            split_endpoints: split,
        }
    }

    /// Get all possible combinations of BC4 settings for testing
    pub fn all_combinations() -> impl Iterator<Item = Self> {
        [false, true]
            .into_iter()
            .map(|split_endpoints| Self { split_endpoints })
    }
}

impl Default for Bc4TransformSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Header version for BC4 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bc4HeaderVersion {
    /// Initial version - supports endpoint splitting
    InitialVersion = 0,
}

impl Bc4HeaderVersion {
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
    /// Packed BC4 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bit 2: Split endpoints flag (1 bit)
    /// - Bits 3-27: Reserved for future use (25 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    struct Bc4TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Whether to split endpoints (1 bit)
    split_endpoints, set_split_endpoints: 2;
    /// Reserved for future use (25 bits)
    reserved, set_reserved: 27, 3;
}

impl Bc4TransformHeaderData {
    /// Create [`Bc4TransformHeaderData`] from [`Bc4TransformSettings`]
    fn from_transform_settings(settings: &Bc4TransformSettings) -> Self {
        let mut header = Self::default();
        header.set_header_version(Bc4HeaderVersion::InitialVersion.to_u32());
        header.set_split_endpoints(settings.split_endpoints);
        header.set_reserved(0);
        header
    }

    /// Convert [`Bc4TransformHeaderData`] to [`Bc4TransformSettings`]
    fn to_transform_settings(self) -> Result<Bc4TransformSettings, EmbedError> {
        // Validate version (from_u32 will error on invalid version)
        let _version = Bc4HeaderVersion::from_u32(self.header_version())?;

        // Reserved bits should be zero for forward compatibility
        if self.reserved() != 0 {
            return Err(EmbedError::CorruptedEmbeddedData);
        }

        Ok(Bc4TransformSettings {
            split_endpoints: self.split_endpoints(),
        })
    }
}

/// BC4 transform details for embedding in headers.
///
/// Contains settings for BC4 single-channel compression processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmbeddableBc4Details(Bc4TransformSettings);

impl EmbeddableTransformDetails for EmbeddableBc4Details {
    const FORMAT: TransformFormat = TransformFormat::Bc4;

    fn pack(&self) -> u32 {
        Bc4TransformHeaderData::from_transform_settings(&self.0).0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        Ok(Self(Bc4TransformHeaderData(data).to_transform_settings()?))
    }
}

impl EmbeddableBc4Details {
    /// Create new BC4 details with default settings (no optimization)
    pub fn new() -> Self {
        Self(Bc4TransformSettings::new())
    }

    /// Create new BC4 details with specified endpoint splitting setting
    pub fn with_split_endpoints(split: bool) -> Self {
        Self(Bc4TransformSettings::with_split_endpoints(split))
    }

    /// Convert to a [`TransformHeader`]
    pub fn to_header(self) -> TransformHeader {
        crate::embed::TransformHeader::new(Self::FORMAT, self.pack())
    }

    /// Create from core BC4 transform settings (internal use only)
    pub(crate) fn from_settings(settings: Bc4TransformSettings) -> Self {
        Self(settings)
    }

    /// Convert to core BC4 transform settings (internal use only)
    pub(crate) fn to_settings(self) -> Bc4TransformSettings {
        self.0
    }
}

impl Default for EmbeddableBc4Details {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc4_details_default() {
        let details = EmbeddableBc4Details::new();
        // Test that default details can be created
        assert_eq!(details, EmbeddableBc4Details::with_split_endpoints(false));
    }

    #[test]
    fn test_bc4_details_with_optimization() {
        let details_true = EmbeddableBc4Details::with_split_endpoints(true);
        let details_false = EmbeddableBc4Details::with_split_endpoints(false);

        // Test that different split settings create different details
        assert_ne!(details_true, details_false);
    }

    #[test]
    fn test_bc4_pack_unpack_roundtrip() {
        let original = EmbeddableBc4Details::with_split_endpoints(true);
        let packed = original.pack();
        let unpacked = EmbeddableBc4Details::unpack(packed).unwrap();

        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_bc4_header_roundtrip() {
        let details = EmbeddableBc4Details::with_split_endpoints(true);
        let header = details.to_header();

        assert_eq!(header.format(), Some(TransformFormat::Bc4));

        let recovered = EmbeddableBc4Details::from_header(header).unwrap();
        assert_eq!(details, recovered);
    }

    #[test]
    fn test_roundtrip_all_possible_transform_details() {
        for settings in Bc4TransformSettings::all_combinations() {
            let embeddable = EmbeddableBc4Details::from_settings(settings);

            let packed = embeddable.pack();
            let recovered = EmbeddableBc4Details::unpack(packed).unwrap();
            assert_eq!(
                settings,
                recovered.to_settings(),
                "Failed for settings {settings:?}",
            );
        }
    }

    #[test]
    fn test_header_version_and_reserved_fields() {
        let settings = Bc4TransformSettings {
            split_endpoints: true,
        };

        let header = Bc4TransformHeaderData::from_transform_settings(&settings);

        // Verify version is set correctly
        assert_eq!(
            header.header_version(),
            Bc4HeaderVersion::InitialVersion.to_u32()
        );
        // Verify reserved field is set to zero
        assert_eq!(header.reserved(), 0);

        // Verify actual data fields are set correctly
        assert!(header.split_endpoints());
    }

    #[test]
    fn test_invalid_header_version() {
        // Create header with invalid version
        let mut invalid_header = Bc4TransformHeaderData::default();
        invalid_header.set_header_version(3); // Invalid version (only 0 is valid)

        // Should fail to convert to transform settings due to invalid version
        assert_eq!(
            invalid_header.to_transform_settings(),
            Err(EmbedError::CorruptedEmbeddedData)
        );
    }

    #[test]
    fn test_invalid_reserved_bits() {
        // Create header with non-zero reserved bits
        let mut invalid_header = Bc4TransformHeaderData::default();
        invalid_header.set_header_version(Bc4HeaderVersion::InitialVersion.to_u32());
        invalid_header.set_reserved(1); // Should be zero

        // Should fail to convert due to non-zero reserved bits
        assert_eq!(
            invalid_header.to_transform_settings(),
            Err(EmbedError::CorruptedEmbeddedData)
        );
    }

    #[test]
    fn test_format_association() {
        // Verify the format association is correct
        assert_eq!(EmbeddableBc4Details::FORMAT, TransformFormat::Bc4);
    }
}
