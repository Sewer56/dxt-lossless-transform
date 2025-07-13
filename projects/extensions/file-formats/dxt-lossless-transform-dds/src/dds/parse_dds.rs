use super::{constants::*, likely_dds};
use core::hint::unreachable_unchecked;
use endian_writer::{EndianReader, LittleEndianReader};

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
    BC6H = 5,
    BC7 = 6,
    /// RGBA8888 format (32-bit with alpha)
    RGBA8888 = 7,
    /// BGRA8888 format (32-bit with alpha, different byte order)
    BGRA8888 = 8,
    /// BGR888 format (24-bit RGB)
    BGR888 = 9,
}

/// The information of the DDS file supplied to the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct DdsInfo {
    pub format: DdsFormat,
    pub data_offset: u8,
    pub data_length: u32,
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
/// `None` if the length is insufficient to read the headers, format is unknown or
/// the header is invalid.
///
/// Otherwise, a [`DdsInfo`] with the format and data offset.
#[inline]
pub fn parse_dds_ignore_magic(data: &[u8]) -> Option<DdsInfo> {
    // Check minimum length for DDS header
    if data.len() < DDS_HEADER_SIZE {
        return None;
    }

    // SAFETY: We checked data.len() >= DDS_HEADER_SIZE (128), so FOURCC_OFFSET (0x54) + 4 is safe
    let mut reader = unsafe { LittleEndianReader::new(data.as_ptr()) };
    let fourcc = unsafe { reader.read_u32_at(FOURCC_OFFSET as isize) };

    let (format, data_offset) = if fourcc == FOURCC_DX10 {
        // DX10 header present, ensure the data is long enough.
        if data.len() < DDS_HEADER_SIZE + DX10_HEADER_SIZE {
            return None;
        }

        // SAFETY: We checked data.len() >= DDS_HEADER_SIZE + DX10_HEADER_SIZE (148),
        // so DX10_FORMAT_OFFSET (0x80) + 4 is safe
        let dxgi_format = unsafe { reader.read_u32_at(DX10_FORMAT_OFFSET as isize) };
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
            DXGI_FORMAT_BC6H_TYPELESS | DXGI_FORMAT_BC6H_UF16 | DXGI_FORMAT_BC6H_SF16 => {
                DdsFormat::BC6H
            }
            DXGI_FORMAT_BC7_TYPELESS | DXGI_FORMAT_BC7_UNORM | DXGI_FORMAT_BC7_UNORM_SRGB => {
                DdsFormat::BC7
            }
            DXGI_FORMAT_R8G8B8A8_TYPELESS
            | DXGI_FORMAT_R8G8B8A8_UNORM
            | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB
            | DXGI_FORMAT_R8G8B8A8_UINT
            | DXGI_FORMAT_R8G8B8A8_SNORM
            | DXGI_FORMAT_R8G8B8A8_SINT => DdsFormat::RGBA8888,
            DXGI_FORMAT_B8G8R8A8_UNORM
            | DXGI_FORMAT_B8G8R8A8_TYPELESS
            | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB => DdsFormat::BGRA8888,
            _ => DdsFormat::Unknown,
        };

        // 148 bytes: 128 byte header + 20 byte DX10 header
        (format, DDS_HEADER_SIZE + DX10_HEADER_SIZE)
    } else {
        // Legacy header - check pixel format flags to determine format type
        let pixel_flags = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_FLAGS_OFFSET as isize) };

        let format = if (pixel_flags & DDPF_FOURCC) != 0 {
            // Block-compressed format with FOURCC
            match fourcc {
                FOURCC_DXT1 => DdsFormat::BC1,
                FOURCC_DXT2 | FOURCC_DXT3 => DdsFormat::BC2,
                FOURCC_DXT4 | FOURCC_DXT5 => DdsFormat::BC3,
                _ => DdsFormat::Unknown,
            }
        } else if (pixel_flags & DDPF_RGB) != 0 {
            // Uncompressed RGB format
            detect_uncompressed_format(data)
        } else {
            // Other formats (YUV, Luminance, Alpha-only, etc.) are not supported
            DdsFormat::Unknown
        };

        // 128 bytes: standard header size
        (format, DDS_HEADER_SIZE)
    };

    // Calculate texture data length based on format and header fields
    let data_length = calculate_data_length(format, data).unwrap_or(0);

    Some(DdsInfo {
        format,
        data_offset: data_offset as u8,
        data_length,
    })
}

