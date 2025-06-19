//! Optimal BC1 Transform Determination
//!
//! This module provides functionality to determine the best transformation parameters for BC1
//! (DXT1) compressed texture data to achieve optimal compression ratios.
//!
//! ## Overview
//!
//! BC1 compression can be further optimized by applying various transformations before
//! final compression. This module analyzes different transformation options and [`Bc1TransformDetails`]
//! and selects the combination that results in the smallest compressed file size.
//!
//! ## Performance Characteristics
//!
//! The functions in this module perform brute force testing of different transformations
//! as follows:
//!
//! 1. Transform the data into a specific format.
//! 2. Estimate the compressed size using a provided file size estimator function.
//! 3. Compare the estimated sizes to find the best transformation.
//!
//! The typical throughput for the transformation is ~24GB/s transformation speed on single thread (Ryzen 9950X3D),
//! so in practice the speed depends on how fast the estimator function can run. When ran with `zstandard -1`,
//! the speed is ~265MiB/s for the estimator function. With `lossless-transform-utils` it is 641.30MiB/s
//!
//! For memory, the usage is the same as the size of the input data, we need a buffer for the transformed
//! output.
//!
//! ## Usage Example
//!
//! ```rust,no_run
//! # use dxt_lossless_transform_bc1::determine_optimal_transform::{determine_best_transform_details, Bc1EstimateOptions};
//!
//! // Define a compression estimator function
//! fn my_compression_estimator(data: *const u8, len: usize) -> usize {
//!     // Your compression size estimation logic here
//!     len // Placeholder
//! }
//!
//! let bc1_data = vec![0u8; 8]; // Example BC1 block data
//! let options = Bc1EstimateOptions {
//!     file_size_estimator: my_compression_estimator,
//!     use_all_decorrelation_modes: false, // Fast mode
//! };
//!
//! // Determine optimal transform (unsafe due to raw pointers)
//! let transform_details = unsafe {
//!     determine_best_transform_details(bc1_data.as_ptr(), bc1_data.len(), std::ptr::null_mut(), options)
//! }.expect("Transform determination failed");
//! ```
//!
//! Your 'estimator' function needs to use the same 'concepts' as the actual final compression function.
//! For example, an LZ compressor will work well for another LZ compressor, but not for something
//! based on the Burrows-Wheeler Transform (BWT).
//!
//! [See my blog post](https://sewer56.dev/blog/2025/03/11/a-program-for-helping-create-lossless-transforms.html#estimator-accuracy-vs-bzip3) for reference.
//!
//! ## Optimization Strategy
//!
//! Determines the best [`Bc1TransformDetails`] by brute force testing of different transformation
//! combinations and selecting the one that produces the smallest estimated compressed size.
//!
//! ## Implementation Notes
//!
//! - Index data is excluded from size estimation as it has poor compressibility
//!   (entropy ≈ 7.0, minimal LZ matches) with negligible impact on results
//! - The brute force approach ensures finding the global optimum within tested parameters
//! - Memory allocation uses 64-byte alignment for optimal SIMD performance

use crate::Bc1TransformDetails;
use dxt_lossless_transform_common::{
    allocate::{allocate_align_64, AllocateError},
    color_565::YCoCgVariant,
};
use thiserror::Error;

/// The options for [`determine_best_transform_details`], regarding how the estimation is done,
/// and other related factors.
pub struct Bc1EstimateOptions<F>
where
    F: Fn(*const u8, usize) -> usize,
{
    /// A function that returns an estimated file size for the given passed in data+len tuple.
    ///
    /// # Parameters
    ///
    /// - `input_ptr`: A pointer to the input data
    /// - `len`: The length of the input data in bytes
    ///
    /// # Returns
    ///
    /// The estimated file size in bytes
    ///
    /// # Remarks
    ///
    /// For minimizing file size, use the exact same compression function as the final file will
    /// be compressed.
    ///
    /// Otherwise consider using a slightly lower level of the same compression function, both to
    /// maximize speed of [`determine_best_transform_details`], and to improve decompression speed
    /// by reducing the size of the sliding window (so more data in cache) and increasing minimum
    /// match length.
    pub file_size_estimator: F,

    /// Controls which decorrelation modes are tested during optimization.
    ///
    /// When `false` (default), only tests [`YCoCgVariant::Variant1`] and [`YCoCgVariant::None`]
    /// for faster optimization with good results.
    ///
    /// When `true`, tests all available decorrelation modes ([`YCoCgVariant::Variant1`],
    /// [`YCoCgVariant::Variant2`], [`YCoCgVariant::Variant3`], and [`YCoCgVariant::None`])
    /// for potentially better compression at the cost of longer optimization time.
    pub use_all_decorrelation_modes: bool,
}


