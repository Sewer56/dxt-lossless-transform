//! Trait for transform details that can be embedded in file headers.

use super::{EmbedError, TransformFormat, TransformHeader};

/// Trait for transform details that can be embedded in a 4-byte header.
///
/// Each BCx format implements this trait to define how its transform details
/// are packed into the 28 bits of format-specific data in the header.
pub trait EmbeddableTransformDetails: Sized {
    /// The transform format this implementation is for
    const FORMAT: TransformFormat;

    /// Pack the transform details into a 28-bit value
    fn pack(&self) -> u32;

    /// Unpack transform details from a 28-bit value
    fn unpack(data: u32) -> Result<Self, EmbedError>;

    /// Convert to a complete transform header
    fn to_header(&self) -> TransformHeader {
        TransformHeader::new(Self::FORMAT, self.pack())
    }

    /// Extract from a complete transform header
    fn from_header(header: TransformHeader) -> Result<Self, EmbedError> {
        if header.format() != Self::FORMAT {
            return Err(EmbedError::FormatMismatch);
        }
        Self::unpack(header.format_data())
    }
}
