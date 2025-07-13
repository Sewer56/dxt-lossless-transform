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
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;
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

// -----------------------------------------
// Shared test helpers for split alphas tests
// -----------------------------------------
/// Common type alias for BC3 split alphas transform functions used across tests.
pub(crate) type SplitAlphasTransformFn =
    unsafe fn(*const u8, *mut u8, *mut u8, *mut u16, *mut u32, *mut u32, usize);
/// Common type alias for BC3 split alphas untransform functions used across tests.
pub(crate) type SplitAlphasUntransformFn =
    unsafe fn(*const u8, *const u8, *const u16, *const u32, *const u32, *mut u8, usize);

// -----------------------------------------
// Shared test helpers for split colour tests
// -----------------------------------------
/// Common type alias for BC3 split colour transform functions used across tests.
pub(crate) type SplitColourTransformFn =
    unsafe fn(*const u8, *mut u16, *mut u16, *mut u16, *mut u16, *mut u32, usize);
/// Common type alias for BC3 split colour untransform functions used across tests.
pub(crate) type SplitColourUntransformFn =
    unsafe fn(*const u16, *const u16, *const u16, *const u16, *const u32, *mut u8, usize);

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
            crate::transform::standard::unsplit_blocks(
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
            crate::transform::standard::split_blocks(
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

/// Executes a split alphas transform → untransform round-trip test
/// using the specified transform function, asserting that the final data
/// matches the original input.
#[inline]
pub(crate) fn run_split_alphas_transform_roundtrip_test(
    transform_fn: SplitAlphasTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas::untransform_with_split_alphas;
    for num_blocks in 1..=max_blocks {
        let original = generate_bc3_test_data(num_blocks);
        // Allocate separate arrays for split alphas data
        let mut alpha0_data = allocate_align_64(num_blocks * 1);
        let mut alpha1_data = allocate_align_64(num_blocks * 1);
        let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
        let mut colors_data = allocate_align_64(num_blocks * 4);
        let mut color_indices_data = allocate_align_64(num_blocks * 4);
        let mut reconstructed = allocate_align_64(original.len());
        unsafe {
            // Transform using the function being tested
            transform_fn(
                original.as_ptr(),
                alpha0_data.as_mut_ptr(),
                alpha1_data.as_mut_ptr(),
                alpha_indices_data.as_mut_ptr() as *mut u16,
                colors_data.as_mut_ptr() as *mut u32,
                color_indices_data.as_mut_ptr() as *mut u32,
                num_blocks,
            );
            // Untransform using the generic dispatcher
            untransform_with_split_alphas(
                alpha0_data.as_ptr(),
                alpha1_data.as_ptr(),
                alpha_indices_data.as_ptr() as *const u16,
                colors_data.as_ptr() as *const u32,
                color_indices_data.as_ptr() as *const u32,
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

/// Executes an unaligned untransform test for split alphas operations.
/// Tests a transform→untransform roundtrip with deliberately misaligned buffers.
///
/// The `max_blocks` parameter should equal twice the number of bytes processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 16).
#[inline]
pub(crate) fn run_with_split_alphas_untransform_unaligned_test(
    untransform_fn: SplitAlphasUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas::transform_with_split_alphas;
    for block_count in 1..=max_blocks {
        // Generate test data
        let original = generate_bc3_test_data(block_count);
        // Create separate arrays for split alphas data
        let mut alpha0_data = allocate_align_64(block_count * 1);
        let mut alpha1_data = allocate_align_64(block_count * 1);
        let mut alpha_indices_data = allocate_align_64(block_count * 6);
        let mut colors_data = allocate_align_64(block_count * 4);
        let mut color_indices_data = allocate_align_64(block_count * 4);
        // Create unaligned reconstruction buffer
        let mut unaligned_reconstructed = allocate_align_64(original.len() + 1);
        unsafe {
            // First, transform using split alphas transform
            transform_with_split_alphas(
                original.as_ptr(),
                alpha0_data.as_mut_ptr(),
                alpha1_data.as_mut_ptr(),
                alpha_indices_data.as_mut_ptr() as *mut u16,
                colors_data.as_mut_ptr() as *mut u32,
                color_indices_data.as_mut_ptr() as *mut u32,
                block_count,
            );
            // Then, untransform using the function being tested with unaligned output
            untransform_fn(
                alpha0_data.as_ptr(),
                alpha1_data.as_ptr(),
                alpha_indices_data.as_ptr() as *const u16,
                colors_data.as_ptr() as *const u32,
                color_indices_data.as_ptr() as *const u32,
                unaligned_reconstructed.as_mut_ptr().add(1),
                block_count,
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
        let original = generate_bc3_test_data(num_blocks);
        // Allocate separate arrays for split colour data
        let mut alpha_endpoints_data = allocate_align_64(num_blocks * 2);
        let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
        let mut color0_data = allocate_align_64(num_blocks * 2);
        let mut color1_data = allocate_align_64(num_blocks * 2);
        let mut color_indices_data = allocate_align_64(num_blocks * 4);
        let mut reconstructed = allocate_align_64(original.len());
        unsafe {
            // Transform using the function being tested
            transform_fn(
                original.as_ptr(),
                alpha_endpoints_data.as_mut_ptr() as *mut u16,
                alpha_indices_data.as_mut_ptr() as *mut u16,
                color0_data.as_mut_ptr() as *mut u16,
                color1_data.as_mut_ptr() as *mut u16,
                color_indices_data.as_mut_ptr() as *mut u32,
                num_blocks,
            );
            // Untransform using the generic dispatcher
            untransform_with_split_colour(
                alpha_endpoints_data.as_ptr() as *const u16,
                alpha_indices_data.as_ptr() as *const u16,
                color0_data.as_ptr() as *const u16,
                color1_data.as_ptr() as *const u16,
                color_indices_data.as_ptr() as *const u32,
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
///
/// The `max_blocks` parameter should equal twice the number of bytes processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 16).
#[inline]
pub(crate) fn run_with_split_colour_untransform_unaligned_test(
    untransform_fn: SplitColourUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour::transform_with_split_colour;
    for block_count in 1..=max_blocks {
        // Generate test data
        let original = generate_bc3_test_data(block_count);
        // Create separate arrays for split colour data
        let mut alpha_endpoints_data = allocate_align_64(block_count * 2);
        let mut alpha_indices_data = allocate_align_64(block_count * 6);
        let mut color0_data = allocate_align_64(block_count * 2);
        let mut color1_data = allocate_align_64(block_count * 2);
        let mut color_indices_data = allocate_align_64(block_count * 4);
        // Create unaligned reconstruction buffer
        let mut unaligned_reconstructed = allocate_align_64(original.len() + 1);
        unsafe {
            // First, transform using split colour transform
            transform_with_split_colour(
                original.as_ptr(),
                alpha_endpoints_data.as_mut_ptr() as *mut u16,
                alpha_indices_data.as_mut_ptr() as *mut u16,
                color0_data.as_mut_ptr() as *mut u16,
                color1_data.as_mut_ptr() as *mut u16,
                color_indices_data.as_mut_ptr() as *mut u32,
                block_count,
            );
            // Then, untransform using the function being tested with unaligned output
            untransform_fn(
                alpha_endpoints_data.as_ptr() as *const u16,
                alpha_indices_data.as_ptr() as *const u16,
                color0_data.as_ptr() as *const u16,
                color1_data.as_ptr() as *const u16,
                color_indices_data.as_ptr() as *const u32,
                unaligned_reconstructed.as_mut_ptr().add(1),
                block_count,
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

// -------------------------------------------------------------------------------------------------
// Shared test helpers for BC3 with_recorrelate tests
// -------------------------------------------------------------------------------------------------

/// Common type alias for BC3 transform functions with decorrelation used across tests.
pub(crate) type WithDecorrelateTransformFn =
    unsafe fn(*const u8, *mut u16, *mut u16, *mut u32, *mut u32, usize);

/// Common type alias for BC3 untransform functions with recorrelation used across tests.
pub(crate) type WithRecorrelateUntransformFn =
    unsafe fn(*const u16, *const u16, *const u32, *const u32, *mut u8, usize);

/// Executes a decorrelate transform → untransform round-trip on 1‥=max_blocks BC3 blocks
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
        let original = generate_bc3_test_data(num_blocks);
        let len = original.len();

        // Allocate combined buffer for separated data (like BC2 does)
        let mut transformed = allocate_align_64(len);
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            // Transform with decorrelation using combined buffer layout
            transform_fn(
                original.as_ptr(),
                transformed.as_mut_ptr() as *mut u16, // alpha_endpoints at start
                transformed.as_mut_ptr().add(len / 8) as *mut u16, // alpha_indices at len/8
                transformed.as_mut_ptr().add(len / 2) as *mut u32, // colors at len/2
                transformed.as_mut_ptr().add(len / 2 + len / 4) as *mut u32, // color_indices at 3*len/4
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
        let original = generate_bc3_test_data(num_blocks);
        let len = original.len();

        // Allocate buffers
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            // First transform with decorrelation using public dispatcher
            let mut transformed = allocate_align_64(len);
            transform_with_decorrelate(original.as_ptr(), transformed.as_mut_ptr(), len, variant);

            // Extract separated pointers from the combined buffer
            let alpha_endpoints_ptr = transformed.as_ptr() as *const u16;
            let alpha_indices_ptr = transformed.as_ptr().add(len / 8) as *const u16;
            let colors_ptr = transformed.as_ptr().add(len / 2) as *const u32;
            let color_indices_ptr = transformed.as_ptr().add(len / 2 + len / 4) as *const u32;

            // Then untransform with recorrelation using individual function
            untransform_fn(
                alpha_endpoints_ptr,
                alpha_indices_ptr,
                colors_ptr,
                color_indices_ptr,
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

// -----------------------------------------
// Shared test helpers for combination variant tests
// -----------------------------------------

/// Common type alias for BC3 split alphas and colour transform functions used across tests.
pub(crate) type SplitAlphasAndColourTransformFn =
    unsafe fn(*const u8, *mut u8, *mut u8, *mut u16, *mut u16, *mut u16, *mut u32, usize);

/// Common type alias for BC3 split alphas and colour untransform functions used across tests.
pub(crate) type SplitAlphasAndColourUntransformFn =
    unsafe fn(*const u8, *const u8, *const u16, *const u16, *const u16, *const u32, *mut u8, usize);

/// Common type alias for BC3 split alphas and recorrelate transform functions used across tests.
pub(crate) type SplitAlphasAndRecorrTransformFn =
    unsafe fn(*const u8, *mut u8, *mut u8, *mut u16, *mut u32, *mut u32, usize, YCoCgVariant);

/// Common type alias for BC3 split alphas and recorrelate untransform functions used across tests.
pub(crate) type SplitAlphasAndRecorrUntransformFn =
    unsafe fn(*const u8, *const u8, *const u16, *const u32, *const u32, *mut u8, usize, YCoCgVariant);

/// Common type alias for BC3 split colour and recorrelate transform functions used across tests.
pub(crate) type SplitColourAndRecorrTransformFn =
    unsafe fn(*const u8, *mut u16, *mut u16, *mut u16, *mut u16, *mut u32, usize, YCoCgVariant);

/// Common type alias for BC3 split colour and recorrelate untransform functions used across tests.
pub(crate) type SplitColourAndRecorrUntransformFn =
    unsafe fn(*const u16, *const u16, *const u16, *const u16, *const u32, *mut u8, usize, YCoCgVariant);

/// Common type alias for BC3 split alphas, colour and recorrelate transform functions used across tests.
pub(crate) type SplitAlphasColourAndRecorrTransformFn =
    unsafe fn(*const u8, *mut u8, *mut u8, *mut u16, *mut u16, *mut u16, *mut u32, usize, YCoCgVariant);

/// Common type alias for BC3 split alphas, colour and recorrelate untransform functions used across tests.
pub(crate) type SplitAlphasColourAndRecorrUntransformFn =
    unsafe fn(*const u8, *const u8, *const u16, *const u16, *const u16, *const u32, *mut u8, usize, YCoCgVariant);

/// Executes a split alphas and colour transform → untransform round-trip on 1‥=max_blocks BC3 blocks
/// using the specified transform function, asserting that the final data matches the original input.
#[inline]
pub(crate) fn run_split_alphas_and_colour_transform_roundtrip_test(
    transform_fn: SplitAlphasAndColourTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas_and_colour::untransform_with_split_alphas_and_colour;
    for num_blocks in 1..=max_blocks {
        let original = generate_bc3_test_data(num_blocks);
        // Allocate separate arrays for split alphas and colour data
        let mut alpha0_data = allocate_align_64(num_blocks);
        let mut alpha1_data = allocate_align_64(num_blocks);
        let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
        let mut color0_data = allocate_align_64(num_blocks * 2);
        let mut color1_data = allocate_align_64(num_blocks * 2);
        let mut color_indices_data = allocate_align_64(num_blocks * 4);
        let mut reconstructed = allocate_align_64(original.len());
        unsafe {
            // Transform using the function being tested
            transform_fn(
                original.as_ptr(),
                alpha0_data.as_mut_ptr(),
                alpha1_data.as_mut_ptr(),
                alpha_indices_data.as_mut_ptr() as *mut u16,
                color0_data.as_mut_ptr() as *mut u16,
                color1_data.as_mut_ptr() as *mut u16,
                color_indices_data.as_mut_ptr() as *mut u32,
                num_blocks,
            );
            // Untransform using the generic dispatcher
            untransform_with_split_alphas_and_colour(
                alpha0_data.as_ptr(),
                alpha1_data.as_ptr(),
                alpha_indices_data.as_ptr() as *const u16,
                color0_data.as_ptr() as *const u16,
                color1_data.as_ptr() as *const u16,
                color_indices_data.as_ptr() as *const u32,
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

/// Executes an unaligned untransform test for split alphas and colour operations.
#[inline]
pub(crate) fn run_with_split_alphas_and_colour_untransform_unaligned_test(
    untransform_fn: SplitAlphasAndColourUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas_and_colour::transform_with_split_alphas_and_colour;
    for num_blocks in 1..=max_blocks {
        let original = generate_bc3_test_data(num_blocks);
        let mut alpha0_data = allocate_align_64(num_blocks);
        let mut alpha1_data = allocate_align_64(num_blocks);
        let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
        let mut color0_data = allocate_align_64(num_blocks * 2);
        let mut color1_data = allocate_align_64(num_blocks * 2);
        let mut color_indices_data = allocate_align_64(num_blocks * 4);
        let mut reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // Transform first
            transform_with_split_alphas_and_colour(
                original.as_ptr(),
                alpha0_data.as_mut_ptr(),
                alpha1_data.as_mut_ptr(),
                alpha_indices_data.as_mut_ptr() as *mut u16,
                color0_data.as_mut_ptr() as *mut u16,
                color1_data.as_mut_ptr() as *mut u16,
                color_indices_data.as_mut_ptr() as *mut u32,
                num_blocks,
            );
            // Untransform with unaligned output only
            untransform_fn(
                alpha0_data.as_ptr(),
                alpha1_data.as_ptr(),
                alpha_indices_data.as_ptr() as *const u16,
                color0_data.as_ptr() as *const u16,
                color1_data.as_ptr() as *const u16,
                color_indices_data.as_ptr() as *const u32,
                reconstructed.as_mut_ptr().add(1),
                num_blocks,
            );
        }
        assert_eq!(
            original.as_slice(),
            &reconstructed.as_slice()[1..original.len() + 1],
            "Mismatch in {impl_name} unaligned test for {num_blocks} blocks",
        );
    }
}

/// Executes a split alphas and recorrelate transform → untransform round-trip test.
#[inline]
pub(crate) fn run_split_alphas_and_recorr_transform_roundtrip_test(
    transform_fn: SplitAlphasAndRecorrTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas_and_recorr::untransform_with_split_alphas_and_recorr;
    for variant in [YCoCgVariant::Variant1, YCoCgVariant::Variant2, YCoCgVariant::Variant3] {
        for num_blocks in 1..=max_blocks {
            let original = generate_bc3_test_data(num_blocks);
            let mut alpha0_data = allocate_align_64(num_blocks);
            let mut alpha1_data = allocate_align_64(num_blocks);
            let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
            let mut decorrelated_colors_data = allocate_align_64(num_blocks * 4);
            let mut color_indices_data = allocate_align_64(num_blocks * 4);
            let mut reconstructed = allocate_align_64(original.len());
            unsafe {
                transform_fn(
                    original.as_ptr(),
                    alpha0_data.as_mut_ptr(),
                    alpha1_data.as_mut_ptr(),
                    alpha_indices_data.as_mut_ptr() as *mut u16,
                    decorrelated_colors_data.as_mut_ptr() as *mut u32,
                    color_indices_data.as_mut_ptr() as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_with_split_alphas_and_recorr(
                    alpha0_data.as_ptr(),
                    alpha1_data.as_ptr(),
                    alpha_indices_data.as_ptr() as *const u16,
                    decorrelated_colors_data.as_ptr() as *const u32,
                    color_indices_data.as_ptr() as *const u32,
                    reconstructed.as_mut_ptr(),
                    num_blocks,
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
}

/// Executes an unaligned untransform test for split alphas and recorrelate operations.
#[inline]
pub(crate) fn run_with_split_alphas_and_recorr_untransform_unaligned_test(
    untransform_fn: SplitAlphasAndRecorrUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas_and_recorr::transform_with_split_alphas_and_recorr;
    for variant in [YCoCgVariant::Variant1, YCoCgVariant::Variant2, YCoCgVariant::Variant3] {
        for num_blocks in 1..=max_blocks {
            let original = generate_bc3_test_data(num_blocks);
            let mut alpha0_data = allocate_align_64(num_blocks);
            let mut alpha1_data = allocate_align_64(num_blocks);
            let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
            let mut decorrelated_colors_data = allocate_align_64(num_blocks * 4);
            let mut color_indices_data = allocate_align_64(num_blocks * 4);
            let mut reconstructed = allocate_align_64(original.len() + 1);

            unsafe {
                transform_with_split_alphas_and_recorr(
                    original.as_ptr(),
                    alpha0_data.as_mut_ptr(),
                    alpha1_data.as_mut_ptr(),
                    alpha_indices_data.as_mut_ptr() as *mut u16,
                    decorrelated_colors_data.as_mut_ptr() as *mut u32,
                    color_indices_data.as_mut_ptr() as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_fn(
                    alpha0_data.as_ptr(),
                    alpha1_data.as_ptr(),
                    alpha_indices_data.as_ptr() as *const u16,
                    decorrelated_colors_data.as_ptr() as *const u32,
                    color_indices_data.as_ptr() as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                    variant,
                );
            }
            assert_eq!(
                original.as_slice(),
                &reconstructed.as_slice()[1..original.len() + 1],
                "Mismatch in {impl_name} unaligned test for {num_blocks} blocks with {variant:?}",
            );
        }
    }
}

/// Executes a split colour and recorrelate transform → untransform round-trip test.
#[inline]
pub(crate) fn run_split_colour_and_recorr_transform_roundtrip_test(
    transform_fn: SplitColourAndRecorrTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour_and_recorr::untransform_with_split_colour_and_recorr;
    for variant in [YCoCgVariant::Variant1, YCoCgVariant::Variant2, YCoCgVariant::Variant3] {
        for num_blocks in 1..=max_blocks {
            let original = generate_bc3_test_data(num_blocks);
            let mut alpha_endpoints_data = allocate_align_64(num_blocks * 2);
            let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
            let mut decorrelated_color0_data = allocate_align_64(num_blocks * 2);
            let mut decorrelated_color1_data = allocate_align_64(num_blocks * 2);
            let mut color_indices_data = allocate_align_64(num_blocks * 4);
            let mut reconstructed = allocate_align_64(original.len());
            unsafe {
                transform_fn(
                    original.as_ptr(),
                    alpha_endpoints_data.as_mut_ptr() as *mut u16,
                    alpha_indices_data.as_mut_ptr() as *mut u16,
                    decorrelated_color0_data.as_mut_ptr() as *mut u16,
                    decorrelated_color1_data.as_mut_ptr() as *mut u16,
                    color_indices_data.as_mut_ptr() as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_with_split_colour_and_recorr(
                    alpha_endpoints_data.as_ptr() as *const u16,
                    alpha_indices_data.as_ptr() as *const u16,
                    decorrelated_color0_data.as_ptr() as *const u16,
                    decorrelated_color1_data.as_ptr() as *const u16,
                    color_indices_data.as_ptr() as *const u32,
                    reconstructed.as_mut_ptr(),
                    num_blocks,
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
}

/// Executes an unaligned untransform test for split colour and recorrelate operations.
#[inline]
pub(crate) fn run_with_split_colour_and_recorr_untransform_unaligned_test(
    untransform_fn: SplitColourAndRecorrUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour_and_recorr::transform_with_split_colour_and_recorr;
    for variant in [YCoCgVariant::Variant1, YCoCgVariant::Variant2, YCoCgVariant::Variant3] {
        for num_blocks in 1..=max_blocks {
            let original = generate_bc3_test_data(num_blocks);
            let mut alpha_endpoints_data = allocate_align_64(num_blocks * 2);
            let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
            let mut decorrelated_color0_data = allocate_align_64(num_blocks * 2);
            let mut decorrelated_color1_data = allocate_align_64(num_blocks * 2);
            let mut color_indices_data = allocate_align_64(num_blocks * 4);
            let mut reconstructed = allocate_align_64(original.len() + 1);

            unsafe {
                transform_with_split_colour_and_recorr(
                    original.as_ptr(),
                    alpha_endpoints_data.as_mut_ptr() as *mut u16,
                    alpha_indices_data.as_mut_ptr() as *mut u16,
                    decorrelated_color0_data.as_mut_ptr() as *mut u16,
                    decorrelated_color1_data.as_mut_ptr() as *mut u16,
                    color_indices_data.as_mut_ptr() as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_fn(
                    alpha_endpoints_data.as_ptr() as *const u16,
                    alpha_indices_data.as_ptr() as *const u16,
                    decorrelated_color0_data.as_ptr() as *const u16,
                    decorrelated_color1_data.as_ptr() as *const u16,
                    color_indices_data.as_ptr() as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                    variant,
                );
            }
            assert_eq!(
                original.as_slice(),
                &reconstructed.as_slice()[1..original.len() + 1],
                "Mismatch in {impl_name} unaligned test for {num_blocks} blocks with {variant:?}",
            );
        }
    }
}

/// Executes a split alphas, colour and recorrelate transform → untransform round-trip test.
#[inline]
pub(crate) fn run_split_alphas_colour_and_recorr_transform_roundtrip_test(
    transform_fn: SplitAlphasColourAndRecorrTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas_colour_and_recorr::untransform_with_split_alphas_colour_and_recorr;
    for variant in [YCoCgVariant::Variant1, YCoCgVariant::Variant2, YCoCgVariant::Variant3] {
        for num_blocks in 1..=max_blocks {
            let original = generate_bc3_test_data(num_blocks);
            let mut alpha0_data = allocate_align_64(num_blocks);
            let mut alpha1_data = allocate_align_64(num_blocks);
            let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
            let mut decorrelated_color0_data = allocate_align_64(num_blocks * 2);
            let mut decorrelated_color1_data = allocate_align_64(num_blocks * 2);
            let mut color_indices_data = allocate_align_64(num_blocks * 4);
            let mut reconstructed = allocate_align_64(original.len());
            unsafe {
                transform_fn(
                    original.as_ptr(),
                    alpha0_data.as_mut_ptr(),
                    alpha1_data.as_mut_ptr(),
                    alpha_indices_data.as_mut_ptr() as *mut u16,
                    decorrelated_color0_data.as_mut_ptr() as *mut u16,
                    decorrelated_color1_data.as_mut_ptr() as *mut u16,
                    color_indices_data.as_mut_ptr() as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_with_split_alphas_colour_and_recorr(
                    alpha0_data.as_ptr(),
                    alpha1_data.as_ptr(),
                    alpha_indices_data.as_ptr() as *const u16,
                    decorrelated_color0_data.as_ptr() as *const u16,
                    decorrelated_color1_data.as_ptr() as *const u16,
                    color_indices_data.as_ptr() as *const u32,
                    reconstructed.as_mut_ptr(),
                    num_blocks,
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
}

/// Executes an unaligned untransform test for split alphas, colour and recorrelate operations.
#[inline]
pub(crate) fn run_with_split_alphas_colour_and_recorr_untransform_unaligned_test(
    untransform_fn: SplitAlphasColourAndRecorrUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_alphas_colour_and_recorr::transform_with_split_alphas_colour_and_recorr;
    for variant in [YCoCgVariant::Variant1, YCoCgVariant::Variant2, YCoCgVariant::Variant3] {
        for num_blocks in 1..=max_blocks {
            let original = generate_bc3_test_data(num_blocks);
            let mut alpha0_data = allocate_align_64(num_blocks);
            let mut alpha1_data = allocate_align_64(num_blocks);
            let mut alpha_indices_data = allocate_align_64(num_blocks * 6);
            let mut decorrelated_color0_data = allocate_align_64(num_blocks * 2);
            let mut decorrelated_color1_data = allocate_align_64(num_blocks * 2);
            let mut color_indices_data = allocate_align_64(num_blocks * 4);
            let mut reconstructed = allocate_align_64(original.len() + 1);

            unsafe {
                transform_with_split_alphas_colour_and_recorr(
                    original.as_ptr(),
                    alpha0_data.as_mut_ptr(),
                    alpha1_data.as_mut_ptr(),
                    alpha_indices_data.as_mut_ptr() as *mut u16,
                    decorrelated_color0_data.as_mut_ptr() as *mut u16,
                    decorrelated_color1_data.as_mut_ptr() as *mut u16,
                    color_indices_data.as_mut_ptr() as *mut u32,
                    num_blocks,
                    variant,
                );
                untransform_fn(
                    alpha0_data.as_ptr(),
                    alpha1_data.as_ptr(),
                    alpha_indices_data.as_ptr() as *const u16,
                    decorrelated_color0_data.as_ptr() as *const u16,
                    decorrelated_color1_data.as_ptr() as *const u16,
                    color_indices_data.as_ptr() as *const u32,
                    reconstructed.as_mut_ptr().add(1),
                    num_blocks,
                    variant,
                );
            }
            assert_eq!(
                original.as_slice(),
                &reconstructed.as_slice()[1..original.len() + 1],
                "Mismatch in {impl_name} unaligned test for {num_blocks} blocks with {variant:?}",
            );
        }
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
