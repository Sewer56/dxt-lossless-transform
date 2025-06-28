//! This module contains the primitives for embedding information about the transformation used
//! inside file headers. Specifically, this module contains all of the
//! common code that's shared between the various file formats.
//!
//! When we perform a lossless transform on a file, we need to know how to 'undo' it.
//! The various transforms have types such as `Bc1TransformSettings` (from dxt_lossless_transform_bc1 crate)
//! which are passed onto the 'untransform' functions.
//!
//! # An Example
//!
//! In the DDS format for instance (implemented in [`dxt-lossless-transform-dds`] crate), we can embed the
//! transform details into the first 4 bytes of the file.
//!
//! Each DDS starts with a 'magic' header, which is 4 bytes long. If we know (from context) that
//! what we're dealing is a DDS file, then said header becomes insignificant, we can store
//! whatever we want in it.
//!
//! And that happens to be our transform details.
//!
//! ## Header Overwriting and Restoration
//!
//! Format-specific implementations should follow this pattern:
//!
//! 1. **Validation**: Verify the file has the expected header format before embedding
//! 2. **Embed functions**: Overwrite the original file header with transform details
//! 3. **Unembed functions**: Extract transform details AND restore the original header
//!
//! This ensures that after the unembed operation, the file is returned to a valid state
//! and can be processed by standard tools that expect the original header format.
//!
//! **Critical Safety Note**: Embed functions should never blindly overwrite headers.
//! Callers must validate the file format first - never trust file extensions alone.
//!
//! [`dxt-lossless-transform-dds`]: https://github.com/Sewer56/dxt-lossless-transform/tree/main/projects/extensions/dxt-lossless-transform-dds
//!
//! # The Header Format
//!
//! We assume 4 byte headers.
//!
//! The header is packed as a [`u32`] little endian integer.
//! The bits are represented as the following.
//!
//! `u4`  - Transform Format [`TransformFormat`]
//! `u28` - Transform format specific data.
//!
//! Each Transform Format is responsible for versioning itself; this header merely stores the
//! format type itself.
//!
//! Generally it's expected that changes to existing formats will be rare, 28 bits is quite rich,
//! most transforms will not even use half the space. This number was chosen as it's sufficient
//! to store 3 bits for each of BC7's 8 modes, with 4 bits left over for miscellaneous use.
//!
//! There is no 'MAGIC' number here or anything to identify the header; the user has to know
//! in context we're dealing with a valid header.
//!
//! ### Transform Specific Data Representation
//!
//! The transform specific data is represented using bitfield structures.
//! The higher bits are used first, with the first 4 bits reserved for the transform format.
//! Fields are populated from the highest bits down, allowing for future alterations.
use bitfield::bitfield;

// Sub-modules
pub mod embed_error;
pub mod embeddable_transform_details;
pub mod transform_format;

// Re-exports for backwards compatibility and convenience
pub use embed_error::EmbedError;
pub use embeddable_transform_details::EmbeddableTransformDetails;
pub use transform_format::TransformFormat;

/// Size of the transform header in bytes.
///
/// The transform header is always 4 bytes (32 bits) containing:
/// - 4 bits for transform format type
/// - 28 bits for format-specific data
pub const TRANSFORM_HEADER_SIZE: usize = 4;

bitfield! {
    /// Common header structure for all transform formats.
    ///
    /// This is a 32-bit header where:
    /// - Bits 0-3: Transform format type
    /// - Bits 4-31: Format-specific data
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TransformHeader(u32);
    impl Debug;
    u32;

    /// Transform format type (4 bits)
    pub format_raw, set_format_raw: 3, 0;
    /// Format-specific data (28 bits)
    pub format_data, set_format_data: 31, 4;
}

impl TransformHeader {
    /// Create a new transform header with the given format and data.
    pub fn new(format: TransformFormat, data: u32) -> Self {
        let mut header = Self::default();
        header.set_format_raw(format.to_u8() as u32);
        header.set_format_data(data);
        header
    }

