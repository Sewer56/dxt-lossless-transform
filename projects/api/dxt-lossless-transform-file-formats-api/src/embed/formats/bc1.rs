//! BC1 format file format support.
//!
//! This module provides BC1-specific implementations of the file format traits.

use crate::embed::{EmbedError, EmbeddableTransformDetails, TransformFormat};
use bitfield::bitfield;
use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Header version for BC1 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Bc1HeaderVersion {
    /// Initial version - supports split color endpoints and decorrelation variants
    InitialVersion = 0,
}

impl Bc1HeaderVersion {
    /// Convert from u32 value
    pub fn from_u32(value: u32) -> Result<Self, EmbedError> {
        match value {
            0 => Ok(Self::InitialVersion),
            _ => Err(EmbedError::CorruptedEmbeddedData),
        }
    }

    /// Convert to u32 value  
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

bitfield! {
    /// Packed BC1 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bit 2: Split colour endpoints flag (1 bit)
    /// - Bits 3-4: Decorrelation variant (2 bits)
    /// - Bits 5-27: Reserved for future use (23 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Bc1TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    pub header_version, set_header_version: 1, 0;
    /// Whether color endpoints were split (1 bit)
    pub split_colour_endpoints, set_split_colour_endpoints: 2;
    /// YCoCg decorrelation variant (0=Variant1, 1=Variant2, 2=Variant3, 3=None) (2 bits)
    pub decorrelation_variant, set_decorrelation_variant: 4, 3;
    /// Reserved bits for future use (23 bits)
    pub reserved, set_reserved: 27, 5;
}

impl Bc1TransformHeaderData {
    /// Convert YCoCgVariant to its packed representation
    fn variant_to_u32(variant: YCoCgVariant) -> u32 {
        match variant {
            // These are 1:1 mappings at time of writing, so a no-op.
            YCoCgVariant::Variant1 => 0,
            YCoCgVariant::Variant2 => 1,
            YCoCgVariant::Variant3 => 2,
            YCoCgVariant::None => 3,
        }
    }

    /// Convert packed representation back to YCoCgVariant
    fn u32_to_variant(value: u32) -> Result<YCoCgVariant, EmbedError> {
        match value {
            0 => Ok(YCoCgVariant::Variant1),
            1 => Ok(YCoCgVariant::Variant2),
            2 => Ok(YCoCgVariant::Variant3),
            3 => Ok(YCoCgVariant::None),
            _ => Err(EmbedError::CorruptedEmbeddedData),
        }
    }

    /// Create [`Bc1TransformHeaderData`] from [`Bc1TransformSettings`]
    pub fn from_transform_settings(settings: &Bc1TransformSettings) -> Self {
        let mut header = Self::default();
        header.set_header_version(Bc1HeaderVersion::InitialVersion.to_u32());
        header.set_split_colour_endpoints(settings.split_colour_endpoints);
        header.set_decorrelation_variant(Self::variant_to_u32(settings.decorrelation_mode));
        header.set_reserved(0);
        header
    }

    /// Convert [`Bc1TransformHeaderData`] to [`Bc1TransformSettings`]
    pub fn to_transform_settings(&self) -> Result<Bc1TransformSettings, EmbedError> {
        // Validate version (from_u32 will error on invalid version)
        let _version = Bc1HeaderVersion::from_u32(self.header_version())?;
        let decorrelation_mode = Self::u32_to_variant(self.decorrelation_variant())?;

        Ok(Bc1TransformSettings {
            decorrelation_mode,
            split_colour_endpoints: self.split_colour_endpoints(),
        })
    }
}

/// Wrapper type for BC1 transform settings that can be stored in file headers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmbeddableBc1Details(pub Bc1TransformSettings);

impl From<Bc1TransformSettings> for EmbeddableBc1Details {
    fn from(settings: Bc1TransformSettings) -> Self {
        Self(settings)
    }
}

impl From<EmbeddableBc1Details> for Bc1TransformSettings {
    fn from(embeddable: EmbeddableBc1Details) -> Self {
        embeddable.0
    }
}

impl EmbeddableTransformDetails for EmbeddableBc1Details {
    const FORMAT: TransformFormat = TransformFormat::Bc1;

    fn pack(&self) -> u32 {
        Bc1TransformHeaderData::from_transform_settings(&self.0).0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        Ok(Self(Bc1TransformHeaderData(data).to_transform_settings()?))
    }
}

impl EmbeddableBc1Details {
    /// Create a [`TransformHeader`] from this embeddable BC1 details.
    ///
    /// This is a convenience method for external format handlers.
    ///
    /// [`TransformHeader`]: crate::embed::TransformHeader
    pub fn to_header(&self) -> crate::embed::TransformHeader {
        crate::embed::TransformHeader::new(Self::FORMAT, self.pack())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_all_possible_transform_details() {
        for settings in Bc1TransformSettings::all_combinations() {
            let embeddable = EmbeddableBc1Details(settings);

            let packed = embeddable.pack();
            let recovered = EmbeddableBc1Details::unpack(packed).unwrap();
            assert_eq!(settings, recovered.0, "Failed for settings {settings:?}",);
        }
    }

    #[test]
    fn test_header_version_and_reserved_fields() {
        let settings = Bc1TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let header = Bc1TransformHeaderData::from_transform_settings(&settings);

        // Verify version is set correctly
        assert_eq!(
            header.header_version(),
            Bc1HeaderVersion::InitialVersion.to_u32()
        );
        // Verify reserved field is set to zero
        assert_eq!(header.reserved(), 0);

        // Verify actual data fields are set correctly
        assert!(header.split_colour_endpoints());
        assert_eq!(header.decorrelation_variant(), 0); // Variant1 = 0
    }

    #[test]
    fn test_invalid_header_version() {
        // Create header with invalid version
        let mut invalid_header = Bc1TransformHeaderData::default();
        invalid_header.set_header_version(3); // Invalid version (only 0 is valid)

        // Should fail to convert to transform settings due to invalid version
        assert_eq!(
            invalid_header.to_transform_settings(),
            Err(EmbedError::CorruptedEmbeddedData)
        );
    }

    #[test]
    fn test_format_association() {
        // Verify the format association is correct
        assert_eq!(EmbeddableBc1Details::FORMAT, TransformFormat::Bc1);
    }
}
