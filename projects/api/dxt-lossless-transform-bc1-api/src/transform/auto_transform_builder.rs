//! Builder pattern implementation for BC1 automatic transform optimization.

use crate::Bc1Error;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::{
    Bc1EstimateSettings, Bc1TransformSettings, transform_bc1_auto_safe,
};

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

    /// Build the estimate settings using the configured values and provided estimator.
    pub fn build<T>(self, size_estimator: T) -> Bc1EstimateSettings<T>
    where
        T: SizeEstimationOperations,
    {
        Bc1EstimateSettings {
            size_estimator,
            use_all_decorrelation_modes: self.use_all_decorrelation_modes.unwrap_or(false),
        }
    }

    /// Build the estimate settings and transform BC1 data in one operation.
    ///
    /// This is a convenience method that combines building the settings and calling
    /// [`transform_bc1_auto`] in a single operation.
    ///
    /// # Parameters
    /// - `input`: The BC1 data to transform
    /// - `output`: The output buffer where transformed data will be written  
    /// - `size_estimator`: The size estimator to use for optimization
    ///
    /// # Returns
    /// The transform details on success, or an error on failure.
    ///
    /// # Errors
    /// Returns [`Bc1Error`] if the transformation fails.
    pub fn build_and_transform<T>(
        self,
        input: &[u8],
        output: &mut [u8],
        size_estimator: T,
    ) -> Result<Bc1TransformSettings, Bc1Error<T::Error>>
    where
        T: SizeEstimationOperations,
        T::Error: core::fmt::Debug,
    {
        let settings = self.build(size_estimator);

        // Call the core crate's safe wrapper function
        transform_bc1_auto_safe(input, output, settings).map_err(Bc1Error::from)
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
    fn test_auto_transform_builder() {
        let settings = Bc1AutoTransformBuilder::new()
            .use_all_decorrelation_modes(true)
            .build(DummyEstimator);

        assert!(settings.use_all_decorrelation_modes);
    }

    #[test]
    fn test_auto_transform_builder_build_and_transform() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 8];

        let result = Bc1AutoTransformBuilder::new()
            .use_all_decorrelation_modes(false)
            .build_and_transform(&bc1_data, &mut output, DummyEstimator);

        assert!(
            result.is_ok(),
            "build_and_transform should not fail with valid BC1 data"
        );
    }
}
