//! High-level convenience APIs for file format operations.

use crate::bundle::TransformBundle;
use crate::embed::{
    EmbeddableBc1Details, EmbeddableTransformDetails, TransformFormat, TransformHeader,
};
use crate::error::{FormatHandlerError, TransformError, TransformResult};
use crate::traits::FileFormatHandler;
use core::fmt::Debug;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Transform a slice using the specified format handler and transform bundle.
///
/// This is the main entry point for transforming file data. The handler will:
/// 1. Detect the specific BCx format in the file
/// 2. Use the appropriate builder from the bundle
/// 3. Embed transform details in the output header
///
/// # Parameters
///
/// - `handler`: The file format handler (e.g., DdsHandler)
/// - `input`: Input buffer containing the file data
/// - `output`: Output buffer (must be at least the same size as input)
/// - `bundle`: Bundle containing transform builders for different BCx formats
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_with_bundle};
/// use dxt_lossless_transform_api_common::estimate::NoEstimation;
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_file_formats_api::TransformResult;
///
/// fn example_transform(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let bundle = TransformBundle::<NoEstimation>::default_all();
///     let mut output = vec![0u8; input.len()];
///     transform_slice_with_bundle(&DdsHandler, input, &mut output, &bundle)?;
///     Ok(output)
/// }
/// ```
pub fn transform_slice_with_bundle<H: FileFormatHandler, T>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
    bundle: &TransformBundle<T>,
) -> TransformResult<()>
where
    T: SizeEstimationOperations,
    T::Error: Debug,
{
    if output.len() < input.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            },
        ));
    }

    handler.transform_bundle(input, output, bundle)
}

/// Untransform a slice using the specified format handler.
///
/// This will:
/// 1. Extract transform details from the header
/// 2. Restore the original file format header
/// 3. Dispatch to the appropriate untransform function
///
/// # Parameters
///
/// - `handler`: The file format handler (e.g., DdsHandler)
/// - `input`: Input buffer containing transformed data
/// - `output`: Output buffer (must be at least the same size as input)
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{untransform_slice};
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_file_formats_api::TransformResult;
///
/// fn example_untransform(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let mut output = vec![0u8; input.len()];
///     untransform_slice(&DdsHandler, input, &mut output)?;
///     Ok(output)
/// }
/// ```
pub fn untransform_slice<H: FileFormatHandler>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
) -> TransformResult<()> {
    if output.len() < input.len() {
        return Err(TransformError::FormatHandler(
            FormatHandlerError::OutputBufferTooSmall {
                required: input.len(),
                actual: output.len(),
            },
        ));
    }

    handler.untransform(input, output)
}

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
        TransformFormat::Bc1 => {
            let details = EmbeddableBc1Details::from_header(header)?;

            // BC1 untransform using unsafe API with safe wrapper
            if input_texture_data.len() % 8 != 0 {
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
                    details.into(),
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