/// Determine the best transform details for the given BC1 blocks.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `result_buffer_ptr`: A mutable pointer to the working buffer, or null to allocate internally
/// - `transform_options`: Options for the estimation including the file size estimator
///
/// # Returns
///
/// The best (smallest size) format for the given data.
///
/// # Remarks
///
/// This function is a brute force approach that tests all standard transform options:
/// - 1/4th of the compression speed in fast mode (2 [`YCoCgVariant`] * 2 (split_colours))
/// - 1/8th of the compression speed in comprehensive mode (4 [`YCoCgVariant`] * 2 (split_colours))
/// - If `result_buffer_ptr` is null, allocates memory internally; otherwise uses the provided buffer
///
/// For experimental normalization support, use the functions in the experimental module instead.
///
/// # Safety
///
/// Function is unsafe because it deals with raw pointers which must be correct.
/// If `result_buffer_ptr` is not null, it must point to at least `len` bytes of valid memory.
pub unsafe fn determine_best_transform_details<F>(
    input_ptr: *const u8,
    len: usize,
    result_buffer_ptr: *mut u8,
    transform_options: Bc1EstimateOptions<F>,
) -> Result<Bc1TransformDetails, DetermineBestTransformError>
where
    F: Fn(*const u8, usize) -> usize,
{
    // Check if we need to allocate memory or use the provided buffer
    let (buffer_ptr, _allocated_buffer) = if result_buffer_ptr.is_null() {
        let mut allocated = allocate_align_64(len)?;
        (allocated.as_mut_ptr(), Some(allocated))
    } else {
        (result_buffer_ptr, None)
    };

    let mut best_transform_details = Bc1TransformDetails::default();
    let mut best_size = usize::MAX;

    // Choose decorrelation modes to test based on the flag
    let decorrelation_modes = if transform_options.use_all_decorrelation_modes {
        // Test all available decorrelation modes
        YCoCgVariant::all_values()
    } else {
        // Test only Variant1 and None for faster optimization
        &[YCoCgVariant::Variant1, YCoCgVariant::None]
    };

    for decorrelation_mode in decorrelation_modes {
        for split_colours in [true, false] {
            // Get the current mode we're testing.
            let current_mode = Bc1TransformDetails {
                decorrelation_mode: *decorrelation_mode,
                split_colour_endpoints: split_colours,
            };

            // Apply a full transformation (~24GB/s on 1 thread, Ryzen 9950X3D)
            crate::transform_bc1(input_ptr, buffer_ptr, len, current_mode);

            // Note(sewer): The indices are very poorly compressible (entropy == ~7.0 , no lz matches).
            // Excluding them from the estimation has negligible effect on results, with a doubling of
            // speed.

            // Test the current mode by measuring the compressed size
            let result_size = (transform_options.file_size_estimator)(buffer_ptr, len / 2);
            if result_size < best_size {
                best_size = result_size;
                best_transform_details = current_mode;
            }
        }
    }

    Ok(best_transform_details)
}

/// An error that happened in memory allocation within the library.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DetermineBestTransformError {
    /// An error that happened in memory allocation within the library
    #[error(transparent)]
    AllocateError(#[from] AllocateError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    /// Simple dummy file size estimator that just returns the input length
    fn dummy_file_size_estimator(_data: *const u8, len: usize) -> usize {
        len
    }

    /// Test that determine_best_transform_details doesn't crash with minimal BC1 data
    #[rstest]
    fn determine_best_transform_details_does_not_crash_and_burn() {
        // Create minimal BC1 block data (8 bytes per block)
        // This is a simple red block
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];

        let transform_options = Bc1EstimateOptions {
            file_size_estimator: dummy_file_size_estimator,
            use_all_decorrelation_modes: false,
        };

        // This should not crash
        let result = unsafe {
            determine_best_transform_details(bc1_data.as_ptr(), bc1_data.len(), std::ptr::null_mut(), transform_options)
        };

        // Just verify it returns Ok, we don't care about the specific transform details
        assert!(
            result.is_ok(),
            "Function should not crash with valid BC1 data"
        );
    }
}