/// Detects uncompressed DDS format by examining bit masks and bit count
///
/// # Preconditions
///
/// This function assumes that the DDPF_RGB flag has already been checked by the caller.
fn detect_uncompressed_format(data: &[u8]) -> DdsFormat {
    let mut reader = unsafe { LittleEndianReader::new(data.as_ptr()) };

    // Read pixel format information
    let pixel_flags = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_FLAGS_OFFSET as isize) };
    let rgb_bit_count = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_RGBBITCOUNT_OFFSET as isize) };

    // Read the bit masks to determine the format
    let r_mask = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_RBITMASK_OFFSET as isize) };
    let g_mask = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_GBITMASK_OFFSET as isize) };
    let b_mask = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_BBITMASK_OFFSET as isize) };
    let a_mask = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_ABITMASK_OFFSET as isize) };

    match rgb_bit_count {
        24 => {
            // BGR888: 24-bit RGB format
            if r_mask == BGR888_RED_MASK
                && g_mask == BGR888_GREEN_MASK
                && b_mask == BGR888_BLUE_MASK
                && a_mask == 0x00000000
            {
                DdsFormat::BGR888
            } else {
                DdsFormat::Unknown
            }
        }
        32 => {
            // For 32-bit formats, check if alpha channel is present
            if (pixel_flags & DDPF_ALPHAPIXELS) != 0 {
                // Check for RGBA8888 (R8G8B8A8_UNORM)
                if r_mask == RGBA8888_RED_MASK
                    && g_mask == RGBA8888_GREEN_MASK
                    && b_mask == RGBA8888_BLUE_MASK
                    && a_mask == RGBA8888_ALPHA_MASK
                {
                    DdsFormat::RGBA8888
                }
                // Check for BGRA8888 (B8G8R8A8_UNORM)
                else if r_mask == BGRA8888_RED_MASK
                    && g_mask == BGRA8888_GREEN_MASK
                    && b_mask == BGRA8888_BLUE_MASK
                    && a_mask == BGRA8888_ALPHA_MASK
                {
                    DdsFormat::BGRA8888
                } else {
                    DdsFormat::Unknown
                }
            } else {
                // 32-bit RGB format without alpha (not currently supported)
                DdsFormat::Unknown
            }
        }
        _ => {
            // Other bit depths are not currently supported
            DdsFormat::Unknown
        }
    }
}

/// Calculate texture data length for a DDS format
#[inline(always)]
fn calculate_data_length(format: DdsFormat, data: &[u8]) -> Option<u32> {
    // SAFETY: We checked data.len() >= DDS_HEADER_SIZE (128) in caller, so DDS_FLAGS_OFFSET (0x10) + 4 is safe

    // Read header fields using little-endian byte order
    let mut reader = unsafe { LittleEndianReader::new(data.as_ptr()) };
    let flags = unsafe { reader.read_u32_at(DDS_FLAGS_OFFSET as isize) };
    let height = unsafe { reader.read_u32_at(DDS_HEIGHT_OFFSET as isize) };
    let width = unsafe { reader.read_u32_at(DDS_WIDTH_OFFSET as isize) };
    let raw_mipmap_count = unsafe { reader.read_u32_at(DDS_MIPMAP_COUNT_OFFSET as isize) };

    // Determine mipmap count
    let mipmap_count = if (flags & DDSD_MIPMAPCOUNT) != 0 {
        raw_mipmap_count.max(1)
    } else {
        1
    };

    // For block-compressed formats, use the dimension-based calculation
    match format {
        DdsFormat::BC1 | DdsFormat::BC2 | DdsFormat::BC3 | DdsFormat::BC6H | DdsFormat::BC7 => {
            calculate_data_length_for_block_compression(format, width, height, mipmap_count)
        }
        DdsFormat::RGBA8888 | DdsFormat::BGRA8888 => {
            // 32-bit formats (4 bytes per pixel)
            calculate_data_length_for_pixel_formats(width, height, mipmap_count, 4)
        }
        DdsFormat::BGR888 => {
            // 24-bit format (3 bytes per pixel)
            calculate_data_length_for_pixel_formats(width, height, mipmap_count, 3)
        }
        DdsFormat::Unknown => {
            // Try to determine from pixel format for uncompressed formats
            calculate_uncompressed_data_length(data, width, height, mipmap_count)
        }
        DdsFormat::NotADds => None,
    }
}

