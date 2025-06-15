//! Common test imports and utilities for BC2 tests
//!
//! This module provides a common prelude for test modules to avoid
//! duplicate imports across the codebase.

// External crates commonly used in tests
pub use rstest::rstest;

// Core functionality from this crate
pub use crate::{transform_bc2, untransform_bc2, BC2TransformDetails};

// Test utilities from transforms module are used internally
// but not re-exported due to visibility constraints

// Common types from dxt_lossless_transform_common
pub use dxt_lossless_transform_common::color_8888::Color8888;
#[allow(unused_imports)] // Might be unused in some CPU architectures, and that's ok.
pub use dxt_lossless_transform_common::cpu_detect::*;
pub use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;

// Standard library imports commonly used in tests
pub use core::ptr::{copy_nonoverlapping, write_bytes};
pub use safe_allocator_api::RawAlloc;

// Re-export super for convenience in test modules
pub use super::*;

/// Helper to generate test data of specified size (in blocks)
pub(crate) fn generate_bc2_test_data(num_blocks: usize) -> RawAlloc {
    let mut data = allocate_align_64(num_blocks * 16);
    let mut data_ptr = data.as_mut_ptr();

    // Reference byte ranges to make testing easy:
    // alpha: 0x00 - 0x80
    // colors: 0x80 - 0xC0
    // indices: 0xC0 - 0xFF
    let mut alpha_byte = 0_u8;
    let mut color_byte = 0x80_u8;
    let mut index_byte = 0xC0_u8;
    unsafe {
        for _ in 0..num_blocks {
            *data_ptr.add(0) = alpha_byte.wrapping_add(0);
            *data_ptr.add(1) = alpha_byte.wrapping_add(1);
            *data_ptr.add(2) = alpha_byte.wrapping_add(2);
            *data_ptr.add(3) = alpha_byte.wrapping_add(3);
            *data_ptr.add(4) = alpha_byte.wrapping_add(4);
            *data_ptr.add(5) = alpha_byte.wrapping_add(5);
            *data_ptr.add(6) = alpha_byte.wrapping_add(6);
            *data_ptr.add(7) = alpha_byte.wrapping_add(7);
            alpha_byte = alpha_byte.wrapping_add(8);

            *data_ptr.add(8) = color_byte.wrapping_add(0);
            *data_ptr.add(9) = color_byte.wrapping_add(1);
            *data_ptr.add(10) = color_byte.wrapping_add(2);
            *data_ptr.add(11) = color_byte.wrapping_add(3);
            color_byte = color_byte.wrapping_add(4);

            *data_ptr.add(12) = index_byte.wrapping_add(0);
            *data_ptr.add(13) = index_byte.wrapping_add(1);
            *data_ptr.add(14) = index_byte.wrapping_add(2);
            *data_ptr.add(15) = index_byte.wrapping_add(3);
            index_byte = index_byte.wrapping_add(4);
            data_ptr = data_ptr.add(16);
        }
    }

    data
}

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
        Alpha: Sequential 0-7 + (block_num * 8)\n\
        Colors: Sequential 0x80-0x83 + (block_num * 4)\n\
        Indices: Sequential 0xC0-0xC3 + (block_num * 4)"
    );
}

/// Allocates data with an alignment of 64 bytes.
///
/// # Parameters
///
/// - `num_bytes`: The number of bytes to allocate
///
/// # Returns
///
/// A [`RawAlloc`] containing the allocated data
fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    dxt_lossless_transform_common::allocate::allocate_align_64(num_bytes).unwrap()
}

// ---------------------------------------
// Shared test helpers for transform tests
// ---------------------------------------

/// Common type alias for transform/split functions used across BC2 tests.
pub(crate) type StandardTransformFn = unsafe fn(*const u8, *mut u8, usize);

/// Executes an unaligned transform test for split operations with input unalignment.
/// Tests transform with deliberately misaligned input and output buffers.
///
/// The `max_blocks` parameter should equal twice the number of bytes processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 16).
#[inline]
pub(crate) fn run_standard_transform_unaligned_test(
    transform_fn: StandardTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc2_test_data(num_blocks);

        // Add 1 extra byte at the beginning to create misaligned buffers
        let mut original_unaligned = allocate_align_64(original.len() + 1);
        original_unaligned.as_mut_slice()[1..].copy_from_slice(original.as_slice());

        let mut transformed = allocate_align_64(original.len() + 1);
        let mut reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // Step 1: Transform using the test function with unaligned pointers
            transform_fn(
                original_unaligned.as_ptr().add(1),
                transformed.as_mut_ptr().add(1),
                original.len(),
            );

            // Step 2: Untransform using standard function with unaligned pointers
            crate::transforms::standard::unsplit_blocks(
                transformed.as_ptr().add(1),
                reconstructed.as_mut_ptr().add(1),
                original.len(),
            );
        }

        // Step 3: Compare reconstructed data against original input
        assert_eq!(
            original.as_slice(),
            &reconstructed.as_slice()[1..],
            "Mismatch {impl_name} roundtrip (unaligned) for {num_blocks} blocks",
        );
    }
}

// --------------------------------------
// Helper functions for untransform tests
// --------------------------------------

/// Executes an unaligned detransform test for unsplit operations.
/// Tests a transform→detransform roundtrip with deliberately misaligned buffers.
///
/// The `max_blocks` parameter should equal twice the number of bytes processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 16).
#[inline]
pub(crate) fn run_standard_untransform_unaligned_test(
    untransform_fn: StandardTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for block_count in 1..=max_blocks {
        // Generate test data
        let original = generate_bc2_test_data(block_count);

        // Create unaligned buffers (allocate an extra byte and offset by 1)
        let mut unaligned_transformed = allocate_align_64(original.len() + 1);
        let mut unaligned_reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // First, transform using standard split_blocks
            crate::transforms::standard::split_blocks(
                original.as_ptr(),
                unaligned_transformed.as_mut_ptr().add(1),
                original.len(),
            );

            // Then, untransform using the function being tested
            untransform_fn(
                unaligned_transformed.as_mut_ptr().add(1),
                unaligned_reconstructed.as_mut_ptr().add(1),
                original.len(),
            );
        }

        // Verify the results match
        assert_implementation_matches_reference(
            original.as_slice(),
            &unaligned_reconstructed.as_slice()[1..],
            impl_name,
            block_count,
        );
    }
}

#[test]
fn validate_bc2_test_data_generator() {
    let expected: Vec<u8> = vec![
        0x00, 0x01, 0x02, 0x03, // block 1 alpha
        0x04, 0x05, 0x06, 0x07, // block 1 alpha
        0x80, 0x81, 0x82, 0x83, // block 1 colours
        0xC0, 0xC1, 0xC2, 0xC3, // block 1 indices
        // block 2
        0x08, 0x09, 0x0A, 0x0B, // block 2 alpha
        0x0C, 0x0D, 0x0E, 0x0F, // block 2 alpha
        0x84, 0x85, 0x86, 0x87, // block 2 colours
        0xC4, 0xC5, 0xC6, 0xC7, // block 2 indices
        // block 3
        0x10, 0x11, 0x12, 0x13, // block 3 alpha
        0x14, 0x15, 0x16, 0x17, // block 3 alpha
        0x88, 0x89, 0x8A, 0x8B, // block 3 colours
        0xC8, 0xC9, 0xCA, 0xCB, // block 3 indices
    ];
    let output = generate_bc2_test_data(3);
    assert_eq!(output.as_slice(), expected.as_slice());
}
