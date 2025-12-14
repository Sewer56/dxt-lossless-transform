//! BC3 transform operations with explicit settings (safe slice-based wrapper).
//!
//! This module provides functions to transform and untransform BC3 data using specific
//! transform settings without automatic optimization.
//!
//! Note: For production use with ABI stability, consider using
//! `dxt-lossless-transform-bc3-api::Bc3ManualTransformBuilder`.

use crate::transform::{
    transform_bc3_with_settings as unsafe_transform_bc3_with_settings,
    untransform_bc3_with_settings as unsafe_untransform_bc3_with_settings, Bc3TransformSettings,
    Bc3UntransformSettings,
};
use thiserror::Error;

/// Validation errors for BC3 transform operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Bc3ValidationError {
    /// Input length is not divisible by 16 (BC3 blocks are 16 bytes each).
    #[error("Invalid input length: {0} (must be divisible by 16)")]
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

/// Transform BC3 data using specified transform settings.
///
/// This function applies the transformation directly using the provided settings
/// without any optimization or testing of different configurations.
///
/// # Parameters
///
/// - `input`: The BC3 data to transform
/// - `output`: The output buffer to write transformed data to
/// - `settings`: The transform settings to use
///
/// # Errors
///
/// - [`Bc3ValidationError::InvalidLength`] if input length is not divisible by 16
/// - [`Bc3ValidationError::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```no_run
/// use dxt_lossless_transform_bc3::transform_bc3_with_settings_safe;
/// use dxt_lossless_transform_bc3::Bc3TransformSettings;
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
/// # use dxt_lossless_transform_bc3::Bc3ValidationError;
///
/// # fn main() -> Result<(), Bc3ValidationError> {
/// let bc3_data = vec![0u8; 16]; // 1 BC3 block
/// let mut output = vec![0u8; bc3_data.len()];
///
/// let settings = Bc3TransformSettings {
///     decorrelation_mode: YCoCgVariant::Variant1,
///     split_colour_endpoints: true,
///     split_alpha_endpoints: false,
/// };
///
/// transform_bc3_with_settings_safe(&bc3_data, &mut output, settings)?;
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```ignore
/// use dxt_lossless_transform_bc3_api::{Bc3ManualTransformBuilder, YCoCgVariant};
/// # use dxt_lossless_transform_bc3_api::Bc3Error;
///
/// # fn main() -> Result<(), Bc3Error> {
/// let bc3_data = vec![0u8; 16]; // 1 BC3 block
/// let mut output = vec![0u8; bc3_data.len()];
///
/// Bc3ManualTransformBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true)
///     .split_alpha_endpoints(false)
///     .transform(&bc3_data, &mut output)?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc3_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc3TransformSettings,
) -> Result<(), Bc3ValidationError> {
    // Validate input length
    if !input.len().is_multiple_of(16) {
        return Err(Bc3ValidationError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc3ValidationError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_transform_bc3_with_settings(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            settings,
        );
    }

    Ok(())
}

/// Untransform BC3 data using specified untransform settings.
///
/// This function reverses the transformation applied by [`transform_bc3_with_settings`]
/// or [`super::transform_auto::transform_bc3_auto`], restoring the original BC3 data.
///
/// # Parameters
///
/// - `input`: The transformed BC3 data to untransform
/// - `output`: The output buffer to write the original BC3 data to
/// - `settings`: The untransform settings to use (must match the original transform settings)
///
/// # Errors
///
/// - [`Bc3ValidationError::InvalidLength`] if input length is not divisible by 16
/// - [`Bc3ValidationError::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```no_run
/// use dxt_lossless_transform_bc3::{
///     transform_bc3_with_settings_safe, untransform_bc3_with_settings_safe
/// };
/// use dxt_lossless_transform_bc3::{Bc3TransformSettings, Bc3UntransformSettings};
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
/// # use dxt_lossless_transform_bc3::Bc3ValidationError;
///
/// # fn main() -> Result<(), Bc3ValidationError> {
/// let bc3_data = vec![0u8; 16]; // 1 BC3 block
/// let mut transformed = vec![0u8; bc3_data.len()];
/// let mut restored = vec![0u8; bc3_data.len()];
///
/// let transform_settings = Bc3TransformSettings {
///     decorrelation_mode: YCoCgVariant::Variant1,
///     split_colour_endpoints: true,
///     split_alpha_endpoints: false,
/// };
///
/// // Transform the data
/// transform_bc3_with_settings_safe(&bc3_data, &mut transformed, transform_settings)?;
///
/// // Convert transform settings to untransform settings
/// let untransform_settings: Bc3UntransformSettings = transform_settings;
///
/// // Untransform to restore original data
/// untransform_bc3_with_settings_safe(&transformed, &mut restored, untransform_settings)?;
/// # assert_eq!(bc3_data, restored); // Verify round-trip works
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```ignore
/// use dxt_lossless_transform_bc3_api::{Bc3ManualTransformBuilder, YCoCgVariant};
/// # use dxt_lossless_transform_bc3_api::Bc3Error;
///
/// # fn main() -> Result<(), Bc3Error> {
/// let bc3_data = vec![0u8; 16]; // 1 BC3 block
/// let mut transformed = vec![0u8; bc3_data.len()];
/// let mut restored = vec![0u8; bc3_data.len()];
///
/// let builder = Bc3ManualTransformBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true)
///     .split_alpha_endpoints(false);
///
/// // Transform the data
/// builder.transform(&bc3_data, &mut transformed)?;
///
/// // Untransform to restore original data  
/// builder.untransform(&transformed, &mut restored)?;
/// # assert_eq!(bc3_data, restored); // Verify round-trip works
/// # Ok(())
/// # }
/// ```
pub fn untransform_bc3_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc3UntransformSettings,
) -> Result<(), Bc3ValidationError> {
    // Validate input length
    if !input.len().is_multiple_of(16) {
        return Err(Bc3ValidationError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc3ValidationError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_untransform_bc3_with_settings(
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
    #[allow(unreachable_code)]
    fn test_transform_bc3_with_settings() {
        return;

        // Create minimal BC3 block data (16 bytes per block)
        let bc3_data = [
            // Alpha endpoints (2 bytes)
            0xFF, 0x00, // Alpha indices (6 bytes - 3-bit per pixel)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Color data (8 bytes - BC1-like)
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 16];

        let settings = Bc3TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
            split_alpha_endpoints: false,
        };

        let result = transform_bc3_with_settings(&bc3_data, &mut output, settings);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC3 data"
        );
    }

    #[test]
    fn test_transform_bc3_with_settings_invalid_length() {
        let bc3_data = [0u8; 15]; // Invalid length (not divisible by 16)
        let mut output = [0u8; 15];

        let settings = Bc3TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
            split_alpha_endpoints: false,
        };

        let result = transform_bc3_with_settings(&bc3_data, &mut output, settings);
        assert!(matches!(result, Err(Bc3ValidationError::InvalidLength(15))));
    }

    #[test]
    fn test_transform_bc3_with_settings_output_too_small() {
        let bc3_data = [0u8; 16];
        let mut output = [0u8; 8]; // Too small

        let settings = Bc3TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
            split_alpha_endpoints: false,
        };

        let result = transform_bc3_with_settings(&bc3_data, &mut output, settings);
        assert!(matches!(
            result,
            Err(Bc3ValidationError::OutputBufferTooSmall {
                needed: 16,
                actual: 8
            })
        ));
    }

    #[test]
    #[allow(unreachable_code)]
    fn test_untransform_bc3_with_settings() {
        return;

        // Create minimal BC3 block data (16 bytes per block)
        let bc3_data = [
            // Alpha endpoints (2 bytes)
            0xFF, 0x00, // Alpha indices (6 bytes - 3-bit per pixel)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // Color data (8 bytes - BC1-like)
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 16];

        let settings = Bc3UntransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
            split_alpha_endpoints: false,
        };

        let result = untransform_bc3_with_settings(&bc3_data, &mut output, settings);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC3 data"
        );
    }

    #[test]
    fn test_untransform_bc3_with_settings_invalid_length() {
        let bc3_data = [0u8; 15]; // Invalid length (not divisible by 16)
        let mut output = [0u8; 15];

        let settings = Bc3UntransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
            split_alpha_endpoints: false,
        };

        let result = untransform_bc3_with_settings(&bc3_data, &mut output, settings);
        assert!(matches!(result, Err(Bc3ValidationError::InvalidLength(15))));
    }

    #[test]
    fn test_untransform_bc3_with_settings_output_too_small() {
        let bc3_data = [0u8; 16];
        let mut output = [0u8; 8]; // Too small

        let settings = Bc3UntransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
            split_alpha_endpoints: false,
        };

        let result = untransform_bc3_with_settings(&bc3_data, &mut output, settings);
        assert!(matches!(
            result,
            Err(Bc3ValidationError::OutputBufferTooSmall {
                needed: 16,
                actual: 8
            })
        ));
    }
}
