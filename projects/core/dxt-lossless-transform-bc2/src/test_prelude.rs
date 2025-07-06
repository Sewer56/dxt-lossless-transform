//! Common test imports and utilities for BC2 tests
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
pub use crate::transform::*;

// Common types from dxt_lossless_transform_api_common
pub use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

// Test utilities from transforms module are used internally
// but not re-exported due to visibility constraints

// Common types from dxt_lossless_transform_common
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

// -------------------------------------------------------------------------------------------------
// Shared test helpers for BC2 with_recorrelate tests
// -------------------------------------------------------------------------------------------------

/// Common type alias for BC2 transform functions with decorrelation used across tests.
pub(crate) type WithDecorrelateTransformFn =
    unsafe fn(*const u8, *mut u64, *mut u32, *mut u32, usize);

/// Common type alias for BC2 untransform functions with recorrelation used across tests.
pub(crate) type WithRecorrelateUntransformFn =
    unsafe fn(*const u64, *const u32, *const u32, *mut u8, usize);

/// Executes a decorrelate transform → untransform round-trip on 1‥=max_blocks BC2 blocks
/// using the specified transform function and YCoCg variant, asserting that the final data
/// matches the original input.
#[inline]
pub(crate) fn run_with_decorrelate_transform_roundtrip_test(
    transform_fn: WithDecorrelateTransformFn,
    variant: YCoCgVariant,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_recorrelate::untransform::untransform_with_recorrelate;

    for num_blocks in 1..=max_blocks {
        let original = generate_bc2_test_data(num_blocks);
        let len = original.len();

        // Allocate combined buffer for separated data (like BC1 does)
        let mut transformed = allocate_align_64(len);
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            // Transform with decorrelation using combined buffer layout
            transform_fn(
                original.as_ptr(),
                transformed.as_mut_ptr() as *mut u64, // alphas at start
                transformed.as_mut_ptr().add(len / 2) as *mut u32, // colors at len/2
                transformed.as_mut_ptr().add(len / 2 + len / 4) as *mut u32, // indices at len/2 + len/4
                num_blocks,
            );

            // Untransform with recorrelation using public dispatcher
            untransform_with_recorrelate(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                len,
                variant,
            );
        }

        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "Mismatch in {impl_name} roundtrip for {num_blocks} blocks with {variant:?}",
        );
    }
}

/// Executes a recorrelate untransform round-trip test by first applying the matching
/// transform with decorrelation, then the specified untransform function, asserting
/// that the final data matches the original input.
#[inline]
pub(crate) fn run_with_recorrelate_untransform_roundtrip_test(
    untransform_fn: WithRecorrelateUntransformFn,
    variant: YCoCgVariant,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_recorrelate::transform::transform_with_decorrelate;

    for num_blocks in 1..=max_blocks {
        let original = generate_bc2_test_data(num_blocks);
        let len = original.len();

        // Allocate buffers
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            // First transform with decorrelation using public dispatcher
            let mut transformed = allocate_align_64(len);
            transform_with_decorrelate(original.as_ptr(), transformed.as_mut_ptr(), len, variant);

            // Extract separated pointers from the combined buffer
            let alphas_ptr = transformed.as_ptr() as *const u64;
            let colors_ptr = transformed.as_ptr().add(len / 2) as *const u32;
            let indices_ptr = transformed.as_ptr().add(len / 2 + len / 4) as *const u32;

            // Then untransform with recorrelation using individual function
            untransform_fn(
                alphas_ptr,
                colors_ptr,
                indices_ptr,
                reconstructed.as_mut_ptr(),
                num_blocks,
            );
        }

        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "Mismatch in {impl_name} roundtrip for {num_blocks} blocks with {variant:?}",
        );
    }
}

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

/// A simple dummy estimator for testing purposes.
///
/// This estimator doesn't perform actual compression estimation but provides
/// a predictable implementation for testing API behavior.
pub struct DummyEstimator;

impl SizeEstimationOperations for DummyEstimator {
    type Error = &'static str;

    fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
        Ok(0) // No buffer needed for dummy estimator
    }

    unsafe fn estimate_compressed_size(
        &self,
        _input_ptr: *const u8,
        len_bytes: usize,
        _output_ptr: *mut u8,
        _output_len: usize,
    ) -> Result<usize, Self::Error> {
        Ok(len_bytes) // Just return the input length
    }
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
            crate::transform::standard::untransform(
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

// -----------------------------------------
// Shared test helpers for split colour tests
// -----------------------------------------

/// Common type alias for BC2 split colour transform functions used across tests.
pub(crate) type SplitColourTransformFn =
    unsafe fn(*const u8, *mut u64, *mut u16, *mut u16, *mut u32, usize);

/// Common type alias for BC2 split colour untransform functions used across tests.
pub(crate) type SplitColourUntransformFn =
    unsafe fn(*const u64, *const u16, *const u16, *const u32, *mut u8, usize);

/// Executes a split colour transform → untransform round-trip test
/// using the specified transform function, asserting that the final data
/// matches the original input.
#[inline]
pub(crate) fn run_split_colour_transform_roundtrip_test(
    transform_fn: SplitColourTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour::untransform_with_split_colour;

    for num_blocks in 1..=max_blocks {
        let original = generate_bc2_test_data(num_blocks);

        // Allocate separate arrays for split colour data
        let mut alpha_data = allocate_align_64(num_blocks * 8);
        let mut color0_data = allocate_align_64(num_blocks * 2);
        let mut color1_data = allocate_align_64(num_blocks * 2);
        let mut indices_data = allocate_align_64(num_blocks * 4);
        let mut reconstructed = allocate_align_64(original.len());

        unsafe {
            // Transform using the function being tested
            transform_fn(
                original.as_ptr(),
                alpha_data.as_mut_ptr() as *mut u64,
                color0_data.as_mut_ptr() as *mut u16,
                color1_data.as_mut_ptr() as *mut u16,
                indices_data.as_mut_ptr() as *mut u32,
                num_blocks,
            );

            // Untransform using the generic dispatcher
            untransform_with_split_colour(
                alpha_data.as_ptr() as *const u64,
                color0_data.as_ptr() as *const u16,
                color1_data.as_ptr() as *const u16,
                indices_data.as_ptr() as *const u32,
                reconstructed.as_mut_ptr(),
                num_blocks,
            );
        }

        assert_eq!(
            original.as_slice(),
            reconstructed.as_slice(),
            "Mismatch in {impl_name} roundtrip for {num_blocks} blocks",
        );
    }
}

/// Executes an unaligned untransform test for split colour operations.
/// Tests a transform→untransform roundtrip with deliberately misaligned buffers.
#[inline]
pub(crate) fn run_with_split_colour_untransform_unaligned_test(
    untransform_fn: SplitColourUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour::transform_with_split_colour;

    for block_count in 1..=max_blocks {
        // Generate test data
        let original = generate_bc2_test_data(block_count);

        // Create separate arrays for split colour data
        let mut alpha_data = allocate_align_64(block_count * 8);
        let mut color0_data = allocate_align_64(block_count * 2);
        let mut color1_data = allocate_align_64(block_count * 2);
        let mut indices_data = allocate_align_64(block_count * 4);

        // Create unaligned reconstruction buffer
        let mut unaligned_reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // First, transform using split colour transform
            transform_with_split_colour(
                original.as_ptr(),
                alpha_data.as_mut_ptr() as *mut u64,
                color0_data.as_mut_ptr() as *mut u16,
                color1_data.as_mut_ptr() as *mut u16,
                indices_data.as_mut_ptr() as *mut u32,
                block_count,
            );

            // Then, untransform using the function being tested with unaligned output
            untransform_fn(
                alpha_data.as_ptr() as *const u64,
                color0_data.as_ptr() as *const u16,
                color1_data.as_ptr() as *const u16,
                indices_data.as_ptr() as *const u32,
                unaligned_reconstructed.as_mut_ptr().add(1),
                block_count,
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
        let original = generate_bc2_test_data(block_count);

        // Create unaligned buffers (allocate an extra byte and offset by 1)
        let mut unaligned_transformed = allocate_align_64(original.len() + 1);
        let mut unaligned_reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // First, transform using standard split_blocks
            crate::transform::standard::transform(
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
