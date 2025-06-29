//! Common test imports and utilities for BC3 tests
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

// Core functionality from this crate
pub use crate::{transform_bc3, untransform_bc3, BC3TransformDetails};

// Test utilities from transforms module are used internally
// but not re-exported due to visibility constraints

// Common types from dxt_lossless_transform_common
pub use dxt_lossless_transform_common::color_8888::Color8888;
#[allow(unused_imports)] // Might be unused in some CPU architectures, and that's ok.
pub use dxt_lossless_transform_common::cpu_detect::*;
pub use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;

use core::alloc::Layout;
// Standard library imports commonly used in tests
pub use core::ptr::{copy_nonoverlapping, write_bytes};
pub use safe_allocator_api::RawAlloc;

// Re-export super for convenience in test modules
pub use super::*;

pub(crate) fn allocate_align_64(num_bytes: usize) -> RawAlloc {
    let layout = Layout::from_size_align(num_bytes, 64).unwrap();
    RawAlloc::new(layout).unwrap()
}

/// Helper to generate test data of specified size (in blocks)
pub(crate) fn generate_bc3_test_data(num_blocks: usize) -> RawAlloc {
    let mut data = allocate_align_64(num_blocks * 16);
    let mut data_ptr = data.as_mut_ptr();

    // Reference byte ranges to make testing easy:
    // alpha: 00 - 31
    // alpha_indices: 32 - 127
    // colors: 128 - 191
    // indices: 192 - 255
    let mut alpha_byte: u8 = 0_u8;
    let mut alpha_index_byte = 32_u8;
    let mut color_byte = 128_u8;
    let mut index_byte = 192_u8;
    unsafe {
        for _ in 0..num_blocks {
            *data_ptr.add(0) = alpha_byte.wrapping_add(0);
            *data_ptr.add(1) = alpha_byte.wrapping_add(1);
            alpha_byte = alpha_byte.wrapping_add(2);
            if alpha_byte >= 32 {
                alpha_byte = alpha_byte.wrapping_sub(32);
            }

            *data_ptr.add(2) = alpha_index_byte.wrapping_add(0);
            *data_ptr.add(3) = alpha_index_byte.wrapping_add(1);
            *data_ptr.add(4) = alpha_index_byte.wrapping_add(2);
            *data_ptr.add(5) = alpha_index_byte.wrapping_add(3);
            *data_ptr.add(6) = alpha_index_byte.wrapping_add(4);
            *data_ptr.add(7) = alpha_index_byte.wrapping_add(5);
            alpha_index_byte = alpha_index_byte.wrapping_add(6);
            if alpha_index_byte >= 128 {
                alpha_index_byte = alpha_index_byte.wrapping_sub(96);
            }

            *data_ptr.add(8) = color_byte.wrapping_add(0);
            *data_ptr.add(9) = color_byte.wrapping_add(1);
            *data_ptr.add(10) = color_byte.wrapping_add(2);
            *data_ptr.add(11) = color_byte.wrapping_add(3);
            color_byte = color_byte.wrapping_add(4);
            if color_byte >= 192 {
                color_byte = color_byte.wrapping_sub(64);
            }

            *data_ptr.add(12) = index_byte.wrapping_add(0);
            *data_ptr.add(13) = index_byte.wrapping_add(1);
            *data_ptr.add(14) = index_byte.wrapping_add(2);
            *data_ptr.add(15) = index_byte.wrapping_add(3);
            index_byte = index_byte.wrapping_add(4);
            if index_byte < 192 {
                index_byte = index_byte.wrapping_sub(64);
            }

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
        Alpha: Sequential 00-31\n\
        Alpha Indices: Sequential 32-127\n\
        Colors: Sequential 128-191\n\
        Indices: Sequential 192-255"
    );
}

// ---------------------------------------
// Shared test helpers for transform tests
// ---------------------------------------

/// Common type alias for transform/split functions used across BC3 tests.
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
        let original = generate_bc3_test_data(num_blocks);

        // Add 1 extra byte at the beginning to create misaligned buffers
        let mut original_unaligned = allocate_align_64(original.len() + 1);
        unsafe {
            copy_nonoverlapping(
                original.as_ptr(),
                original_unaligned.as_mut_ptr().add(1),
                original.len(),
            );
        }

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
            &reconstructed.as_slice()[1..original.len() + 1],
            "Mismatch {impl_name} roundtrip (unaligned) for {num_blocks} blocks",
        );
    }
}

// --------------------------------------
// Helper functions for untransform tests
// --------------------------------------

/// Executes an unaligned untransform test for unsplit operations.
/// Tests a transform→untransform roundtrip with deliberately misaligned buffers.
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
        let original = generate_bc3_test_data(block_count);

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
            &unaligned_reconstructed.as_slice()[1..original.len() + 1],
            impl_name,
            block_count,
        );
    }
}

#[test]
fn validate_bc3_test_data_generator() {
    let expected: Vec<u8> = vec![
        0, 1, // block 1 alpha
        32, 33, 34, 35, 36, 37, // block 1 alpha indices
        128, 129, 130, 131, // block 1 colours
        192, 193, 194, 195, // block 1 indices
        // block 2
        2, 3, // block 2 alpha
        38, 39, 40, 41, 42, 43, // block 2 alpha indices
        132, 133, 134, 135, // block 2 colours
        196, 197, 198, 199, // block 2 indices
        // block 3
        4, 5, // block 3 alpha
        44, 45, 46, 47, 48, 49, // block 3 alpha indices
        136, 137, 138, 139, // block 3 colours
        200, 201, 202, 203, // block 3 indices
    ];
    let output = generate_bc3_test_data(3);
    assert_eq!(output.as_slice(), expected.as_slice());
}
