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
//! ```ignore
//! use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_bundle};
//! use dxt_lossless_transform_dds::DdsHandler;
//!
//! // Create a bundle with auto settings for all formats
//! let bundle = TransformBundle::auto_all();
//!
//! // Transform a DDS file in memory
//! let mut output = vec![0u8; input.len()];
//! transform_slice_bundle(&DdsHandler, &input, &mut output, &bundle)?;
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

// Core modules
pub mod api;
pub mod bundle;
pub mod embed;
pub mod error;
pub mod formats;
pub mod traits;

#[cfg(feature = "file_io")]
pub mod file_io;

// Re-export key types
pub use bundle::{TransformBundle, UntransformResult};
pub use error::{FileFormatError, FileFormatResult};
pub use traits::FileFormatHandler;

// Re-export convenience functions
pub use api::{transform_slice_bundle, untransform_slice_with};
