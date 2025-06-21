//! Trait for types that can be embedded into transform headers.

use crate::embed::{EmbedError, TransformFormat, TransformHeader};

/// Trait for types that can be embedded into a transform header.
///
/// Implementors of this trait can be packed into the 28-bit format-specific
/// data section of a [`TransformHeader`].
pub trait EmbeddableTransformDetails: Sized {
    /// The transform format this type is associated with.
    const FORMAT: TransformFormat;

    /// Pack the transform details into a 28-bit value.
    ///
    /// The returned value must fit within 28 bits (i.e., be less than 2^28).
    fn pack(&self) -> u32;

    /// Unpack transform details from a 28-bit value.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if unpacking succeeds, or [`EmbedError`] if
    /// the data is invalid or corrupted.
    fn unpack(data: u32) -> Result<Self, EmbedError>;

    /// Create a complete transform header from these details.
    fn to_header(&self) -> TransformHeader {
        TransformHeader::new(Self::FORMAT, self.pack())
    }

    /// Extract transform details from a complete header.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if the header format matches and unpacking succeeds,
    /// or [`EmbedError`] if the format doesn't match or data is invalid.
    fn from_header(header: TransformHeader) -> Result<Self, EmbedError> {
        if header.format() != Self::FORMAT {
            return Err(EmbedError::InvalidHeaderFormat);
        }
        Self::unpack(header.format_data())
    }
}
