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

// -------------------------------------------------------------------------------------------------
// Shared test helpers lifted from individual transform tests
// -------------------------------------------------------------------------------------------------

// Re-export the reference untransform function so tests can access it through the prelude
pub use crate::transforms::standard;

/// Common type alias for transform/permute functions used across BC1 tests.
#[allow(clippy::type_complexity)]
pub type TransformFn = unsafe fn(*const u8, *mut u8, usize);

/// Executes a transform → untransform round-trip on 1‥=max_blocks BC1 blocks and
/// asserts that the final data matches the original input.
#[inline]
pub fn run_standard_transform_roundtrip_test(
    transform_fn: TransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let input = generate_bc1_test_data(num_blocks);
        let len = input.len();
        let mut transformed = vec![0u8; len];
        let mut reconstructed = vec![0u8; len];

        unsafe {
            transform_fn(input.as_ptr(), transformed.as_mut_ptr(), len);
            standard::untransform(transformed.as_ptr(), reconstructed.as_mut_ptr(), len);
        }

        assert_eq!(
            reconstructed.as_slice(),
            input.as_slice(),
            "Mismatch {impl_name} roundtrip for {num_blocks} blocks",
        );
    }
}

// --------------------------------------
// Helper functions for untransform tests
// --------------------------------------

/// Executes a standard reference transform followed by the given untransform implementation
/// on 1‥=max_blocks BC1 blocks, using aligned input/output buffers.
#[inline]
pub fn run_standard_untransform_aligned_test(
    detransform_fn: TransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc1_test_data(num_blocks);
        let mut transformed = allocate_align_64(original.len()).unwrap();
        let mut reconstructed = allocate_align_64(original.len()).unwrap();

        unsafe {
            // Transform with the reference path
            standard::transform(original.as_ptr(), transformed.as_mut_ptr(), original.len());

            // Reconstruct with the implementation under test
            reconstructed.as_mut_slice().fill(0);
            detransform_fn(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                transformed.len(),
            );
        }

        assert_implementation_matches_reference(
            original.as_slice(),
            reconstructed.as_slice(),
            &format!("{impl_name} (aligned)"),
            num_blocks,
        );
    }
}

/// Same as [`run_standard_untransform_aligned_test`] but also validates the implementation
/// with deliberately mis-aligned (offset by 1 byte) input and output pointers.
#[inline]
pub fn run_standard_untransform_unaligned_test(
    detransform_fn: TransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc1_test_data(num_blocks);

        // Transform using the reference path
        let mut transformed = vec![0u8; original.len()];
        unsafe {
            standard::transform(original.as_ptr(), transformed.as_mut_ptr(), original.len());

            // Shift by one byte to mis-align the buffers
            let mut transformed_unaligned = vec![0u8; transformed.len() + 1];
            transformed_unaligned[1..].copy_from_slice(&transformed);

            let mut reconstructed = vec![0u8; original.len() + 1];

            reconstructed.as_mut_slice().fill(0);
            detransform_fn(
                transformed_unaligned.as_ptr().add(1),
                reconstructed.as_mut_ptr().add(1),
                transformed.len(),
            );

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}
