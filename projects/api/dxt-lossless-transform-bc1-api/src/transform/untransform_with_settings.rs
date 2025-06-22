//! BC1 untransform operations with explicit settings.
//!
//! This module provides functions to untransform BC1 data using specific detransform settings.

use crate::error::Bc1Error;
use dxt_lossless_transform_bc1::{
    Bc1DetransformSettings, untransform_bc1_with_settings as core_untransform_bc1_with_settings,
};

/// Untransform BC1 data using specified detransform settings.
///
/// This function reverses the transformation applied by [`transform_bc1_with_settings`](crate::transform_bc1_with_settings)
/// or [`transform_bc1_auto`](crate::transform_bc1_auto), restoring the original BC1 data.
///
/// # Parameters
///
/// - `input`: The transformed BC1 data to untransform
/// - `output`: The output buffer to write the original BC1 data to
/// - `settings`: The detransform settings to use (must match the original transform settings)
///
/// # Errors
///
/// - [`Bc1Error::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1Error::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Safety
///
/// The underlying function is safe to call with validated inputs.
///
/// # Examples
///
/// ```ignore
/// use dxt_lossless_transform_bc1_api::{
///     transform_bc1_with_settings, untransform_bc1_with_settings,
///     Bc1TransformSettingsBuilder, Bc1DetransformSettingsBuilder
/// };
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
///
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let mut transformed = vec![0u8; bc1_data.len()];
/// let mut restored = vec![0u8; bc1_data.len()];
///
/// let transform_settings = Bc1TransformSettingsBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true)
///     .build();
///
/// // Transform the data
/// transform_bc1_with_settings(&bc1_data, &mut transformed, transform_settings)?;
///
/// // Create matching detransform settings
/// let detransform_settings = Bc1DetransformSettingsBuilder::from_transform_settings(transform_settings)
///     .build();
///
/// // Untransform to restore original data
/// untransform_bc1_with_settings(&transformed, &mut restored, detransform_settings)?;
/// ```
pub fn untransform_bc1_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc1DetransformSettings,
) -> Result<(), Bc1Error> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1Error::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1Error::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        core_untransform_bc1_with_settings(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            settings,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_common::color_565::YCoCgVariant;

    #[test]
    fn test_untransform_bc1_with_settings() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 8];

        let settings = Bc1DetransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = untransform_bc1_with_settings(&bc1_data, &mut output, settings);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC1 data"
        );
    }

    #[test]
    fn test_untransform_bc1_with_settings_invalid_length() {
        let bc1_data = [0u8; 7]; // Invalid length (not divisible by 8)
        let mut output = [0u8; 7];

        let settings = Bc1DetransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = untransform_bc1_with_settings(&bc1_data, &mut output, settings);
        assert!(matches!(result, Err(Bc1Error::InvalidLength(7))));
    }

    #[test]
    fn test_untransform_bc1_with_settings_output_too_small() {
        let bc1_data = [0u8; 8];
        let mut output = [0u8; 4]; // Too small

        let settings = Bc1DetransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = untransform_bc1_with_settings(&bc1_data, &mut output, settings);
        assert!(matches!(
            result,
            Err(Bc1Error::OutputBufferTooSmall {
                needed: 8,
                actual: 4
            })
        ));
    }
}
