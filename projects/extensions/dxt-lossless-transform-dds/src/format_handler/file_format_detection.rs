use crate::dds::parse_dds::parse_dds;
use dxt_lossless_transform_file_formats_api::traits::FileFormatDetection;

use super::DdsHandler;

impl FileFormatDetection for DdsHandler {
    fn can_handle(&self, input: &[u8]) -> bool {
        parse_dds(input).is_some()
    }

    fn supported_extensions(&self) -> &[&str] {
        &["dds"]
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
        let valid_dds = create_valid_bc1_dds(DDS_HEADER_SIZE);
        assert!(handler.can_handle(&valid_dds));
    }

    #[test]
    fn can_handle_rejects_invalid_data_no_magic() {
        let handler = DdsHandler;
        let invalid_data = [0u8; DDS_HEADER_SIZE];
        assert!(!handler.can_handle(&invalid_data));
    }

    #[test]
    fn can_handle_accepts_minimum_valid_size() {
        let handler = DdsHandler;
        let min_valid = create_valid_bc1_dds(DDS_HEADER_SIZE);
        assert!(handler.can_handle(&min_valid));
    }

    #[test]
    fn can_handle_rejects_just_under_minimum_size() {
        let handler = DdsHandler;
        let too_small = create_valid_bc1_dds(DDS_HEADER_SIZE - 1);
        assert!(!handler.can_handle(&too_small));
    }
}
