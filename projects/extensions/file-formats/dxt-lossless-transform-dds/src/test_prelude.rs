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
pub use std::is_x86_feature_detected;

// External crates commonly used in tests
pub use rstest::rstest;

// Common DDS test data helpers
use crate::dds::constants::{DDS_HEADER_SIZE, DDS_MAGIC, DX10_HEADER_SIZE};

/// Total size of DDS header + DX10 header (used in tests)
pub const DDS_DX10_TOTAL_HEADER_SIZE: usize = DDS_HEADER_SIZE + DX10_HEADER_SIZE;

/// Helper function to create a valid DDS header with BC1 format
pub fn create_valid_bc1_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if size >= DDS_HEADER_SIZE {
        data[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());
        // Set FOURCC to DXT1 (BC1)
        data[0x54..0x58].copy_from_slice(b"DXT1");
    }
    data
}

/// Helper function to create a valid DDS header with BC2 format
pub fn create_valid_bc2_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if size >= DDS_HEADER_SIZE {
        data[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());
        // Set FOURCC to DXT3 (BC2)
        data[0x54..0x58].copy_from_slice(b"DXT3");
    }
    data
}

/// Helper function to create a valid DDS header with BC3 format
pub fn create_valid_bc3_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if size >= DDS_HEADER_SIZE {
        data[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());
        // Set FOURCC to DXT5 (BC3)
        data[0x54..0x58].copy_from_slice(b"DXT5");
    }
    data
}

/// Helper function to create a valid DDS header with BC6H format (DX10 header)
pub fn create_valid_bc6h_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if size >= DDS_DX10_TOTAL_HEADER_SIZE {
        data[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());
        // Set FOURCC to DX10
        data[0x54..0x58].copy_from_slice(b"DX10");
        // Set DXGI format to BC6H
        unsafe {
            (data.as_mut_ptr().add(0x80) as *mut u32)
                .write_unaligned(crate::dds::constants::DXGI_FORMAT_BC6H_UF16);
        }
    }
    data
}

/// Helper function to create a valid DDS header with BC7 format (DX10 header)
pub fn create_valid_bc7_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if size >= DDS_DX10_TOTAL_HEADER_SIZE {
        data[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());
        // Set FOURCC to DX10
        data[0x54..0x58].copy_from_slice(b"DX10");
        // Set DXGI format to BC7
        unsafe {
            (data.as_mut_ptr().add(0x80) as *mut u32)
                .write_unaligned(crate::dds::constants::DXGI_FORMAT_BC7_UNORM);
        }
    }
    data
}

/// Helper function to create a valid DDS header with unknown format
pub fn create_unknown_format_dds(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    if size >= DDS_HEADER_SIZE {
        data[0..4].copy_from_slice(&DDS_MAGIC.to_ne_bytes());
        // Set FOURCC to unknown format
        data[0x54..0x58].copy_from_slice(b"UNKN");
    }
    data
}
