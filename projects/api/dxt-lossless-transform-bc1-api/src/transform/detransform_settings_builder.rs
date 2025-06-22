//! Builder pattern implementation for BC1 detransform settings.

use dxt_lossless_transform_bc1::{Bc1DetransformSettings, Bc1TransformSettings};
use dxt_lossless_transform_common::color_565::YCoCgVariant;

/// Builder for BC1 detransform options with convenient configuration methods.
#[derive(Debug, Clone, Copy)]
pub struct Bc1DetransformSettingsBuilder {
    decorrelation_mode: Option<YCoCgVariant>,
    split_colour_endpoints: Option<bool>,
}

impl Bc1DetransformSettingsBuilder {
    /// Create a new detransform settings builder.
    pub fn new() -> Self {
        Self {
            decorrelation_mode: None,
            split_colour_endpoints: None,
        }
    }

    /// Create a detransform settings builder from existing transform settings.
    pub fn from_transform_settings(settings: Bc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: Some(settings.decorrelation_mode),
            split_colour_endpoints: Some(settings.split_colour_endpoints),
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

    /// Build the detransform settings using the configured values.
    pub fn build(self) -> Bc1DetransformSettings {
        Bc1DetransformSettings {
            decorrelation_mode: self.decorrelation_mode.unwrap_or(YCoCgVariant::Variant1),
            split_colour_endpoints: self.split_colour_endpoints.unwrap_or(true),
        }
    }
}

impl Default for Bc1DetransformSettingsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Bc1TransformSettings> for Bc1DetransformSettingsBuilder {
    fn from(settings: Bc1TransformSettings) -> Self {
        Self::from_transform_settings(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detransform_settings_builder() {
        let settings = Bc1DetransformSettingsBuilder::new()
            .decorrelation_mode(YCoCgVariant::None)
            .split_colour_endpoints(false)
            .build();

        assert_eq!(settings.decorrelation_mode, YCoCgVariant::None);
        assert!(!settings.split_colour_endpoints);
    }

    #[test]
    fn test_detransform_settings_builder_defaults() {
        let settings = Bc1DetransformSettingsBuilder::new().build();

        assert_eq!(settings.decorrelation_mode, YCoCgVariant::Variant1);
        assert!(settings.split_colour_endpoints);
    }

    #[test]
    fn test_detransform_settings_from_transform_settings() {
        let transform_settings = Bc1TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant2,
            split_colour_endpoints: false,
        };

        let detransform_settings =
            Bc1DetransformSettingsBuilder::from_transform_settings(transform_settings).build();

        assert_eq!(
            detransform_settings.decorrelation_mode,
            YCoCgVariant::Variant2
        );
        assert!(!detransform_settings.split_colour_endpoints);
    }
}
