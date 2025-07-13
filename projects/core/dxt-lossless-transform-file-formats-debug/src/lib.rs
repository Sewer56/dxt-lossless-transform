#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
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