    /// Get the transform format from the header.
    pub fn format(&self) -> TransformFormat {
        TransformFormat::from_u8(self.format_raw() as u8)
    }

    /// Read a transform header from a byte pointer.
    ///
    /// Reads the header as a little-endian [`u32`] value as specified in the format.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid for reads of at least [`TRANSFORM_HEADER_SIZE`] bytes
    pub unsafe fn read_from_ptr(ptr: *const u8) -> Self {
        let value = (ptr as *const u32).read_unaligned();
        Self(u32::from_le(value))
    }

    /// Write a transform header to a byte pointer.
    ///
    /// Writes the header as a little-endian [`u32`] value as specified in the format.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid for writes of at least [`TRANSFORM_HEADER_SIZE`] bytes
    pub unsafe fn write_to_ptr(&self, ptr: *mut u8) {
        (ptr as *mut u32).write_unaligned(self.0.to_le());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_format_conversion() {
        assert_eq!(TransformFormat::from_u8(0x00), TransformFormat::Bc1);
        assert_eq!(TransformFormat::from_u8(0x01), TransformFormat::Bc2);
        assert_eq!(TransformFormat::from_u8(0x02), TransformFormat::Bc3);
        assert_eq!(TransformFormat::from_u8(0x03), TransformFormat::Bc7);
        assert_eq!(TransformFormat::from_u8(0x0F), TransformFormat::Reserved15);

        assert_eq!(TransformFormat::Bc1.to_u8(), 0x00);
        assert_eq!(TransformFormat::Bc2.to_u8(), 0x01);
        assert_eq!(TransformFormat::Bc3.to_u8(), 0x02);
        assert_eq!(TransformFormat::Bc7.to_u8(), 0x03);
    }

    #[test]
    fn test_transform_header_bitfield() {
        let header = TransformHeader::new(TransformFormat::Bc1, 0x0ABCDEF0);
        assert_eq!(header.format(), TransformFormat::Bc1);
        assert_eq!(header.format_data(), 0x0ABCDEF0);

        // Test that data is properly masked to 28 bits
        let header2 = TransformHeader::new(TransformFormat::Bc3, 0xFFFFFFFF);
        assert_eq!(header2.format(), TransformFormat::Bc3);
        assert_eq!(header2.format_data(), 0x0FFFFFFF);
    }

    #[test]
    fn test_header_read_write() {
        let mut buffer = [0u8; TRANSFORM_HEADER_SIZE];
        let original = TransformHeader::new(TransformFormat::Bc7, 0x12345678);

        unsafe {
            original.write_to_ptr(buffer.as_mut_ptr());
            let read_back = TransformHeader::read_from_ptr(buffer.as_ptr());
            assert_eq!(original, read_back);
        }
    }

    #[test]
    fn test_little_endian_byte_order() {
        let mut buffer = [0u8; TRANSFORM_HEADER_SIZE];

        // Create a header with known bit pattern: format=0x3 (BC7), data=0x1234567
        let header = TransformHeader::new(TransformFormat::Bc7, 0x1234567);

        unsafe {
            header.write_to_ptr(buffer.as_mut_ptr());
        }

        // The expected little-endian byte representation
        // header.0 should be 0x12345673 (format 0x3 in bits 0-3, data 0x1234567 in bits 4-31)
        // In little-endian: [0x73, 0x56, 0x34, 0x12]
        let expected_value = 0x12345673u32;
        let expected_bytes = expected_value.to_le_bytes();

        assert_eq!(
            buffer, expected_bytes,
            "Header should be written in little-endian byte order"
        );

        // Verify we can read it back correctly
        unsafe {
            let read_back = TransformHeader::read_from_ptr(buffer.as_ptr());
            assert_eq!(read_back, header);
            assert_eq!(read_back.format(), TransformFormat::Bc7);
            assert_eq!(read_back.format_data(), 0x1234567);
        }
    }
}
