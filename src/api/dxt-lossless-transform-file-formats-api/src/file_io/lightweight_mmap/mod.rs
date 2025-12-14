//! Lightweight memory-mapped file I/O operations.
//!
//! This module provides efficient file I/O operations using memory mapping for transforming
//! DXT texture data. It supports various combinations of input/output types:
//! - File to file transformations
//! - File to slice transformations  
//! - Slice to file transformations
//!
//! All operations use memory mapping for optimal performance and support both single
//! handler and multiple handler (auto-detection) variants.

use std::path::Path;
use std::string::String;

pub mod file;
pub mod file_to_slice;
pub mod slice_to_file;

#[cfg(test)]
pub mod test_prelude;

// Re-export all public functions
pub use file::*;
pub use file_to_slice::*;
pub use slice_to_file::*;

/// Extract file extension from a path and convert to lowercase.
///
/// # Arguments
///
/// * `path` - The file path to extract extension from
///
/// # Returns
///
/// * `Some(extension)` - The lowercase extension string without leading dot
/// * `None` - If the path has no extension
pub fn extract_lowercase_extension(path: &Path) -> Option<String> {
    // Note(sewer): Performance here is kinda oof, due to heap allocation, but this is on the
    // slow path of unknown file types; so I don't mind for the time being.
    path.extension()?.to_str().map(|s| s.to_lowercase())
}
