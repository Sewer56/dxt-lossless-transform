//! DDS format handler implementation.

use super::{
    constants::DDS_MAGIC,
    parse_dds::{parse_dds, parse_dds_ignore_magic, DdsFormat},
};
use dxt_lossless_transform_file_formats_api::{
    bundle::{Bc1TransformBuilderExt, TransformBundle},
    embed::{EmbeddableTransformDetails, TransformHeader},
    error::{FileFormatError, FileFormatResult},
    formats::{EmbeddableBc1Details, EmbeddableBc2Details, EmbeddableBc3Details},
    traits::{FileFormatDetection, FileFormatHandler},
};

/// Handler for DDS file format.
///
/// This handler supports BC1/BC2/BC3/BC7 formats within DDS files,
/// embedding transform details in the 4-byte DDS magic header.
/// Currently only BC1 supports configurable transform options.
pub struct DdsHandler;

impl FileFormatHandler for DdsHandler {
    fn can_handle(&self, input: &[u8]) -> bool {
        unsafe { parse_dds(input.as_ptr(), input.len()).is_some() }
    }

    fn transform_bundle(
        &self,
        input: &[u8],
        output: &mut [u8],
        bundle: &TransformBundle,
    ) -> FileFormatResult<()> {
        // Parse DDS header
        let info = unsafe { parse_dds(input.as_ptr(), input.len()) }
            .ok_or_else(|| FileFormatError::InvalidFileData("Not a valid DDS file".to_string()))?;

        let data_offset = info.data_offset as usize;

        // Copy headers to output
        output[..data_offset].copy_from_slice(&input[..data_offset]);

        // Transform based on detected format
        let header = match info.format {
            DdsFormat::BC1 => {
                let builder = bundle
                    .bc1
                    .as_ref()
                    .ok_or(FileFormatError::NoBuilderForFormat("BC1"))?;

                let details = builder.transform_slice_with_details(
                    &input[data_offset..],
                    &mut output[data_offset..],
                )?;

                EmbeddableBc1Details::from(details).to_header()
            }
            DdsFormat::BC2 => {
                return Err(FileFormatError::UnsupportedFormat(
                    "BC2 not yet implemented",
                ));
            }
            DdsFormat::BC3 => {
                return Err(FileFormatError::UnsupportedFormat(
                    "BC3 not yet implemented",
                ));
            }
            DdsFormat::BC7 => {
                return Err(FileFormatError::UnsupportedFormat(
                    "BC7 not yet implemented",
                ));
            }
            DdsFormat::Unknown => {
                return Err(FileFormatError::UnsupportedFormat("Unknown DDS format"));
            }
            DdsFormat::NotADds => {
                return Err(FileFormatError::InvalidFileData(
                    "Not a DDS file".to_string(),
                ));
            }
        };

        // Embed transform header (overwrites DDS magic)
        unsafe {
            header.write_to_ptr(output.as_mut_ptr());
        }

        Ok(())
    }

    fn untransform(&self, input: &[u8], output: &mut [u8]) -> FileFormatResult<()> {
        // Read transform header from first 4 bytes
        let header = unsafe { TransformHeader::read_from_ptr(input.as_ptr()) };

        // Copy entire input to output
        output.copy_from_slice(input);

        // Restore DDS magic
        output[0..4].copy_from_slice(&DDS_MAGIC.to_le_bytes());

        // Validate restored DDS and get info
        let info = unsafe { parse_dds(output.as_ptr(), output.len()) }.ok_or_else(|| {
            FileFormatError::InvalidFileData("Corrupted DDS data after restore".to_string())
        })?;

        let data_offset = info.data_offset as usize;

        // Dispatch untransform based on header format
        dxt_lossless_transform_file_formats_api::api::dispatch_untransform(
            header,
            &mut output[data_offset..],
        )?;

        Ok(())
    }
}

impl FileFormatDetection for DdsHandler {
    fn can_handle_untransform(&self, input: &[u8]) -> bool {
        if input.len() < 4 {
            return false;
        }

        // Try to parse as DDS ignoring the magic header (which contains transform data)
        unsafe { parse_dds_ignore_magic(input.as_ptr(), input.len()).is_some() }
    }

    fn supported_extensions(&self) -> &[&str] {
        &["dds"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dds_handler_can_handle() {
        let handler = DdsHandler;

        // Valid DDS data
        let mut valid_dds = vec![0u8; 128];
        valid_dds[0..4].copy_from_slice(&DDS_MAGIC.to_le_bytes());
        assert!(handler.can_handle(&valid_dds));

        // Invalid data
        let invalid_data = vec![0u8; 128];
        assert!(!handler.can_handle(&invalid_data));

        // Too small
        let too_small = vec![0u8; 10];
        assert!(!handler.can_handle(&too_small));
    }
}
