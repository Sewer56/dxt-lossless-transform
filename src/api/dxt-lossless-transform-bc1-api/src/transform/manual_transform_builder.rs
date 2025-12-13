//! Builder pattern implementation for BC1 manual transform configuration.

use super::YCoCgVariant;
use crate::Bc1Error;
use dxt_lossless_transform_bc1::{
    Bc1TransformSettings, Bc1UntransformSettings, transform_bc1_with_settings_safe,
    untransform_bc1_with_settings_safe,
};

/// Manual BC1 transform configuration builder.
///
/// Allows precise control over transform parameters like decorrelation mode
/// and color endpoint splitting. Ideal when you know what settings work
/// best for your specific use case.
///
/// For automatic optimization, use [`crate::Bc1AutoTransformBuilder`].
#[derive(Debug, Clone, Copy)]
pub struct Bc1ManualTransformBuilder {
    settings: Bc1TransformSettings,
}

impl Bc1ManualTransformBuilder {
    /// Create a new manual transform builder.
    pub fn new() -> Self {
        Self {
            settings: Bc1TransformSettings::default(),
        }
    }

    /// Get the current transform settings.
    ///
    /// **Internal API**: This method exposes internal transform settings from the unstable core crate.
    /// This is not intended for public use and may change or be removed in future versions.
    ///
    /// Returns a copy of the current transform settings configured on this builder.
    #[doc(hidden)]
    pub fn get_settings(&self) -> Bc1TransformSettings {
        self.settings
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
    /// For automatic optimization, consider using [`crate::Bc1AutoTransformBuilder`] instead.
    pub fn decorrelation_mode(mut self, mode: YCoCgVariant) -> Self {
        self.settings.decorrelation_mode = mode.to_internal_variant();
        self
    }

    /// Set whether to split colour endpoints.
    ///
    /// This setting controls whether BC1 texture color endpoints are separated during processing,
    /// which can improve compression efficiency for many textures.
    ///
    /// **File Size**: This setting reduces file size around 78% of the time.
    ///
    /// For automatic optimization, consider using [`crate::Bc1AutoTransformBuilder`] instead.
    pub fn split_colour_endpoints(mut self, split: bool) -> Self {
        self.settings.split_colour_endpoints = split;
        self
    }

    /// Transform BC1 data using the configured settings.
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
    /// ```
    /// use dxt_lossless_transform_bc1_api::{Bc1ManualTransformBuilder, YCoCgVariant};
    /// # use dxt_lossless_transform_bc1_api::Bc1Error;
    ///
    /// # fn main() -> Result<(), Bc1Error> {
    /// let bc1_data = vec![0u8; 8]; // 1 BC1 block
    /// let mut transformed = vec![0u8; 8];
    /// let mut restored = vec![0u8; 8];
    ///
    /// let builder = Bc1ManualTransformBuilder::new()
    ///     .decorrelation_mode(YCoCgVariant::Variant1)
    ///     .split_colour_endpoints(true);
    ///
    /// // Transform
    /// builder.transform(&bc1_data, &mut transformed)?;
    ///
    /// // Later, untransform with the same builder
    /// builder.untransform(&transformed, &mut restored)?;
    /// # assert_eq!(bc1_data, restored); // Verify round-trip works
    /// # Ok(())
    /// # }
    /// ```
    pub fn transform(&self, input: &[u8], output: &mut [u8]) -> Result<(), Bc1Error> {
        transform_bc1_with_settings_safe(input, output, self.settings)
            .map_err(Bc1Error::from_validation_error)
    }

    /// Untransform BC1 data using the configured settings.
    ///
    /// This method reverses the transformation applied by [`transform`](Self::transform),
    /// using the same configuration that was used for the original transformation.
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
    /// ```
    /// use dxt_lossless_transform_bc1_api::{Bc1ManualTransformBuilder, YCoCgVariant};
    /// # use dxt_lossless_transform_bc1_api::Bc1Error;
    ///
    /// # fn main() -> Result<(), Bc1Error> {
    /// let transformed_data = vec![0u8; 8]; // 1 transformed BC1 block
    /// let mut output = vec![0u8; 8];
    ///
    /// let builder = Bc1ManualTransformBuilder::new()
    ///     .decorrelation_mode(YCoCgVariant::Variant1)
    ///     .split_colour_endpoints(true);
    ///
    /// builder.untransform(&transformed_data, &mut output)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn untransform(&self, input: &[u8], output: &mut [u8]) -> Result<(), Bc1Error> {
        let untransform_settings: Bc1UntransformSettings = self.settings;
        untransform_bc1_with_settings_safe(input, output, untransform_settings)
            .map_err(Bc1Error::from_validation_error)
    }
}

impl Default for Bc1ManualTransformBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_transform_builder_transform() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 8];

        let builder = Bc1ManualTransformBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .split_colour_endpoints(true);

        let result = builder.transform(&bc1_data, &mut output);

        assert!(
            result.is_ok(),
            "transform should not fail with valid BC1 data"
        );
    }

    #[test]
    fn test_manual_transform_builder_round_trip() {
        // First transform some data
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut transformed = [0u8; 8];
        let mut restored = [0u8; 8];

        let builder = Bc1ManualTransformBuilder::new()
            .decorrelation_mode(YCoCgVariant::Variant1)
            .split_colour_endpoints(true);

        // Transform
        let transform_result = builder.transform(&bc1_data, &mut transformed);
        assert!(
            transform_result.is_ok(),
            "Transform should not fail with valid BC1 data"
        );

        // Untransform with same settings
        let untransform_result = builder.untransform(&transformed, &mut restored);
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
