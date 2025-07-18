//! Block extraction implementation for DDS files.

use super::format_conversion::dds_format_to_transform_format;
use crate::dds::parse_dds;
use dxt_lossless_transform_file_formats_api::{error::TransformResult, TransformError};
use dxt_lossless_transform_file_formats_debug::{
    ExtractedBlocks, FileFormatBlockExtraction, TransformFormatFilter,
};

impl FileFormatBlockExtraction for super::DdsHandler {
    fn extract_blocks<'a>(
        &self,
        data: &'a [u8],
        filter: TransformFormatFilter,
    ) -> TransformResult<Option<ExtractedBlocks<'a>>> {
        // Parse DDS header to get format information
        let dds_info = parse_dds(data).ok_or({
            TransformError::FormatHandler(
                dxt_lossless_transform_file_formats_api::error::FormatHandlerError::UnknownFileFormat,
            )
        })?;

        // Convert DDS format to transform format (allow unimplemented for debug/extraction)
        let transform_format = dds_format_to_transform_format(dds_info.format, true)?;

        // Check if the format matches the filter
        if !filter.accepts(transform_format) {
            return Ok(None);
        }

        // Calculate block data location and size
        let data_offset = dds_info.data_offset as usize;
        let data_length = dds_info.data_length as usize;
        let total_required = data_offset + data_length;

        // Validate input buffer contains enough data for declared texture size
        if data.len() < total_required {
            return Err(TransformError::FormatHandler(
                dxt_lossless_transform_file_formats_api::error::FormatHandlerError::InputTooShortForStatedTextureSize {
                    required: total_required,
                    actual: data.len(),
                },
            ));
        }

        // Extract the block data slice using calculated data length
        let block_data = &data[data_offset..data_offset + data_length];

        // Create the extracted blocks result
        let extracted = ExtractedBlocks::new(block_data, transform_format);

        Ok(Some(extracted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_blocks_invalid_dds() {
        let handler = super::super::DdsHandler;
        let invalid_data = b"not a dds file";

        let result = handler.extract_blocks(invalid_data, TransformFormatFilter::All);

        assert!(result.is_err());
    }
}
