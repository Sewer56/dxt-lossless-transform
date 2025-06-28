//! Builder pattern implementation for BC1 automatic transform optimization.

use super::YCoCgVariant;
use crate::{Bc1Error, Bc1ManualTransformBuilder};
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{Bc1EstimateSettings, transform_bc1_auto_safe};

/// Automatic BC1 transform optimization builder.
///
/// Uses a size estimator to automatically determine the best transform settings
/// for optimal compression. Ideal when you want the best compression without
/// manual tuning.
///
/// For manual control over transform parameters, use [`crate::Bc1ManualTransformBuilder`].
pub struct Bc1AutoTransformBuilder<T>
where
    T: SizeEstimationOperations,
{
    settings: Bc1EstimateSettings<T>,
}

impl<T> Bc1AutoTransformBuilder<T>
where
    T: SizeEstimationOperations,
{
    /// Create a new automatic transform builder with the provided estimator.
    ///
    /// The estimator should have its compression level and other parameters already configured.
    /// This allows for more flexible usage patterns where different estimators can have
    /// completely different configuration approaches.
    ///
    /// # Parameters
    /// - `estimator`: The size estimator to use for finding the best possible transform.
    ///   This will test different transform configurations and choose the one that results
    ///   in the smallest estimated compressed size according to this estimator.
    pub fn new(estimator: T) -> Self {
        Self {
            settings: Bc1EstimateSettings {
                size_estimator: estimator,
                use_all_decorrelation_modes: false, // Default value
            },
        }
    }

    /// Set whether to use all decorrelation modes.
    ///
    /// When `false` (default), only tests common configurations for faster optimization.
    /// When `true`, tests all decorrelation modes for potentially better compression
    /// at the cost of twice as long optimization time.
    ///
    /// **Note**: The typical improvement from testing all decorrelation modes is <0.1% in practice.
    /// For better compression gains, it's recommended to use a compression level on the
    /// estimator (e.g., ZStandard estimator) closer to your final compression level instead.
    pub fn use_all_decorrelation_modes(mut self, use_all: bool) -> Self {
        self.settings.use_all_decorrelation_modes = use_all;
        self
    }

    /// Transform BC1 data with automatically optimized settings and return a builder for untransformation.
    ///
    /// This method determines the best transform settings using the configured estimator,
    /// applies the transformation to the input data, and returns a pre-configured
    /// [`Bc1ManualTransformBuilder`] that can be used to untransform the data later.
    ///
    /// # Parameters
    /// - `input`: The BC1 data to transform
    /// - `output`: The output buffer where transformed data will be written
    ///
    /// # Returns
    /// A [`Bc1ManualTransformBuilder`] configured with the optimal settings used for transformation.
    ///
    /// # Errors
    /// Returns [`Bc1Error`] if the optimization or transformation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
    /// use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
    /// # use dxt_lossless_transform_bc1_api::Bc1Error;
    /// # use dxt_lossless_transform_ltu::LosslessTransformUtilsError;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let bc1_data = vec![0u8; 8]; // 1 BC1 block
    /// let mut transformed = vec![0u8; 8];
    /// let mut restored = vec![0u8; 8];
    ///
    /// // Create LTU estimator for fast size estimation
    /// let estimator = LosslessTransformUtilsSizeEstimation::new();
    ///
    /// // Transform with optimal settings and get builder for untransformation
    /// let untransform_builder = Bc1AutoTransformBuilder::new(estimator)
    ///     .use_all_decorrelation_modes(false)
    ///     .transform(&bc1_data, &mut transformed)?;
    ///
    /// // Later, untransform using the returned builder
    /// untransform_builder.untransform(&transformed, &mut restored)?;
    /// # assert_eq!(bc1_data, restored); // Verify round-trip works
    /// # Ok(())
    /// # }
    /// ```
    pub fn transform(
        self,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<Bc1ManualTransformBuilder, Bc1Error<T::Error>>
    where
        T::Error: core::fmt::Debug,
    {
        // Use the configured settings directly
        let optimal_settings = transform_bc1_auto_safe(input, output, self.settings)
            .map_err(Bc1Error::from_auto_transform_error)?;

        // Return a manual builder configured with these optimal settings
        Ok(Bc1ManualTransformBuilder::new()
            .decorrelation_mode(YCoCgVariant::from_internal_variant(
                optimal_settings.decorrelation_mode,
            ))
            .split_colour_endpoints(optimal_settings.split_colour_endpoints))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};

    /// Dummy estimator for testing
    struct DummyEstimator;

    impl SizeEstimationOperations for DummyEstimator {
        type Error = &'static str;

        fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
            Ok(0)
        }

        unsafe fn estimate_compressed_size(
            &self,
            _input_ptr: *const u8,
            len_bytes: usize,
            _data_type: DataType,
            _output_ptr: *mut u8,
            _output_len: usize,
        ) -> Result<usize, Self::Error> {
            Ok(len_bytes)
        }
    }

    #[test]
    fn test_auto_transform_builder_transform() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut transformed = [0u8; 8];

        let result = Bc1AutoTransformBuilder::new(DummyEstimator)
            .use_all_decorrelation_modes(false)
            .transform(&bc1_data, &mut transformed);

        assert!(
            result.is_ok(),
            "transform should not fail with valid BC1 data"
        );

        // Verify we can use the returned builder for untransformation
        let untransform_builder = result.unwrap();
        let mut restored = [0u8; 8];
        let untransform_result = untransform_builder.untransform(&transformed, &mut restored);
        assert!(untransform_result.is_ok(), "untransform should succeed");
    }

    #[test]
    fn test_auto_transform_builder_construction() {
        // Test that builder can be constructed with an estimator
        let _builder = Bc1AutoTransformBuilder::new(DummyEstimator);

        // Test builder method chaining
        let _builder_with_options = Bc1AutoTransformBuilder::new(DummyEstimator)
            .use_all_decorrelation_modes(true)
            .use_all_decorrelation_modes(false);
    }
}
