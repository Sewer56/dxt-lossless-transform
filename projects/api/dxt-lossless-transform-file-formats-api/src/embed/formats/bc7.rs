//! BC7 format file format support.
//!
//! BC7 is not yet fully implemented in the transform library,
//! so this is a placeholder for future use.

use super::EmbeddableTransformDetails;
use crate::embed::{EmbedError, TransformFormat};
use bitfield::bitfield;

/// Placeholder BC7 untransform details
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Bc7UntransformDetails;

/// Header version for BC7 format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum Bc7HeaderVersion {
    /// Initial version
    InitialVersion = 0,
}

impl Bc7HeaderVersion {
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
    /// Packed BC7 transform data for storage in headers.
    ///
    /// Bit layout (within the 28-bit format data):
    /// - Bits 0-1: Header version (2 bits)
    /// - Bits 2-27: Reserved for BC7 mode masks and settings (26 bits)
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    struct Bc7TransformHeaderData(u32);
    impl Debug;
    u32;

    /// Header version (2 bits)
    header_version, set_header_version: 1, 0;
    /// Reserved bits for future BC7 implementation (26 bits)
    reserved, set_reserved: 27, 2;
}

/// Wrapper type for BC7 untransform details that can be stored in file headers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EmbeddableBc7Details(Bc7UntransformDetails);

impl From<Bc7UntransformDetails> for EmbeddableBc7Details {
    fn from(details: Bc7UntransformDetails) -> Self {
        Self(details)
    }
}

impl From<EmbeddableBc7Details> for Bc7UntransformDetails {
    fn from(embeddable: EmbeddableBc7Details) -> Self {
        embeddable.0
    }
}

impl EmbeddableTransformDetails for EmbeddableBc7Details {
    const FORMAT: TransformFormat = TransformFormat::Bc7;

    fn pack(&self) -> u32 {
        let mut header = Bc7TransformHeaderData::default();
        header.set_header_version(Bc7HeaderVersion::InitialVersion.to_u32());
        header.set_reserved(0);
        header.0
    }

    fn unpack(data: u32) -> Result<Self, EmbedError> {
        let header = Bc7TransformHeaderData(data);

        // Validate version
        let _version = Bc7HeaderVersion::from_u32(header.header_version())?;

        // BC7 is not yet implemented
        Ok(Self(Bc7UntransformDetails))
    }
}
