use core::slice;

use dxt_lossless_transform_common::{
    allocate::{AllocateError, FixedRawAllocArray},
    color_565::{Color565, YCoCgVariant},
    transforms::split_565_color_endpoints::split_color_endpoints,
};
use thiserror::Error;

use crate::{
    normalize_blocks::{normalize_blocks_all_modes, ColorNormalizationMode},
    split_blocks::split_blocks,
    Bc1TransformDetails,
};

/// The options for [`determine_best_transform_details`], regarding how the estimation is done,
/// and other related factors.
pub struct Bc1TransformOptions {
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
    pub file_size_estimator: Box<dyn Fn(*const u8, usize) -> usize>,
}

/// Determine the best transform details for the given BC1 blocks.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `len`: The length of the input data in bytes
///
/// # Returns
///
/// The best (smallest size) format for the given data.
///
/// # Remarks
///
/// This function is a brute force, the characteristics of this function are:
///
/// - 1/24th of the compression speed ([`ColorNormalizationMode`] * [`YCoCgVariant`] * 2 (split_colours))
/// - Uses 6x the memory of input size
///
/// # Safety
///
/// Function is unsafe because it deals with raw pointers which must be correct.
pub unsafe fn determine_best_transform_details(
    input_ptr: *const u8,
    len: usize,
    transform_options: Bc1TransformOptions,
) -> Result<Bc1TransformDetails, DetermineBestTransformError> {
    // TODO: Write a 'fast' variant of this, which basically means defaulting to a single normalize
    //       as we can't test them all.

    const NUM_NORMALIZE: usize = ColorNormalizationMode::all_values().len();
    let mut normalize_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let mut split_blocks_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let normalize_buffers_ptrs = normalize_buffers.get_pointer_slice();
    let split_blocks_buffers_ptrs = split_blocks_buffers.get_pointer_slice();

    // Normalize blocks into all possible modes.
    normalize_blocks_all_modes(input_ptr, &normalize_buffers_ptrs, len);

    // Now split all blocks.
    for x in 0..NUM_NORMALIZE {
        split_blocks(normalize_buffers_ptrs[x], split_blocks_buffers_ptrs[x], len);
    }

    // Now we got all blocks normalized and split, and have to test all the different possibilities.
    // We can repurpose the normalize_buffers
    let mut best_transform_details = Bc1TransformDetails::default();
    let mut best_size = usize::MAX;

    // split_blocks_buffers_ptrs: buffer_a
    // result_pointers: buffer_b (output)
    let result_pointers = normalize_buffers.get_pointer_slice();
    for norm_idx in 0..NUM_NORMALIZE {
        for decorrelation_mode in YCoCgVariant::all_values() {
            for split_colours in [true, false] {
                // Get the current mode we're testing.
                let current_mode = Bc1TransformDetails {
                    color_normalization_mode: ColorNormalizationMode::all_values()[norm_idx],
                    decorrelation_mode: *decorrelation_mode,
                    split_colour_endpoints: split_colours,
                };

                // Get input/output buffers.
                let input = split_blocks_buffers_ptrs[norm_idx];
                let output = result_pointers[norm_idx];

                // So this is the fun part.
                if split_colours {
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
                    let colors_out_arr = slice::from_raw_parts_mut(
                        output as *mut Color565,
                        (len / 2) / size_of::<Color565>(),
                    );
                    Color565::decorrelate_ycocg_r_slice(
                        colors_in_arr,
                        colors_out_arr,
                        *decorrelation_mode,
                    );
                } else {
                    // Decorrelate directly into the target buffer.
                    let colors_in_arr = slice::from_raw_parts(
                        input as *const Color565,
                        len / 2 / size_of::<Color565>(),
                    );
                    let colors_out_arr = slice::from_raw_parts_mut(
                        output as *mut Color565,
                        len / 2 / size_of::<Color565>(),
                    );
                    Color565::decorrelate_ycocg_r_slice(
                        colors_in_arr,
                        colors_out_arr,
                        *decorrelation_mode,
                    );
                }

                // Now copy the indices verbatim.
                let indices_in_arr = slice::from_raw_parts(input.add(len / 2), len / 2);
                let indices_out_arr = slice::from_raw_parts_mut(output.add(len / 2), len / 2);
                indices_out_arr.copy_from_slice(indices_in_arr);

                // Test the current mode.
                let result_size = (transform_options.file_size_estimator)(output, len);
                if result_size < best_size {
                    best_size = result_size;
                    best_transform_details = current_mode;
                }
            }
        }
    }

    Ok(best_transform_details)
}

/// An error that happened in memory allocation within the library.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DetermineBestTransformError {
    #[error(transparent)]
    AllocateError(#[from] AllocateError),
}
