//! This module contains the primitives for embedding information about the transformation used
//! inside file headers. Specifically, this module contains all of the
//! common code that's shared between the various file formats.
//!
//! When we perform a lossless transform on a file, we need to know how to 'undo' it.
//! The various transforms have types such as [`Bc1DetransformDetails`] which are passed onto the
//! 'untransform' functions.
//!
//! [`Bc1DetransformDetails`]: https://docs.rs/dxt-lossless-transform-bc1/latest/dxt_lossless_transform_bc1/struct.Bc1DetransformDetails.html
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
//! 1. **Embed functions**: Overwrite the original file header with transform details
//! 2. **Unembed functions**: Extract transform details AND restore the original header
//!
//! This ensures that after the unembed operation, the file is returned to a valid state
//! and can be processed by standard tools that expect the original header format.
//!
//! [`DDS_MAGIC`]: https://docs.rs/dxt-lossless-transform-dds/latest/dxt_lossless_transform_dds/dds/constants/constant.DDS_MAGIC.html
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
//!
//! Here's an example of how BC1 format implements its 28-bit format-specific data:
//!
//! ```
//! use bitfield::bitfield;
//! # use dxt_lossless_transform_api_common::embed::{EmbeddableTransformDetails, TransformFormat, EmbedError};
//!
//! /// Header version for BC1 format
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! #[repr(u8)]
//! pub enum Bc1HeaderVersion {
//!     /// Initial version - supports split color endpoints and 3 decorrelation variants
//!     InitialVersion = 0,
//! }
//!
//! impl Bc1HeaderVersion {
//!     /// Convert from u32 value
//!     pub fn from_u32(value: u32) -> Result<Self, EmbedError> {
//!         match value {
//!             0 => Ok(Self::InitialVersion),
//!             _ => Err(EmbedError::CorruptedEmbeddedData),
//!         }
//!     }
//!
//!     /// Convert to u32 value  
//!     pub fn to_u32(self) -> u32 {
//!         self as u32
//!     }
//! }
//!
//! bitfield! {
//!     /// Packed BC1 transform data for embedding in headers.
//!     ///
//!     /// Bit layout (within the 28-bit format data):
//!     /// - Bits 0-1: Header version (2 bits)
//!     /// - Bit 2: Split colour endpoints flag (1 bit)
//!     /// - Bits 3-4: Decorrelation variant (2 bits)
//!     /// - Bits 5-27: Reserved for future use (23 bits)
//!     #[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
//!     pub struct Bc1TransformHeaderData(u32);
//!     impl Debug;
//!     u32;
//!
//!     /// Header version (2 bits)
//!     pub header_version, set_header_version: 1, 0;
//!     /// Whether color endpoints were split (1 bit)
//!     pub split_colour_endpoints, set_split_colour_endpoints: 2;
//!     /// YCoCg decorrelation variant (0=Variant1, 1=Variant2, 2=Variant3, 3=None) (2 bits)
//!     pub decorrelation_variant, set_decorrelation_variant: 4, 3;
//!     /// Reserved bits for future use (23 bits)
//!     pub reserved, set_reserved: 27, 5;
//! }
//!
//! /// Example wrapper type for BC1 transform details
//! #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//! pub struct ExampleBc1Details {
//!     pub split_colours: bool,
//!     pub decorrelation: u8,
//! }
//!
//! impl EmbeddableTransformDetails for ExampleBc1Details {
//!     const FORMAT: TransformFormat = TransformFormat::Bc1;
//!
//!     fn pack(&self) -> u32 {
//!         let mut header = Bc1TransformHeaderData::default();
//!         header.set_header_version(Bc1HeaderVersion::InitialVersion.to_u32());
//!         header.set_split_colour_endpoints(self.split_colours);
//!         header.set_decorrelation_variant(self.decorrelation as u32);
//!         header.set_reserved(0);
//!         header.0
//!     }
//!
//!     fn unpack(data: u32) -> Result<Self, EmbedError> {
//!         let header = Bc1TransformHeaderData(data);
//!         
//!         // Validate version
//!         let _version = Bc1HeaderVersion::from_u32(header.header_version())?;
//!         
//!         Ok(Self {
//!             split_colours: header.split_colour_endpoints(),
//!             decorrelation: header.decorrelation_variant() as u8,
//!         })
//!     }
//! }
//! ```
//!
//! # Why not pad the files
//!
//! Padding the files is possible, and technically safer; but that leaves performance on the table,
//! and can lead to texture data no longer being aligned. Aligned reads/writes help us gain
//! extra purr~formance 😼. Don't want to miss out on that 😉.
//!
//! If file or transform formats with different max sizes come, a second variant will be created
//! which does pad; but for now, it's not needed.

use bitfield::bitfield;

// Sub-modules
pub mod embed_error;
pub mod embeddable_transform_details;
pub mod transform_format;

// Re-exports for backwards compatibility and convenience
pub use embed_error::EmbedError;
pub use embeddable_transform_details::EmbeddableTransformDetails;
pub use transform_format::TransformFormat;

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
    /// - `ptr` must be valid for reads of at least 4 bytes
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
    /// - `ptr` must be valid for writes of at least 4 bytes
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
        let mut buffer = [0u8; 4];
        let original = TransformHeader::new(TransformFormat::Bc7, 0x12345678);

        unsafe {
            original.write_to_ptr(buffer.as_mut_ptr());
            let read_back = TransformHeader::read_from_ptr(buffer.as_ptr());
            assert_eq!(original, read_back);
        }
    }

    #[test]
    fn test_little_endian_byte_order() {
        let mut buffer = [0u8; 4];

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
