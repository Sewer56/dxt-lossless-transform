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
#[derive(Debug, Clone, Copy)]
pub struct Bc1AutoTransformBuilder {
    use_all_decorrelation_modes: Option<bool>,
}

impl Bc1AutoTransformBuilder {
    /// Create a new automatic transform builder.
    pub fn new() -> Self {
        Self {
            use_all_decorrelation_modes: None,
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
        self.use_all_decorrelation_modes = Some(use_all);
        self
    }

    /// Transform BC1 data with automatically optimized settings and return a builder for detransformation.
    ///
    /// This method determines the best transform settings using the provided estimator,
    /// applies the transformation to the input data, and returns a pre-configured
    /// [`Bc1ManualTransformBuilder`] that can be used to detransform the data later.
    ///
    /// # Parameters
    /// - `input`: The BC1 data to transform
    /// - `output`: The output buffer where transformed data will be written
    /// - `estimator`: The size estimator to use for optimization
    ///
    /// # Returns
    /// A [`Bc1ManualTransformBuilder`] configured with the optimal settings used for transformation.
    ///
    /// # Errors
    /// Returns [`Bc1Error`] if the optimization or transformation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use dxt_lossless_transform_bc1_api::Bc1AutoTransformBuilder;
    /// use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
    ///
    /// let bc1_data = vec![0u8; 8]; // 1 BC1 block
    /// let mut transformed = vec![0u8; 8];
    /// let mut restored = vec![0u8; 8];
    ///
    /// // Transform with optimal settings and get builder for detransformation
    /// let detransform_builder = Bc1AutoTransformBuilder::new()
    ///     .use_all_decorrelation_modes(false)
    ///     .transform(&bc1_data, &mut transformed, my_estimator)?;
    ///
    /// // Later, detransform using the returned builder
    /// detransform_builder.detransform(&transformed, &mut restored)?;
    /// ```
    pub fn transform<T>(
        self,
        input: &[u8],
        output: &mut [u8],
        estimator: T,
    ) -> Result<Bc1ManualTransformBuilder, Bc1Error<T::Error>>
    where
        T: SizeEstimationOperations,
        T::Error: core::fmt::Debug,
    {
        // Build internal settings and find optimal transform
        let settings = Bc1EstimateSettings {
            size_estimator: estimator,
            use_all_decorrelation_modes: self.use_all_decorrelation_modes.unwrap_or(false),
        };
        let optimal_settings =
            transform_bc1_auto_safe(input, output, settings).map_err(Bc1Error::from)?;

        // Return a manual builder configured with these optimal settings
        Ok(Bc1ManualTransformBuilder::new()
            .decorrelation_mode(YCoCgVariant::from_internal_variant(
                optimal_settings.decorrelation_mode,
            ))
            .split_colour_endpoints(optimal_settings.split_colour_endpoints))
    }
}

impl Default for Bc1AutoTransformBuilder {
    fn default() -> Self {
        Self::new()
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

        let result = Bc1AutoTransformBuilder::new()
            .use_all_decorrelation_modes(false)
            .transform(&bc1_data, &mut transformed, DummyEstimator);

        assert!(
            result.is_ok(),
            "transform should not fail with valid BC1 data"
        );

        // Verify we can use the returned builder for detransformation
        let detransform_builder = result.unwrap();
        let mut restored = [0u8; 8];
        let detransform_result = detransform_builder.detransform(&transformed, &mut restored);
        assert!(detransform_result.is_ok(), "detransform should succeed");
    }
}
