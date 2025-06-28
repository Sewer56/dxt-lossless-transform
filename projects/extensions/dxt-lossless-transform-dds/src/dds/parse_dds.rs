use super::{constants::*, likely_dds};

/// Defines a known data format within a DDS file; suitable for lossless transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DdsFormat {
    /// Indicates the data is not a DDS file.
    /// This is mostly reserved for the C API, where a native 'Option' type is not available.
    NotADds = 0,
    /// This is a DDS file, but not in a format we know.
    Unknown = 1,
    /// a.k.a. DXT1
    BC1 = 2,
    /// a.k.a. DXT2/3
    BC2 = 3,
    /// a.k.a. DXT4/5
    BC3 = 4,
    BC7 = 5,
}

/// The information of the DDS file supplied to the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct DdsInfo {
    pub format: DdsFormat,
    pub data_offset: u8,
}

/// Attempts to parse a the data format of a DDS file from the given slice.
///
/// # Return
///
/// `None` if the file is not a valid DDS file with a known format, or if
/// the length is insufficient to read the headers.
///
/// Otherwise, a [`DdsInfo`] with the format and data offset.
///
/// # Notes
///
/// The function will return [`None`] if the format is not known.
/// For DX10 headers, it will check the `DXGI_FORMAT` field to determine
/// the format.
#[inline]
pub fn parse_dds(data: &[u8]) -> Option<DdsInfo> {
    if !likely_dds(data) {
        return None;
    }

    parse_dds_ignore_magic(data)
}