/// Calculate texture data length for given format and dimensions
///
/// This is a utility function for tests and other scenarios where you want to calculate
/// the expected data size without parsing an actual DDS buffer.
#[inline(always)] // Avoid double matching in release builds
pub(crate) fn calculate_data_length_for_block_compression(
    format: DdsFormat,
    width: u32,
    height: u32,
    mipmap_count: u32,
) -> Option<u32> {
    // Calculate data size based on format type
    match format {
        DdsFormat::BC1 | DdsFormat::BC2 | DdsFormat::BC3 | DdsFormat::BC6H | DdsFormat::BC7 => {
            // Block-compressed formats
            let block_size = match format {
                DdsFormat::BC1 => 8,   // DXT1: 8 bytes per 4x4 block
                DdsFormat::BC2 => 16,  // DXT2/3: 16 bytes per 4x4 block
                DdsFormat::BC3 => 16,  // DXT4/5: 16 bytes per 4x4 block
                DdsFormat::BC6H => 16, // BC6H: 16 bytes per 4x4 block
                DdsFormat::BC7 => 16,  // BC7: 16 bytes per 4x4 block
                _ => unsafe { unreachable_unchecked() },
            };

            let mut total_size = 0u32;
            let mut w = width;
            let mut h = height;

            for _ in 0..mipmap_count {
                // Round up dimensions to next multiple of 4 for block compression
                let blocks_wide = w.div_ceil(4);
                let blocks_high = h.div_ceil(4);

                // Calculate size for this mipmap level
                let level_size = blocks_wide * blocks_high * block_size;
                total_size = total_size.saturating_add(level_size);

                // Calculate next mipmap level dimensions (minimum 1x1)
                w = (w / 2).max(1);
                h = (h / 2).max(1);
            }

            Some(total_size)
        }
        DdsFormat::RGBA8888 | DdsFormat::BGRA8888 => {
            // 32-bit uncompressed formats
            calculate_data_length_for_pixel_formats(width, height, mipmap_count, 4)
        }
        DdsFormat::BGR888 => {
            // 24-bit uncompressed format
            calculate_data_length_for_pixel_formats(width, height, mipmap_count, 3)
        }
        DdsFormat::Unknown => {
            // Don't make assumptions about unknown formats - return 0
            Some(0)
        }
        DdsFormat::NotADds => None,
    }
}

/// Calculate data length for uncompressed pixel formats
fn calculate_uncompressed_data_length(
    data: &[u8],
    width: u32,
    height: u32,
    mipmap_count: u32,
) -> Option<u32> {
    // Read pixel format information using little-endian byte order
    let mut reader = unsafe { LittleEndianReader::new(data.as_ptr()) };
    let pixel_flags = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_FLAGS_OFFSET as isize) };
    let rgb_bit_count = unsafe { reader.read_u32_at(DDS_PIXELFORMAT_RGBBITCOUNT_OFFSET as isize) };

    // Check if this is a recognized uncompressed format
    if (pixel_flags & (DDPF_RGB | DDPF_LUMINANCE | DDPF_YUV | DDPF_ALPHA)) == 0 {
        return None; // Not a recognized uncompressed format
    }

    // Calculate bytes per pixel from bit count
    if !rgb_bit_count.is_multiple_of(8) {
        return None; // Invalid bit count
    }
    let bytes_per_pixel = rgb_bit_count / 8;

    // Use the shared function to calculate total size for all mipmap levels
    calculate_data_length_for_pixel_formats(width, height, mipmap_count, bytes_per_pixel)
}

