//! File format aware API for DXT lossless transform operations.
//!
//! This crate provides high-level APIs that automatically handle file format detection,
//! header embedding, and restoration during DXT transform operations.
//!
//! # Features
//!
//! - Automatic file format detection (DDS, KTX, etc.)
//! - Seamless header embedding of transform details
//! - Type-safe transform bundles for different BCx formats
//! - Memory-mapped file support for efficient I/O
//!
//! # Example
//!
//! ```
//! use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_bundle, TransformResult};
//! use dxt_lossless_transform_dds::DdsHandler;
//!
//! fn example_file_transform(input: &[u8]) -> TransformResult<Vec<u8>> {
//!     // Create a bundle with default settings for all formats
//!     let bundle = TransformBundle::default_all();
//!
//!     // Transform a DDS file in memory
//!     let mut output = vec![0u8; input.len()];
//!     transform_slice_bundle(&DdsHandler, input, &mut output, &bundle)?;
//!     Ok(output)
//! }
//! ```

#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

// Core modules
pub mod api;
pub mod bundle;
pub mod embed;
pub mod error;
pub mod formats;
pub mod traits;

#[cfg(feature = "file-io")]
pub mod file_io;

// Re-export key types
pub use bundle::{TransformBundle, UntransformResult};
pub use error::{TransformError, TransformResult};
pub use traits::FileFormatHandler;

// Re-export file operation types when file-io feature is enabled
#[cfg(feature = "file-io")]
pub use file_io::{FileOperationError, FileOperationResult};

// Re-export convenience functions
pub use api::{transform_slice_bundle, untransform_slice_with};
