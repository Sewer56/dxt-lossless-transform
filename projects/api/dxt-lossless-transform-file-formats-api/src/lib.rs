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
//! - Support for both manual and automatic BC1 optimization
//! - Memory-mapped file support for efficient I/O
//!
//! # Example
//!
//! ```
//! use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_with_bundle, TransformResult};
//! use dxt_lossless_transform_api_common::estimate::NoEstimation;
//! use dxt_lossless_transform_dds::DdsHandler;
//!
//! fn example_file_transform(input: &[u8]) -> TransformResult<Vec<u8>> {
//!     // Create a bundle with default manual settings for all formats
//!     let bundle = TransformBundle::<NoEstimation>::default_all();
//!
//!     // Transform a DDS file in memory
//!     let mut output = vec![0u8; input.len()];
//!     transform_slice_with_bundle(&DdsHandler, input, &mut output, &bundle)?;
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
pub mod traits;

#[cfg(feature = "file-io")]
pub mod file_io;

// Re-export key types
pub use bundle::{Bc1Builder, TransformBundle, UntransformResult};
pub use error::{FormatHandlerError, FormatHandlerResult, TransformError, TransformResult};
pub use traits::FileFormatHandler;

// Re-export file operation types when file-io feature is enabled
#[cfg(feature = "file-io")]
pub use file_io::{FileOperationError, FileOperationResult};

// Re-export convenience functions
pub use api::{transform_slice_with_bundle, untransform_slice};
