use super::{constants::*, is_dds};

/// Defines a known data format within a DDS file; suitable for lossless transform.
#[derive(Debug, PartialEq)]
pub enum DdsFormat {
    Unknown,
    /// a.k.a. DXT1
    BC1,
    /// a.k.a. DXT2/3
    BC2,
    /// a.k.a. DXT4/5
    BC3,
    BC7,
}

/// The information of the DDS file supplied to the reader.
pub struct DdsInfo {
    pub format: DdsFormat,
    pub data_offset: usize,
}

/// Attempts to parse a the data format of a DDS file from the given pointer and length.
///
/// # Safety
///
/// Any input which passes [`is_dds`] check should be a valid input;
/// but you do not need to explicitly call [`is_dds`], this function will return `None`
/// if the file is not a DDS.
///
/// - `ptr` must be valid for reads of `len` bytes
/// - `len` must accurately represent the length of the file
///
/// # Return
///
/// `None` if the file is not a valid DDS file with a known format, or if
/// the length is insufficient to read the headers.
///
/// Otherwise, a `DdsInfo` with the format and data offset.
///
/// # Notes
///
/// The function will return `None` if the format is not known.
/// For DX10 headers, it will check the `DXGI_FORMAT` field to determine
/// the format.
///
/// [`is_dds`]: crate::dds::is_dds::is_dds
#[inline]
#[no_mangle]
pub unsafe fn parse_dds(ptr: *const u8, len: usize) -> Option<DdsInfo> {
    if !is_dds(ptr, len) {
        return None;
    }

    let fourcc = (ptr.add(FOURCC_OFFSET) as *const u32).read();

    let (format, data_offset) = if fourcc == FOURCC_DX10 {
        // DX10 header present, ensure the data is long enough.
        if len < DDS_HEADER_SIZE + DX10_HEADER_SIZE {
            return None;
        }

        let dxgi_format = (ptr.add(DX10_FORMAT_OFFSET) as *const u32).read();
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
        data_offset,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(FOURCC_DXT1, DdsFormat::BC1)]
    #[case(FOURCC_DXT2, DdsFormat::BC2)]
    #[case(FOURCC_DXT3, DdsFormat::BC2)]
    #[case(FOURCC_DXT4, DdsFormat::BC3)]
    #[case(FOURCC_DXT5, DdsFormat::BC3)]
    fn can_parse_legacy_formats(#[case] fourcc: u32, #[case] expected_format: DdsFormat) {
        let mut data = vec![0u8; 0x80];

        unsafe {
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(fourcc);
        }

        let info = unsafe { parse_dds(data.as_ptr(), data.len()) }.unwrap();
        assert_eq!(info.format, expected_format);
        assert_eq!(info.data_offset, 0x80);
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
    fn can_parse_dx10_formats(#[case] dxgi_format: u32, #[case] expected_format: DdsFormat) {
        let mut data = vec![0u8; 0x94];

        unsafe {
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(FOURCC_DX10);
            (data.as_mut_ptr().add(DX10_FORMAT_OFFSET) as *mut u32).write_unaligned(dxgi_format);
        }

        let info = unsafe { parse_dds(data.as_ptr(), data.len()) }.unwrap();
        assert_eq!(info.format, expected_format);
        assert_eq!(info.data_offset, 0x94);
    }

    #[test]
    fn can_detect_unknown_formats() {
        let mut data = vec![0u8; 0x94];

        // Test invalid legacy format
        unsafe {
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(0x12345678);
        }

        let info = unsafe { parse_dds(data.as_ptr(), data.len()) }.unwrap();
        assert_eq!(info.format, DdsFormat::Unknown);

        // Test invalid DX10 format
        unsafe {
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(FOURCC_DX10);
            (data.as_mut_ptr().add(DX10_FORMAT_OFFSET) as *mut u32).write_unaligned(0x12345678);
        }

        let info = unsafe { parse_dds(data.as_ptr(), data.len()) }.unwrap();
        assert_eq!(info.format, DdsFormat::Unknown);
    }

    #[test]
    fn can_ignore_invalid_dds() {
        // Too short for legacy header
        let data = [0u8; 0x7F];
        assert!(unsafe { parse_dds(data.as_ptr(), data.len()) }.is_none());

        // Too short for DX10 header
        let mut data = vec![0u8; 0x80];
        unsafe {
            (data.as_mut_ptr().add(0) as *mut u32).write_unaligned(DDS_MAGIC);
            (data.as_mut_ptr().add(FOURCC_OFFSET) as *mut u32).write_unaligned(FOURCC_DX10);
        }
        assert!(unsafe { parse_dds(data.as_ptr(), data.len()) }.is_none());
    }
}
