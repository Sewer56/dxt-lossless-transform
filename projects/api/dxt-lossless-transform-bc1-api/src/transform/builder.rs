//! Builder pattern implementation for BC1 estimate options.

use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_bc1::Bc1EstimateOptions;
use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Builder for BC1 estimate options with convenient configuration methods.
#[derive(Debug, Clone, Copy)]
pub struct Bc1EstimateOptionsBuilder {
    use_all_decorrelation_modes: Option<bool>,
}

impl Bc1EstimateOptionsBuilder {
    /// Create a new options builder.
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
    pub fn use_all_decorrelation_modes(mut self, use_all: bool) -> Self {
        self.use_all_decorrelation_modes = Some(use_all);
        self
    }

    /// Build the estimate options using the configured values and provided estimator.
    pub fn build<T>(self, size_estimator: T) -> Bc1EstimateOptions<T>
    where
        T: SizeEstimationOperations,
    {
        Bc1EstimateOptions {
            size_estimator,
            use_all_decorrelation_modes: self.use_all_decorrelation_modes.unwrap_or(false),
        }
    }
}

impl Default for Bc1EstimateOptionsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for BC1 transform options with convenient configuration methods.
#[derive(Debug, Clone, Copy)]
pub struct Bc1TransformOptionsBuilder {
    decorrelation_mode: Option<YCoCgVariant>,
    split_colour_endpoints: Option<bool>,
}

impl Bc1TransformOptionsBuilder {
    /// Create a new transform options builder.
    pub fn new() -> Self {
        Self {
            decorrelation_mode: None,
            split_colour_endpoints: None,
        }
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

    /// Build the transform settings using the configured values.
    pub fn build(self) -> Bc1TransformSettings {
        Bc1TransformSettings {
            decorrelation_mode: self.decorrelation_mode.unwrap_or(YCoCgVariant::Variant1),
            split_colour_endpoints: self.split_colour_endpoints.unwrap_or(true),
        }
    }
}

impl Default for Bc1TransformOptionsBuilder {
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
    fn test_estimate_options_builder() {
        let options = Bc1EstimateOptionsBuilder::new()
            .use_all_decorrelation_modes(true)
            .build(DummyEstimator);

        assert!(options.use_all_decorrelation_modes);
    }

    #[test]
    fn test_transform_options_builder() {
        let settings = Bc1TransformOptionsBuilder::new()
            .decorrelation_mode(YCoCgVariant::None)
            .split_colour_endpoints(false)
            .build();

        assert_eq!(settings.decorrelation_mode, YCoCgVariant::None);
        assert!(!settings.split_colour_endpoints);
    }

    #[test]
    fn test_transform_options_builder_defaults() {
        let settings = Bc1TransformOptionsBuilder::new().build();

        assert_eq!(settings.decorrelation_mode, YCoCgVariant::Variant1);
        assert!(settings.split_colour_endpoints);
    }
}
