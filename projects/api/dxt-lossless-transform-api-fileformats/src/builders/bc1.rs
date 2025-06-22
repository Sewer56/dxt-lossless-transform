//! BC1 file format aware builders.
//!
//! This module provides extension traits that add file format handling capabilities
//! to the existing [`Bc1TransformOptionsBuilder`] and [`Bc1EstimateOptionsBuilder`]
//! from the stable API, without requiring method duplication.

use crate::error::FileFormatResult;
use crate::formats::bc1::EmbeddableBc1Details;
use crate::traits::file_format::FileFormatHandler;
use dxt_lossless_transform_api_common::embed::EmbeddableTransformDetails;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{Bc1DetransformDetails, Bc1TransformDetails};
use dxt_lossless_transform_bc1_api::{Bc1EstimateOptionsBuilder, Bc1TransformOptionsBuilder};
use std::path::Path;

/// Extension trait that adds file format handling capabilities to [`Bc1TransformOptionsBuilder`].
///
/// This trait extends the original builder with methods for working with file formats,
/// without requiring duplication of the core builder methods.
pub trait Bc1TransformFileFormatExt {
    /// Build embeddable transform details.
    ///
    /// This converts the transform details to a format that can be embedded in file headers.
    fn build_embeddable(self) -> EmbeddableBc1Details;

    /// Transform a single file with the configured settings.
    ///
    /// Transform details are automatically stored in the file header.
    ///
    /// # Parameters
    ///
    /// - `input_path`: Path to the input file
    /// - `output_path`: Path to the output file
    ///
    /// # Returns
    ///
    /// [`Ok(())`] on success, or a [`FileFormatError`] on failure.
    fn transform_file<H: FileFormatHandler>(
        self,
        input_path: &Path,
        output_path: &Path,
    ) -> FileFormatResult<()>;
}

impl Bc1TransformFileFormatExt for Bc1TransformOptionsBuilder {
    fn build_embeddable(self) -> EmbeddableBc1Details {
        let details = self.build();
        EmbeddableBc1Details::from(Bc1DetransformDetails::from(details))
    }

    fn transform_file<H: FileFormatHandler>(
        self,
        input_path: &Path,
        output_path: &Path,
    ) -> FileFormatResult<()> {
        use lightweight_mmap::handles::{ReadOnlyFileHandle, ReadWriteFileHandle};
        use lightweight_mmap::mmap::{ReadOnlyMmap, ReadWriteMmap};
        use std::fs;
        use std::ptr::copy_nonoverlapping;

        // Open and map the input file
        let source_handle = ReadOnlyFileHandle::open(input_path.to_str().ok_or_else(|| {
            crate::error::FileFormatError::InvalidFileData("Invalid input path".to_string())
        })?)
        .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?;

        let source_size = source_handle
            .size()
            .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?
            as usize;

        let source_mapping = ReadOnlyMmap::new(&source_handle, 0, source_size)
            .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?;

        // Detect and validate file format
        let file_info = H::detect_format(unsafe {
            std::slice::from_raw_parts(source_mapping.data(), source_mapping.len())
        })
        .ok_or_else(|| {
            crate::error::FileFormatError::FormatNotDetected(
                input_path.to_string_lossy().to_string(),
            )
        })?;

        let data_offset = H::get_data_offset(&file_info);

        // Create output directory if needed
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(crate::error::FileFormatError::Io)?;
        }

        // Create output file
        let target_handle = ReadWriteFileHandle::create_preallocated(
            output_path.to_str().ok_or_else(|| {
                crate::error::FileFormatError::InvalidFileData("Invalid output path".to_string())
            })?,
            source_size as i64,
        )
        .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?;

        let target_mapping = ReadWriteMmap::new(&target_handle, 0, source_size)
            .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?;

        // Copy headers
        unsafe {
            copy_nonoverlapping(source_mapping.data(), target_mapping.data(), data_offset);
        }

        // Store transform details in header (always enabled)
        let embeddable_details = self.build_embeddable();
        let header = embeddable_details.to_header();
        unsafe {
            H::embed_transform_header(target_mapping.data(), header)
                .map_err(|e| crate::error::FileFormatError::Embed(e.into()))?;
        }

