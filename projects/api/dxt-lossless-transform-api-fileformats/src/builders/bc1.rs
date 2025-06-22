//! BC1 file format aware builders.
//!
//! This module provides builders that wrap the existing [`Bc1TransformOptionsBuilder`] and
//! [`Bc1EstimateOptionsBuilder`] from the stable API, adding file format handling capabilities while
//! maintaining the separation of concerns.

use crate::error::FileFormatResult;
use crate::formats::bc1::EmbeddableBc1Details;
use crate::traits::file_format::FileFormatHandler;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc1::{Bc1DetransformDetails, Bc1TransformDetails};
use dxt_lossless_transform_bc1_api::{
    error::Bc1Error, Bc1EstimateOptionsBuilder, Bc1TransformOptionsBuilder,
};
use std::path::Path;

/// Transform-focused builder that wraps [`Bc1TransformOptionsBuilder`] with file format handling capabilities.
///
/// This builder maintains the same API as the original builder but adds file format handling
/// functionality for file operations.
pub struct Bc1FileFormatTransformBuilder {
    inner: Bc1TransformOptionsBuilder,
}

impl Bc1FileFormatTransformBuilder {
    /// Create a new BC1 file format transform builder.
    pub fn new() -> Self {
        Self {
            inner: Bc1TransformOptionsBuilder::new(),
        }
    }

    /// Set the decorrelation mode.
    ///
    /// This delegates to the underlying [`Bc1TransformOptionsBuilder`].
    pub fn decorrelation_mode(mut self, mode: YCoCgVariant) -> Self {
        self.inner = self.inner.decorrelation_mode(mode);
        self
    }

    /// Set whether to split colour endpoints.
    ///
    /// This delegates to the underlying [`Bc1TransformOptionsBuilder`].
    pub fn split_colour_endpoints(mut self, split: bool) -> Self {
        self.inner = self.inner.split_colour_endpoints(split);
        self
    }

    /// Build the transform options using the configured values or defaults.
    ///
    /// This delegates to the underlying [`Bc1TransformOptionsBuilder`].
    pub fn build(self) -> Bc1TransformDetails {
        self.inner.build()
    }

    /// Build embeddable transform details.
    ///
    /// This converts the transform details to a format that can be embedded in file headers.
    pub fn build_embeddable(self) -> EmbeddableBc1Details {
        EmbeddableBc1Details::from(Bc1DetransformDetails::from(self.build()))
    }

    /// Transform a single file with the configured settings.
    ///
    /// # Parameters
    ///
    /// - `input_path`: Path to the input file
    /// - `output_path`: Path to the output file
    ///
    /// # Returns
    ///
    /// [`Ok(())`] on success, or a [`FileFormatError`] on failure.
    pub fn transform_file<H: FileFormatHandler>(
        self,
        input_path: &Path,
        output_path: &Path,
    ) -> FileFormatResult<()> {
        let details = self.build();

        super::super::api::transform_bc1_file_with_details::<H>(
            input_path,
            output_path,
            details,
            true, // Always store in header when using file format builder
        )
    }
}

impl Default for Bc1FileFormatTransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimation-focused builder that wraps [`Bc1EstimateOptionsBuilder`] with file format handling capabilities.
///
/// This builder maintains the same API as the original builder but adds file format handling
/// functionality and file-level operations.
pub struct Bc1FileFormatEstimateBuilder<T: SizeEstimationOperations> {
    inner: Bc1EstimateOptionsBuilder,
    estimator: T,
}

impl<T: SizeEstimationOperations> Bc1FileFormatEstimateBuilder<T> {
    /// Create a new BC1 file format estimate builder.
    ///
    /// # Parameters
    ///
    /// - `estimator`: The size estimation operations to use
    pub fn new(estimator: T) -> Self {
        Self {
            inner: Bc1EstimateOptionsBuilder::new(),
            estimator,
        }
    }

    /// Set whether to use all decorrelation modes.
    ///
    /// This delegates to the underlying [`Bc1EstimateOptionsBuilder`].
    pub fn use_all_decorrelation_modes(mut self, use_all: bool) -> Self {
        self.inner = self.inner.use_all_decorrelation_modes(use_all);
        self
    }

    /// Determine optimal transform parameters for the given BC1 data.
    ///
    /// This delegates to the underlying API but returns the result in a format
    /// suitable for file format operations.
    ///
    /// # Parameters
    ///
    /// - `data`: The BC1 data to analyze
    ///
    /// # Returns
    ///
    /// The optimal [`Bc1TransformDetails`] for the given data.
    pub fn determine_optimal_for_data(
        self,
        data: &[u8],
    ) -> Result<Bc1TransformDetails, Bc1Error<T::Error>>
    where
        T::Error: core::fmt::Debug,
    {
        let options = self.inner.build(self.estimator);
        dxt_lossless_transform_bc1_api::determine_optimal_transform(data, options)
    }

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
    pub fn determine_optimal_for_file<H: FileFormatHandler>(
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
        let options = self.inner.build(self.estimator);
        let optimal_details = determine_optimal_transform(bc1_data, options).map_err(|e| {
            crate::error::FileFormatError::Transform(format!("Optimization failed: {e:?}"))
        })?;

        // Convert to embeddable details
        let embeddable_details =
            EmbeddableBc1Details::from(Bc1DetransformDetails::from(optimal_details));
        Ok(embeddable_details)
    }

    /// Transform a file with automatically determined optimal settings.
    ///
    /// This determines the optimal transform settings for the file and then applies
    /// the transformation in a single operation.
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
    pub fn transform_file_with_optimal<H: FileFormatHandler>(
        self,
        input_path: &Path,
        output_path: &Path,
    ) -> FileFormatResult<TransformResult>
    where
        T::Error: core::fmt::Debug,
    {
        // Get use_all_modes setting from the inner builder
        // Since the builder doesn't expose this, we'll use the default of false for now
        let use_all_modes = false; // TODO: Add introspection to builder or change API

        super::super::api::transform_bc1_file_with_optimal::<H, T>(
            input_path,
            output_path,
            self.estimator,
            use_all_modes,
            true, // Always embed when using file format builder
        )
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

// Note: We'll need to fix the implementation once we have the API functions implemented
// For now, this shows the structure and interface

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;

    #[test]
    fn test_bc1_file_format_transform_builder() {
        let builder = Bc1FileFormatTransformBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .split_colour_endpoints(true);

        let details = builder.build();
        assert_eq!(
            details.decorrelation_mode,
            YCoCgVariant::Variant1.to_internal_variant()
        );
        assert!(details.split_colour_endpoints);
    }

    #[test]
    fn test_bc1_file_format_estimate_builder() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let builder =
            Bc1FileFormatEstimateBuilder::new(estimator).use_all_decorrelation_modes(true);

        // Test that the builder can be created and configured
        // Actual functionality tests will need real data
        let bc1_data = vec![0u8; 8]; // Minimal BC1 block
        let result = builder.determine_optimal_for_data(&bc1_data);

        // Should succeed with valid BC1 data
        assert!(result.is_ok());
    }
}
