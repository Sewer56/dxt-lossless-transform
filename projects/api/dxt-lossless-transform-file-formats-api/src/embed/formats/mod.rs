//! BCx format-specific embeddable implementations.

#[allow(dead_code)]
mod argb8888;
mod bc1;
mod bc2;
#[allow(dead_code)]
mod bc3; // code not ready, placeholder
#[allow(dead_code)]
mod bc7; // code not ready, placeholder
#[allow(dead_code)]
mod rgba8888;

pub(crate) use argb8888::EmbeddableArgb8888Details;
pub(crate) use bc1::EmbeddableBc1Details;
pub(crate) use bc2::EmbeddableBc2Details;
pub(crate) use rgba8888::EmbeddableRgba8888Details;

use super::{EmbedError, TransformFormat, TransformHeader};

/// Trait for transform details that can be embedded in a 4-byte header.
///
/// Each BCx format implements this trait to define how its transform details
/// are packed into the 28 bits of format-specific data in the header.
pub(crate) trait EmbeddableTransformDetails: Sized {
    /// The transform format this implementation is for
    const FORMAT: TransformFormat;

    /// Pack the transform details into a 28-bit value
    fn pack(&self) -> u32;

    /// Unpack transform details from a 28-bit value
    fn unpack(data: u32) -> Result<Self, EmbedError>;

    /// Extract from a complete transform header
    fn from_header(header: TransformHeader) -> Result<Self, EmbedError> {
        match header.format() {
            Some(format) if format == Self::FORMAT => Self::unpack(header.format_data()),
            Some(_) => Err(EmbedError::UnknownFormat),
            None => Err(EmbedError::UnknownFormat),
        }
    }
}
