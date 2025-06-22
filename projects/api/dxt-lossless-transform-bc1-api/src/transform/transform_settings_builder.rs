//! Builder pattern implementation for BC1 transform settings.

use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Builder for BC1 transform options with convenient configuration methods.
#[derive(Debug, Clone, Copy)]
pub struct Bc1TransformSettingsBuilder {
    decorrelation_mode: Option<YCoCgVariant>,
    split_colour_endpoints: Option<bool>,
}

impl Bc1TransformSettingsBuilder {
    /// Create a new transform options builder.
    pub fn new() -> Self {
        Self {
            decorrelation_mode: None,
            split_colour_endpoints: None,
        }
    }

    /// Set the decorrelation mode.
    ///
    /// Controls the YCoCg-R color space decorrelation variant used for transformation.
    /// Different variants can provide varying compression ratios depending on the texture content.
    ///
    /// **Note**: When manually testing decorrelation modes, the typical improvement from
    /// using different variants is <0.1% in practice. For better compression gains,
    /// it's recommended to use a compression level on the estimator (e.g., ZStandard estimator)
    /// closer to your final compression level instead.
    ///
    /// For automatic optimization, consider using [`transform_bc1_auto`] instead.
    ///
    /// [`transform_bc1_auto`]: crate::transform_bc1_auto
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

impl Default for Bc1TransformSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_settings_builder() {
        let settings = Bc1TransformSettingsBuilder::new()
            .decorrelation_mode(YCoCgVariant::None)
            .split_colour_endpoints(false)
            .build();

        assert_eq!(settings.decorrelation_mode, YCoCgVariant::None);
        assert!(!settings.split_colour_endpoints);
    }

    #[test]
    fn test_transform_settings_builder_defaults() {
        let settings = Bc1TransformSettingsBuilder::new().build();

        assert_eq!(settings.decorrelation_mode, YCoCgVariant::Variant1);
        assert!(settings.split_colour_endpoints);
    }
}
