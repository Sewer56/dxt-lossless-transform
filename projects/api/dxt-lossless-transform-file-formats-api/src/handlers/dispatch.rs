//! Lower-level dispatch operations for file format handlers.
//!
//! This module provides dispatch functions for transforming and untransforming texture data
//! based on detected formats. These functions operate directly on texture data and are
//! primarily intended for use by file format handlers.

use crate::bundle::TransformBundle;
use crate::embed::formats::{
    EmbeddableBc1Details, EmbeddableBc2Details, EmbeddableTransformDetails,
};
use crate::embed::{TransformFormat, TransformHeader};
use crate::error::{FormatHandlerError, TransformError, TransformResult};
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Dispatch untransform operation based on the transform header format.
///
/// This is a lower-level function that operates directly on texture data,
/// assuming the file format headers have already been processed.
///
/// # Parameters
///
/// - `header`: The transform header containing format and settings
/// - `input_texture_data`: Input slice containing the transformed texture data
/// - `output_texture_data`: Output slice where the untransformed texture data will be written (must be at least the same size as input)
///
/// # Safety Requirements
///
/// Both input and output texture data must be properly sized for the format:
/// - BC1: Must be multiple of 8 bytes
/// - BC2/BC3: Must be multiple of 16 bytes  
/// - BC7: Must be multiple of 16 bytes
///
/// Output buffer must be at least the same size as the input buffer.
///
/// # Example
///
/// See: `dxt-lossless-transform-dds` crate.
pub fn dispatch_untransform(
    header: TransformHeader,
    input_texture_data: &[u8],
    output_texture_data: &mut [u8],
) -> TransformResult<()> {
    if output_texture_data.len() < input_texture_data.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input_texture_data.len(),
                actual: output_texture_data.len(),
            },
        ));
    }

    match header.format() {
        Some(TransformFormat::Bc1) => {
            let details = EmbeddableBc1Details::from_header(header)?;

            // BC1 untransform using unsafe API with safe wrapper
            if !input_texture_data.len().is_multiple_of(8) {
                return Err(TransformError::InvalidDataAlignment {
                    size: input_texture_data.len(),
                    required_divisor: 8,
                });
            }

            unsafe {
                dxt_lossless_transform_bc1::untransform_bc1_with_settings(
                    input_texture_data.as_ptr(),
                    output_texture_data.as_mut_ptr(),
                    input_texture_data.len(),
                    details.to_settings(),
                );
            }
        }
        Some(TransformFormat::Bc2) => {
            let details = EmbeddableBc2Details::from_header(header)?;

            // BC2 untransform using unsafe API with safe wrapper
            if !input_texture_data.len().is_multiple_of(16) {
                return Err(TransformError::InvalidDataAlignment {
                    size: input_texture_data.len(),
                    required_divisor: 16,
                });
            }

            unsafe {
                dxt_lossless_transform_bc2::untransform_bc2_with_settings(
                    input_texture_data.as_ptr(),
                    output_texture_data.as_mut_ptr(),
                    input_texture_data.len(),
                    details.get_settings(),
                );
            }
        }
        _ => {
            return Err(TransformError::UnknownTransformFormat);
        }
    }

    Ok(())
}

/// Dispatch transform operation based on the detected format.
///
/// This is a lower-level function that operates directly on texture data,
/// assuming the file format headers have already been processed.
///
/// # Parameters
///
/// - `format`: The detected texture format to transform
/// - `input_texture_data`: Input slice containing the original texture data
/// - `output_texture_data`: Output slice where the transformed texture data will be written (must be at least the same size as input)
/// - `bundle`: Bundle containing transform builders for different BCx formats
///
/// # Returns
///
/// Returns a [`TransformHeader`] containing the transform details that should be embedded in the file.
///
/// # Safety Requirements
///
/// Both input and output texture data must be properly sized for the format:
/// - BC1: Must be multiple of 8 bytes
/// - BC2/BC3: Must be multiple of 16 bytes  
/// - BC7: Must be multiple of 16 bytes
///
/// Output buffer must be at least the same size as the input buffer.
///
/// # Example
///
/// See: `dxt-lossless-transform-dds` crate.
pub fn dispatch_transform<T>(
    format: TransformFormat,
    input_texture_data: &[u8],
    output_texture_data: &mut [u8],
    bundle: &TransformBundle<T>,
) -> TransformResult<TransformHeader>
where
    T: SizeEstimationOperations,
    T::Error: Debug,
{
    bundle.dispatch_transform(format, input_texture_data, output_texture_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use alloc::vec;

    #[test]
    fn test_dispatch_untransform_invalid_alignment() {
        let header = create_test_bc1_header();
        let input = vec![0u8; 15]; // Not multiple of 8
        let mut output = vec![0u8; 15];

        let result = dispatch_untransform(header, &input, &mut output);
        assert!(matches!(
            result,
            Err(TransformError::InvalidDataAlignment { .. })
        ));
    }
}
