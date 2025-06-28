use crate::dds::parse_dds::parse_dds_ignore_magic;
use dxt_lossless_transform_file_formats_api::traits::FileFormatUntransformDetection;

use super::DdsHandler;

impl FileFormatUntransformDetection for DdsHandler {
    fn can_handle_untransform(&self, input: &[u8]) -> bool {
        if input.len() < 4 {
            return false;
        }

        // Try to parse as DDS ignoring the magic header (which contains transform data)
        parse_dds_ignore_magic(input).is_some()
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
    fn can_handle_untransform_accepts_transformed_dds() {
        let handler = DdsHandler;
        let mut transformed_dds = create_valid_bc1_dds(DDS_HEADER_SIZE);
        // Overwrite magic with transform header
        transformed_dds[0..4].copy_from_slice(&[0xAB, 0xCD, 0xEF, 0x12]);
        assert!(handler.can_handle_untransform(&transformed_dds));
    }

    #[test]
    fn can_handle_untransform_accepts_minimum_transformed_size() {
        let handler = DdsHandler;
        let mut min_transformed = create_valid_bc1_dds(DDS_HEADER_SIZE);
        min_transformed[0..4].copy_from_slice(&[0xAB, 0xCD, 0xEF, 0x12]);
        assert!(handler.can_handle_untransform(&min_transformed));
    }

    #[test]
    fn can_handle_untransform_rejects_just_under_minimum_size() {
        let handler = DdsHandler;
        let too_small_transform = [0u8; DDS_HEADER_SIZE - 1];
        assert!(!handler.can_handle_untransform(&too_small_transform));
    }

    #[test]
    fn supported_extensions_returns_dds() {
        let handler = DdsHandler;
        assert_eq!(handler.supported_extensions(), &["dds"]);
    }
}
