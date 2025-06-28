//! High-level convenience APIs for file format operations.

use crate::bundle::TransformBundle;
use crate::embed::{EmbeddableTransformDetails, TransformFormat, TransformHeader};
use crate::error::{TransformError, TransformResult};
use crate::formats::EmbeddableBc1Details;
use crate::traits::FileFormatHandler;

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
/// - `output`: Output buffer (must be same size as input)
/// - `bundle`: Bundle containing transform builders for different BCx formats
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_bundle};
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_file_formats_api::TransformResult;
///
/// fn example_transform(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let bundle = TransformBundle::default_all();
///     let mut output = vec![0u8; input.len()];
///     transform_slice_bundle(&DdsHandler, input, &mut output, &bundle)?;
///     Ok(output)
/// }
/// ```
pub fn transform_slice_bundle<H: FileFormatHandler>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
    bundle: &TransformBundle,
) -> TransformResult<()> {
    if input.len() != output.len() {
        return Err(TransformError::BufferSizeMismatch {
            input_len: input.len(),
            output_len: output.len(),
        });
    }

    handler.transform_bundle(input, output, bundle)
}

/// Untransform a slice using the specified format handler.
///
/// This will:
/// 1. Extract transform details from the header
/// 2. Restore the original file format headers
/// 3. Apply the reverse transform to the texture data
///
/// # Parameters
///
/// - `handler`: The file format handler (e.g., DdsHandler)
/// - `input`: Input buffer containing the transformed file data
/// - `output`: Output buffer (must be same size as input)
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{untransform_slice_with};
/// use dxt_lossless_transform_dds::DdsHandler;
/// use dxt_lossless_transform_file_formats_api::TransformResult;
///
/// fn example_untransform(input: &[u8]) -> TransformResult<Vec<u8>> {
///     let mut output = vec![0u8; input.len()];
///     untransform_slice_with(&DdsHandler, input, &mut output)?;
///     Ok(output)
/// }
/// ```
pub fn untransform_slice_with<H: FileFormatHandler>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
) -> TransformResult<()> {
    if input.len() != output.len() {
        return Err(TransformError::BufferSizeMismatch {
            input_len: input.len(),
            output_len: output.len(),
        });
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
/// - `output_texture_data`: Output slice where the untransformed texture data will be written
///
/// # Safety Requirements
///
/// Both input and output texture data must be properly sized for the format:
/// - BC1: Must be multiple of 8 bytes
/// - BC2/BC3: Must be multiple of 16 bytes  
/// - BC7: Must be multiple of 16 bytes
///
/// Input and output buffers must be the same size.
///
/// # Example
///
/// See: `dxt-lossless-transform-dds` crate.
pub fn dispatch_untransform(
    header: TransformHeader,
    input_texture_data: &[u8],
    output_texture_data: &mut [u8],
) -> TransformResult<()> {
    if input_texture_data.len() != output_texture_data.len() {
        return Err(TransformError::BufferSizeMismatch {
            input_len: input_texture_data.len(),
            output_len: output_texture_data.len(),
        });
    }

    match header.format() {
        TransformFormat::Bc1 => {
            let details = EmbeddableBc1Details::unpack(header.format_data())?;

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
        TransformFormat::Bc2 => {
            return Err(TransformError::FormatNotImplemented(TransformFormat::Bc2));
        }
        TransformFormat::Bc3 => {
            return Err(TransformError::FormatNotImplemented(TransformFormat::Bc3));
        }
        TransformFormat::Bc7 => {
            return Err(TransformError::FormatNotImplemented(TransformFormat::Bc7));
        }
        _ => {
            return Err(TransformError::UnknownTransformFormat);
        }
    }

    Ok(())
}
