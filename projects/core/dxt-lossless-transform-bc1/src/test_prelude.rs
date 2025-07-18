//! Common test imports and utilities for BC1 tests
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
pub use crate::{transform_bc1_with_settings, Bc1TransformSettings};

// Common types from dxt_lossless_transform_api_common
pub use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

// Common types from dxt_lossless_transform_common
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;
pub use dxt_lossless_transform_common::color_8888::Color8888;
pub use dxt_lossless_transform_common::cpu_detect::*;
pub use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;

// Standard library imports commonly used in tests
pub use core::ptr::{copy_nonoverlapping, write_bytes};
pub use safe_allocator_api::RawAlloc;

// Re-export super for convenience in test modules
pub use super::*;

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
    let mut data = allocate_align_64(num_blocks * 8);
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

// -------------------------------------------------------------------------------------------------
// Shared test helpers lifted from individual transform tests
// -------------------------------------------------------------------------------------------------

/// Common type alias for transform/permute functions used across BC1 tests.
pub(crate) type StandardTransformFn = unsafe fn(*const u8, *mut u8, usize);

/// Common type alias for decorrelate transform functions used across BC1 with_recorrelate tests.
pub(crate) type WithDecorrelateTransformFn = unsafe fn(*const u8, *mut u32, *mut u32, usize);

/// Common type alias for split-colour transform functions used across BC1 tests.
pub(crate) type SplitColourTransformFn = unsafe fn(*const u8, *mut u16, *mut u16, *mut u32, usize);

/// Common type alias for split-colour with decorrelation transform functions used across BC1 tests.
pub(crate) type SplitColourWithDecorrTransformFn =
    unsafe fn(*const u8, *mut u16, *mut u16, *mut u32, usize, YCoCgVariant);

/// Executes a transform → untransform round-trip on 1‥=max_blocks BC1 blocks and
/// asserts that the final data matches the original input.
#[inline]
pub(crate) fn run_standard_transform_roundtrip_test(
    transform_fn: StandardTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let input = generate_bc1_test_data(num_blocks);
        let len = input.len();
        let mut transformed = allocate_align_64(len);
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            transform_fn(input.as_ptr(), transformed.as_mut_ptr(), len);
            crate::transform::standard::untransform(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                len,
            );
        }

        assert_eq!(
            reconstructed.as_slice(),
            input.as_slice(),
            "Mismatch {impl_name} roundtrip for {num_blocks} blocks",
        );
    }
}

/// Executes a decorrelate transform → untransform round-trip on 1‥=max_blocks BC1 blocks
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
        let input = generate_bc1_test_data(num_blocks);
        let len = input.len();
        let mut transformed = allocate_align_64(len);
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            transform_fn(
                input.as_ptr(),
                transformed.as_mut_ptr() as *mut u32,
                transformed.as_mut_ptr().add(len / 2) as *mut u32,
                num_blocks,
            );
            untransform_with_recorrelate(
                transformed.as_ptr(),
                reconstructed.as_mut_ptr(),
                num_blocks * 8,
                variant,
            );
        }

        assert_eq!(
            reconstructed.as_slice(),
            input.as_slice(),
            "Mismatch {impl_name} roundtrip variant {variant:?} for {num_blocks} blocks",
        );
    }
}

/// Executes a split-colour transform → untransform round-trip on 1‥=max_blocks BC1 blocks and
/// asserts that the final data matches the original input.
///
/// The `max_blocks` parameter should equal twice the number of blocks processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 8).
#[inline]
pub(crate) fn run_split_colour_transform_roundtrip_test(
    transform_fn: SplitColourTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour::untransform::untransform_with_split_colour;

    for num_blocks in 1..=max_blocks {
        let input = generate_bc1_test_data(num_blocks);
        let len = input.len();
        let mut colour0 = allocate_align_64(num_blocks * 2); // u16 = 2 bytes
        let mut colour1 = allocate_align_64(num_blocks * 2); // u16 = 2 bytes
        let mut indices = allocate_align_64(num_blocks * 4); // u32 = 4 bytes
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            transform_fn(
                input.as_ptr(),
                colour0.as_mut_ptr() as *mut u16,
                colour1.as_mut_ptr() as *mut u16,
                indices.as_mut_ptr() as *mut u32,
                num_blocks,
            );
            untransform_with_split_colour(
                colour0.as_ptr() as *const u16,
                colour1.as_ptr() as *const u16,
                indices.as_ptr() as *const u32,
                reconstructed.as_mut_ptr(),
                num_blocks,
            );
        }

        assert_eq!(
            reconstructed.as_slice(),
            input.as_slice(),
            "Mismatch {impl_name} roundtrip split-colour for {num_blocks} blocks",
        );
    }
}

