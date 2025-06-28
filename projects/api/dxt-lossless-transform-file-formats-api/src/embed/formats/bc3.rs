//! BC3 format file format support.

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat};
use bitfield::bitfield;

/// Header version for BC3 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bc3HeaderVersion {
    /// Initial version
    InitialVersion = 0,
}

impl Bc3HeaderVersion {
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
    /// Packed BC3 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bits 2-27: Reserved for future use (26 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    struct Bc3TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Reserved bits for future use (26 bits)
    reserved, set_reserved: 27, 2;
}

/// BC3 transform details that can be stored in file headers
///
/// BC3 currently has no configurable options, so this is just a marker type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EmbeddableBc3Details;

impl EmbeddableTransformDetails for EmbeddableBc3Details {
    const FORMAT: TransformFormat = TransformFormat::Bc3;

    fn pack(&self) -> u32 {
        let mut header = Bc3TransformHeaderData::default();
        header.set_header_version(Bc3HeaderVersion::InitialVersion.to_u32());
        header.set_reserved(0);
        header.0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        let header = Bc3TransformHeaderData(data);

        // Validate version
        let _version = Bc3HeaderVersion::from_u32(header.header_version())?;

        // BC3 currently has no configurable options
        Ok(Self)
    }
}
