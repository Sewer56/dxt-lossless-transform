//! High-level convenience APIs for file format operations.

use crate::bundle::TransformBundle;
use crate::embed::{EmbeddableTransformDetails, TransformHeader};
use crate::error::{FileFormatError, FileFormatResult};
use crate::formats::EmbeddableBc1Details;
use crate::traits::FileFormatHandler;
use alloc::string::ToString;

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
/// use dxt_lossless_transform_file_formats_api::FileFormatResult;
///
/// fn example_transform(input: &[u8]) -> FileFormatResult<Vec<u8>> {
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
) -> FileFormatResult<()> {
    if input.len() != output.len() {
        return Err(FileFormatError::InvalidFileData(
            "Input and output buffers must be the same size".to_string(),
        ));
    }

    handler.transform_bundle(input, output, bundle)
}

/// Untransform a slice using the specified format handler.
///
/// This will:
/// 1. Extract transform details from the header
/// 2. Restore the original file format header
/// 3. Apply the appropriate untransform based on the detected format
///
/// # Parameters
///
/// - `handler`: The file format handler
/// - `input`: Input buffer containing transformed data
/// - `output`: Output buffer (must be same size as input)
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{untransform_slice_with, FileFormatResult};
/// use dxt_lossless_transform_dds::DdsHandler;
///
/// fn example_untransform(input: &[u8]) -> FileFormatResult<Vec<u8>> {
///     let mut output = vec![0u8; input.len()];
///     untransform_slice_with(&DdsHandler, input, &mut output)?;
///     Ok(output)
/// }
/// ```
pub fn untransform_slice_with<H: FileFormatHandler>(
    handler: &H,
    input: &[u8],
    output: &mut [u8],
) -> FileFormatResult<()> {
    if input.len() != output.len() {
        return Err(FileFormatError::InvalidFileData(
            "Input and output buffers must be the same size".to_string(),
        ));
    }

    handler.untransform(input, output)
}

/// Helper function to dispatch untransform based on header format.
///
/// This is used internally by format handlers after they extract the header.
/// It examines the transform format in the header and calls the appropriate
/// untransform function with the embedded settings.
///
/// # Parameters
///
/// - `header`: Transform header containing format type and settings
/// - `texture_data`: Mutable slice containing the texture data to untransform
///
/// # Example
///
/// ```
/// use dxt_lossless_transform_file_formats_api::{api::dispatch_untransform, FileFormatResult};
/// use dxt_lossless_transform_file_formats_api::embed::{TransformHeader, TransformFormat};
///
/// fn example_dispatch(file_data: &[u8], data_offset: usize) -> FileFormatResult<Vec<u8>> {
///     // Read header from file
///     let header = unsafe { TransformHeader::read_from_ptr(file_data.as_ptr()) };
///     
///     let mut texture_data = file_data[data_offset..].to_vec();
///     // Dispatch to appropriate untransform
///     dispatch_untransform(header, &mut texture_data)?;
///     Ok(texture_data)
/// }
/// ```
pub fn dispatch_untransform(
    header: TransformHeader,
    texture_data: &mut [u8],
) -> FileFormatResult<()> {
    use crate::embed::TransformFormat;

    match header.format() {
        TransformFormat::Bc1 => {
            let details = EmbeddableBc1Details::unpack(header.format_data())?;

            // BC1 untransform using unsafe API with safe wrapper
            if texture_data.len() % 8 != 0 {
                return Err(FileFormatError::InvalidFileData(
                    "BC1 data must be 8-byte aligned".to_string(),
                ));
            }

            unsafe {
                dxt_lossless_transform_bc1::untransform_bc1_with_settings(
                    texture_data.as_ptr(),
                    texture_data.as_mut_ptr(),
                    texture_data.len(),
                    details.into(),
                );
            }
        }
        TransformFormat::Bc2 => {
            // BC2 not yet implemented
            return Err(FileFormatError::UnsupportedFormat("BC2"));
        }
        TransformFormat::Bc3 => {
            // BC3 not yet implemented
            return Err(FileFormatError::UnsupportedFormat("BC3"));
        }
        TransformFormat::Bc7 => {
            // BC7 not yet implemented
            return Err(FileFormatError::UnsupportedFormat("BC7"));
        }
        _ => {
            return Err(FileFormatError::UnsupportedFormat(
                "Unknown transform format",
            ));
        }
    }

    Ok(())
}
