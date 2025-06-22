//! BC1 Transform Optimization
//!
//! This module provides optimization functionality to determine the best
//! transformation parameters for BC1 data compression.

use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_common::allocate::{allocate_align_64, AllocateError};
use thiserror::Error;

use super::operations::transform_bc1_with_settings;
use super::settings::{Bc1TransformSettings, COMPREHENSIVE_TEST_ORDER, FAST_TEST_ORDER};

/// An error that happened during transform determination.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DetermineBestTransformError<E> {
    /// An error that happened in memory allocation within the library
    #[error(transparent)]
    AllocateError(#[from] AllocateError),

    /// An error that happened during size estimation
    #[error("Size estimation failed: {0:?}")]
    SizeEstimationError(E),
}

/// The options for [`transform_bc1_auto`], regarding how the estimation is done,
/// and other related factors.
pub struct Bc1EstimateOptions<T>
where
    T: SizeEstimationOperations,
{
    /// A trait-based size estimator that provides size estimation operations.
    ///
    /// # Remarks
    ///
    /// The estimator should have its compression level and other parameters already configured.
    /// This allows for more flexible usage patterns where different estimators can have
    /// completely different configuration approaches.
    ///
    /// For minimizing file size, use the exact same compression algorithm as the final file will
    /// be compressed with.
    ///
    /// Otherwise consider using a slightly lower level of the same compression function, both to
    /// maximize speed of [`transform_bc1_auto`], and to improve decompression speed
    /// by reducing the size of the sliding window (so more data in cache) and increasing minimum
    /// match length.
    pub size_estimator: T,

    /// Controls which decorrelation modes are tested during optimization.
    ///
    /// When `false` (default), only tests [YCoCgVariant::Variant1] and [YCoCgVariant::None]
    /// for faster optimization with good results.
    ///
    /// When `true`, tests all available decorrelation modes ([YCoCgVariant::Variant1],
    /// [YCoCgVariant::Variant2], [YCoCgVariant::Variant3], and [YCoCgVariant::None])
    /// for potentially better compression at the cost of twice as long optimization
    /// time (tests 4 options instead of 2) for negligible gains (typically <0.1% extra savings).
    ///
    /// [YCoCgVariant::Variant1]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1
    /// [YCoCgVariant::Variant2]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2
    /// [YCoCgVariant::Variant3]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3
    /// [YCoCgVariant::None]: dxt_lossless_transform_common::color_565::YCoCgVariant::None
    pub use_all_decorrelation_modes: bool,
}

