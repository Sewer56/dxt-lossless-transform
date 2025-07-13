//! Common test imports and utilities for DDS extension tests
//!
//! This module provides a common prelude for test modules to avoid
//! duplicate imports across the codebase.
#![allow(unused_imports)]

// External crate declaration for no_std compatibility
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// Re-export commonly used alloc types for tests
pub use alloc::{boxed::Box, format, string::String, vec, vec::Vec};

// Re-export std items for tests that need them
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub use std::is_x86_feature_detected;

// External crates commonly used in tests
pub use rstest::rstest;

// Common DDS test data helpers
use crate::dds::constants::*;
use endian_writer::{EndianWriter, LittleEndianWriter};

/// Total size of DDS header + DX10 header (used in tests)
pub const DDS_DX10_TOTAL_HEADER_SIZE: usize = DDS_HEADER_SIZE + DX10_HEADER_SIZE;

// Import the existing DDS Parser functionality
use crate::dds::parse_dds::*;

/// Helper function to create a basic DDS header with common fields
fn create_dds_header_base(
    data: &mut [u8],
    width: u32,
    height: u32,
    mipmap_count: u32,
    is_dx10: bool,
) {
    let required_size = if is_dx10 {
        DDS_DX10_TOTAL_HEADER_SIZE
    } else {
        DDS_HEADER_SIZE
    };
    if data.len() < required_size {
        return;
    }

    let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };

    unsafe {
        // DDS magic
        writer.write_u32_at(DDS_MAGIC, 0);
        // Set header size (dwSize field at offset 4)
        writer.write_u32_at(124, 4);

        // Set flags to include required fields
        let mut flags = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT | DDSD_LINEARSIZE;
        if mipmap_count > 1 {
            flags |= DDSD_MIPMAPCOUNT;
        }
        writer.write_u32_at(flags, DDS_FLAGS_OFFSET as isize);

        // Set dimensions
        writer.write_u32_at(height, DDS_HEIGHT_OFFSET as isize);
        writer.write_u32_at(width, DDS_WIDTH_OFFSET as isize);

        // Set mipmap count if more than 1
        if mipmap_count > 1 {
            writer.write_u32_at(mipmap_count, DDS_MIPMAP_COUNT_OFFSET as isize);
        }
    }
}

/// Helper function to write pixel format flags for FOURCC-based formats
fn write_fourcc_pixel_format(data: &mut [u8], fourcc: &[u8; 4]) {
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(fourcc);
    unsafe {
        let mut writer = LittleEndianWriter::new(data.as_mut_ptr());
        writer.write_u32_at(DDPF_FOURCC, DDS_PIXELFORMAT_FLAGS_OFFSET as isize);
    }
}

/// Helper function to write DX10 format information
fn write_dx10_format(data: &mut [u8], dxgi_format: u32) {
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(b"DX10");
    unsafe {
        let mut writer = LittleEndianWriter::new(data.as_mut_ptr());
        writer.write_u32_at(DDPF_FOURCC, DDS_PIXELFORMAT_FLAGS_OFFSET as isize);
        writer.write_u32_at(dxgi_format, 0x80);
    }
}

/// Helper function to write uncompressed pixel format information
fn write_uncompressed_pixel_format(
    data: &mut [u8],
    red_mask: u32,
    green_mask: u32,
    blue_mask: u32,
    alpha_mask: u32,
) {
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(b"\0\0\0\0");
    unsafe {
        let mut writer = LittleEndianWriter::new(data.as_mut_ptr());
        writer.write_u32_at(
            DDPF_RGB | DDPF_ALPHAPIXELS,
            DDS_PIXELFORMAT_FLAGS_OFFSET as isize,
        );
        writer.write_u32_at(32, DDS_PIXELFORMAT_RGBBITCOUNT_OFFSET as isize);
        writer.write_u32_at(red_mask, DDS_PIXELFORMAT_RBITMASK_OFFSET as isize);
        writer.write_u32_at(green_mask, DDS_PIXELFORMAT_GBITMASK_OFFSET as isize);
        writer.write_u32_at(blue_mask, DDS_PIXELFORMAT_BBITMASK_OFFSET as isize);
        writer.write_u32_at(alpha_mask, DDS_PIXELFORMAT_ABITMASK_OFFSET as isize);
    }
}

