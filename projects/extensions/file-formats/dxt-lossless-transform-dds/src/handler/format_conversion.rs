//! Shared format conversion utilities for DDS handlers.

use crate::dds::DdsFormat;
use dxt_lossless_transform_file_formats_api::{
    embed::TransformFormat,
    error::{FormatHandlerError, TransformError, TransformResult},
};

/// Convert a [`DdsFormat`] to a [`TransformFormat`].
///
/// This function provides a centralized conversion from DDS-specific formats
/// to the generic [`TransformFormat`] used by the lossless transform API.
///
/// # Parameters
///
/// - `dds_format`: The DDS format to convert
/// - `allow_unimplemented`: If `true`, returns `Ok` for known but unimplemented formats.
///   If `false`, returns `Err(FormatNotImplemented)` for unimplemented formats.
///
/// # Returns
///
/// - `Ok(format)`: Successfully converted to a [`TransformFormat`]
/// - `Err(TransformError::FormatHandler(FormatNotImplemented))`: Format is known but not implemented (when `allow_unimplemented` is `false`)
/// - `Err(TransformError::FormatHandler(UnknownFileFormat))`: DDS format is not supported
///
/// # Supported Formats
///
/// - BC1 (DXT1) - implemented
/// - BC2 (DXT2/3) - implemented
/// - BC3 (DXT4/5) - known but unimplemented
/// - BC6H - known but unimplemented
/// - BC7 - known but unimplemented
/// - RGBA8888 - implemented
/// - BGRA8888 - implemented
/// - RGB888 - implemented
///
/// # Unsupported Formats
///
/// - Unknown or invalid formats
#[inline(always)]
pub(crate) fn dds_format_to_transform_format(
    dds_format: DdsFormat,
    allow_unimplemented: bool,
) -> TransformResult<TransformFormat> {
    match dds_format {
        DdsFormat::BC1 => Ok(TransformFormat::Bc1),
        DdsFormat::BC2 => Ok(TransformFormat::Bc2),
        DdsFormat::BC3 => {
            if allow_unimplemented {
                Ok(TransformFormat::Bc3)
            } else {
                Err(TransformError::FormatHandler(
                    FormatHandlerError::FormatNotImplemented(TransformFormat::Bc3),
                ))
            }
        }
        DdsFormat::BC6H => {
            if allow_unimplemented {
                Ok(TransformFormat::Bc6H)
            } else {
                Err(TransformError::FormatHandler(
                    FormatHandlerError::FormatNotImplemented(TransformFormat::Bc6H),
                ))
            }
        }
        DdsFormat::BC7 => {
            if allow_unimplemented {
                Ok(TransformFormat::Bc7)
            } else {
                Err(TransformError::FormatHandler(
                    FormatHandlerError::FormatNotImplemented(TransformFormat::Bc7),
                ))
            }
        }
        DdsFormat::RGBA8888 => Ok(TransformFormat::Rgba8888),
        DdsFormat::BGRA8888 => Ok(TransformFormat::Bgra8888),
        DdsFormat::RGB888 => Ok(TransformFormat::Rgb888),
        DdsFormat::NotADds | DdsFormat::Unknown => Err(TransformError::FormatHandler(
            FormatHandlerError::UnknownFileFormat,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implemented_formats() {
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC1, false).unwrap(),
            TransformFormat::Bc1
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC2, false).unwrap(),
            TransformFormat::Bc2
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::RGBA8888, false).unwrap(),
            TransformFormat::Rgba8888
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BGRA8888, false).unwrap(),
            TransformFormat::Bgra8888
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::RGB888, false).unwrap(),
            TransformFormat::Rgb888
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC1, true).unwrap(),
            TransformFormat::Bc1
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC2, true).unwrap(),
            TransformFormat::Bc2
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::RGBA8888, true).unwrap(),
            TransformFormat::Rgba8888
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BGRA8888, true).unwrap(),
            TransformFormat::Bgra8888
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::RGB888, true).unwrap(),
            TransformFormat::Rgb888
        );
    }

    #[test]
    fn test_unimplemented_formats_allowed() {
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC3, true).unwrap(),
            TransformFormat::Bc3
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC6H, true).unwrap(),
            TransformFormat::Bc6H
        );
        assert_eq!(
            dds_format_to_transform_format(DdsFormat::BC7, true).unwrap(),
            TransformFormat::Bc7
        );
    }

    #[test]
    fn test_unimplemented_formats_disallowed() {
        match dds_format_to_transform_format(DdsFormat::BC3, false) {
            Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
                TransformFormat::Bc3,
            ))) => {}
            _ => panic!("Expected FormatNotImplemented for BC3"),
        }
        match dds_format_to_transform_format(DdsFormat::BC6H, false) {
            Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
                TransformFormat::Bc6H,
            ))) => {}
            _ => panic!("Expected FormatNotImplemented for BC6H"),
        }
        match dds_format_to_transform_format(DdsFormat::BC7, false) {
            Err(TransformError::FormatHandler(FormatHandlerError::FormatNotImplemented(
                TransformFormat::Bc7,
            ))) => {}
            _ => panic!("Expected FormatNotImplemented for BC7"),
        }
    }

    #[test]
    fn test_unsupported_formats() {
        assert!(matches!(
            dds_format_to_transform_format(DdsFormat::Unknown, false),
            Err(TransformError::FormatHandler(
                FormatHandlerError::UnknownFileFormat
            ))
        ));
        assert!(matches!(
            dds_format_to_transform_format(DdsFormat::NotADds, false),
            Err(TransformError::FormatHandler(
                FormatHandlerError::UnknownFileFormat
            ))
        ));
    }
}
