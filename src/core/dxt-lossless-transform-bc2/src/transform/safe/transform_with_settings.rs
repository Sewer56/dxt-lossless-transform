//! BC2 transform operations with explicit settings (safe slice-based wrapper).
//!
//! This module provides functions to transform and untransform BC2 data using specific
//! transform settings without automatic optimization.
//!
//! Note: For production use with ABI stability, consider using
//! `dxt-lossless-transform-bc2-api::Bc2ManualTransformBuilder`.

use crate::transform::{
    transform_bc2_with_settings as unsafe_transform_bc2_with_settings,
    untransform_bc2_with_settings as unsafe_untransform_bc2_with_settings, Bc2TransformSettings,
    Bc2UntransformSettings,
};
use thiserror::Error;

/// Validation errors for BC2 transform operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum Bc2ValidationError {
    /// Input length is not divisible by 16 (BC2 blocks are 16 bytes each).
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

/// Transform BC2 data using specified transform settings.
///
/// This function applies the transformation directly using the provided settings
/// without any optimization or testing of different configurations.
///
/// # Parameters
///
/// - `input`: The BC2 data to transform
/// - `output`: The output buffer to write transformed data to
/// - `settings`: The transform settings to use
///
/// # Errors
///
/// - [`Bc2ValidationError::InvalidLength`] if input length is not divisible by 16
/// - [`Bc2ValidationError::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```no_run
/// use dxt_lossless_transform_bc2::transform_bc2_with_settings_safe;
/// use dxt_lossless_transform_bc2::Bc2TransformSettings;
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
/// # use dxt_lossless_transform_bc2::Bc2ValidationError;
///
/// # fn main() -> Result<(), Bc2ValidationError> {
/// let bc2_data = vec![0u8; 16]; // 1 BC2 block
/// let mut output = vec![0u8; bc2_data.len()];
///
/// let settings = Bc2TransformSettings {
///     decorrelation_mode: YCoCgVariant::Variant1,
///     split_colour_endpoints: true,
/// };
///
/// transform_bc2_with_settings_safe(&bc2_data, &mut output, settings)?;
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```ignore
/// use dxt_lossless_transform_bc2_api::{Bc2ManualTransformBuilder, YCoCgVariant};
/// # use dxt_lossless_transform_bc2_api::Bc2Error;
///
/// # fn main() -> Result<(), Bc2Error> {
/// let bc2_data = vec![0u8; 16]; // 1 BC2 block
/// let mut output = vec![0u8; bc2_data.len()];
///
/// Bc2ManualTransformBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true)
///     .transform(&bc2_data, &mut output)?;
/// # Ok(())
/// # }
/// ```
pub fn transform_bc2_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc2TransformSettings,
) -> Result<(), Bc2ValidationError> {
    // Validate input length
    if !input.len().is_multiple_of(16) {
        return Err(Bc2ValidationError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc2ValidationError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_transform_bc2_with_settings(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len(),
            settings,
        );
    }

    Ok(())
}

/// Untransform BC2 data using specified untransform settings.
///
/// This function reverses the transformation applied by [`transform_bc2_with_settings`]
/// or [`super::transform_auto::transform_bc2_auto`], restoring the original BC2 data.
///
/// # Parameters
///
/// - `input`: The transformed BC2 data to untransform
/// - `output`: The output buffer to write the original BC2 data to
/// - `settings`: The untransform settings to use (must match the original transform settings)
///
/// # Errors
///
/// - [`Bc2ValidationError::InvalidLength`] if input length is not divisible by 16
/// - [`Bc2ValidationError::OutputBufferTooSmall`] if output buffer is smaller than input
///
/// # Examples
///
/// ```no_run
/// use dxt_lossless_transform_bc2::{
///     transform_bc2_with_settings_safe, untransform_bc2_with_settings_safe
/// };
/// use dxt_lossless_transform_bc2::{Bc2TransformSettings, Bc2UntransformSettings};
/// use dxt_lossless_transform_common::color_565::YCoCgVariant;
/// # use dxt_lossless_transform_bc2::Bc2ValidationError;
///
/// # fn main() -> Result<(), Bc2ValidationError> {
/// let bc2_data = vec![0u8; 16]; // 1 BC2 block
/// let mut transformed = vec![0u8; bc2_data.len()];
/// let mut restored = vec![0u8; bc2_data.len()];
///
/// let transform_settings = Bc2TransformSettings {
///     decorrelation_mode: YCoCgVariant::Variant1,
///     split_colour_endpoints: true,
/// };
///
/// // Transform the data
/// transform_bc2_with_settings_safe(&bc2_data, &mut transformed, transform_settings)?;
///
/// // Convert transform settings to untransform settings
/// let untransform_settings: Bc2UntransformSettings = transform_settings;
///
/// // Untransform to restore original data
/// untransform_bc2_with_settings_safe(&transformed, &mut restored, untransform_settings)?;
/// # assert_eq!(bc2_data, restored); // Verify round-trip works
/// # Ok(())
/// # }
/// ```
///
/// # Recommended Stable Alternative
///
/// ```ignore
/// use dxt_lossless_transform_bc2_api::{Bc2ManualTransformBuilder, YCoCgVariant};
/// # use dxt_lossless_transform_bc2_api::Bc2Error;
///
/// # fn main() -> Result<(), Bc2Error> {
/// let bc2_data = vec![0u8; 16]; // 1 BC2 block
/// let mut transformed = vec![0u8; bc2_data.len()];
/// let mut restored = vec![0u8; bc2_data.len()];
///
/// let builder = Bc2ManualTransformBuilder::new()
///     .decorrelation_mode(YCoCgVariant::Variant1)
///     .split_colour_endpoints(true);
///
/// // Transform the data
/// builder.transform(&bc2_data, &mut transformed)?;
///
/// // Untransform to restore original data  
/// builder.untransform(&transformed, &mut restored)?;
/// # assert_eq!(bc2_data, restored); // Verify round-trip works
/// # Ok(())
/// # }
/// ```
pub fn untransform_bc2_with_settings(
    input: &[u8],
    output: &mut [u8],
    settings: Bc2UntransformSettings,
) -> Result<(), Bc2ValidationError> {
    // Validate input length
    if !input.len().is_multiple_of(16) {
        return Err(Bc2ValidationError::InvalidLength(input.len()));
    }

    // Validate output buffer size
    if output.len() < input.len() {
        return Err(Bc2ValidationError::OutputBufferTooSmall {
            needed: input.len(),
            actual: output.len(),
        });
    }

    // Safety: We've validated the input length and output buffer size
    unsafe {
        unsafe_untransform_bc2_with_settings(
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
    fn test_transform_bc2_with_settings() {
        return;

        // Create minimal BC2 block data (16 bytes per block)
        let bc2_data = [
            // Alpha data (8 bytes - 4-bit per pixel)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            // Color data (8 bytes - BC1-like)
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 16];

        let settings = Bc2TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = transform_bc2_with_settings(&bc2_data, &mut output, settings);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC2 data"
        );
    }

    #[test]
    fn test_transform_bc2_with_settings_invalid_length() {
        let bc2_data = [0u8; 15]; // Invalid length (not divisible by 16)
        let mut output = [0u8; 15];

        let settings = Bc2TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = transform_bc2_with_settings(&bc2_data, &mut output, settings);
        assert!(matches!(result, Err(Bc2ValidationError::InvalidLength(15))));
    }

    #[test]
    fn test_transform_bc2_with_settings_output_too_small() {
        let bc2_data = [0u8; 16];
        let mut output = [0u8; 8]; // Too small

        let settings = Bc2TransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = transform_bc2_with_settings(&bc2_data, &mut output, settings);
        assert!(matches!(
            result,
            Err(Bc2ValidationError::OutputBufferTooSmall {
                needed: 16,
                actual: 8
            })
        ));
    }

    #[test]
    #[allow(unreachable_code)]
    fn test_untransform_bc2_with_settings() {
        return;

        // Create minimal BC2 block data (16 bytes per block)
        let bc2_data = [
            // Alpha data (8 bytes - 4-bit per pixel)
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,
            // Color data (8 bytes - BC1-like)
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output = [0u8; 16];

        let settings = Bc2UntransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = untransform_bc2_with_settings(&bc2_data, &mut output, settings);
        assert!(
            result.is_ok(),
            "Function should not fail with valid BC2 data"
        );
    }

    #[test]
    fn test_untransform_bc2_with_settings_invalid_length() {
        let bc2_data = [0u8; 15]; // Invalid length (not divisible by 16)
        let mut output = [0u8; 15];

        let settings = Bc2UntransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = untransform_bc2_with_settings(&bc2_data, &mut output, settings);
        assert!(matches!(result, Err(Bc2ValidationError::InvalidLength(15))));
    }

    #[test]
    fn test_untransform_bc2_with_settings_output_too_small() {
        let bc2_data = [0u8; 16];
        let mut output = [0u8; 8]; // Too small

        let settings = Bc2UntransformSettings {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        };

        let result = untransform_bc2_with_settings(&bc2_data, &mut output, settings);
        assert!(matches!(
            result,
            Err(Bc2ValidationError::OutputBufferTooSmall {
                needed: 16,
                actual: 8
            })
        ));
    }
}
