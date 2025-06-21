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
    /// When `false` (default), only tests [`YCoCgVariant::Variant1`] and [`YCoCgVariant::None`]
    /// for faster optimization with good results.
    ///
    /// When `true`, tests all available decorrelation modes for potentially better
    /// compression at the cost of twice as long optimization time (tests 4 options
    /// instead of 2) for negligible gains (typically <0.1% extra savings).
    ///
    /// [`YCoCgVariant::Variant1`]: dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant1
    /// [`YCoCgVariant::None`]: dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::None
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
    use crate::test_prelude::DummyEstimator;

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
