//! Functions for determining the best transform options with experimental normalization support.

use crate::{
    determine_optimal_transform::DetermineBestTransformError, transforms::standard::transform,
    YCoCgVariant,
};
use core::mem::size_of;
use core::slice;
use dxt_lossless_transform_common::allocate::FixedRawAllocArray;
use dxt_lossless_transform_common::{
    color_565::Color565, transforms::split_565_color_endpoints::split_color_endpoints,
};

use super::{
    normalize_blocks_all_modes, Bc1TransformDetailsWithNormalization, ColorNormalizationMode,
};

/// The options for [`determine_best_transform_details_with_normalization`], regarding how the estimation is done,
/// and other related factors.
pub struct Bc1EstimateOptionsWithNormalization<F>
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
    /// maximize speed of [`determine_best_transform_details_with_normalization`], and to improve decompression speed
    /// by reducing the size of the sliding window (so more data in cache) and increasing minimum
    /// match length.
    pub file_size_estimator: F,

    /// Whether to test all normalization options or skip them for faster processing.
    ///
    /// When `true`, all [`ColorNormalizationMode`] variants will be tested.
    /// When `false`, only [`ColorNormalizationMode::None`] will be used, significantly
    /// improving performance at the cost of potentially less optimal compression.
    ///
    /// This is off by default for the time being. In the future, we'll have a better 'normalize'
    /// function, where brute forcing will not be necessary.
    pub test_normalize_options: bool,
}

