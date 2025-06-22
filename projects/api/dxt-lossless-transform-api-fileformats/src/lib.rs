//! # DXT Lossless Transform File Formats
//!
//! This crate provides high-level file format APIs for DXT lossless transforms with file format integration.
//! It wraps the stable transform APIs from the individual format crates and adds file format handling capabilities
//! for storing transform details in file headers.
//!
//! ## Architecture
//!
//! This crate maintains the separation of concerns from the stable APIs:
//!
//! - **Transform builders**: Configure and apply specific transform settings
//! - **Estimate builders**: Determine optimal transform settings automatically  
//! - **Function APIs**: Direct function calls without builder overhead
//! - **File format handlers**: Abstract over different file formats (DDS, KTX, etc.)
//!
//! ## Features
//!
//! - **Extension traits**: Add file format capabilities to existing stable builders
//! - **File operations**: Transform entire files with automatic header handling
//! - **Batch processing**: Directory-level operations similar to the CLI tool
//! - **Extensible design**: Support for new file formats and transform types
//! - **CLI integration**: Patterns that match the existing CLI tool's `transform_dir_entry`
//!
//! ## Quick Start
//!
//! ### Builder API - Transform with Specific Settings
//!
//! ```ignore
//! use dxt_lossless_transform_api_fileformats::{
//!     builders::Bc1TransformFileFormatExt,
//!     traits::FileFormatHandler,
//! };
//! use dxt_lossless_transform_bc1_api::Bc1TransformOptionsBuilder;
//! use dxt_lossless_transform_common::color_565::YCoCgVariant;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Use the extension trait to add file format capabilities
//! let result = Bc1TransformOptionsBuilder::new()
//!     .decorrelation_mode(YCoCgVariant::Variant1)
//!     .split_colour_endpoints(true)
//!     .transform_file::<DdsHandler>(input_path, output_path)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Builder API - Determine Optimal Settings
//!
//! ```ignore
//! use dxt_lossless_transform_api_fileformats::builders::{
//!     Bc1EstimateFileFormatExt,
//!     Bc1EstimateOptionsBuilderExt,
//! };
//! use dxt_lossless_transform_bc1_api::Bc1EstimateOptionsBuilder;
//! use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let estimator = LosslessTransformUtilsSizeEstimation::new();
//!
//! // Use extension traits to add file format capabilities
//! let optimal_details = Bc1EstimateOptionsBuilder::new()
//!     .with_estimator(estimator)
//!     .use_all_decorrelation_modes(false)
//!     .determine_optimal_for_file::<DdsHandler>(input_path)?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Function API - Direct Operations
//!
//! ```ignore
//! use dxt_lossless_transform_api_fileformats::api::{
//!     transform_bc1_file_with_details,
//!     transform_bc1_file_with_optimal,
//! };
//! use dxt_lossless_transform_bc1_api::Bc1TransformDetails;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Transform with specific settings (no builder overhead)
//! transform_bc1_file_with_details::<DdsHandler>(
//!     input_path,
//!     output_path,
//!     Bc1TransformDetails::default(),
//!     true // store_in_header
//! )?;
//!
//! // Transform with optimal settings in one call
//! let result = transform_bc1_file_with_optimal::<DdsHandler, _>(
//!     input_path,
//!     output_path,
//!     estimator,
//!     false, // use_all_modes
//!     true   // store_in_header
//! )?;
//! # Ok(())
//! # }
//! ```
//!
//! ### CLI-Style Batch Processing
//!
//! ```ignore
//! use dxt_lossless_transform_api_fileformats::api::transform_bc1_directory_with_details;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let result = transform_bc1_directory_with_details::<DdsHandler>(
//!     input_dir,
//!     output_dir,
//!     Bc1TransformDetails::default(),
//!     true, // store_in_header
//!     true  // parallel
//! )?;
//!
//! println!("Processed {} files, {} successful, {} failed",
//!     result.total_files,
//!     result.successful_transforms,
//!     result.failed_transforms
//! );
//! # Ok(())
//! # }
//! ```
//!
//! ## File Format Support
//!
//! File formats are supported through the [`FileFormatHandler`] trait. Currently supported:
//!
//! - **DDS**: Via `dxt-lossless-transform-dds` crate
//!
//! New file formats can be added by implementing the [`FileFormatHandler`] trait
//! in their respective crates.
//!
//! ## Integration with Stable APIs
//!
//! This crate wraps and extends the stable transform APIs:
//!
//! - [`dxt-lossless-transform-bc1-api`]: BC1 transform and estimation
//! - [`dxt-lossless-transform-bc2-api`]: BC2 transform (future)
//! - [`dxt-lossless-transform-bc3-api`]: BC3 transform (future)
//! - [`dxt-lossless-transform-bc7-api`]: BC7 transform (future)
//!
//! The file format functionality does not modify the stable APIs - it provides
//! additional layers that can be adopted incrementally.

#![cfg_attr(not(feature = "std"), no_std)]

// Public modules
pub mod api;
pub mod builders;
pub mod error;
pub mod formats;
pub mod traits;

// Re-exports for convenience
pub use error::{FileFormatError, FileFormatResult};
pub use traits::{FileFormatFilter, FileFormatHandler};

// Re-export core embedding types from stable API
pub use dxt_lossless_transform_api_common::embed::{
    EmbedError, EmbeddableTransformDetails, TransformFormat, TransformHeader,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reexports() {
        // Test that re-exports work
        let _format = TransformFormat::Bc1;
        let _header = TransformHeader::new(TransformFormat::Bc1, 0x12345678);
    }
}
