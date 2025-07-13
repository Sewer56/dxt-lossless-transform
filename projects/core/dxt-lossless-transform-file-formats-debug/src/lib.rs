//! Debug and research utilities for dxt-lossless-transform file format handling.
//!
//! This crate provides debug-only functionality for working with DXT compressed texture
//! file formats. It contains utilities useful for research, analysis, debugging, and
//! CLI tools, but are not intended for production use.
//!
//! # Example
//!
//! ```rust
//! use dxt_lossless_transform_file_formats_debug::{
//!     FileFormatBlockExtraction, TransformFormatCheck, TransformFormatFilter, ExtractedBlocks,
//! };
//! use dxt_lossless_transform_file_formats_api::{
//!     embed::TransformFormat,
//!     error::{TransformResult, TransformError, FormatHandlerError},
//! };
//!
//! // Example custom handler
//! struct MyFormatHandler;
//!
//! impl TransformFormatCheck for MyFormatHandler {
//!     fn get_transform_format(
//!         &self,
//!         data: &[u8],
//!         filter: TransformFormatFilter,
//!     ) -> TransformResult<Option<TransformFormat>> {
//!         // Implementation for format detection
//!         Ok(Some(TransformFormat::Bc1))
//!     }
//! }
//!
//! impl FileFormatBlockExtraction for MyFormatHandler {
//!     fn extract_blocks<'a>(
//!         &self,
//!         data: &'a [u8],
//!         filter: TransformFormatFilter,
//!     ) -> TransformResult<Option<ExtractedBlocks<'a>>> {
//!         // Implementation for extracting raw blocks
//!         Ok(None)
//!     }
//! }
//! ```
#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

// Core module with block extraction functionality
pub mod block_extraction;

// Format detection functionality
pub mod format_detection;

// Re-export main types and traits
pub use block_extraction::{ExtractedBlocks, FileFormatBlockExtraction, TransformFormatFilter};
pub use format_detection::TransformFormatCheck;

// File I/O operations for block extraction
#[cfg(feature = "file-io")]
pub mod file_io;

#[cfg(feature = "file-io")]
pub use file_io::{extract_blocks_from_file_format, get_transform_format};
