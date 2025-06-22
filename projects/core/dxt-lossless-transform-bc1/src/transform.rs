//! BC1 Transform Operations
//!
//! This module provides the core transformation and optimization functionality for BC1
//! (DXT1) compressed texture data to achieve optimal compression ratios.
//!
//! ## Overview
//!
//! BC1 compression can be further optimized by applying various transformations before
//! final compression. This module provides both manual transform operations and automatic
//! optimization to determine the best transformation parameters.
//!
//! ## Performance Characteristics
//!
//! The optimization functions in this module perform brute force testing of different transformations:
//!
//! 1. Transform the data into a specific format.
//! 2. Estimate the compressed size using a provided file size estimator function.
//! 3. Compare the estimated sizes to find the best transformation.
//!
//! The typical throughput for the transformation is ~24GB/s transformation speed on single thread (Ryzen 9950X3D),
//! so in practice the speed depends on how fast the estimator function can run. When ran with `zstandard -1`,
//! the speed is ~265MiB/s for the estimator function. With `lossless-transform-utils` it is 641.30MiB/s
//!
//! For memory, the usage is the same as the size of the input data plus the compression buffer needed
//! by the estimator.

use crate::transforms::{
    standard::{transform, untransform},
    with_recorrelate, with_split_colour, with_split_colour_and_recorr,
};
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};
use dxt_lossless_transform_common::{
    allocate::{allocate_align_64, AllocateError},
    color_565::YCoCgVariant,
};
use thiserror::Error;

/// The information about the BC1 transform that was just performed.
/// Each item transformed via [`transform_bc1_with_settings`] will produce an instance of this struct.
/// To undo the transform, you'll need to pass [`Bc1DetransformSettings`] to [`untransform_bc1_with_settings`],
/// which can be obtained from this struct using the `into` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1TransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

/// Settings required to detransform BC1 data.
///
/// This struct contains only the information needed to reverse the transform operation.
/// Note that color normalization is a preprocessing step that doesn't need to be reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bc1DetransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,

    /// Whether or not the colour endpoints are to be split or not.
    pub split_colour_endpoints: bool,
}

impl From<Bc1TransformSettings> for Bc1DetransformSettings {
    fn from(transform_settings: Bc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: transform_settings.decorrelation_mode,
            split_colour_endpoints: transform_settings.split_colour_endpoints,
        }
    }
}

