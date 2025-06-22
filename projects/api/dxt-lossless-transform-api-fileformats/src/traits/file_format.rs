//! File format handling traits for file format operations.

use core::fmt::Debug;
use dxt_lossless_transform_api_common::embed::{EmbedError, TransformHeader};

/// Trait for handling file format-specific header operations.
///
/// This trait abstracts over different file formats (DDS, KTX, etc.) and provides
/// a common interface for storing and extracting transform details from file headers.
///
/// # Safety
///
/// Implementations must ensure that:
/// - Header storage/extraction operations are reversible
/// - Original file headers are properly restored after extraction
/// - File format validation is performed before attempting header modification
pub trait FileFormatHandler: Send + Sync {
    /// Information about the detected file format
    type Info: Send + Sync + Debug;

    /// The compression format enum (e.g., DdsFormat with BC1, BC2, etc.)
    type Format: Copy + Debug + PartialEq + Eq + Send + Sync;

    /// Error type specific to this file format
    type Error: Into<EmbedError> + Debug;

    /// Detect if the data represents this file format and extract format information.
    ///
    /// # Parameters
    ///
    /// - `data`: The file data to analyze
    ///
    /// # Returns
    ///
    /// [`Some(Info)`] if the data is a valid file of this format, [`None`] otherwise.
    fn detect_format(data: &[u8]) -> Option<Self::Info>;

    /// Get the offset where the actual texture data begins.
    ///
    /// # Parameters
    ///
    /// - `info`: Format information returned by [`detect_format`]
    ///
    /// # Returns
    ///
    /// The byte offset from the start of the file where texture data begins.
    fn get_data_offset(info: &Self::Info) -> usize;

    /// Get the compression format from the file information.
    ///
    /// # Parameters
    ///
    /// - `info`: Format information returned by [`detect_format`]
    ///
    /// # Returns
    ///
    /// The compression format (e.g., BC1, BC2, etc.).
    fn get_format(info: &Self::Info) -> Self::Format;

    /// Store a transform header into the file, overwriting the original header.
    ///
    /// This function **overwrites** the original file header with transform details.
    /// The original header can be restored using [`extract_transform_header`].
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid for reads and writes of at least the header size
    /// - `ptr` must point to a valid file of this format
    /// - **The caller must verify the file format** using [`detect_format`] before calling this function
    /// - Do not rely solely on file extensions - always validate the actual header content
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the file data (the header will be overwritten)
    /// - `header`: Transform header to store
    ///
    /// # Errors
    ///
    /// Returns an error if the storage operation fails.
    ///
    /// [`extract_transform_header`]: Self::extract_transform_header
    unsafe fn embed_transform_header(
        ptr: *mut u8,
        header: TransformHeader,
    ) -> Result<(), Self::Error>;

    /// Extract transform header from the file and restore the original header.
    ///
    /// This function reads the transform details from the file header and then **restores**
    /// the original file header, returning the file to its original state.
    ///
    /// # Safety
    ///
    /// - `ptr` must be valid for reads and writes of at least the header size
    /// - The file must have been previously modified with transform details using [`embed_transform_header`]
    /// - **The caller must ensure the data contains valid stored transform details**
    /// - Do not call this on arbitrary files - only on files that were previously modified
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the file data with stored details (the header will be restored)
    ///
    /// # Returns
    ///
    /// - [`Ok((TransformHeader, Info))`] on success, containing the extracted header and file info
    /// - [`Err`] on failure
    ///
    /// # Remarks
    ///
    /// After successful extraction, the file header is restored to its original
    /// state, making the file a valid file again for standard tools.
    ///
    /// [`embed_transform_header`]: Self::embed_transform_header
    unsafe fn unembed_transform_header(
        ptr: *mut u8,
    ) -> Result<(TransformHeader, Self::Info), Self::Error>;
}

/// Convenience trait for file format filters.
///
/// This trait allows file format handlers to provide filtering capabilities
/// for batch operations.
pub trait FileFormatFilter: Copy + Clone + Debug + PartialEq + Eq {
    type Format: Copy + Debug + PartialEq + Eq;

    /// Check if the given format should be processed according to this filter.
    fn matches(&self, format: Self::Format) -> bool;

    /// Get all formats that this filter accepts.
    fn accepted_formats(&self) -> &[Self::Format];
}
