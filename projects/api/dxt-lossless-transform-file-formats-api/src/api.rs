//! High-level convenience APIs for file format operations.

use crate::bundle::TransformBundle;
use crate::embed::{EmbeddableTransformDetails, TransformHeader};
use crate::error::{FileFormatError, FileFormatResult};
use crate::formats::{EmbeddableBc1Details, EmbeddableBc2Details, EmbeddableBc3Details};
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
/// ```ignore
/// use dxt_lossless_transform_file_formats_api::{TransformBundle, transform_slice_bundle};
/// use dxt_lossless_transform_dds::DdsHandler;
///
/// let bundle = TransformBundle::auto_all();
/// let mut output = vec![0u8; input.len()];
/// transform_slice_bundle(&DdsHandler, &input, &mut output, &bundle)?;
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
/// ```ignore
/// use dxt_lossless_transform_file_formats_api::untransform_slice_with;
/// use dxt_lossless_transform_dds::DdsHandler;
///
/// let mut output = vec![0u8; input.len()];
/// untransform_slice_with(&DdsHandler, &input, &mut output)?;
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
/// ```ignore
/// use dxt_lossless_transform_file_formats_api::{TransformHeader, dispatch_untransform};
/// use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
///
/// // Read header from file
/// let header = unsafe { TransformHeader::read_from_ptr(file_data.as_ptr()) };
///
/// // Dispatch to appropriate untransform
/// dispatch_untransform(header, &mut texture_data[data_offset..])?;
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
            let _details = EmbeddableBc2Details::unpack(header.format_data())?;

            // BC2 untransform using unsafe API
            if texture_data.len() % 16 != 0 {
                return Err(FileFormatError::InvalidFileData(
                    "BC2 data must be 16-byte aligned".to_string(),
                ));
            }

            unsafe {
                dxt_lossless_transform_bc2::untransform_bc2(
                    texture_data.as_ptr(),
                    texture_data.as_mut_ptr(),
                    texture_data.len(),
                    dxt_lossless_transform_bc2::BC2TransformDetails {},
                );
            }
        }
        TransformFormat::Bc3 => {
            let _details = EmbeddableBc3Details::unpack(header.format_data())?;

            // BC3 untransform using unsafe API
            if texture_data.len() % 16 != 0 {
                return Err(FileFormatError::InvalidFileData(
                    "BC3 data must be 16-byte aligned".to_string(),
                ));
            }

            unsafe {
                dxt_lossless_transform_bc3::untransform_bc3(
                    texture_data.as_ptr(),
                    texture_data.as_mut_ptr(),
                    texture_data.len(),
                    dxt_lossless_transform_bc3::BC3TransformDetails {},
                );
            }
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