/// Executes a split-colour with decorrelation transform → untransform round-trip on 1‥=max_blocks
/// BC1 blocks using the specified transform function and YCoCg variant, asserting that the final data
/// matches the original input.
///
/// The `max_blocks` parameter should equal twice the number of blocks processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 8).
#[inline]
pub(crate) fn run_split_colour_with_decorr_transform_roundtrip_test(
    transform_fn: SplitColourWithDecorrTransformFn,
    variant: YCoCgVariant,
    max_blocks: usize,
    impl_name: &str,
) {
    use crate::transform::with_split_colour_and_recorr::untransform::untransform_with_split_colour_and_recorr;

    for num_blocks in 1..=max_blocks {
        let input = generate_bc1_test_data(num_blocks);
        let len = input.len();
        let mut colour0 = allocate_align_64(num_blocks * 2); // u16 = 2 bytes
        let mut colour1 = allocate_align_64(num_blocks * 2); // u16 = 2 bytes
        let mut indices = allocate_align_64(num_blocks * 4); // u32 = 4 bytes
        let mut reconstructed = allocate_align_64(len);

        unsafe {
            transform_fn(
                input.as_ptr(),
                colour0.as_mut_ptr() as *mut u16,
                colour1.as_mut_ptr() as *mut u16,
                indices.as_mut_ptr() as *mut u32,
                num_blocks,
                variant,
            );
            untransform_with_split_colour_and_recorr(
                colour0.as_ptr() as *const u16,
                colour1.as_ptr() as *const u16,
                indices.as_ptr() as *const u32,
                reconstructed.as_mut_ptr(),
                num_blocks,
                variant,
            );
        }

        assert_eq!(
            reconstructed.as_slice(),
            input.as_slice(),
            "Mismatch {impl_name} roundtrip split-colour with decorr {variant:?} for {num_blocks} blocks",
        );
    }
}

// --------------------------------------
// Helper functions for untransform tests
// --------------------------------------

/// Common type alias for with_recorrelate untransform functions used across BC1 tests.
pub(crate) type WithRecorrelateUntransformFn = unsafe fn(*const u32, *const u32, *mut u8, usize);

/// Common type alias for with_split_colour untransform functions used across BC1 tests.
pub(crate) type WithSplitColourUntransformFn =
    unsafe fn(*const u16, *const u16, *const u32, *mut u8, usize);

/// Common type alias for with_split_colour_and_recorr generic untransform functions used across BC1 tests.
pub(crate) type WithSplitColourAndRecorrUntransformFn =
    unsafe fn(*const u16, *const u16, *const u32, *mut u8, usize, YCoCgVariant);

/// Executes a with_recorrelate untransform test on 1‥=max_blocks BC1 blocks using unaligned buffers.
/// This helper eliminates code duplication across AVX2, AVX512, SSE2, and generic untransform tests.
///
/// The `max_blocks` parameter should equal twice the number of blocks processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 8).
#[inline]
pub(crate) fn run_with_recorrelate_untransform_unaligned_test(
    untransform_fn: WithRecorrelateUntransformFn,
    decorr_variant: YCoCgVariant,
    impl_name: &str,
    max_blocks: usize,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc1_test_data(num_blocks);

        // Transform using standard implementation
        let mut transformed = allocate_align_64(original.len());
        unsafe {
            transform_bc1_with_settings(
                original.as_ptr(),
                transformed.as_mut_ptr(),
                original.len(),
                Bc1TransformSettings {
                    decorrelation_mode: decorr_variant,
                    split_colour_endpoints: false,
                },
            );
        }

        // Add 1 extra byte at the beginning to create misaligned buffers
        let mut transformed_unaligned = allocate_align_64(transformed.len() + 1);
        transformed_unaligned.as_mut_slice()[1..].copy_from_slice(transformed.as_slice());
        let mut reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            untransform_fn(
                transformed_unaligned.as_ptr().add(1) as *const u32,
                transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                reconstructed.as_mut_ptr().add(1),
                num_blocks,
            );
        }

        assert_implementation_matches_reference(
            original.as_slice(),
            &reconstructed.as_slice()[1..],
            impl_name,
            num_blocks,
        );
    }
}

/// Same as [`run_standard_untransform_aligned_test`] but also validates the implementation
/// with deliberately mis-aligned (offset by 1 byte) input and output pointers.
///
/// The `max_blocks` parameter should equal twice the number of blocks processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 8).
#[inline]
pub(crate) fn run_standard_untransform_unaligned_test(
    untransform_fn: StandardTransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc1_test_data(num_blocks);

        // Transform using the reference path
        let mut transformed = allocate_align_64(original.len());
        unsafe {
            crate::transform::standard::transform(
                original.as_ptr(),
                transformed.as_mut_ptr(),
                original.len(),
            );

            // Shift by one byte to mis-align the buffers
            let mut transformed_unaligned = allocate_align_64(transformed.len() + 1);
            transformed_unaligned.as_mut_slice()[1..].copy_from_slice(transformed.as_slice());

            let mut reconstructed = allocate_align_64(original.len() + 1);
            untransform_fn(
                transformed_unaligned.as_ptr().add(1),
                reconstructed.as_mut_ptr().add(1),
                transformed.len(),
            );

            assert_implementation_matches_reference(
                original.as_slice(),
                &reconstructed.as_slice()[1..],
                &format!("{impl_name} (unaligned)"),
                num_blocks,
            );
        }
    }
}

