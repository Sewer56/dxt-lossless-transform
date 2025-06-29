//! BC2 format file format support.

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat};
use bitfield::bitfield;

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
    /// - Bits 2-27: Reserved for future use (26 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
     struct Bc2TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Reserved bits for future use (26 bits)
    reserved, set_reserved: 27, 2;
}

/// BC2 transform details that can be stored in file headers
///
/// BC2 currently has no configurable options, so this is just a marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EmbeddableBc2Details;

impl EmbeddableTransformDetails for EmbeddableBc2Details {
    const FORMAT: TransformFormat = TransformFormat::Bc2;

    fn pack(&self) -> u32 {
        let mut header = Bc2TransformHeaderData::default();
        header.set_header_version(Bc2HeaderVersion::InitialVersion.to_u32());
        header.set_reserved(0);
        header.0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        let header = Bc2TransformHeaderData(data);

        // Validate version
        let _version = Bc2HeaderVersion::from_u32(header.header_version())?;

        // BC2 currently has no configurable options
        Ok(Self)
    }
}
