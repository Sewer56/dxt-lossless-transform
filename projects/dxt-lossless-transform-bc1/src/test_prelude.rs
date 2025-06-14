//! Common test imports and utilities for BC1 tests
//!
//! This module provides a common prelude for test modules to avoid
//! duplicate imports across the codebase.

// External crates commonly used in tests
pub use rstest::rstest;

// Core functionality from this crate
pub use crate::{transform_bc1, Bc1TransformDetails};

// Experimental features commonly tested
pub use crate::experimental::normalize_blocks::*;

// Common types from dxt_lossless_transform_common
pub use dxt_lossless_transform_common::allocate::allocate_align_64;
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;
pub use dxt_lossless_transform_common::color_8888::Color8888;
#[allow(unused_imports)] // Might be unused in some CPU architectures, and that's ok.
pub use dxt_lossless_transform_common::cpu_detect::*;
pub use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;

// Standard library imports commonly used in tests
pub use core::ptr::{copy_nonoverlapping, write_bytes};
pub use safe_allocator_api::RawAlloc;

// Re-export super for convenience in test modules
pub use super::*;

/// Helper to assert implementation results match reference implementation
pub(crate) fn assert_implementation_matches_reference(
    output_expected: &[u8],
    output_test: &[u8],
    impl_name: &str,
    num_blocks: usize,
) {
    assert_eq!(
            output_expected, output_test,
            "{impl_name} implementation produced different results than reference for {num_blocks} blocks.\n\
            First differing block will have predictable values:\n\
            Colors: Sequential 0-3 + (block_num * 4)\n\
            Indices: Sequential 128-131 + (block_num * 4)"
        );
}

// Helper to generate test data of specified size (in blocks)
pub(crate) fn generate_bc1_test_data(num_blocks: usize) -> RawAlloc {
    let mut data = allocate_align_64(num_blocks * 8).unwrap();
    let mut data_ptr = data.as_mut_ptr();

    let mut color_byte = 0_u8;
    let mut index_byte = 128_u8;
    unsafe {
        for _ in 0..num_blocks {
            *data_ptr = color_byte.wrapping_add(0);
            *data_ptr.add(1) = color_byte.wrapping_add(1);
            *data_ptr.add(2) = color_byte.wrapping_add(2);
            *data_ptr.add(3) = color_byte.wrapping_add(3);
            color_byte = color_byte.wrapping_add(4);

            *data_ptr.add(4) = index_byte.wrapping_add(0);
            *data_ptr.add(5) = index_byte.wrapping_add(1);
            *data_ptr.add(6) = index_byte.wrapping_add(2);
            *data_ptr.add(7) = index_byte.wrapping_add(3);
            index_byte = index_byte.wrapping_add(4);
            data_ptr = data_ptr.add(8);
        }
    }

    data
}

#[test]
fn validate_bc1_test_data_generator() {
    let expected: Vec<u8> = vec![
        0x00, 0x01, 0x02, 0x03, // block 1 colours
        0x80, 0x81, 0x82, 0x83, // block 1 indices
        0x04, 0x05, 0x06, 0x07, // block 2 colours
        0x84, 0x85, 0x86, 0x87, // block 2 indices
        0x08, 0x09, 0x0A, 0x0B, // block 3 colours
        0x88, 0x89, 0x8A, 0x8B, // block 3 indices
    ];
    let output = generate_bc1_test_data(3);
    assert_eq!(output.as_slice(), expected.as_slice());
}
