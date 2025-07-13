//! Format inspection implementation for DDS files.

use super::format_conversion::dds_format_to_transform_format;
use crate::dds::parse_dds;
use dxt_lossless_transform_file_formats_api::{
    embed::TransformFormat, error::TransformResult, TransformError,
};
use dxt_lossless_transform_file_formats_debug::{TransformFormatCheck, TransformFormatFilter};

impl TransformFormatCheck for super::DdsHandler {
    fn get_transform_format(
        &self,
        data: &[u8],
        filter: TransformFormatFilter,
    ) -> TransformResult<Option<TransformFormat>> {
        // Parse DDS header to get format information
        let dds_info = parse_dds(data).ok_or({
            TransformError::FormatHandler(
                dxt_lossless_transform_file_formats_api::error::FormatHandlerError::UnknownFileFormat,
            )
        })?;

        // Convert DDS format to transform format (allow unimplemented for debug/inspection)
        let transform_format = dds_format_to_transform_format(dds_info.format, true)?;

        // Check if the format matches the filter
        if !filter.accepts(transform_format) {
            return Ok(None);
        }

        Ok(Some(transform_format))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    #[test]
    fn test_get_transform_format_bc1() {
        let handler = super::super::DdsHandler;
        let dds_data = create_valid_bc1_dds_with_dimensions(64, 64, 1);

        let result = handler.get_transform_format(&dds_data, TransformFormatFilter::All);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(TransformFormat::Bc1));
    }

    #[test]
    fn test_get_transform_format_bc7() {
        let handler = super::super::DdsHandler;
        let dds_data = create_valid_bc7_dds();

        let result = handler.get_transform_format(&dds_data, TransformFormatFilter::All);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(TransformFormat::Bc7));
    }

    #[test]
    fn test_get_transform_format_with_filter_match() {
        let handler = super::super::DdsHandler;
        let dds_data = create_valid_bc1_dds_with_dimensions(64, 64, 1);

        let result = handler.get_transform_format(&dds_data, TransformFormatFilter::Bc1);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(TransformFormat::Bc1));
    }

    #[test]
    fn test_get_transform_format_with_filter_no_match() {
        let handler = super::super::DdsHandler;
        let dds_data = create_valid_bc1_dds_with_dimensions(64, 64, 1);

        let result = handler.get_transform_format(&dds_data, TransformFormatFilter::Bc7);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_get_transform_format_invalid_dds() {
        let handler = super::super::DdsHandler;
        let invalid_data = b"not a dds file";

        let result = handler.get_transform_format(invalid_data, TransformFormatFilter::All);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_transform_format_unsupported_format() {
        let handler = super::super::DdsHandler;
        let dds_data = create_valid_rgba8888_dds_with_dimensions(64, 64, 1);

        let result = handler.get_transform_format(&dds_data, TransformFormatFilter::All);

        assert!(result.is_err());
    }
}
