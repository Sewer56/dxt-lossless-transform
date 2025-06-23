//! BC1 transform operations with explicit settings (safe slice-based wrapper).
//!
//! This module provides functions to transform and untransform BC1 data using specific
//! transform settings without automatic optimization.
//!
//! Note: For production use with ABI stability, consider using
//! `dxt-lossless-transform-bc1-api::Bc1ManualTransformBuilder`.

use crate::transform::{
    transform_bc1_with_settings as unsafe_transform_bc1_with_settings,
    untransform_bc1_with_settings as unsafe_untransform_bc1_with_settings, Bc1DetransformSettings,
    Bc1TransformSettings,
};
use thiserror::Error;

/// Validation errors for BC1 transform operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Bc1ValidationError {
    /// Input length is not divisible by 8 (BC1 blocks are 8 bytes each).
    #[error("Invalid input length: {0} (must be divisible by 8)")]
    InvalidLength(usize),

    /// Output buffer is too small to hold the transformed data.
    #[error("Output buffer too small: needed {needed}, got {actual}")]
    OutputBufferTooSmall {
        /// The required buffer size.
        needed: usize,
        /// The actual buffer size provided.
        actual: usize,
    },
}

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
/// - [`Bc1ValidationError::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1ValidationError::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```
/// use dxt_lossless_transform_bc1::transform_bc1_with_settings_safe;
/// use dxt_lossless_transform_bc1::Bc1TransformSettings;
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
/// # use dxt_lossless_transform_bc1::Bc1ValidationError;
///
/// # fn main() -> Result<(), Bc1ValidationError> {
/// let bc1_data = vec![0u8; 8]; // 1 BC1 block
/// let mut output = vec![0u8; bc1_data.len()];
///
/// let settings = Bc1TransformSettings {
///     decorrelation_mode: YCoCgVariant::Variant1,
///     split_colour_endpoints: true,
/// };
///
/// transform_bc1_with_settings_safe(&bc1_data, &mut output, settings)?;
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```
/// use dxt_lossless_transform_bc1_api::{Bc1ManualTransformBuilder, YCoCgVariant};
/// # use dxt_lossless_transform_bc1_api::Bc1Error;
///
/// # fn main() -> Result<(), Bc1Error> {
/// let bc1_data = vec![0u8; 8]; // 1 BC1 block
/// let mut output = vec![0u8; bc1_data.len()];
///
/// Bc1ManualTransformBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true)
///     .transform(&bc1_data, &mut output)?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc1_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc1TransformSettings,
) -> Result<(), Bc1ValidationError> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1ValidationError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1ValidationError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_transform_bc1_with_settings(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            settings,
        );
    }

    Ok(())
}

/// Untransform BC1 data using specified detransform settings.
///
/// This function reverses the transformation applied by [`transform_bc1_with_settings`]
/// or [`super::transform_auto::transform_bc1_auto`], restoring the original BC1 data.
///
/// # Parameters
///
/// - `input`: The transformed BC1 data to untransform
/// - `output`: The output buffer to write the original BC1 data to
/// - `settings`: The detransform settings to use (must match the original transform settings)
///
/// # Errors
///
/// - [`Bc1ValidationError::InvalidLength`] if input length is not divisible by 8
/// - [`Bc1ValidationError::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```
/// use dxt_lossless_transform_bc1::{
///     transform_bc1_with_settings_safe, untransform_bc1_with_settings_safe
/// };
/// use dxt_lossless_transform_bc1::{Bc1TransformSettings, Bc1DetransformSettings};
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
/// # use dxt_lossless_transform_bc1::Bc1ValidationError;
///
/// # fn main() -> Result<(), Bc1ValidationError> {
/// let bc1_data = vec![0u8; 8]; // 1 BC1 block
/// let mut transformed = vec![0u8; bc1_data.len()];
/// let mut restored = vec![0u8; bc1_data.len()];
///
/// let transform_settings = Bc1TransformSettings {
///     decorrelation_mode: YCoCgVariant::Variant1,
///     split_colour_endpoints: true,
/// };
///
/// // Transform the data
/// transform_bc1_with_settings_safe(&bc1_data, &mut transformed, transform_settings)?;
///
/// // Convert transform settings to detransform settings
/// let detransform_settings: Bc1DetransformSettings = transform_settings.into();
///
/// // Untransform to restore original data
/// untransform_bc1_with_settings_safe(&transformed, &mut restored, detransform_settings)?;
/// # assert_eq!(bc1_data, restored); // Verify round-trip works
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```
/// use dxt_lossless_transform_bc1_api::{Bc1ManualTransformBuilder, YCoCgVariant};
/// # use dxt_lossless_transform_bc1_api::Bc1Error;
///
/// # fn main() -> Result<(), Bc1Error> {
/// let bc1_data = vec![0u8; 8]; // 1 BC1 block
/// let mut transformed = vec![0u8; bc1_data.len()];
/// let mut restored = vec![0u8; bc1_data.len()];
///
/// let builder = Bc1ManualTransformBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true);
///
/// // Transform the data
/// builder.transform(&bc1_data, &mut transformed)?;
///
/// // Untransform to restore original data  
/// builder.detransform(&transformed, &mut restored)?;
/// # assert_eq!(bc1_data, restored); // Verify round-trip works
/// # Ok(())
/// # }
/// ```
pub fn untransform_bc1_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc1DetransformSettings,
) -> Result<(), Bc1ValidationError> {
    // Validate input length
    if input.len() % 8 != 0 {
        return Err(Bc1ValidationError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc1ValidationError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_untransform_bc1_with_settings(
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
        assert!(matches!(result, Err(Bc1ValidationError::InvalidLength(7))));
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
            Err(Bc1ValidationError::OutputBufferTooSmall {
                needed: 8,
                actual: 4
            })
        ));
    }

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
        assert!(matches!(result, Err(Bc1ValidationError::InvalidLength(7))));
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
            Err(Bc1ValidationError::OutputBufferTooSmall {
                needed: 8,
                actual: 4
            })
        ));
    }
}