/// Attempts to parse a DDS file format ignoring the magic header validation.
///
/// This function is used for transformed files where the original DDS magic
/// has been overwritten with transform details. It assumes the input has a
/// valid DDS structure except for the magic header.
///
/// # Return
///
/// `None` if the length is insufficient to read the headers or format is unknown.
/// Otherwise, a [`DdsInfo`] with the format and data offset.
#[inline]
pub fn parse_dds_ignore_magic(data: &[u8]) -> Option<DdsInfo> {
    // Check minimum length for DDS header
    if data.len() < DDS_HEADER_SIZE {
        return None;
    }

    // SAFETY: We checked data.len() >= DDS_HEADER_SIZE (128), so FOURCC_OFFSET (0x54) + 4 is safe
    let fourcc = unsafe { (data.as_ptr().add(FOURCC_OFFSET) as *const u32).read_unaligned() };

    let (format, data_offset) = if fourcc == FOURCC_DX10 {
        // DX10 header present, ensure the data is long enough.
        if data.len() < DDS_HEADER_SIZE + DX10_HEADER_SIZE {
            return None;
        }

        // SAFETY: We checked data.len() >= DDS_HEADER_SIZE + DX10_HEADER_SIZE (148),
        // so DX10_FORMAT_OFFSET (0x80) + 4 is safe
        let dxgi_format =
            unsafe { (data.as_ptr().add(DX10_FORMAT_OFFSET) as *const u32).read_unaligned() };
        let format = match dxgi_format {
            DXGI_FORMAT_BC1_TYPELESS | DXGI_FORMAT_BC1_UNORM | DXGI_FORMAT_BC1_UNORM_SRGB => {
                DdsFormat::BC1
            }
            DXGI_FORMAT_BC2_TYPELESS | DXGI_FORMAT_BC2_UNORM | DXGI_FORMAT_BC2_UNORM_SRGB => {
                DdsFormat::BC2
            }
            DXGI_FORMAT_BC3_TYPELESS | DXGI_FORMAT_BC3_UNORM | DXGI_FORMAT_BC3_UNORM_SRGB => {
                DdsFormat::BC3
            }
            DXGI_FORMAT_BC7_TYPELESS | DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB => {
                DdsFormat::BC7
            }
            _ => DdsFormat::Unknown,
        };

        // 148 bytes: 128 byte header + 20 byte DX10 header
        (format, DDS_HEADER_SIZE + DX10_HEADER_SIZE)
    } else {
        // Legacy header
        let format = match fourcc {
            FOURCC_DXT1 => DdsFormat::BC1,
            FOURCC_DXT2 | FOURCC_DXT3 => DdsFormat::BC2,
            FOURCC_DXT4 | FOURCC_DXT5 => DdsFormat::BC3,
            _ => DdsFormat::Unknown,
        };

        // 128 bytes: standard header size
        (format, DDS_HEADER_SIZE)
    };

    Some(DdsInfo {
        format,
        data_offset: data_offset as u8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::DDS_HEADER_SIZE;
    use crate::test_prelude::DDS_DX10_TOTAL_HEADER_SIZE;
    use crate::test_prelude::*;

    #[rstest]
    #[case(FOURCC_DXT1, DdsFormat::BC1)]
    #[case(FOURCC_DXT2, DdsFormat::BC2)]
    #[case(FOURCC_DXT3, DdsFormat::BC2)]
    #[case(FOURCC_DXT4, DdsFormat::BC3)]
    #[case(FOURCC_DXT5, DdsFormat::BC3)]
    fn parse_dds_handles_legacy_formats(#[case] fourcc: u32, #[case] expected_format: DdsFormat) {
        let mut data = create_valid_bc1_dds(DDS_HEADER_SIZE);

        // Override the FOURCC with the test case
        unsafe {
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(fourcc);
        }

        let info = parse_dds(&data).unwrap();
        assert_eq!(info.format, expected_format);
        assert_eq!(info.data_offset, DDS_HEADER_SIZE as u8);
    }

    #[rstest]
    #[case(FOURCC_DXT1, DdsFormat::BC1)]
    #[case(FOURCC_DXT2, DdsFormat::BC2)]
    #[case(FOURCC_DXT3, DdsFormat::BC2)]
    #[case(FOURCC_DXT4, DdsFormat::BC3)]
    #[case(FOURCC_DXT5, DdsFormat::BC3)]
    fn parse_dds_ignore_magic_handles_legacy_formats(
        #[case] fourcc: u32,
        #[case] expected_format: DdsFormat,
    ) {
        // Verifies can parse with ignore magic, nothing more.
        let mut data = create_valid_bc1_dds(DDS_HEADER_SIZE);

        // Override the magic and FOURCC for the test case
        unsafe {
            // Set invalid magic header (simulating transform header)
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(0xDEADBEEF);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(fourcc);
        }

        // Regular parse_dds should fail
        assert!(parse_dds(&data).is_none());

        // parse_dds_ignore_magic should succeed
        let info = parse_dds_ignore_magic(&data).unwrap();
        assert_eq!(info.format, expected_format);
        assert_eq!(info.data_offset, DDS_HEADER_SIZE as u8);
    }

    #[rstest]
    #[case(DXGI_FORMAT_BC1_TYPELESS, DdsFormat::BC1)]
    #[case(DXGI_FORMAT_BC1_UNORM, DdsFormat::BC1)]
    #[case(DXGI_FORMAT_BC1_UNORM_SRGB, DdsFormat::BC1)]
    #[case(DXGI_FORMAT_BC2_TYPELESS, DdsFormat::BC2)]
    #[case(DXGI_FORMAT_BC2_UNORM, DdsFormat::BC2)]
    #[case(DXGI_FORMAT_BC2_UNORM_SRGB, DdsFormat::BC2)]
    #[case(DXGI_FORMAT_BC3_TYPELESS, DdsFormat::BC3)]
    #[case(DXGI_FORMAT_BC3_UNORM, DdsFormat::BC3)]
    #[case(DXGI_FORMAT_BC3_UNORM_SRGB, DdsFormat::BC3)]
    #[case(DXGI_FORMAT_BC7_TYPELESS, DdsFormat::BC7)]
    #[case(DXGI_FORMAT_BC7_UNORM, DdsFormat::BC7)]
    #[case(DXGI_FORMAT_BC7_UNORM_SRGB, DdsFormat::BC7)]
    fn parse_dds_handles_dx10_formats(
        #[case] dxgi_format: u32,
        #[case] expected_format: DdsFormat,
    ) {
        let mut data = create_valid_bc7_dds(DDS_DX10_TOTAL_HEADER_SIZE);

        // Override the DXGI format with the test case
        unsafe {
            (data.as_mut_ptr().add(DX10_FORMAT_OFFSET) as *mut u32).write_unaligned(dxgi_format);
        }

        let info = parse_dds(&data).unwrap();
        assert_eq!(info.format, expected_format);
        assert_eq!(info.data_offset, DDS_DX10_TOTAL_HEADER_SIZE as u8);
    }

    #[test]
    fn parse_dds_detects_unknown_legacy_format() {
        // Test invalid legacy format
        let unknown_legacy = create_unknown_format_dds(DDS_DX10_TOTAL_HEADER_SIZE);
        let info = parse_dds(&unknown_legacy).unwrap();
        assert_eq!(info.format, DdsFormat::Unknown);
    }

    #[test]
    fn parse_dds_detects_unknown_dx10_format() {
        // Test invalid DX10 format
        let mut unknown_dx10 = create_valid_bc7_dds(DDS_DX10_TOTAL_HEADER_SIZE);
        unsafe {
            (unknown_dx10.as_mut_ptr().add(DX10_FORMAT_OFFSET) as *mut u32)
                .write_unaligned(0x12345678);
        }

        let info = parse_dds(&unknown_dx10).unwrap();
        assert_eq!(info.format, DdsFormat::Unknown);
    }

    #[test]
    fn parse_dds_handles_data_too_short_for_legacy_header() {
        // Too short for legacy header
        let data = [0u8; DDS_HEADER_SIZE - 1];
        assert!(parse_dds(&data).is_none());
    }

    #[test]
    fn parse_dds_handles_data_too_short_for_dx10_header() {
        // Too short for DX10 header
        let mut data = [0u8; DDS_DX10_TOTAL_HEADER_SIZE - 1];
        unsafe {
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(FOURCC_DX10);
        }
        assert!(parse_dds(&data).is_none());
    }

    #[test]
    fn parse_dds_handles_unaligned_reads() {
        // Test with data at an unaligned address to ensure read_unaligned works correctly
        let mut buffer = [0u8; 0x81]; // DDS_HEADER_SIZE + 1 = 0x80 + 1 = 0x81
        let data = &mut buffer[1..]; // Start at offset 1 to create misalignment

        unsafe {
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(FOURCC_DXT1);
        }

        let info = parse_dds(data).unwrap();
        assert_eq!(info.format, DdsFormat::BC1);
        assert_eq!(info.data_offset, DDS_HEADER_SIZE as u8);
    }
}
