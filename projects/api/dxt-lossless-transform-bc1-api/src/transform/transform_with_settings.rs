//! BC1 transform operations with explicit settings.
//!
//! This module provides functions to transform BC1 data using specific transform settings
//! without automatic optimization.

use crate::error::Bc1Error;
use dxt_lossless_transform_bc1::{
    Bc1TransformSettings, transform_bc1_with_settings as core_transform_bc1_with_settings,
};

/// Transform BC1 data using specified transform settings.
///
/// This function applies the transformation directly using the provided settings
/// without any optimization or testing of different configurations.
///
/// # Parameters
///
/// - `input`: The BC1 data to transform
/// - `output`: The output buffer to write transformed data to
/// - `settings`: The transform settings to use
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
/// use dxt_lossless_transform_bc1_api::{transform_bc1_with_settings, Bc1TransformSettingsBuilder};
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
///
/// let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
/// let mut output = vec![0u8; bc1_data.len()];
///
/// let settings = Bc1TransformSettingsBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true)
///     .build();
///
/// transform_bc1_with_settings(&bc1_data, &mut output, settings)?;
/// ```
pub fn transform_bc1_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc1TransformSettings,
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
        core_transform_bc1_with_settings(
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
    fn test_transform_bc1_with_settings() {
        // Create minimal BC1 block data (8 bytes per block)
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 8];

        let settings = Bc1TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = transform_bc1_with_settings(&bc1_data, &mut output, settings);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC1 data"
        );
    }

    #[test]
    fn test_transform_bc1_with_settings_invalid_length() {
        let bc1_data = [0u8; 7]; // Invalid length (not divisible by 8)
        let mut output = [0u8; 7];

        let settings = Bc1TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = transform_bc1_with_settings(&bc1_data, &mut output, settings);
        assert!(matches!(result, Err(Bc1Error::InvalidLength(7))));
    }

    #[test]
    fn test_transform_bc1_with_settings_output_too_small() {
        let bc1_data = [0u8; 8];
        let mut output = [0u8; 4]; // Too small

        let settings = Bc1TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = transform_bc1_with_settings(&bc1_data, &mut output, settings);
        assert!(matches!(
            result,
            Err(Bc1Error::OutputBufferTooSmall {
                needed: 8,
                actual: 4
            })
        ));
    }
}