impl Default for Bc1DetransformSettings {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Default for Bc1TransformSettings {
    fn default() -> Self {
        // Best (on average) results, but of course not perfect, as is with brute-force method.
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Bc1TransformSettings {
    /// Returns an iterator over all possible combinations of [`Bc1TransformSettings`] values.
    ///
    /// This function generates all possible combinations by iterating through:
    /// - All [`YCoCgVariant`] variants  
    /// - Both `true` and `false` values for `split_colour_endpoints`
    ///
    /// The total number of combinations is:
    /// [`YCoCgVariant`] variants × 2 bool values
    ///
    /// # Examples
    ///
    /// ```
    /// use dxt_lossless_transform_bc1::transform::Bc1TransformSettings;
    ///
    /// let all_combinations: Vec<_> = Bc1TransformSettings::all_combinations().collect();
    /// println!("Total combinations: {}", all_combinations.len());
    ///
    /// for settings in Bc1TransformSettings::all_combinations() {
    ///     println!("{:?}", settings);
    /// }
    /// ```
    #[cfg(not(tarpaulin_include))]
    pub fn all_combinations() -> impl Iterator<Item = Bc1TransformSettings> {
        YCoCgVariant::all_values().iter().flat_map(|decorr_mode| {
            [true, false]
                .into_iter()
                .map(move |split_endpoints| Bc1TransformSettings {
                    decorrelation_mode: *decorr_mode,
                    split_colour_endpoints: split_endpoints,
                })
        })
    }

    /// Determines the appropriate [`DataType`] for size estimation based on the transform options.
    ///
    /// This method maps the transform configuration to the corresponding data type that
    /// should be used for compression size estimation and caching.
    ///
    /// # Returns
    /// The [`DataType`] that represents the data format after applying these transform options
    pub fn to_data_type(&self) -> DataType {
        match (self.decorrelation_mode, self.split_colour_endpoints) {
            (YCoCgVariant::None, false) => DataType::Bc1Colours,
            (YCoCgVariant::None, true) => DataType::Bc1SplitColours,
            (_, true) => DataType::Bc1SplitDecorrelatedColours, // Split colours with decorrelation
            (_, false) => DataType::Bc1DecorrelatedColours,     // Decorrelated but not split
        }
    }
}

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`transform_with_best_options`] or
///   [`Bc1TransformSettings::default`] for less optimal result(s).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: Bc1TransformSettings,
) {
    debug_assert!(len % 8 == 0);

    let has_split_colours = transform_options.split_colour_endpoints;

    if has_split_colours {
        if transform_options.decorrelation_mode == YCoCgVariant::None {
            with_split_colour::transform_with_split_colour(
                input_ptr,
                output_ptr as *mut u16,              // color0 values
                output_ptr.add(len / 4) as *mut u16, // color1 values
                output_ptr.add(len / 2) as *mut u32, // indices in last half
                len / 8,                             // number of blocks (8 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::transform_with_split_colour_and_recorr(
                input_ptr,
                output_ptr as *mut u16,              // color0 values
                output_ptr.add(len / 4) as *mut u16, // color1 values
                output_ptr.add(len / 2) as *mut u32, // indices in last half
                len / 8,                             // number of blocks (8 bytes per block)
                transform_options.decorrelation_mode,
            );
        }
    } else if transform_options.decorrelation_mode == YCoCgVariant::None {
        // Standard transform – no split-colour and no decorrelation.
        transform(input_ptr, output_ptr, len);
    } else {
        // Standard transform + decorrelate.
        with_recorrelate::transform_with_decorrelate(
            input_ptr,
            output_ptr,
            len,
            transform_options.decorrelation_mode,
        );
    }
}

/// Untransform BC1 file back to its original format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks).
///   Output from [`transform_bc1_with_settings`].
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `detransform_options`: A struct containing information about the transform that was originally performed.
///   Must match the settings used in [`transform_bc1_with_settings`] function (excluding color normalization).
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn untransform_bc1_with_settings(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    detransform_options: Bc1DetransformSettings,
) {
    debug_assert!(len % 8 == 0);

    let has_split_colours = detransform_options.split_colour_endpoints;

    if has_split_colours {
        if detransform_options.decorrelation_mode == YCoCgVariant::None {
            // Optimized single-pass operation: unsplit split colors and combine with indices
            // directly into BC1 blocks, avoiding intermediate memory copies
            with_split_colour::untransform_with_split_colour(
                input_ptr as *const u16,              // color0 values
                input_ptr.add(len / 4) as *const u16, // color1 values
                input_ptr.add(len / 2) as *const u32, // indices
                output_ptr,                           // output BC1 blocks
                len / 8,                              // number of blocks (8 bytes per block)
            );
        } else {
            with_split_colour_and_recorr::untransform_with_split_colour_and_recorr(
                input_ptr as *const u16,              // color0 values
                input_ptr.add(len / 4) as *const u16, // color1 values
                input_ptr.add(len / 2) as *const u32, // indices
                output_ptr,                           // output BC1 blocks
                len / 8,                              // number of blocks (8 bytes per block)
                detransform_options.decorrelation_mode,
            );
        }
    } else if detransform_options.decorrelation_mode == YCoCgVariant::None {
        // Standard transform – no split-colour and no decorrelation.
        untransform(input_ptr, output_ptr, len);
    } else {
        // Standard transform + recorrelate.
        with_recorrelate::untransform_with_recorrelate(
            input_ptr,
            output_ptr,
            len,
            detransform_options.decorrelation_mode,
        );
    }
}

/// Test order for fast mode optimization (tests only common combinations)
pub(crate) static FAST_TEST_ORDER: &[(YCoCgVariant, bool)] = &[
    (YCoCgVariant::None, false),     // None/NoSplit
    (YCoCgVariant::None, true),      // None/Split
    (YCoCgVariant::Variant1, false), // YCoCg1/NoSplit (17.9%)
    (YCoCgVariant::Variant1, true),  // YCoCg1/Split (71.1%) - most common, test last
];

/// Test order for comprehensive mode optimization (tests all combinations)
pub(crate) static COMPREHENSIVE_TEST_ORDER: &[(YCoCgVariant, bool)] = &[
    (YCoCgVariant::Variant2, false), // YCoCg2/NoSplit (0.9%)
    (YCoCgVariant::None, false),     // None/NoSplit (1.0%)
    (YCoCgVariant::None, true),      // None/Split (1.1%)
    (YCoCgVariant::Variant3, false), // YCoCg3/NoSplit (1.9%)
    (YCoCgVariant::Variant3, true),  // YCoCg3/Split (2.7%)
    (YCoCgVariant::Variant2, true),  // YCoCg2/Split (3.5%)
    (YCoCgVariant::Variant1, false), // YCoCg1/NoSplit (17.9%)
    (YCoCgVariant::Variant1, true),  // YCoCg1/Split (71.1%) - most common, test last
];

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

/// The options for [`transform_with_best_options`], regarding how the estimation is done,
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
    /// maximize speed of [`transform_with_best_options`], and to improve decompression speed
    /// by reducing the size of the sliding window (so more data in cache) and increasing minimum
    /// match length.
    pub size_estimator: T,

