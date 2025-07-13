//! File I/O operations for block extraction.
//!
//! This module provides file I/O operations for extracting raw block data from files
//! for debug, analysis, and testing purposes.

use crate::{FileFormatBlockExtraction, TransformFormatCheck, TransformFormatFilter};
use dxt_lossless_transform_file_formats_api::{
    embed::TransformFormat,
    file_io::{extract_lowercase_extension, FileOperationError, FileOperationResult},
    handlers::FileFormatDetection,
    TransformError,
};
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::path::Path;

/// Extracts BC1-BC7 blocks from a file using the file-formats-api.
/// Raw block data from found files is passed to the `test_fn` parameter for processing.
///
/// This function uses memory mapping for optimal performance and supports any file format handler
/// that implements both [`FileFormatDetection`] and [`FileFormatBlockExtraction`].
///
/// # Arguments
///
/// * `file_path` - Path to the file to extract blocks from
/// * `handlers` - Array of file format handlers to try for block extraction
/// * `filter` - Filter specifying which block formats to extract
/// * `test_fn` - Callback function that receives the extracted block data as a slice
///
/// # Returns
///
/// * `Ok(())` - Successfully extracted and processed blocks
/// * `Err(TransformError::IgnoredByFilter)` - File format doesn't match filter or no handler supports it
/// * `Err(other)` - File I/O error or block extraction failed
///
/// # Example
///
/// ```rust,ignore
/// // Note: This example requires dxt-lossless-transform-dds with the
/// // "debug-block-extraction" feature enabled
/// use dxt_lossless_transform_file_formats_debug::{
///     extract_blocks_from_file_format,
///     TransformFormatFilter,
/// };
/// use dxt_lossless_transform_file_formats_api::{
///     embed::TransformFormat,
///     TransformError,
/// };
/// use dxt_lossless_transform_dds::DdsHandler;
/// use std::path::Path;
///
/// fn example_extract(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
///     let handlers = [DdsHandler];
///     let result = extract_blocks_from_file_format(
///         file_path,
///         &handlers,
///         TransformFormatFilter::Bc1,
///         |data: &[u8], format: TransformFormat| -> Result<(), TransformError> {
///             // Process the block data here
///             println!("Extracted {} bytes of {:?} data", data.len(), format);
///             Ok(())
///         }
///     );
///     match result {
///         Ok(()) => println!("Successfully extracted blocks"),
///         Err(e) => println!("Failed to extract: {:?}", e),
///     }
///     Ok(())
/// }
/// ```
pub fn extract_blocks_from_file_format<H, TFunction>(
    file_path: &Path,
    handlers: &[H],
    filter: TransformFormatFilter,
    mut test_fn: TFunction,
) -> FileOperationResult<()>
where
    H: FileFormatDetection + FileFormatBlockExtraction,
    TFunction: FnMut(&[u8], TransformFormat) -> Result<(), TransformError>,
{
    // Use file-formats-api to open the file
    let source_handle = ReadOnlyFileHandle::open(file_path)?;
    let source_size = source_handle.size()? as usize;
    let source_mapping = ReadOnlyMmap::new(&source_handle, 0, source_size)?;
    let data = source_mapping.as_slice();

    // Get file extension for handler detection
    let file_extension = extract_lowercase_extension(file_path);
    let file_extension_ref = file_extension.as_deref();

    // Try each handler until one can process the file
    for handler in handlers {
        if handler.can_handle(data, file_extension_ref) {
            // Extract blocks using the file-formats-api
            match handler.extract_blocks(data, filter) {
                Ok(blocks_opt) => match blocks_opt {
                    Some(blocks) => {
                        // Call the test function with the extracted blocks
                        return test_fn(blocks.data, blocks.format)
                            .map_err(FileOperationError::Transform);
                    }
                    None => {
                        // Format doesn't match filter - this is expected behaviour
                        return Err(FileOperationError::Transform(
                            TransformError::NoSupportedHandler,
                        ));
                    }
                },
                Err(api_err) => {
                    // Convert file-formats-api error to file operation error
                    return Err(FileOperationError::Transform(api_err));
                }
            }
        }
    }

    // No handler could process the file
    Err(FileOperationError::Transform(
        TransformError::NoSupportedHandler,
    ))
}

/// Extracts the [`TransformFormat`] from a file using the file-formats-api.
///
/// This function uses memory mapping for optimal performance and supports any file format handler
/// that implements both [`FileFormatDetection`] and [`TransformFormatCheck`].
///
/// # Arguments
///
/// * `file_path` - Path to the file to inspect
/// * `handlers` - Array of file format handlers to try for format detection
/// * `filter` - Filter specifying which block formats to accept
///
/// # Returns
///
/// * `Ok(Some(format))` - Successfully identified format matching the filter
/// * `Ok(None)` - File format doesn't match the filter or no handler supports it
/// * `Err(error)` - File I/O error or format detection failed
///
/// # Example
///
/// ```rust,ignore
/// // Note: This example requires dxt-lossless-transform-dds with the
/// // "debug-block-extraction" feature enabled
/// use dxt_lossless_transform_file_formats_debug::{
///     get_file_format,
///     TransformFormatFilter,
/// };
/// use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
/// use dxt_lossless_transform_dds::DdsHandler;
/// use std::path::Path;
///
/// fn example_get_format(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
///     let handlers = [DdsHandler];
///     let result = get_file_format(
///         file_path,
///         &handlers,
///         TransformFormatFilter::All,
///     );
///     match result {
///         Ok(Some(format)) => println!("File format: {:?}", format),
///         Ok(None) => println!("No supported format found or doesn't match filter"),
///         Err(e) => println!("Failed to detect format: {:?}", e),
///     }
///     Ok(())
/// }
/// ```
pub fn get_file_format<H>(
    file_path: &Path,
    handlers: &[H],
    filter: TransformFormatFilter,
) -> FileOperationResult<Option<TransformFormat>>
where
    H: FileFormatDetection + TransformFormatCheck,
{
    // Use file-formats-api to open the file
    let source_handle = ReadOnlyFileHandle::open(file_path)?;
    let source_size = source_handle.size()? as usize;
    let source_mapping = ReadOnlyMmap::new(&source_handle, 0, source_size)?;
    let data = source_mapping.as_slice();

    // Get file extension for handler detection
    let file_extension = extract_lowercase_extension(file_path);
    let file_extension_ref = file_extension.as_deref();

    // Try each handler until one can process the file
    for handler in handlers {
        if handler.can_handle(data, file_extension_ref) {
            // Get format using the format detection trait
            match handler.get_transform_format(data, filter) {
                Ok(format_opt) => return Ok(format_opt),
                Err(api_err) => {
                    // Convert file-formats-api error to file operation error
                    return Err(FileOperationError::Transform(api_err));
                }
            }
        }
    }

    // No handler could process the file
    Ok(None)
}
