//! Builder pattern implementation for BC1 transform options.

use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc1::Bc1TransformDetails;

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
}
