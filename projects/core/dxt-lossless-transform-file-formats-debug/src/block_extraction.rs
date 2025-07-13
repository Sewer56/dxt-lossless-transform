//! Block extraction trait for debug and analysis operations.
//!
//! This module provides a trait for extracting raw block data from file formats
//! for debug, analysis, and testing purposes.

use alloc::{format, string::String};
use dxt_lossless_transform_file_formats_api::{embed::TransformFormat, error::TransformResult};

/// Block extraction result containing raw block data and format information.
#[derive(Debug)]
pub struct ExtractedBlocks<'a> {
    /// The raw block data
    pub data: &'a [u8],
    /// The format of the blocks
    pub format: TransformFormat,
}

impl<'a> ExtractedBlocks<'a> {
    /// Create a new `ExtractedBlocks` instance.
    ///
    /// # Parameters
    ///
    /// - `data`: The raw block data slice
    /// - `format`: The format of the blocks
    ///
    /// # Remarks
    ///
    /// The slice must contain valid block data properly aligned for the specified format.
    pub fn new(data: &'a [u8], format: TransformFormat) -> Self {
        Self { data, format }
    }
}

/// Filter for specifying which [`TransformFormat`]s to extract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformFormatFilter {
    /// Extract only BC1 blocks
    Bc1,
    /// Extract only BC2 blocks
    Bc2,
    /// Extract only BC3 blocks
    Bc3,
    /// Extract only BC7 blocks
    Bc7,
    /// Extract only BC6H blocks
    Bc6H,
    /// Extract only RGBA8888 pixels
    Rgba8888,
    /// Extract only BGRA8888 pixels
    Bgra8888,
    /// Extract all supported [`TransformFormat`]s
    All,
}

impl TransformFormatFilter {
    /// Check if this filter accepts the given format.
    pub fn accepts(&self, format: TransformFormat) -> bool {
        matches!(
            (self, format),
            (TransformFormatFilter::Bc1, TransformFormat::Bc1)
                | (TransformFormatFilter::Bc2, TransformFormat::Bc2)
                | (TransformFormatFilter::Bc3, TransformFormat::Bc3)
                | (TransformFormatFilter::Bc7, TransformFormat::Bc7)
                | (TransformFormatFilter::Bc6H, TransformFormat::Bc6H)
                | (TransformFormatFilter::Rgba8888, TransformFormat::Rgba8888)
                | (TransformFormatFilter::Bgra8888, TransformFormat::Bgra8888)
                | (TransformFormatFilter::All, _)
        )
    }
}

impl core::str::FromStr for TransformFormatFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bc1" => Ok(TransformFormatFilter::Bc1),
            "bc2" => Ok(TransformFormatFilter::Bc2),
            "bc3" => Ok(TransformFormatFilter::Bc3),
            "bc7" => Ok(TransformFormatFilter::Bc7),
            "bc6h" => Ok(TransformFormatFilter::Bc6H),
            "rgba8888" => Ok(TransformFormatFilter::Rgba8888),
            "bgra8888" => Ok(TransformFormatFilter::Bgra8888),
            "all" => Ok(TransformFormatFilter::All),
            _ => Err(format!(
                "Invalid TransformFormat filter: {s}. Valid types are: bc1, bc2, bc3, bc7, bc6h, rgba8888, bgra8888, all"
            )),
        }
    }
}

/// Trait for extracting raw block data from file formats.
///
/// This trait is designed for debug, analysis, and testing purposes where you need
/// direct access to the raw block data within a file format. It's separated into
/// this debug crate to avoid including debug-only functionality in production builds.
///
/// # Safety
///
/// Implementations of this trait work with raw pointers and must ensure memory safety.
/// The extracted block data must remain valid for the lifetime specified in the result.
///
/// # Example Implementation
///
/// ```rust
/// use dxt_lossless_transform_file_formats_debug::{
///     FileFormatBlockExtraction, TransformFormatFilter, ExtractedBlocks,
/// };
/// use dxt_lossless_transform_file_formats_api::{
///     embed::TransformFormat,
///     error::{TransformResult, TransformError, FormatHandlerError},
/// };
///
/// // Example custom handler
/// struct MyFormatHandler;
///
/// impl FileFormatBlockExtraction for MyFormatHandler {
///     fn extract_blocks<'a>(
///         &self,
///         data: &'a [u8],
///         filter: TransformFormatFilter,
///     ) -> TransformResult<Option<ExtractedBlocks<'a>>> {
///         // Parse your custom format header
///         if data.len() < 4 {
///             return Err(TransformError::FormatHandler(
///                 FormatHandlerError::UnknownFileFormat
///             ));
///         }
///         
///         // Determine format from header
///         let transform_format = TransformFormat::Bc1; // Example
///         
///         if !filter.accepts(transform_format) {
///             return Ok(None);
///         }
///         
///         // Extract block data
///         let data_offset = 4; // Skip header
///         let block_data = &data[data_offset..];
///         
///         let extracted = ExtractedBlocks::new(block_data, transform_format);
///         
///         Ok(Some(extracted))
///     }
/// }
/// ```
pub trait FileFormatBlockExtraction {
    /// Extract raw block data from the file format.
    ///
    /// This method parses the file format and extracts the raw block data
    /// that matches the specified filter. If the file format doesn't contain
    /// blocks matching the filter, returns `Ok(None)`.
    ///
    /// # Parameters
    ///
    /// - `data`: The complete file data to extract blocks from
    /// - `filter`: Filter specifying which [`TransformFormat`]s to extract
    ///
    /// # Returns
    ///
    /// - `Ok(Some(blocks))`: Successfully extracted blocks matching the filter
    /// - `Ok(None)`: File format doesn't contain blocks matching the filter
    /// - `Err(error)`: File format is invalid or extraction failed
    ///
    /// # Safety
    ///
    /// The returned `ExtractedBlocks` contains raw pointers that are valid
    /// for the lifetime of the input `data`. Callers must ensure the input
    /// data remains valid for the entire lifetime of the returned blocks.
    fn extract_blocks<'a>(
        &self,
        data: &'a [u8],
        filter: TransformFormatFilter,
    ) -> TransformResult<Option<ExtractedBlocks<'a>>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_format_filter_accepts() {
        assert!(TransformFormatFilter::Bc1.accepts(TransformFormat::Bc1));
        assert!(!TransformFormatFilter::Bc1.accepts(TransformFormat::Bc2));
        assert!(TransformFormatFilter::All.accepts(TransformFormat::Bc1));
        assert!(TransformFormatFilter::All.accepts(TransformFormat::Bc7));
    }

    #[test]
    fn test_transform_format_filter_from_str() {
        assert_eq!(
            "bc1".parse::<TransformFormatFilter>().unwrap(),
            TransformFormatFilter::Bc1
        );
        assert_eq!(
            "BC3".parse::<TransformFormatFilter>().unwrap(),
            TransformFormatFilter::Bc3
        );
        assert_eq!(
            "all".parse::<TransformFormatFilter>().unwrap(),
            TransformFormatFilter::All
        );
        assert!("invalid".parse::<TransformFormatFilter>().is_err());
    }

    #[test]
    fn test_extracted_blocks_data_access() {
        let data = [1u8, 2, 3, 4];
        let extracted = ExtractedBlocks::new(&data, TransformFormat::Bc1);
        assert_eq!(extracted.data, &data);
        assert_eq!(extracted.format, TransformFormat::Bc1);
    }
}