    /// Controls which decorrelation modes are tested during optimization.
    ///
    /// When `false` (default), only tests [`YCoCgVariant::Variant1`] and [`YCoCgVariant::None`]
    /// for faster optimization with good results.
    ///
    /// When `true`, tests all available decorrelation modes ([`YCoCgVariant::Variant1`],
    /// [`YCoCgVariant::Variant2`], [`YCoCgVariant::Variant3`], and [`YCoCgVariant::None`])
    /// for potentially better compression at the cost of twice as long optimization
    /// time (tests 4 options instead of 2) for negligible gains (typically <0.1% extra savings).
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
/// - 1/4th of the compression speed in fast mode (2 [`YCoCgVariant`] * 2 (split_colours))
/// - 1/8th of the compression speed in comprehensive mode (4 [`YCoCgVariant`] * 2 (split_colours))
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
/// # use dxt_lossless_transform_bc1::transform::{transform_with_best_options, Bc1EstimateOptions};
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
///     transform_with_best_options(
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
pub unsafe fn transform_with_best_options<T>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;

    /// Test that transform_with_best_options works correctly
    #[rstest]
    fn test_transform_with_best_options() {
        // Create minimal BC1 block data (8 bytes per block)
        // This is a simple red block
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output_buffer = [0u8; 8];

        // Create a simple dummy estimator
        struct DummyEstimator;

        impl SizeEstimationOperations for DummyEstimator {
            type Error = &'static str;

            fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
                Ok(0) // No buffer needed for dummy estimator
            }

            unsafe fn estimate_compressed_size(
                &self,
                _input_ptr: *const u8,
                len_bytes: usize,
                _data_type: DataType,
                _output_ptr: *mut u8,
                _output_len: usize,
            ) -> Result<usize, Self::Error> {
                Ok(len_bytes) // Just return the input length
            }
        }

        let transform_options = Bc1EstimateOptions {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        // This should not crash and should produce transformed data
        let result = unsafe {
            transform_with_best_options(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
                bc1_data.len(),
                transform_options,
            )
        };

        assert!(
            result.is_ok(),
            "Function should not crash with valid BC1 data"
        );
    }

    /// Test that transform_with_best_options handles estimation errors properly
    #[rstest]
    fn test_transform_with_best_options_handles_errors() {
        // Create minimal BC1 block data
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output_buffer = [0u8; 8];

        // Create an estimator that always fails
        struct FailingEstimator;

        impl SizeEstimationOperations for FailingEstimator {
            type Error = &'static str;

            fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
                Err("Estimation failed")
            }

            unsafe fn estimate_compressed_size(
                &self,
                _input_ptr: *const u8,
                _len_bytes: usize,
                _data_type: DataType,
                _output_ptr: *mut u8,
                _output_len: usize,
            ) -> Result<usize, Self::Error> {
                Err("Estimation failed")
            }
        }

        let transform_options = Bc1EstimateOptions {
            size_estimator: FailingEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = unsafe {
            transform_with_best_options(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
                bc1_data.len(),
                transform_options,
            )
        };

        // Should return an error
        assert!(
            result.is_err(),
            "Function should return error when estimator fails"
        );

        // Check that it's specifically a SizeEstimationError
        if let Err(e) = result {
            match e {
                DetermineBestTransformError::SizeEstimationError(_) => {
                    // This is what we expect
                }
                _ => panic!("Expected SizeEstimationError, got {e:?}"),
            }
        }
    }
}
