#![cfg(not(tarpaulin_include))]

pub mod benchmark_common;
pub mod calc_compression_stats_common;
pub mod compressed_data_cache;
pub mod compression;
pub mod compression_size_cache;
pub mod estimation;

use crate::util;
use crate::{error::TransformError, util::all_handlers};
use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
use dxt_lossless_transform_file_formats_debug::{
    extract_blocks_from_file_format, TransformFormatFilter,
};
use std::path::Path;

/// Extracts transform blocks from files using the file-formats-api.
///
/// This function uses the file-formats-api to extract raw transform block data from files
/// for debug, analysis, and testing purposes.
pub(crate) fn extract_blocks_from_file<TFunction>(
    file_path: &Path,
    filter: TransformFormatFilter,
    mut test_fn: TFunction,
) -> Result<(), TransformError>
where
    TFunction: FnMut(&[u8], TransformFormat) -> Result<(), TransformError>,
{
    // Convert CLI TransformError to file-formats-api TransformError
    let transform_format_fn = |data: &[u8], transform_format: TransformFormat| {
        test_fn(data, transform_format).map_err(|_e| {
            // For debug operations, we'll map CLI errors to NoSupportedHandler
            dxt_lossless_transform_file_formats_api::TransformError::NoSupportedHandler
        })
    };

    // Use the file-formats-debug function and convert the error
    extract_blocks_from_file_format(file_path, &all_handlers(), filter, transform_format_fn)
        .map_err(TransformError::FileOperationError)
}

/// Handles errors from debug operations by printing to stdout.
/// Silently ignores NoSupportedHandler errors to allow filtering mixed file types.
pub(crate) fn handle_debug_error(
    file_path: &Path,
    operation: &str,
    result: Result<(), TransformError>,
) {
    if let Err(e) = result {
        // Check if this is a "No file format handler can process the file" error
        if let TransformError::FileOperationError(
            dxt_lossless_transform_file_formats_api::file_io::FileOperationError::Transform(
                transform_error,
            ),
        ) = &e
        {
            if matches!(
                transform_error,
                dxt_lossless_transform_file_formats_api::TransformError::NoSupportedHandler
            ) {
                // Silently ignore files that can't be processed - common when filtering directories
                return;
            }
        }

        // Print all other errors
        println!("✗ Error {operation} {}: {}", file_path.display(), e);
    }
}

/// Calculates XXH3-128 hash of data for use as a cache key.
pub fn calculate_content_hash(data: &[u8]) -> u128 {
    xxhash_rust::xxh3::xxh3_128(data)
}
