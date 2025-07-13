//! BC5 format file format support.
//!
//! This module provides BC5-specific implementations of the file format traits.
//! BC5 is a dual-channel compressed format, typically used for normal maps (red and green channels).

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat, TransformHeader};
use bitfield::bitfield;

/// BC5 transform settings for dual-channel compression.
///
/// BC5 is a dual-channel format (red and green) commonly used for normal maps.
/// This is a placeholder structure until the BC5 core crate is implemented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc5TransformSettings {
    /// Whether to split alpha_0 and alpha_1 into separate arrays
    pub split_endpoints: bool,
}

impl Bc5TransformSettings {
    /// Create default BC5 transform settings
    pub fn new() -> Self {
        Self {
            split_endpoints: false,
        }
    }

    /// Create BC5 settings with specified endpoint splitting
    pub fn with_split_endpoints(split: bool) -> Self {
        Self {
            split_endpoints: split,
        }
    }

    /// Get all possible combinations of BC5 settings for testing
    pub fn all_combinations() -> impl Iterator<Item = Self> {
        [false, true]
            .into_iter()
            .map(|split_endpoints| Self { split_endpoints })
    }
}

impl Default for Bc5TransformSettings {
    fn default() -> Self {
        Self::new()
    }
}

/// Header version for BC5 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bc5HeaderVersion {
    /// Initial version - supports endpoint splitting
    InitialVersion = 0,
}

impl Bc5HeaderVersion {
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
    /// Packed BC5 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bit 2: Split endpoints flag (1 bit)
    /// - Bits 3-27: Reserved for future use (25 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    struct Bc5TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Whether to split endpoints (1 bit)
    split_endpoints, set_split_endpoints: 2;
    /// Reserved for future use (25 bits)
    reserved, set_reserved: 27, 3;
}

impl Bc5TransformHeaderData {
    /// Create [`Bc5TransformHeaderData`] from [`Bc5TransformSettings`]
    fn from_transform_settings(settings: &Bc5TransformSettings) -> Self {
        let mut header = Self::default();
        header.set_header_version(Bc5HeaderVersion::InitialVersion.to_u32());
        header.set_split_endpoints(settings.split_endpoints);
        header.set_reserved(0);
        header
    }

    /// Convert [`Bc5TransformHeaderData`] to [`Bc5TransformSettings`]
    fn to_transform_settings(self) -> Result<Bc5TransformSettings, EmbedError> {
        // Validate version (from_u32 will error on invalid version)
        let _version = Bc5HeaderVersion::from_u32(self.header_version())?;

        // Reserved bits should be zero for forward compatibility
        if self.reserved() != 0 {
            return Err(EmbedError::CorruptedEmbeddedData);
        }

        Ok(Bc5TransformSettings {
            split_endpoints: self.split_endpoints(),
        })
    }
}

/// BC5 transform details for embedding in headers.
///
/// Contains settings for BC5 dual-channel compression processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct EmbeddableBc5Details(Bc5TransformSettings);

impl EmbeddableTransformDetails for EmbeddableBc5Details {
    const FORMAT: TransformFormat = TransformFormat::Bc5;

    fn pack(&self) -> u32 {
        Bc5TransformHeaderData::from_transform_settings(&self.0).0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        Ok(Self(Bc5TransformHeaderData(data).to_transform_settings()?))
    }
}

impl EmbeddableBc5Details {
    /// Create new BC5 details with default settings (no endpoint splitting)
    pub fn new() -> Self {
        Self(Bc5TransformSettings::new())
    }

    /// Create new BC5 details with specified endpoint splitting setting
    pub fn with_split_endpoints(split: bool) -> Self {
        Self(Bc5TransformSettings::with_split_endpoints(split))
    }

    /// Convert to a [`TransformHeader`]
    pub fn to_header(self) -> TransformHeader {
        crate::embed::TransformHeader::new(Self::FORMAT, self.pack())
    }

    /// Create from core BC5 transform settings (internal use only)
    pub(crate) fn from_settings(settings: Bc5TransformSettings) -> Self {
        Self(settings)
    }

    /// Convert to core BC5 transform settings (internal use only)
    pub(crate) fn to_settings(self) -> Bc5TransformSettings {
        self.0
    }
}

impl Default for EmbeddableBc5Details {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc5_details_default() {
        let details = EmbeddableBc5Details::new();
        // Test that default details can be created
        assert_eq!(details, EmbeddableBc5Details::with_split_endpoints(false));
    }

    #[test]
    fn test_bc5_details_with_optimization() {
        let details_true = EmbeddableBc5Details::with_split_endpoints(true);
        let details_false = EmbeddableBc5Details::with_split_endpoints(false);

        // Test that different split settings create different details
        assert_ne!(details_true, details_false);
    }

    #[test]
    fn test_bc5_pack_unpack_roundtrip() {
        let original = EmbeddableBc5Details::with_split_endpoints(true);
        let packed = original.pack();
        let unpacked = EmbeddableBc5Details::unpack(packed).unwrap();

        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_bc5_header_roundtrip() {
        let details = EmbeddableBc5Details::with_split_endpoints(true);
        let header = details.to_header();

        assert_eq!(header.format(), Some(TransformFormat::Bc5));

        let recovered = EmbeddableBc5Details::from_header(header).unwrap();
        assert_eq!(details, recovered);
    }

    #[test]
    fn test_roundtrip_all_possible_transform_details() {
        for settings in Bc5TransformSettings::all_combinations() {
            let embeddable = EmbeddableBc5Details::from_settings(settings);

            let packed = embeddable.pack();
            let recovered = EmbeddableBc5Details::unpack(packed).unwrap();
            assert_eq!(
                settings,
                recovered.to_settings(),
                "Failed for settings {settings:?}",
            );
        }
    }

    #[test]
    fn test_header_version_and_reserved_fields() {
        let settings = Bc5TransformSettings {
            split_endpoints: true,
        };

        let header = Bc5TransformHeaderData::from_transform_settings(&settings);

        // Verify version is set correctly
        assert_eq!(
            header.header_version(),
            Bc5HeaderVersion::InitialVersion.to_u32()
        );
        // Verify reserved field is set to zero
        assert_eq!(header.reserved(), 0);

        // Verify actual data fields are set correctly
        assert!(header.split_endpoints());
    }

    #[test]
    fn test_invalid_header_version() {
        // Create header with invalid version
        let mut invalid_header = Bc5TransformHeaderData::default();
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
        let mut invalid_header = Bc5TransformHeaderData::default();
        invalid_header.set_header_version(Bc5HeaderVersion::InitialVersion.to_u32());
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
        assert_eq!(EmbeddableBc5Details::FORMAT, TransformFormat::Bc5);
    }
}