        // Transform the data section using the builder's memory-to-memory method
        let data_size = source_size - data_offset;
        if data_size % 8 != 0 {
            return Err(crate::error::FileFormatError::InvalidFileData(
                "BC1 data size must be multiple of 8 bytes".to_string(),
            ));
        }

        let input_slice = unsafe {
            std::slice::from_raw_parts(source_mapping.data().add(data_offset), data_size)
        };
        let output_slice = unsafe {
            std::slice::from_raw_parts_mut(target_mapping.data().add(data_offset), data_size)
        };

        // Use the builder's transform method instead of standalone function
        self.transform_slice(input_slice, output_slice)
            .map_err(|e| {
                crate::error::FileFormatError::Transform(format!("BC1 transform failed: {e:?}"))
            })?;

        Ok(())
    }
}

/// Extension trait that adds file format handling capabilities to [`Bc1EstimateOptionsBuilder`].
///
/// This trait extends the original builder with methods for working with file formats,
/// without requiring duplication of the core builder methods.
pub trait Bc1EstimateFileFormatExt<T: SizeEstimationOperations> {
    /// Determine optimal transform parameters for a file.
    ///
    /// This loads the file, extracts BC1 data, and determines optimal transform settings.
    ///
    /// # Parameters
    ///
    /// - `input_path`: Path to the file to analyze
    ///
    /// # Returns
    ///
    /// The optimal [`EmbeddableBc1Details`] for the file.
    fn determine_optimal_for_file<H: FileFormatHandler>(
        self,
        input_path: &Path,
    ) -> FileFormatResult<EmbeddableBc1Details>
    where
        T::Error: core::fmt::Debug;

    /// Transform a file with automatically determined optimal settings.
    ///
    /// This determines the optimal transform settings for the file and then applies
    /// the transformation in a single operation. Transform details are automatically
    /// stored in the file header.
    ///
    /// # Parameters
    ///
    /// - `input_path`: Path to the input file
    /// - `output_path`: Path to the output file
    ///
    /// # Returns
    ///
    /// [`Ok(TransformResult)`] on success containing the details that were used,
    /// or a [`FileFormatError`] on failure.
    fn transform_file_with_optimal<H: FileFormatHandler>(
        self,
        input_path: &Path,
        output_path: &Path,
    ) -> FileFormatResult<TransformResult>
    where
        T::Error: core::fmt::Debug;
}

// Helper struct to pair builder with estimator for the extension trait
pub struct Bc1EstimateBuilderWithEstimator<T: SizeEstimationOperations> {
    builder: Bc1EstimateOptionsBuilder,
    estimator: T,
    use_all_decorrelation_modes: bool,
}

impl<T: SizeEstimationOperations> Bc1EstimateBuilderWithEstimator<T> {
    /// Create a new builder with estimator pair.
    pub fn new(builder: Bc1EstimateOptionsBuilder, estimator: T) -> Self {
        Self {
            builder,
            estimator,
            use_all_decorrelation_modes: false, // Default value
        }
    }

    /// Set whether to use all decorrelation modes (chainable).
    pub fn use_all_decorrelation_modes(mut self, use_all: bool) -> Self {
        self.builder = self.builder.use_all_decorrelation_modes(use_all);
        self.use_all_decorrelation_modes = use_all;
        self
    }
}