/// Calculate data length for uncompressed formats with given dimensions and bytes per pixel
pub(crate) fn calculate_data_length_for_pixel_formats(
    width: u32,
    height: u32,
    mipmap_count: u32,
    bytes_per_pixel: u32,
) -> Option<u32> {
    if bytes_per_pixel == 0 {
        return None;
    }

    let mut total_size = 0u32;
    let mut w = width;
    let mut h = height;

    for _ in 0..mipmap_count {
        let level_size = w * h * bytes_per_pixel;
        total_size = total_size.saturating_add(level_size);

        // Calculate next mipmap level dimensions (minimum 1x1)
        w = (w / 2).max(1);
        h = (h / 2).max(1);
    }

    Some(total_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::constants::DDS_HEADER_SIZE;
    use crate::test_prelude::DDS_DX10_TOTAL_HEADER_SIZE;
    use crate::test_prelude::*;
    use endian_writer::{EndianWriter, LittleEndianWriter};

    #[rstest]
    #[case(FOURCC_DXT1, DdsFormat::BC1)]
    #[case(FOURCC_DXT2, DdsFormat::BC2)]
    #[case(FOURCC_DXT3, DdsFormat::BC2)]
    #[case(FOURCC_DXT4, DdsFormat::BC3)]
    #[case(FOURCC_DXT5, DdsFormat::BC3)]
    fn parse_dds_handles_legacy_formats(#[case] fourcc: u32, #[case] expected_format: DdsFormat) {
        // Create a DDS with actual texture data for proper validation
        let mut data = create_valid_bc1_dds_with_dimensions(4, 4, 1);

        // Override the FOURCC with the test case
        let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
        unsafe { writer.write_u32_at(fourcc, FOURCC_OFFSET as isize) };

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
        // Create a DDS with actual texture data for proper validation
        let mut data = create_valid_bc1_dds_with_dimensions(4, 4, 1);

        // Override the magic and FOURCC for the test case
        let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
        unsafe {
            // Set invalid magic header (simulating transform header)
            writer.write_u32_at(0xDEADBEEF, 0);
            writer.write_u32_at(fourcc, FOURCC_OFFSET as isize);
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
    #[case(DXGI_FORMAT_BC6H_TYPELESS, DdsFormat::BC6H)]
    #[case(DXGI_FORMAT_BC6H_UF16, DdsFormat::BC6H)]
    #[case(DXGI_FORMAT_BC6H_SF16, DdsFormat::BC6H)]
    #[case(DXGI_FORMAT_BC7_TYPELESS, DdsFormat::BC7)]
    #[case(DXGI_FORMAT_BC7_UNORM, DdsFormat::BC7)]
    #[case(DXGI_FORMAT_BC7_UNORM_SRGB, DdsFormat::BC7)]
    fn parse_dds_handles_dx10_formats(
        #[case] dxgi_format: u32,
        #[case] expected_format: DdsFormat,
    ) {
        let mut data = create_incomplete_bc7_dds(); // Header-only BC7 for testing unknown format detection

        // Override the DXGI format with the test case
        let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
        unsafe { writer.write_u32_at(dxgi_format, DX10_FORMAT_OFFSET as isize) };

        let info = parse_dds(&data).unwrap();
        assert_eq!(info.format, expected_format);
        assert_eq!(info.data_offset, DDS_DX10_TOTAL_HEADER_SIZE as u8);
    }

    #[test]
    fn parse_dds_detects_unknown_legacy_format() {
        // Test invalid legacy format
        let unknown_legacy = create_valid_unknown_format_dds(); // Valid unknown format DDS for testing
        let info = parse_dds(&unknown_legacy).unwrap();
        assert_eq!(info.format, DdsFormat::Unknown);
    }

    #[test]
    fn parse_dds_detects_unknown_dx10_format() {
        // Test invalid DX10 format
        let mut unknown_dx10 = create_valid_bc7_dds(); // Valid BC7 DDS for modification
        let mut writer = unsafe { LittleEndianWriter::new(unknown_dx10.as_mut_ptr()) };
        unsafe {
            // Change DXGI format to unknown
            writer.write_u32_at(0x12345678, DX10_FORMAT_OFFSET as isize);
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
        let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
        unsafe {
            writer.write_u32_at(DDS_MAGIC, 0);
            writer.write_u32_at(FOURCC_DX10, FOURCC_OFFSET as isize);
        }
        assert!(parse_dds(&data).is_none());
    }

    #[test]
    fn parse_dds_handles_unaligned_reads() {
        // Create a valid BC1 DDS file using helper functions
        let valid_dds = create_valid_bc1_dds_with_dimensions(64, 64, 1);

        // Create a buffer with an extra byte at the start to make the DDS data misaligned
        let mut misaligned_buffer = vec![0u8; valid_dds.len() + 1];

        // Copy the valid DDS data starting at offset 1 to create misalignment
        misaligned_buffer[1..].copy_from_slice(&valid_dds);

        // Test parsing the misaligned data (skip the first byte)
        let misaligned_data = &misaligned_buffer[1..];

        let info = parse_dds(misaligned_data).unwrap();
        assert_eq!(info.format, DdsFormat::BC1);
        assert_eq!(info.data_offset, DDS_HEADER_SIZE as u8);
    }

    // Data length calculation tests

    #[test]
    fn data_length_bc1_single_level_256x256() {
        let input = create_valid_bc1_dds_with_dimensions(256, 256, 1);
        let info = parse_dds(&input).unwrap();
        // 256x256 = 64x64 blocks * 8 bytes = 32768 bytes
        assert_eq!(info.data_length, 32768);
    }

    #[test]
    fn data_length_bc1_with_mipmaps_256x256() {
        let input = create_valid_bc1_dds_with_dimensions(256, 256, 9);
        let info = parse_dds(&input).unwrap();
        // 256x256 (32768) + 128x128 (8192) + 64x64 (2048) + 32x32 (512) + 16x16 (128) + 8x8 (32) + 4x4 (8) + 2x2 (8) + 1x1 (8) = 43704
        assert_eq!(info.data_length, 43704);
    }

    #[test]
    fn data_length_bc1_with_non_multiple_of_4_dimensions() {
        let input = create_valid_bc1_dds_with_dimensions(17, 13, 1);
        let info = parse_dds(&input).unwrap();
        // (17+3)/4 = 5 blocks wide, (13+3)/4 = 4 blocks high
        // 5 * 4 * 8 = 160 bytes
        assert_eq!(info.data_length, 160);
    }

    #[test]
    fn data_length_zero_dimensions_returns_zero() {
        let mut input = create_valid_bc1_dds_with_dimensions(256, 256, 1);
        // Set width to 0
        let mut writer = unsafe { LittleEndianWriter::new(input.as_mut_ptr()) };
        unsafe { writer.write_u32_at(0, DDS_WIDTH_OFFSET as isize) };
        let info = parse_dds(&input).unwrap();
        // Should return 0 for zero dimensions (0 * height * block_size = 0)
        assert_eq!(info.data_length, 0);
    }

    #[test]
    fn data_length_uncompressed_rgba32() {
        let input = create_valid_rgba8888_dds_with_dimensions(16, 16, 1);
        let info = parse_dds(&input).unwrap();
        // 16x16 * 4 bytes per pixel = 1024 bytes
        assert_eq!(info.data_length, 1024);
    }

    #[test]
    fn data_length_uncompressed_with_mipmaps() {
        let input = create_valid_rgba8888_dds_with_dimensions(4, 4, 3);
        let info = parse_dds(&input).unwrap();
        // 4x4 (64) + 2x2 (16) + 1x1 (4) = 84 bytes
        assert_eq!(info.data_length, 84);
    }
}
