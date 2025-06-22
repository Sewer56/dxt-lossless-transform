//! Builder pattern implementation for BC1 estimate settings.

use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::Bc1EstimateSettings;

/// Builder for BC1 estimate settings with convenient configuration methods.
#[derive(Debug, Clone, Copy)]
pub struct Bc1EstimateSettingsBuilder {
    use_all_decorrelation_modes: Option<bool>,
}

impl Bc1EstimateSettingsBuilder {
    /// Create a new settings builder.
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
}

impl Default for Bc1EstimateSettingsBuilder {
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
    fn test_estimate_settings_builder() {
        let settings = Bc1EstimateSettingsBuilder::new()
            .use_all_decorrelation_modes(true)
            .build(DummyEstimator);

        assert!(settings.use_all_decorrelation_modes);
    }
}