/// Determine the best transform details with full normalization testing.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `transform_options`: Options for the estimation including the file size estimator and normalization settings
///
/// # Returns
///
/// The best (smallest size) format for the given data.
///
/// # Remarks
///
/// This function tests all normalization options, the characteristics of this function are:
///
/// - 1/24th of the compression speed ([`ColorNormalizationMode`] * [`YCoCgVariant`] * 2 (split_colours))
/// - Uses 6x the memory of input size
///
/// # Safety
///
/// Function is unsafe because it deals with raw pointers which must be correct.
pub unsafe fn determine_best_transform_details_with_normalization<F>(
    input_ptr: *const u8,
    len: usize,
    transform_options: Bc1EstimateOptionsWithNormalization<F>,
) -> Result<Bc1TransformDetailsWithNormalization, DetermineBestTransformError>
where
    F: Fn(*const u8, usize) -> usize,
{
    const NUM_NORMALIZE: usize = ColorNormalizationMode::all_values().len();
    let mut normalize_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let mut split_blocks_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let normalize_buffers_ptrs = normalize_buffers.get_pointer_slice();
    let split_blocks_buffers_ptrs = split_blocks_buffers.get_pointer_slice();

    // Normalize blocks into all possible modes.
    let any_normalized = normalize_blocks_all_modes(input_ptr, &normalize_buffers_ptrs, len);

    // Now we got all blocks normalized and split, and have to test all the different possibilities.
    // We can repurpose the normalize_buffers
    let mut best_transform_details = Bc1TransformDetailsWithNormalization::default();
    let mut best_size = usize::MAX;

    // split_blocks_buffers_ptrs: buffer_a
    // result_pointers: buffer_b (output)
    let result_pointers = normalize_buffers.get_pointer_slice();
    if any_normalized {
        // At least 1 block was normalized, so we have to test all options.
        // Now split all blocks.
        for x in 0..NUM_NORMALIZE {
            transform(normalize_buffers_ptrs[x], split_blocks_buffers_ptrs[x], len);
        }

        for norm_idx in 0..NUM_NORMALIZE {
            for decorrelation_mode in YCoCgVariant::all_values() {
                for split_colours in [true, false] {
                    // Get the current mode we're testing.
                    let current_mode = Bc1TransformDetailsWithNormalization {
                        color_normalization_mode: ColorNormalizationMode::all_values()[norm_idx],
                        decorrelation_mode: *decorrelation_mode,
                        split_colour_endpoints: split_colours,
                    };

                    // Get input/output buffers.
                    let input = split_blocks_buffers_ptrs[norm_idx];
                    let output = result_pointers[norm_idx];

                    test_normalize_variant_with_normalization(
                        input,
                        output,
                        len,
                        &transform_options,
                        &mut best_transform_details,
                        &mut best_size,
                        current_mode,
                    );
                }
            }
        }

        Ok(best_transform_details)
    } else {
        // No blocks were normalized, we can skip testing normalize steps
        // Since no normalization occurred, we can use the original input directly after splitting
        transform(input_ptr, split_blocks_buffers_ptrs[0], len);

        for decorrelation_mode in YCoCgVariant::all_values() {
            for split_colours in [true, false] {
                // Get the current mode we're testing.
                let current_mode = Bc1TransformDetailsWithNormalization {
                    color_normalization_mode: ColorNormalizationMode::None, // Skip normalization step
                    decorrelation_mode: *decorrelation_mode,
                    split_colour_endpoints: split_colours,
                };

                // Get input/output buffers.
                let input = split_blocks_buffers_ptrs[0]; // Use first buffer since no normalization variants
                let output = result_pointers[0];

                test_normalize_variant_with_normalization(
                    input,
                    output,
                    len,
                    &transform_options,
                    &mut best_transform_details,
                    &mut best_size,
                    current_mode,
                );
            }
        }

        Ok(best_transform_details)
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
unsafe fn test_normalize_variant_with_normalization<F>(
    input: *mut u8,
    output: *mut u8,
    len: usize,
    transform_options: &Bc1EstimateOptionsWithNormalization<F>,
    best_transform_details: &mut Bc1TransformDetailsWithNormalization,
    best_size: &mut usize,
    current_mode: Bc1TransformDetailsWithNormalization,
) where
    F: Fn(*const u8, usize) -> usize,
{
    // So this is the fun part.
    if current_mode.split_colour_endpoints {
        // Split colour endpoints, then decorrelate in-place.
        // ..
        // Colours represent first half of the data, before indices.
        split_color_endpoints(
            input as *const Color565,
            output as *mut Color565,
            len / 2, // (len / 2): Length of colour endpoints in bytes
        );
        let colors_in_arr = slice::from_raw_parts(
            output as *const Color565, // Using output as both source and destination as data was already copied there
            (len / 2) / size_of::<Color565>(),
        );
        let colors_out_arr =
            slice::from_raw_parts_mut(output as *mut Color565, (len / 2) / size_of::<Color565>());
        Color565::decorrelate_ycocg_r_slice(
            colors_in_arr,
            colors_out_arr,
            current_mode.decorrelation_mode,
        );
    } else {
        // Decorrelate directly into the target buffer.
        let colors_in_arr =
            slice::from_raw_parts(input as *const Color565, len / 2 / size_of::<Color565>());
        let colors_out_arr =
            slice::from_raw_parts_mut(output as *mut Color565, len / 2 / size_of::<Color565>());
        Color565::decorrelate_ycocg_r_slice(
            colors_in_arr,
            colors_out_arr,
            current_mode.decorrelation_mode,
        );
    }

    // Note(sewer): The indices are very poorly compressible (entropy == ~7.0 , no lz matches).
    // Excluding them from the estimation has negligible effect on results, with a doubling of
    // speed. If you want to include them, uncomment the code below, and change len / 2 to len in the
    // `result_size` calculation.

    // Now copy the indices verbatim.
    // let indices_in_arr = slice::from_raw_parts(input.add(len / 2), len / 2);
    // let indices_out_arr = slice::from_raw_parts_mut(output.add(len / 2), len / 2);
    // indices_out_arr.copy_from_slice(indices_in_arr);

    // Test the current mode.
    let result_size = (transform_options.file_size_estimator)(output, len / 2);
    if result_size < *best_size {
        *best_size = result_size;
        *best_transform_details = current_mode;
    }
}
