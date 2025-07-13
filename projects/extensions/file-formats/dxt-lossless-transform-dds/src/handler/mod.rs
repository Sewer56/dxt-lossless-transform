//! DDS format handler implementation.

mod file_format_detection;
mod file_format_handler;
mod file_format_untransform_detection;
mod format_conversion;

#[cfg(feature = "debug-block-extraction")]
mod file_format_block_extraction;

#[cfg(feature = "debug-block-extraction")]
mod file_format_check;

/// Handler for DDS file format.
///
/// This handler supports BC1/BC2/BC3/BC7 formats within DDS files,
/// embedding transform details in the 4-byte DDS magic header.
/// Currently only BC1 supports configurable transform options.
pub struct DdsHandler;
