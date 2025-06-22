//! DDS file format handler for file format operations.

use crate::dds::{constants::DDS_MAGIC, parse_dds, DdsFormat, DdsInfo};
use dxt_lossless_transform_api_common::embed::{EmbedError, TransformHeader};
use dxt_lossless_transform_api_fileformats::traits::{FileFormatFilter, FileFormatHandler};

/// DDS file format handler for file format operations.
pub struct DdsHandler;

impl FileFormatHandler for DdsHandler {
    type Info = DdsInfo;
    type Format = DdsFormat;
    type Error = EmbedError;

    fn detect_format(data: &[u8]) -> Option<Self::Info> {
        if data.len() < 4 {
            return None;
        }

        unsafe { parse_dds(data.as_ptr(), data.len()) }
    }

    fn get_data_offset(info: &Self::Info) -> usize {
        info.data_offset as usize
    }

    fn get_format(info: &Self::Info) -> Self::Format {
        info.format
    }

    unsafe fn embed_transform_header(
        ptr: *mut u8,
        header: TransformHeader,
    ) -> Result<(), Self::Error> {
        header.write_to_ptr(ptr);
        Ok(())
    }

    unsafe fn unembed_transform_header(
        ptr: *mut u8,
    ) -> Result<(TransformHeader, Self::Info), Self::Error> {
        // Read the stored header
        let header = TransformHeader::read_from_ptr(ptr);

        // Restore the original DDS magic header
        (ptr as *mut u32).write_unaligned(DDS_MAGIC);

        // Parse the file info after restoring the header
        // We need to create a slice for parsing, but we only have a pointer
        // This is a limitation - we need the file size to properly parse
        // For now, we'll create a minimal DdsInfo that has what we need
        let info = DdsInfo {
            format: DdsFormat::BC1, // We'll need to improve this
            data_offset: 128,       // Standard DDS header size
                                    // Add other required fields with defaults
        };

        Ok((header, info))
    }
}

/// DDS format filter for batch operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdsFormatFilter {
    /// Process only BC1 files
    BC1,
    /// Process only BC2 files
    BC2,
    /// Process only BC3 files
    BC3,
    /// Process only BC7 files
    BC7,
    /// Process all supported formats
    All,
}

impl FileFormatFilter for DdsFormatFilter {
    type Format = DdsFormat;

    fn matches(&self, format: Self::Format) -> bool {
        matches!(
            (self, format),
            (Self::BC1, DdsFormat::BC1)
                | (Self::BC2, DdsFormat::BC2)
                | (Self::BC3, DdsFormat::BC3)
                | (Self::BC7, DdsFormat::BC7)
                | (
                    Self::All,
                    DdsFormat::BC1 | DdsFormat::BC2 | DdsFormat::BC3 | DdsFormat::BC7
                )
        )
    }

    fn accepted_formats(&self) -> &[Self::Format] {
        match self {
            Self::BC1 => &[DdsFormat::BC1],
            Self::BC2 => &[DdsFormat::BC2],
            Self::BC3 => &[DdsFormat::BC3],
            Self::BC7 => &[DdsFormat::BC7],
            Self::All => &[
                DdsFormat::BC1,
                DdsFormat::BC2,
                DdsFormat::BC3,
                DdsFormat::BC7,
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::embed::TransformFormat;

    fn create_minimal_dds_data() -> Vec<u8> {
        let mut data = vec![0u8; 128]; // Minimal DDS header size
        unsafe {
            (data.as_mut_ptr() as *mut u32).write_unaligned(DDS_MAGIC);
        }
        data
    }

    #[test]
    fn test_dds_handler_detect_format() {
        let dds_data = create_minimal_dds_data();
        let info = DdsHandler::detect_format(&dds_data);

        // Should detect some format (exact format depends on the test data structure)
        assert!(info.is_some());
    }

    #[test]
    fn test_dds_format_filter() {
        let filter = DdsFormatFilter::BC1;
        assert!(filter.matches(DdsFormat::BC1));
        assert!(!filter.matches(DdsFormat::BC2));

        let filter = DdsFormatFilter::All;
        assert!(filter.matches(DdsFormat::BC1));
        assert!(filter.matches(DdsFormat::BC2));
        assert!(filter.matches(DdsFormat::BC3));
        assert!(filter.matches(DdsFormat::BC7));
    }

    #[test]
    fn test_header_store_extract() {
        let mut dds_data = create_minimal_dds_data();
        // Use a 28-bit value (0x02345678) instead of 32-bit to fit in format_data field
        let header = TransformHeader::new(TransformFormat::Bc1, 0x02345678);

        // Test storing header
        unsafe {
            DdsHandler::embed_transform_header(dds_data.as_mut_ptr(), header).unwrap();
        }

        // Verify header was changed
        let stored_header = unsafe { TransformHeader::read_from_ptr(dds_data.as_ptr()) };
        assert_eq!(stored_header.format(), TransformFormat::Bc1);
        assert_eq!(stored_header.format_data(), 0x02345678);

        // Test extracting header
        let (recovered_header, _info) =
            unsafe { DdsHandler::unembed_transform_header(dds_data.as_mut_ptr()).unwrap() };

        // Verify header was recovered correctly
        assert_eq!(recovered_header.format(), TransformFormat::Bc1);
        assert_eq!(recovered_header.format_data(), 0x02345678);

        // Verify DDS magic was restored
        let restored_magic = unsafe { (dds_data.as_ptr() as *const u32).read_unaligned() };
        assert_eq!(restored_magic, DDS_MAGIC);
    }
}