impl<T: SizeEstimationOperations> Bc1EstimateFileFormatExt<T>
    for Bc1EstimateBuilderWithEstimator<T>
{
    fn determine_optimal_for_file<H: FileFormatHandler>(
        self,
        input_path: &Path,
    ) -> FileFormatResult<EmbeddableBc1Details>
    where
        T::Error: core::fmt::Debug,
    {
        use dxt_lossless_transform_bc1_api::determine_optimal_transform;
        use lightweight_mmap::handles::ReadOnlyFileHandle;
        use lightweight_mmap::mmap::ReadOnlyMmap;

        // Open and map the input file for analysis only
        let source_handle = ReadOnlyFileHandle::open(input_path.to_str().ok_or_else(|| {
            crate::error::FileFormatError::InvalidFileData("Invalid input path".to_string())
        })?)
        .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?;

        let source_size = source_handle
            .size()
            .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?
            as usize;

        let source_mapping = ReadOnlyMmap::new(&source_handle, 0, source_size)
            .map_err(|e| crate::error::FileFormatError::MemoryMapping(e.to_string()))?;

        // Detect and validate file format
        let file_info = H::detect_format(unsafe {
            std::slice::from_raw_parts(source_mapping.data(), source_mapping.len())
        })
        .ok_or_else(|| {
            crate::error::FileFormatError::FormatNotDetected(
                input_path.to_string_lossy().to_string(),
            )
        })?;

        let data_offset = H::get_data_offset(&file_info);
        let data_size = source_size - data_offset;

        if data_size % 8 != 0 {
            return Err(crate::error::FileFormatError::InvalidFileData(
                "BC1 data size must be multiple of 8 bytes".to_string(),
            ));
        }

        // Extract BC1 data for analysis
        let bc1_data = unsafe {
            std::slice::from_raw_parts(source_mapping.data().add(data_offset), data_size)
        };

        // Determine optimal transform settings
        let options = self.builder.build(self.estimator);
        let optimal_details = determine_optimal_transform(bc1_data, options).map_err(|e| {
            crate::error::FileFormatError::Transform(format!("Optimization failed: {e:?}"))
        })?;

        // Convert to embeddable details
        let embeddable_details =
            EmbeddableBc1Details::from(Bc1DetransformDetails::from(optimal_details));
        Ok(embeddable_details)
    }

    fn transform_file_with_optimal<H: FileFormatHandler>(
        self,
        input_path: &Path,
        output_path: &Path,
    ) -> FileFormatResult<TransformResult>
    where
        T::Error: core::fmt::Debug,
    {
        crate::api::transform_bc1_file_with_optimal::<H, T>(
            input_path,
            output_path,
            self.estimator,
            self.use_all_decorrelation_modes,
        )
    }
}

/// Extension trait to create a file format builder from [`Bc1EstimateOptionsBuilder`].
pub trait Bc1EstimateOptionsBuilderExt {
    /// Create a file format builder by pairing with an estimator.
    ///
    /// # Parameters
    ///
    /// - `estimator`: The size estimation operations to use
    ///
    /// # Returns
    ///
    /// A builder that supports file format operations.
    fn with_estimator<T: SizeEstimationOperations>(
        self,
        estimator: T,
    ) -> Bc1EstimateBuilderWithEstimator<T>;
}

impl Bc1EstimateOptionsBuilderExt for Bc1EstimateOptionsBuilder {
    fn with_estimator<T: SizeEstimationOperations>(
        self,
        estimator: T,
    ) -> Bc1EstimateBuilderWithEstimator<T> {
        Bc1EstimateBuilderWithEstimator::new(self, estimator)
    }
}

/// Result of a transform operation that includes the details that were used.
#[derive(Debug, Clone)]
pub struct TransformResult {
    /// The transform details that were used
    pub transform_details: Bc1TransformDetails,
    /// The embeddable details
    pub embeddable_details: EmbeddableBc1Details,
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
    use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;

    #[test]
    fn test_bc1_transform_extension() {
        let builder = Bc1TransformOptionsBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .split_colour_endpoints(true);

        // Test that we can build embeddable details
        let embeddable = builder.build_embeddable();
        // Access the inner Bc1DetransformDetails through the wrapper
        // Need to convert from API variant to internal variant for comparison
        assert_eq!(
            embeddable.0.decorrelation_mode,
            YCoCgVariant::Variant1.to_internal_variant()
        );
        assert!(embeddable.0.split_colour_endpoints);
    }

    #[test]
    fn test_bc1_estimate_extension() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let builder = Bc1EstimateOptionsBuilder::new()
            .with_estimator(estimator)
            .use_all_decorrelation_modes(true);

        // Test that the builder can be created and configured
        // Actual functionality tests will need real data
        let bc1_data = vec![0u8; 8]; // Minimal BC1 block

        // We can't test the actual determination without real data,
        // but we can verify the API compiles correctly
    }
}
