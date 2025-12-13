use crate::dds::parse_dds::parse_dds;
use dxt_lossless_transform_file_formats_api::handlers::FileFormatDetection;

use super::DdsHandler;

impl FileFormatDetection for DdsHandler {
    fn can_handle(&self, input: &[u8], file_extension: Option<&str>) -> bool {
        // Check file extension first for performance
        if let Some(ext) = file_extension {
            if ext != "dds" {
                return false;
            }
        }

        // If extension is correct or not provided, check file content
        parse_dds(input).is_some()
    }
}

// These tests exist purely for safety, in case underlying implementation changes.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::DDS_HEADER_SIZE;
    use crate::test_prelude::*;

    #[test]
    fn can_handle_accepts_valid_bc1_dds() {
        let handler = DdsHandler;
        let valid_dds = create_valid_bc1_dds();
        assert!(handler.can_handle(&valid_dds, Some("dds")));
        assert!(handler.can_handle(&valid_dds, None)); // Should also work without extension
    }

    #[test]
    fn can_handle_rejects_invalid_data_no_magic() {
        let handler = DdsHandler;
        let invalid_data = [0u8; DDS_HEADER_SIZE];
        assert!(!handler.can_handle(&invalid_data, Some("dds")));
    }

    #[test]
    fn can_handle_rejects_wrong_extension() {
        let handler = DdsHandler;
        let valid_dds = create_valid_bc1_dds();
        assert!(!handler.can_handle(&valid_dds, Some("txt")));
    }

    #[test]
    fn can_handle_accepts_minimum_valid_size() {
        let handler = DdsHandler;
        let min_valid = create_valid_bc1_dds();
        assert!(handler.can_handle(&min_valid, Some("dds")));
    }

    #[test]
    fn can_handle_rejects_just_under_minimum_size() {
        let handler = DdsHandler;
        let too_small = create_truncated_dds(DDS_HEADER_SIZE - 1);
        assert!(!handler.can_handle(&too_small, Some("dds")));
    }
}