/// Helper function to create a valid DDS with specified format and dimensions
pub fn create_valid_dds_with_dimensions(
    format: DdsFormat,
    width: u32,
    height: u32,
    mipmap_count: u32,
) -> Vec<u8> {
    let (data_size, is_dx10) = match format {
        DdsFormat::BC1 => (
            calculate_data_length_for_block_compression(format, width, height, mipmap_count)
                .unwrap_or(0) as usize,
            false,
        ),
        DdsFormat::BC2 => (
            calculate_data_length_for_block_compression(format, width, height, mipmap_count)
                .unwrap_or(0) as usize,
            false,
        ),
        DdsFormat::BC3 => (
            calculate_data_length_for_block_compression(format, width, height, mipmap_count)
                .unwrap_or(0) as usize,
            false,
        ),
        DdsFormat::BC6H => (
            calculate_data_length_for_block_compression(format, width, height, mipmap_count)
                .unwrap_or(0) as usize,
            true,
        ),
        DdsFormat::BC7 => (
            calculate_data_length_for_block_compression(format, width, height, mipmap_count)
                .unwrap_or(0) as usize,
            true,
        ),
        DdsFormat::RGBA8888 | DdsFormat::BGRA8888 => {
            // 32-bit uncompressed formats
            (
                calculate_data_length_for_pixel_formats(width, height, mipmap_count, 4).unwrap_or(0)
                    as usize,
                false, // Use legacy format for uncompressed formats
            )
        }
        DdsFormat::Unknown => {
            // Unknown formats return 0 data size
            (0, false)
        }
        DdsFormat::NotADds => (0, false), // Should not be used in tests
    };

    let header_size = if is_dx10 {
        DDS_DX10_TOTAL_HEADER_SIZE
    } else {
        DDS_HEADER_SIZE
    };
    let total_size = header_size + data_size;
    let mut data = vec![0u8; total_size];

    create_dds_header_base(&mut data, width, height, mipmap_count, is_dx10);

    // Set format-specific fields
    match format {
        DdsFormat::BC1 => {
            write_fourcc_pixel_format(&mut data, b"DXT1");
        }
        DdsFormat::BC2 => {
            write_fourcc_pixel_format(&mut data, b"DXT3");
        }
        DdsFormat::BC3 => {
            write_fourcc_pixel_format(&mut data, b"DXT5");
        }
        DdsFormat::BC6H => {
            write_dx10_format(&mut data, DXGI_FORMAT_BC6H_UF16);
        }
        DdsFormat::BC7 => {
            write_dx10_format(&mut data, DXGI_FORMAT_BC7_UNORM);
        }
        DdsFormat::RGBA8888 => {
            write_uncompressed_pixel_format(
                &mut data,
                RGBA8888_RED_MASK,
                RGBA8888_GREEN_MASK,
                RGBA8888_BLUE_MASK,
                RGBA8888_ALPHA_MASK,
            );
        }
        DdsFormat::BGRA8888 => {
            write_uncompressed_pixel_format(
                &mut data,
                BGRA8888_RED_MASK,
                BGRA8888_GREEN_MASK,
                BGRA8888_BLUE_MASK,
                BGRA8888_ALPHA_MASK,
            );
        }
        DdsFormat::Unknown => {
            write_fourcc_pixel_format(&mut data, b"UNKN");
        }
        DdsFormat::NotADds => {
            // Should not be used in tests, but handle gracefully
            return data;
        }
    }

    // Fill texture data area with test pattern
    #[allow(clippy::needless_range_loop)]
    for x in header_size..data.len() {
        data[x] = ((x - header_size) % 256) as u8;
    }

    data
}

/// Helper function to create a valid BC1 DDS with proper dimensions and data length
pub fn create_valid_bc1_dds_with_dimensions(width: u32, height: u32, mipmap_count: u32) -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::BC1, width, height, mipmap_count)
}

/// Helper function to create a valid RGBA8888 DDS with proper dimensions and data length
pub fn create_valid_rgba8888_dds_with_dimensions(
    width: u32,
    height: u32,
    mipmap_count: u32,
) -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::RGBA8888, width, height, mipmap_count)
}

