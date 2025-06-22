//! Builder pattern implementation for BC1 transform options.

use crate::determine_optimal_transform::builder::Bc1EstimateOptionsBuilder;
use crate::error::Bc1Error;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc1::Bc1TransformDetails;
use safe_allocator_api::RawAlloc;

/// Builder for BC1 transform options with convenient configuration methods.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bc1TransformOptionsBuilder {
    decorrelation_mode: Option<YCoCgVariant>,
    split_colour_endpoints: Option<bool>,
}

impl Bc1TransformOptionsBuilder {
    /// Create a new options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the decorrelation mode.
    pub fn decorrelation_mode(mut self, mode: YCoCgVariant) -> Self {
        self.decorrelation_mode = Some(mode);
        self
    }

    /// Set whether to split colour endpoints.
    pub fn split_colour_endpoints(mut self, split: bool) -> Self {
        self.split_colour_endpoints = Some(split);
        self
    }

    /// Build the transform options using the configured values or defaults.
    pub fn build(self) -> Bc1TransformDetails {
        let default = Bc1TransformDetails::default();
        Bc1TransformDetails {
            decorrelation_mode: self
                .decorrelation_mode
                .map(|mode| mode.to_internal_variant())
                .unwrap_or(default.decorrelation_mode),
            split_colour_endpoints: self
                .split_colour_endpoints
                .unwrap_or(default.split_colour_endpoints),
        }
    }

    /// Transform BC1 data using the configured settings with pre-allocated output buffer.
    ///
    /// # Parameters
    ///
    /// - `input`: The input BC1 data to transform
    /// - `output`: The output buffer to write transformed data to
    ///
    /// # Returns
    ///
    /// [`Ok`] on success, or an error if validation fails.
    ///
    /// # Errors
    ///
    /// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
    /// - [`Bc1Error::OutputBufferTooSmall`] if output buffer is smaller than input
    pub fn transform_slice(self, input: &[u8], output: &mut [u8]) -> Result<(), Bc1Error> {
        let details = self.build();
        crate::transform::transform_bc1_slice(input, output, details)
    }

    /// Transform BC1 data using the configured settings and return a new allocated buffer.
    ///
    /// This function allocates a new 64-byte aligned buffer for optimal SIMD performance.
    ///
    /// # Parameters
    ///
    /// - `input`: The input BC1 data to transform
    ///
    /// # Returns
    ///
    /// A [`RawAlloc`] containing the transformed data.
    ///
    /// # Errors
    ///
    /// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
    /// - [`Bc1Error::AllocationFailed`] if memory allocation fails
    pub fn transform_allocating(self, input: &[u8]) -> Result<RawAlloc, Bc1Error> {
        let details = self.build();
        crate::transform::transform_bc1_allocating(input, details)
    }

    /// Automatically determine the best options for the given data.
    ///
    /// # Parameters
    ///
    /// - `data`: The BC1 data to analyze
    /// - `estimator`: The size estimation operations to use
    /// - `use_all_modes`: Whether to test all decorrelation modes (twice as slow, tests 4 options instead of 2, typically <0.1% extra savings)
    ///
    /// # Returns
    ///
    /// The optimal transform options for the given data.
    pub fn auto_determine_with<T>(
        self,
        data: &[u8],
        estimator: T,
        use_all_modes: bool,
    ) -> Result<Bc1TransformDetails, Bc1Error<T::Error>>
    where
        T: SizeEstimationOperations,
        T::Error: core::fmt::Debug,
    {
        let options = Bc1EstimateOptionsBuilder::new()
            .use_all_decorrelation_modes(use_all_modes)
            .build(estimator);
        crate::determine_optimal_transform::determine_optimal_transform(data, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder() {
        let options = Bc1TransformOptionsBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant2)
            .split_colour_endpoints(false)
            .build();

        assert_eq!(
            options.decorrelation_mode,
            YCoCgVariant::Variant2.to_internal_variant()
        );
        assert!(!options.split_colour_endpoints);
    }

    #[test]
    fn transform_slice_method() {
        let bc1_data = vec![0u8; 8]; // Single BC1 block
        let mut output = vec![0u8; 8];

        let result = Bc1TransformOptionsBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .transform_slice(&bc1_data, &mut output);

        assert!(result.is_ok());
    }

    #[test]
    fn transform_allocating_method() {
        let bc1_data = vec![0u8; 8]; // Single BC1 block

        let result = Bc1TransformOptionsBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .transform_allocating(&bc1_data);

        assert!(result.is_ok());
        let transformed = result.unwrap();
        assert_eq!(transformed.len(), 8);
    }
}
