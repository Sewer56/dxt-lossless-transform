//! BC2 format file format support.

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat};
use bitfield::bitfield;
use dxt_lossless_transform_bc2::Bc2TransformSettings;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Header version for BC2 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bc2HeaderVersion {
    /// Initial version
    InitialVersion = 0,
}

impl Bc2HeaderVersion {
    fn from_u32(value: u32) -> Result<Self, EmbedError> {
        match value {
            0 => Ok(Self::InitialVersion),
            _ => Err(EmbedError::CorruptedEmbeddedData),
        }
    }

    fn to_u32(self) -> u32 {
        self as u32
    }
}

bitfield! {
    /// Packed BC2 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bit 2: Split colour endpoints (1 bit)
    /// - Bits 3-4: Decorrelation mode (2 bits, [`YCoCgVariant`] as u8)
    /// - Bits 5-27: Reserved for future use (23 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
     struct Bc2TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Split colour endpoints flag (1 bit)
    split_colour_endpoints, set_split_colour_endpoints: 2;
    /// Decorrelation mode (2 bits)
    decorrelation_mode, set_decorrelation_mode: 4, 3;
    /// Reserved bits for future use (23 bits)
    reserved, set_reserved: 27, 5;
}

/// BC2 transform details that can be stored in file headers
///
/// Contains the BC2 transform settings that were used during compression.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct EmbeddableBc2Details(Bc2TransformSettings);

impl Bc2TransformHeaderData {
    /// Convert [`YCoCgVariant`] to u32 representation
    fn variant_to_u32(variant: YCoCgVariant) -> u32 {
        match variant {
            // These are 1:1 mappings at time of writing, so a no-op.
            YCoCgVariant::Variant1 => 0,
            YCoCgVariant::Variant2 => 1,
            YCoCgVariant::Variant3 => 2,
            YCoCgVariant::None => 3,
        }
    }

    /// Convert u32 to [`YCoCgVariant`]
    fn u32_to_variant(value: u32) -> Result<YCoCgVariant, EmbedError> {
        match value {
            0 => Ok(YCoCgVariant::Variant1),
            1 => Ok(YCoCgVariant::Variant2),
            2 => Ok(YCoCgVariant::Variant3),
            3 => Ok(YCoCgVariant::None),
            _ => Err(EmbedError::CorruptedEmbeddedData),
        }
    }

    /// Create [`Bc2TransformHeaderData`] from [`Bc2TransformSettings`]
    fn from_transform_settings(settings: &Bc2TransformSettings) -> Self {
        let mut header = Self::default();
        header.set_header_version(Bc2HeaderVersion::InitialVersion.to_u32());
        header.set_decorrelation_mode(Self::variant_to_u32(settings.decorrelation_mode));
        header.set_split_colour_endpoints(settings.split_colour_endpoints);
        header.set_reserved(0);
        header
    }

    /// Convert [`Bc2TransformHeaderData`] to [`Bc2TransformSettings`]
    fn to_transform_settings(self) -> Result<Bc2TransformSettings, EmbedError> {
        // Validate version (from_u32 will error on invalid version)
        let _version = Bc2HeaderVersion::from_u32(self.header_version())?;
        let decorrelation_mode = Self::u32_to_variant(self.decorrelation_mode())?;

        Ok(Bc2TransformSettings {
            decorrelation_mode,
            split_colour_endpoints: self.split_colour_endpoints(),
        })
    }
}

impl EmbeddableTransformDetails for EmbeddableBc2Details {
    const FORMAT: TransformFormat = TransformFormat::Bc2;

    fn pack(&self) -> u32 {
        Bc2TransformHeaderData::from_transform_settings(&self.0).0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        Ok(Self(Bc2TransformHeaderData(data).to_transform_settings()?))
    }
}

impl EmbeddableBc2Details {
    /// Create a [`TransformHeader`] from this embeddable BC2 details (internal use only).
    ///
    /// [`TransformHeader`]: crate::embed::TransformHeader
    pub(crate) fn to_header(self) -> crate::embed::TransformHeader {
        crate::embed::TransformHeader::new(Self::FORMAT, self.pack())
    }

    /// Create embeddable details from BC2 transform settings.
    ///
    /// # Parameters
    /// - `settings`: The BC2 transform settings to embed
    ///
    /// # Returns
    /// Embeddable details containing the settings
    pub(crate) fn from_settings(settings: Bc2TransformSettings) -> Self {
        Self(settings)
    }

    /// Convert to core BC1 transform settings (internal use only)
    pub(crate) fn to_settings(self) -> Bc2TransformSettings {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_all_possible_transform_details() {
        for settings in Bc2TransformSettings::all_combinations() {
            let embeddable = EmbeddableBc2Details::from_settings(settings);

            let packed = embeddable.pack();
            let recovered = EmbeddableBc2Details::unpack(packed).unwrap();
            assert_eq!(
                settings,
                recovered.to_settings(),
                "Failed for settings {settings:?}",
            );
        }
    }

    #[test]
    fn test_header_version_and_reserved_fields() {
        let settings = Bc2TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let header = Bc2TransformHeaderData::from_transform_settings(&settings);

        // Verify version is set correctly
        assert_eq!(
            header.header_version(),
            Bc2HeaderVersion::InitialVersion.to_u32()
        );
        // Verify reserved field is set to zero
        assert_eq!(header.reserved(), 0);

        // Verify actual data fields are set correctly
        assert!(header.split_colour_endpoints());
        assert_eq!(header.decorrelation_mode(), 0); // Variant1 = 0
    }

    #[test]
    fn test_invalid_header_version() {
        // Create header with invalid version
        let mut invalid_header = Bc2TransformHeaderData::default();
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
        assert_eq!(EmbeddableBc2Details::FORMAT, TransformFormat::Bc2);
    }
}