/// Helper function to create a valid BGRA8888 DDS with proper dimensions and data length
pub fn create_valid_bgra8888_dds_with_dimensions(
    width: u32,
    height: u32,
    mipmap_count: u32,
) -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::BGRA8888, width, height, mipmap_count)
}

// Semantic helper functions for clearer test intent

/// Creates a minimal valid BC1 DDS file (4x4, single mipmap)
/// Use this when you just need any valid BC1 DDS for testing
pub fn create_valid_bc1_dds() -> Vec<u8> {
    create_valid_bc1_dds_with_dimensions(4, 4, 1)
}

/// Creates a minimal valid BC2 DDS file (4x4, single mipmap)
/// Use this when you just need any valid BC2 DDS for testing
pub fn create_valid_bc2_dds() -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::BC2, 4, 4, 1)
}

/// Creates a minimal valid BC3 DDS file (4x4, single mipmap)
/// Use this when you just need any valid BC3 DDS for testing
pub fn create_valid_bc3_dds() -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::BC3, 4, 4, 1)
}

/// Creates a minimal valid BC6H DDS file (4x4, single mipmap)
/// Use this when you just need any valid BC6H DDS for testing
pub fn create_valid_bc6h_dds() -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::BC6H, 4, 4, 1)
}

/// Creates a minimal valid BC7 DDS file (4x4, single mipmap)
/// Use this when you just need any valid BC7 DDS for testing
pub fn create_valid_bc7_dds() -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::BC7, 4, 4, 1)
}

/// Creates a minimal valid unknown format DDS file (1x1, unknown format)
/// Use this when you just need any valid unknown format DDS for testing
pub fn create_valid_unknown_format_dds() -> Vec<u8> {
    create_valid_dds_with_dimensions(DdsFormat::Unknown, 1, 1, 1)
}

/// Creates an incomplete BC1 DDS file (header-only, no texture data)
/// Use this to test error conditions where there's insufficient data
pub fn create_incomplete_bc1_dds() -> Vec<u8> {
    let mut data = vec![0u8; DDS_HEADER_SIZE];

    // Set up a proper header but no texture data
    create_dds_header_base(&mut data, 4, 4, 1, false);
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(b"DXT1");

    data
}

/// Creates an incomplete BC2 DDS file (header-only, no texture data)
/// Use this to test error conditions where there's insufficient data
pub fn create_incomplete_bc2_dds() -> Vec<u8> {
    let mut data = vec![0u8; DDS_HEADER_SIZE];

    create_dds_header_base(&mut data, 4, 4, 1, false);
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(b"DXT3");

    data
}

/// Creates an incomplete BC3 DDS file (header-only, no texture data)
/// Use this to test error conditions where there's insufficient data
pub fn create_incomplete_bc3_dds() -> Vec<u8> {
    let mut data = vec![0u8; DDS_HEADER_SIZE];

    create_dds_header_base(&mut data, 4, 4, 1, false);
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(b"DXT5");

    data
}

/// Creates an incomplete BC7 DDS file (header-only, no texture data)
/// Use this to test error conditions where there's insufficient data
pub fn create_incomplete_bc7_dds() -> Vec<u8> {
    let mut data = vec![0u8; DDS_DX10_TOTAL_HEADER_SIZE];

    create_dds_header_base(&mut data, 4, 4, 1, true);
    data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(b"DX10");
    let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
    unsafe { writer.write_u32_at(DXGI_FORMAT_BC7_UNORM, 0x80) };

    data
}

/// Creates a DDS file that's too small to contain a complete header
/// Use this to test error conditions for truncated files
pub fn create_truncated_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];

    // Only set magic if there's room
    if size >= 4 {
        let mut writer = unsafe { LittleEndianWriter::new(data.as_mut_ptr()) };
        unsafe { writer.write_u32_at(DDS_MAGIC, 0) };
    }

    data
}

/// Helper function to create a BC1 DDS with leftover data for testing
pub fn create_bc1_dds_with_leftover_data(width: u32, height: u32, leftover_data: &[u8]) -> Vec<u8> {
    let mut base_dds = create_valid_bc1_dds_with_dimensions(width, height, 1);
    base_dds.extend_from_slice(leftover_data);
    base_dds
}
