//! BC3 Transform Optimization
//!
//! This module provides optimization functionality to determine the best
//! transformation parameters for BC3 data compression.

use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_common::allocate::{allocate_align_64, AllocateError};
use thiserror::Error;

use super::settings::{Bc3TransformSettings, COMPREHENSIVE_TEST_ORDER, FAST_TEST_ORDER};
use super::transform_with_settings::transform_bc3_with_settings;

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

/// The settings for [`transform_bc3_auto`], regarding how the estimation is done,
/// and other related factors.
pub struct Bc3EstimateSettings<T>
where
    T: SizeEstimationOperations,
{
    /// A trait-based size estimator used to find the best possible transform by testing
    /// different configurations and choosing the one that results in the smallest estimated
    /// compressed size.
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
    /// maximize speed of [`transform_bc3_auto`], and to improve decompression speed
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
    /// time (tests 16 combinations instead of 8).
    ///
    /// **Note**: The typical improvement from testing all decorrelation modes is <0.1% in practice.
    /// For better compression gains, it's recommended to use a compression level on the
    /// estimator (e.g., ZStandard estimator) closer to your final compression level instead.
    ///
    /// [YCoCgVariant::Variant1]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1
    /// [YCoCgVariant::Variant2]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2
    /// [YCoCgVariant::Variant3]: dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3
    /// [YCoCgVariant::None]: dxt_lossless_transform_common::color_565::YCoCgVariant::None
    pub use_all_decorrelation_modes: bool,
}

