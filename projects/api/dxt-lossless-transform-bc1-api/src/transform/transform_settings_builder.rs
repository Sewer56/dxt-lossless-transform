//! Builder pattern implementation for BC1 transform settings.

use crate::Bc1Error;
use crate::transform::{transform_bc1_with_settings, untransform_bc1_with_settings};
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
    ///
    /// This setting controls whether BC1 texture color endpoints are separated during processing,
    /// which can improve compression efficiency for many textures.
    ///
    /// **File Size**: This setting reduces file size around 78% of the time.
    ///
    /// For automatic optimization, consider using [`transform_bc1_auto`] instead.
    ///
    /// [`transform_bc1_auto`]: crate::transform_bc1_auto
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

    /// Build the transform settings and transform BC1 data in one operation.
    ///
    /// This is a convenience method that combines building the settings and calling
    /// [`transform_bc1_with_settings`] in a single operation.
    ///
    /// # Parameters
    /// - `input`: The BC1 data to transform
    /// - `output`: The output buffer where transformed data will be written
    ///
    /// # Returns
    /// Ok(()) on success, or an error on failure.
    ///
    /// # Errors
    /// Returns [`Bc1Error`] if the transformation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use dxt_lossless_transform_bc1_api::Bc1TransformSettingsBuilder;
    /// use dxt_lossless_transform_common::color_565::YCoCgVariant;
    ///
    /// let bc1_data = vec![0u8; 8]; // 1 BC1 block
    /// let mut output = vec![0u8; 8];
    ///
    /// Bc1TransformSettingsBuilder::new()
    ///     .decorrelation_mode(YCoCgVariant::Variant1)
    ///     .split_colour_endpoints(true)
    ///     .build_and_transform(&bc1_data, &mut output)?;
    /// ```
    pub fn build_and_transform(self, input: &[u8], output: &mut [u8]) -> Result<(), Bc1Error> {
        let settings = self.build();
        transform_bc1_with_settings(input, output, settings)
    }

    /// Build the transform settings and untransform BC1 data in one operation.
    ///
    /// This is a convenience method that combines building the settings and calling
    /// [`untransform_bc1_with_settings`] in a single operation.
    ///
    /// # Parameters
    /// - `input`: The transformed BC1 data to untransform
    /// - `output`: The output buffer where original BC1 data will be written
    ///
    /// # Returns
    /// Ok(()) on success, or an error on failure.
    ///
    /// # Errors
    /// Returns [`Bc1Error`] if the untransformation fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use dxt_lossless_transform_bc1_api::Bc1TransformSettingsBuilder;
    /// use dxt_lossless_transform_common::color_565::YCoCgVariant;
    ///
    /// let transformed_data = vec![0u8; 8]; // 1 transformed BC1 block
    /// let mut output = vec![0u8; 8];
    ///
    /// Bc1TransformSettingsBuilder::new()
    ///     .decorrelation_mode(YCoCgVariant::Variant1)
    ///     .split_colour_endpoints(true)
    ///     .build_and_untransform(&transformed_data, &mut output)?;
    /// ```
    pub fn build_and_untransform(self, input: &[u8], output: &mut [u8]) -> Result<(), Bc1Error> {
        let settings = self.build();
        let detransform_settings = settings.into();
        untransform_bc1_with_settings(input, output, detransform_settings)
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

    #[test]
    fn test_transform_settings_builder_build_and_transform() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 8];

        let result = Bc1TransformSettingsBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .split_colour_endpoints(true)
            .build_and_transform(&bc1_data, &mut output);

        assert!(
            result.is_ok(),
            "build_and_transform should not fail with valid BC1 data"
        );
    }

    #[test]
    fn test_transform_settings_builder_build_and_untransform() {
        // First transform some data
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut transformed = [0u8; 8];
        let mut restored = [0u8; 8];

        let builder = Bc1TransformSettingsBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .split_colour_endpoints(true);

        // Transform
        let transform_result = builder.build_and_transform(&bc1_data, &mut transformed);
        assert!(
            transform_result.is_ok(),
            "Transform should not fail with valid BC1 data"
        );

        // Untransform with same settings
        let untransform_result = builder.build_and_untransform(&transformed, &mut restored);
        assert!(
            untransform_result.is_ok(),
            "Untransform should not fail with valid transformed data"
        );

        // Verify round-trip
        assert_eq!(
            bc1_data, restored,
            "Round-trip transform/untransform should restore original data"
        );
    }
}
