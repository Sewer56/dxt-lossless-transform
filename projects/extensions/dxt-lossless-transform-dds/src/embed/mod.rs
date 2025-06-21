//! Embeds information about the various transforms into the DDS header.
//!
//! This module contains the code to pack information about how a file was transformed into the DDS
//! header of each file; such that the file can be detransformed later.
//!
//! # Explanation
//!
//! Each DDS starts with a 'magic' header, which is 4 bytes long.
//! This header is defined as [`DDS_MAGIC`] in this library.
//!
//! If we know (from context) that the input file is a DDS file at load time, then said
//! header becomes insignificant. In which case, we can use those 4 bytes to store the information we need.
//!
//! This module is responsible for embedding this information into the DDS header.
//!
//! [`DDS_MAGIC`]: crate::dds::constants::DDS_MAGIC

pub mod bc1;

use crate::dds::DdsFormat;

// Re-export the functions from BC1 module
pub use bc1::{embed_bc1_details, unembed_bc1_details};

// Re-export transform details from the BC1 transform crate
pub use dxt_lossless_transform_bc1::Bc1DetransformDetails;

/// Error types for embed/unembed operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedError {
    /// The provided data is not a valid DDS file.
    NotADds,
    /// The format specified is not supported for embedding.
    UnsupportedFormat,
    /// The embedded data is corrupted or invalid.
    CorruptedEmbeddedData,
}

/// Generic function to embed transform details based on DDS format.
///
/// This is a convenience function that automatically calls the appropriate
/// format-specific embed function based on the format parameter.
///
/// Currently only BC1 is supported.
///
/// # Safety
///
/// - `ptr` must be valid for reads and writes of `len` bytes
/// - `ptr` must point to a valid DDS file
/// - The format must match the actual DDS file format
///
/// # Parameters
///
/// - `ptr`: Pointer to the DDS data (will be modified)
/// - `len`: Length of the DDS data
/// - `format`: The DDS format of the file
/// - `bc1_details`: BC1 details to embed (only used if format is BC1)
///
/// # Returns
///
/// - `Ok(())` on success
/// - `Err(EmbedError)` on failure
pub unsafe fn embed_details_by_format(
    ptr: *mut u8,
    len: usize,
    format: DdsFormat,
    bc1_details: Bc1DetransformDetails,
) -> Result<(), EmbedError> {
    match format {
        DdsFormat::BC1 => embed_bc1_details(ptr, len, bc1_details),
        DdsFormat::BC2 | DdsFormat::BC3 | DdsFormat::BC7 => Err(EmbedError::UnsupportedFormat),
        DdsFormat::NotADds | DdsFormat::Unknown => Err(EmbedError::UnsupportedFormat),
    }
}
