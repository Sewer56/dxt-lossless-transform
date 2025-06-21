//! Builder pattern implementation for BC1 estimate options.

use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::determine_optimal_transform::Bc1EstimateOptions;

/// Builder for BC1 estimation options with convenient configuration methods.
#[derive(Debug, Clone, Copy, Default)]
pub struct Bc1EstimateOptionsBuilder {
    use_all_decorrelation_modes: Option<bool>,
}

impl Bc1EstimateOptionsBuilder {
    /// Create a new estimate options builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set whether to use all decorrelation modes.
    ///
    /// When `false` (default), only tests `YCoCgVariant::Variant1` and `YCoCgVariant::None`
    /// for faster optimization with good results.
    ///
    /// When `true`, tests all available decorrelation modes for potentially better
    /// compression at the cost of longer optimization time.
    pub fn use_all_decorrelation_modes(mut self, use_all: bool) -> Self {
        self.use_all_decorrelation_modes = Some(use_all);
        self
    }

    /// Build the estimation options using the configured values or defaults.
    ///
    /// # Parameters
    ///
    /// - `estimator`: The size estimation operations to use
    ///
    /// # Returns
    ///
    /// A configured [`Bc1EstimateOptions`] instance.
    pub fn build<T>(self, estimator: T) -> Bc1EstimateOptions<T>
    where
        T: SizeEstimationOperations,
    {
        Bc1EstimateOptions {
            size_estimator: estimator,
            use_all_decorrelation_modes: self.use_all_decorrelation_modes.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};

    // Create a simple dummy estimator for testing
    struct DummyEstimator;

    impl SizeEstimationOperations for DummyEstimator {
        type Error = &'static str;

        fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
            Ok(0) // No buffer needed for dummy estimator
        }

        unsafe fn estimate_compressed_size(
            &self,
            _input_ptr: *const u8,
            len_bytes: usize,
            _data_type: DataType,
            _output_ptr: *mut u8,
            _output_len: usize,
        ) -> Result<usize, Self::Error> {
            Ok(len_bytes) // Just return the input length
        }
    }

    #[test]
    fn test_estimate_options_builder() {
        // Test default builder
        let options = Bc1EstimateOptionsBuilder::new().build(DummyEstimator);
        assert!(!options.use_all_decorrelation_modes);

        // Test with all decorrelation modes enabled
        let options = Bc1EstimateOptionsBuilder::new()
            .use_all_decorrelation_modes(true)
            .build(DummyEstimator);
        assert!(options.use_all_decorrelation_modes);

        // Test with all decorrelation modes disabled
        let options = Bc1EstimateOptionsBuilder::new()
            .use_all_decorrelation_modes(false)
            .build(DummyEstimator);
        assert!(!options.use_all_decorrelation_modes);
    }
}