/// Executes a with_split_colour untransform test on 1‥=max_blocks BC1 blocks using unaligned buffers.
/// This helper eliminates code duplication across AVX2, AVX512, SSE2, and generic untransform tests.
///
/// The `max_blocks` parameter should equal twice the number of blocks processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 8).
#[inline]
pub(crate) fn run_with_split_colour_untransform_unaligned_test(
    untransform_fn: WithSplitColourUntransformFn,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc1_test_data(num_blocks);

        // Transform using standard implementation
        let mut transformed = allocate_align_64(original.len());
        unsafe {
            transform_bc1_with_settings(
                original.as_ptr(),
                transformed.as_mut_ptr(),
                original.len(),
                Bc1TransformSettings {
                    decorrelation_mode: YCoCgVariant::None,
                    split_colour_endpoints: true,
                },
            );
        }

        // Add 1 extra byte at the beginning to create misaligned buffers
        let mut transformed_unaligned = allocate_align_64(transformed.len() + 1);
        transformed_unaligned.as_mut_slice()[1..].copy_from_slice(transformed.as_slice());
        let mut reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // Reconstruct using the implementation being tested with unaligned pointers
            untransform_fn(
                transformed_unaligned.as_ptr().add(1) as *const u16,
                transformed_unaligned.as_ptr().add(1 + num_blocks * 2) as *const u16,
                transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                reconstructed.as_mut_ptr().add(1),
                num_blocks,
            );
        }

        assert_implementation_matches_reference(
            original.as_slice(),
            &reconstructed.as_slice()[1..],
            impl_name,
            num_blocks,
        );
    }
}

/// Executes a with_split_colour_and_recorr untransform test on 1‥=max_blocks BC1 blocks using unaligned buffers.
/// This helper eliminates code duplication across AVX2, AVX512, SSE2, and generic untransform tests.
///
/// The `max_blocks` parameter should equal twice the number of blocks processed in one main loop
/// iteration of the SIMD implementation being tested (i.e., bytes processed × 2 ÷ 8).
#[inline]
pub(crate) fn run_with_split_colour_and_recorr_untransform_unaligned_test(
    untransform_fn: WithSplitColourAndRecorrUntransformFn,
    decorr_variant: YCoCgVariant,
    max_blocks: usize,
    impl_name: &str,
) {
    for num_blocks in 1..=max_blocks {
        let original = generate_bc1_test_data(num_blocks);

        // Transform using standard implementation
        let mut transformed = allocate_align_64(original.len());
        unsafe {
            transform_bc1_with_settings(
                original.as_ptr(),
                transformed.as_mut_ptr(),
                original.len(),
                Bc1TransformSettings {
                    decorrelation_mode: decorr_variant,
                    split_colour_endpoints: true,
                },
            );
        }

        // Add 1 extra byte at the beginning to create misaligned buffers
        let mut transformed_unaligned = allocate_align_64(transformed.len() + 1);
        transformed_unaligned.as_mut_slice()[1..].copy_from_slice(transformed.as_slice());
        let mut reconstructed = allocate_align_64(original.len() + 1);

        unsafe {
            // Reconstruct using the implementation being tested with unaligned pointers
            untransform_fn(
                transformed_unaligned.as_ptr().add(1) as *const u16,
                transformed_unaligned.as_ptr().add(1 + num_blocks * 2) as *const u16,
                transformed_unaligned.as_ptr().add(1 + num_blocks * 4) as *const u32,
                reconstructed.as_mut_ptr().add(1),
                num_blocks,
                decorr_variant,
            );
        }

        assert_implementation_matches_reference(
            original.as_slice(),
            &reconstructed.as_slice()[1..],
            impl_name,
            num_blocks,
        );
    }
}

/// Alias for [`run_with_split_colour_and_recorr_untransform_unaligned_test`] for backward compatibility.
/// All implementations now use functions that take a [`YCoCgVariant`] parameter.
#[inline]
pub(crate) fn run_with_split_colour_and_recorr_generic_untransform_unaligned_test(
    untransform_fn: WithSplitColourAndRecorrUntransformFn,
    decorr_variant: YCoCgVariant,
    max_blocks: usize,
    impl_name: &str,
) {
    run_with_split_colour_and_recorr_untransform_unaligned_test(
        untransform_fn,
        decorr_variant,
        max_blocks,
        impl_name,
    );
}