/// Transform BC3 data using the best determined settings.
///
/// This function tests various transform configurations and applies the one that
/// produces the smallest compressed size according to the provided estimator.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC3 blocks)
/// - `output_ptr`: A pointer to the output buffer where transformed data will be written
/// - `len`: The length of the input data in bytes
/// - `transform_options`: Settings for the estimation including the file size estimator
///
/// # Returns
///
/// The [`Bc3TransformSettings`] that produced the best (smallest) compressed size.
///
/// # Remarks
///
/// This function combines the functionality of determining the best transform options
/// and actually transforming the data, eliminating the need for two separate calls.
/// The output buffer will contain the transformed data using the optimal settings.
///
/// This function is a brute force approach that tests all standard transform options:
/// - 1/8th of the compression speed in fast mode (2 [YCoCgVariant] * 2 (split_alphas) * 2 (split_colours))
/// - 1/16th of the compression speed in comprehensive mode (4 [YCoCgVariant] * 2 (split_alphas) * 2 (split_colours))
///
/// ## Performance Characteristics
///
/// Overall throughput depends on the estimator used:
/// - **LTU estimator**: ~678 MiB/s (fast, ok accuracy)
/// - **ZStandard level 1 estimator**: ~177 MiB/s (slower, higher accuracy)
///
/// The transformation itself runs at ~24GB/s, so the estimator becomes the bottleneck.
///
/// ## Performance Optimization
///
/// The transform options are tested in order of decreasing probability of being optimal,
/// based on [TODO: BC3-specific analysis]:
///
/// Since YCoCg1 with both splitting options is expected to be optimal for the majority of textures,
/// testing it last means we avoid a final redundant transform in the majority of cases.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `output_ptr` must be valid for writes of `len` bytes
/// - `len` must be divisible by 16
/// - It is recommended that `input_ptr` and `output_ptr` are at least 16-byte aligned (recommended 32-byte align)
///
/// # Examples
///
/// ```rust,no_run
/// # use dxt_lossless_transform_bc3::{transform_bc3_auto, Bc3EstimateSettings};
/// # use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
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
///         _output_ptr: *mut u8,
///         _output_len: usize,
///     ) -> Result<usize, Self::Error> {
///         Ok(len_bytes) // Your compression size estimation logic here
///     }
/// }
///
/// let bc3_data = vec![0u8; 16]; // Example BC3 block data
/// let mut output_buffer = vec![0u8; bc3_data.len()]; // Output buffer
/// let options = Bc3EstimateSettings {
///     size_estimator: MyCompressionEstimator,
///     use_all_decorrelation_modes: false, // Fast mode
/// };
///
/// // Transform with optimal settings (unsafe due to raw pointers)
/// let transform_details = unsafe {
///     transform_bc3_auto(
///         bc3_data.as_ptr(),
///         output_buffer.as_mut_ptr(),
///         bc3_data.len(),
///         &options
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
/// ## Optimization Strategy
///
/// Determines the best [`Bc3TransformSettings`] by brute force testing of different transformation
/// combinations and selecting the one that produces the smallest estimated compressed size.
/// The transformed data from the best option is kept in the output buffer.
///
/// ## Implementation Notes
///
/// - Indices data (alpha and color) are excluded from size estimation as it has poor compressibility
///   (entropy ≈ 7.0, minimal LZ matches) with negligible impact on results
/// - The brute force approach ensures finding the global optimum within tested parameters
/// - Memory allocation uses 64-byte alignment for optimal SIMD performance
///
/// [YCoCgVariant]: dxt_lossless_transform_common::color_565::YCoCgVariant
pub unsafe fn transform_bc3_auto<T>(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: &Bc3EstimateSettings<T>,
) -> Result<Bc3TransformSettings, DetermineBestTransformError<T::Error>>
where
    T: SizeEstimationOperations,
{
    let mut best_transform_settings = Bc3TransformSettings::default();
    let mut best_size = usize::MAX;
    let mut last_tested = Bc3TransformSettings::default();

    // Pre-allocate compression buffer once for all iterations
    // Note: len*1/4 for largest estimation call (color endpoints, larger than alpha endpoints at len*1/8)
    // We estimate alpha endpoints and color endpoints separately, so buffer size is based on the larger call
    let max_comp_size = transform_options
        .size_estimator
        .max_compressed_size(len / 4)
        .map_err(DetermineBestTransformError::SizeEstimationError)?;

    // Allocate compression buffer if needed (reused across all calls)
    let (comp_buffer_ptr, comp_buffer_len, _comp_buffer) = if max_comp_size == 0 {
        (core::ptr::null_mut(), 0, None)
    } else {
        let mut comp_buffer = allocate_align_64(max_comp_size)?;
        let ptr = comp_buffer.as_mut_ptr();
        (ptr, max_comp_size, Some(comp_buffer))
    };

    // Test transforms in order of decreasing probability, with most common last
    // This minimizes redundant final transforms since the optimal option is tested last
    let test_order = if transform_options.use_all_decorrelation_modes {
        COMPREHENSIVE_TEST_ORDER
    } else {
        FAST_TEST_ORDER
    };

    for &(decorrelation_mode, split_alphas, split_colours) in test_order {
        // Get the current mode we're testing.
        let current_mode = Bc3TransformSettings {
            decorrelation_mode,
            split_alpha_endpoints: split_alphas,
            split_colour_endpoints: split_colours,
        };

        // Apply a full transformation (~24GB/s on 1 thread, Ryzen 9950X3D)
        transform_bc3_with_settings(input_ptr, output_ptr, len, current_mode);
        last_tested = current_mode;

        // Note: The indices are very poorly compressible (entropy == ~7.0, no lz matches).
        // Excluding them from the estimation has negligible effect on results, with a significant
        // speed improvement.

        // Test the current mode by measuring the compressed size using the trait
        // For BC3, we estimate alpha endpoints and color endpoints only
        // We skip all indices (6 alpha + 4 color indices per 16-byte block) due to poor compressibility
        let num_blocks = len / 16;
        let alpha_endpoints_size = num_blocks * 2; // 2 bytes per block
        let color_endpoints_size = num_blocks * 4; // 4 bytes color endpoints only per block

        // Estimate alpha endpoints
        let alpha_result_size = transform_options
            .size_estimator
            .estimate_compressed_size(
                output_ptr, // alpha endpoints at start
                alpha_endpoints_size,
                comp_buffer_ptr,
                comp_buffer_len,
            )
            .map_err(DetermineBestTransformError::SizeEstimationError)?;

        // Estimate color endpoints only
        let color_result_size = transform_options
            .size_estimator
            .estimate_compressed_size(
                output_ptr.add(len / 2), // skip alpha data, proceed to color data
                color_endpoints_size,
                comp_buffer_ptr,
                comp_buffer_len,
            )
            .map_err(DetermineBestTransformError::SizeEstimationError)?;

        let total_result_size = alpha_result_size + color_result_size;

        if total_result_size < best_size {
            best_size = total_result_size;
            best_transform_settings = current_mode;
        }
    }

    // If the best option wasn't the last one tested, we need to transform again
    if best_transform_settings != last_tested {
        // Transform the data one final time with the best settings
        transform_bc3_with_settings(input_ptr, output_ptr, len, best_transform_settings);
    }

    Ok(best_transform_settings)
}