/// Transform BC1 data using the best determined settings.
///
/// This function tests various transform configurations and applies the one that
/// produces the smallest compressed size according to the provided estimator.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output buffer where transformed data will be written
/// - `len`: The length of the input data in bytes
/// - `transform_options`: Options for the estimation including the file size estimator
///
/// # Returns
///
/// The [`Bc1TransformSettings`] that produced the best (smallest) compressed size.
///
/// # Remarks
///
/// This function combines the functionality of determining the best transform options
/// and actually transforming the data, eliminating the need for two separate calls.
/// The output buffer will contain the transformed data using the optimal settings.
///
/// This function is a brute force approach that tests all standard transform options:
/// - 1/4th of the compression speed in fast mode (2 [YCoCgVariant] * 2 (split_colours))
/// - 1/8th of the compression speed in comprehensive mode (4 [YCoCgVariant] * 2 (split_colours))
///
/// ## Performance Characteristics
///
/// Overall throughput depends on the estimator used:
/// - **LTU estimator**: ~641 MiB/s (fast, good accuracy)
/// - **ZStandard level 1 estimator**: ~265 MiB/s (slower, higher accuracy)
///
/// The transformation itself runs at ~24GB/s, so the estimator becomes the bottleneck.
///
/// ## Performance Optimization
///
/// The transform options are tested in order of decreasing probability of being optimal,
/// based on analysis of 2,130 BC1 texture files (zstd estimator level 1):
/// - YCoCg1/Split (71.1% probability) - **tested last to minimize redundant transforms**
/// - YCoCg1/NoSplit (17.9% probability)
/// - YCoCg2/Split (3.5% probability)
/// - YCoCg3/Split (2.7% probability)
/// - YCoCg3/NoSplit (1.9% probability)
/// - None/Split (1.1% probability)
/// - None/NoSplit (1.0% probability)
/// - YCoCg2/NoSplit (0.9% probability)
///
/// Since YCoCg1/Split is optimal for ~71% of textures, testing it last means we avoid
/// a final redundant transform in the majority of cases.
///
/// For experimental normalization support, use the functions in the experimental module instead.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `output_ptr` must be valid for writes of `len` bytes
/// - `len` must be divisible by 8
/// - It is recommended that `input_ptr` and `output_ptr` are at least 16-byte aligned (recommended 32-byte align)
///
/// # Examples
///
/// ```rust,no_run
/// # use dxt_lossless_transform_bc1::{transform_bc1_auto, Bc1EstimateOptions};
/// # use dxt_lossless_transform_api_common::estimate::{SizeEstimationOperations, DataType};
///
/// // Define a compression estimator implementation
/// struct MyCompressionEstimator;
///
/// impl SizeEstimationOperations for MyCompressionEstimator {
///     type Error = &'static str;
///
///     fn max_compressed_size(
///         &self,
///         _len_bytes: usize,
///     ) -> Result<usize, Self::Error> {
///         Ok(0) // No buffer needed for this simple estimator
///     }
///
///     unsafe fn estimate_compressed_size(
///         &self,
///         _input_ptr: *const u8,
///         len_bytes: usize,
///         _data_type: DataType,
///         _output_ptr: *mut u8,
///         _output_len: usize,
///     ) -> Result<usize, Self::Error> {
///         Ok(len_bytes) // Your compression size estimation logic here
///     }
/// }
///
/// let bc1_data = vec![0u8; 8]; // Example BC1 block data
/// let mut output_buffer = vec![0u8; bc1_data.len()]; // Output buffer
/// let options = Bc1EstimateOptions {
///     size_estimator: MyCompressionEstimator,
///     use_all_decorrelation_modes: false, // Fast mode
/// };
///
/// // Transform with optimal settings (unsafe due to raw pointers)
/// let transform_details = unsafe {
///     transform_bc1_auto(
///         bc1_data.as_ptr(),
///         output_buffer.as_mut_ptr(),
///         bc1_data.len(),
///         options
///     )
/// }.expect("Transform failed");
///
/// // output_buffer now contains the optimally transformed data
/// ```
///
/// Your 'estimator' function needs to use the same 'concepts' as the actual final compression function.
/// For example, an LZ compressor will work well for another LZ compressor, but not for something
/// based on the Burrows-Wheeler Transform (BWT).
///
/// [See my blog post](https://sewer56.dev/blog/2025/03/11/a-program-for-helping-create-lossless-transforms.html#estimator-accuracy-vs-bzip3) for reference.
///
/// ## Optimization Strategy
///
/// Determines the best [`Bc1TransformSettings`] by brute force testing of different transformation
/// combinations and selecting the one that produces the smallest estimated compressed size.
/// The transformed data from the best option is kept in the output buffer.
///
/// ## Implementation Notes
///
/// - Index data is excluded from size estimation as it has poor compressibility
///   (entropy ≈ 7.0, minimal LZ matches) with negligible impact on results
/// - The brute force approach ensures finding the global optimum within tested parameters
/// - Memory allocation uses 64-byte alignment for optimal SIMD performance
///
/// [YCoCgVariant]: dxt_lossless_transform_common::color_565::YCoCgVariant
pub unsafe fn transform_bc1_auto<T>(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: Bc1EstimateOptions<T>,
) -> Result<Bc1TransformSettings, DetermineBestTransformError<T::Error>>
where
    T: SizeEstimationOperations,
{
    let mut best_transform_settings = Bc1TransformSettings::default();
    let mut best_size = usize::MAX;
    let mut last_tested = Bc1TransformSettings::default();

    // Pre-allocate compression buffer once for all iterations
    // Note: len/2 because we only compress color data, not indices
    let max_comp_size = transform_options
        .size_estimator
        .max_compressed_size(len / 2)
        .map_err(DetermineBestTransformError::SizeEstimationError)?;

    // Allocate compression buffer if needed (reused across all calls)
    let (comp_buffer_ptr, comp_buffer_len, _comp_buffer) = if max_comp_size == 0 {
        (core::ptr::null_mut(), 0, None)
    } else {
        let mut comp_buffer = allocate_align_64(max_comp_size)?;
        let ptr = comp_buffer.as_mut_ptr();
        (ptr, max_comp_size, Some(comp_buffer))
    };

    // Test transforms in order of decreasing probability, with most common (YCoCg1/Split) last
    // This minimizes redundant final transforms since YCoCg1/Split is optimal ~71% of the time
    let test_order = if transform_options.use_all_decorrelation_modes {
        COMPREHENSIVE_TEST_ORDER
    } else {
        FAST_TEST_ORDER
    };

    for &(decorrelation_mode, split_colours) in test_order {
        // Get the current mode we're testing.
        let current_mode = Bc1TransformSettings {
            decorrelation_mode,
            split_colour_endpoints: split_colours,
        };

        // Apply a full transformation (~24GB/s on 1 thread, Ryzen 9950X3D)
        transform_bc1_with_settings(input_ptr, output_ptr, len, current_mode);
        last_tested = current_mode;

        // Note(sewer): The indices are very poorly compressible (entropy == ~7.0 , no lz matches).
        // Excluding them from the estimation has negligible effect on results, with a doubling of
        // speed.

        // Test the current mode by measuring the compressed size using the trait
        let data_type = current_mode.to_data_type();

        let result_size = transform_options
            .size_estimator
            .estimate_compressed_size(
                output_ptr,
                len / 2,
                data_type,
                comp_buffer_ptr,
                comp_buffer_len,
            )
            .map_err(DetermineBestTransformError::SizeEstimationError)?;
        if result_size < best_size {
            best_size = result_size;
            best_transform_settings = current_mode;
        }
    }

    // If the best option wasn't the last one tested, we need to transform again
    if best_transform_settings != last_tested {
        // Transform the data one final time with the best settings
        transform_bc1_with_settings(input_ptr, output_ptr, len, best_transform_settings);
    }

    Ok(best_transform_settings)
}
