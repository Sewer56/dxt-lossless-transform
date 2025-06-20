//! Functions for determining the best transform options with experimental normalization support.

use crate::determine_optimal_transform::{
    determine_best_transform_details, Bc1EstimateOptions, DetermineBestTransformError,
};
use crate::{transforms::standard::transform, Bc1TransformDetails, YCoCgVariant};
use core::mem::size_of;
use core::ptr::null_mut;
use core::slice;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_common::allocate::{allocate_align_64, FixedRawAllocArray};
use dxt_lossless_transform_common::{
    color_565::Color565, transforms::split_565_color_endpoints::split_color_endpoints,
};

use super::{
    normalize_blocks_all_modes, Bc1TransformDetailsWithNormalization, ColorNormalizationMode,
};

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
/// - Uses 3x the memory of input size when no normalization is needed, 6x when normalization is required
///
/// # Safety
///
/// Function is unsafe because it deals with raw pointers which must be correct.
pub unsafe fn determine_best_transform_details_with_normalization<T>(
    input_ptr: *const u8,
    len: usize,
    transform_options: Bc1EstimateOptions<T>,
) -> Result<Bc1TransformDetailsWithNormalization, DetermineBestTransformError<T::Error>>
where
    T: SizeEstimationOperations,
{
    const NUM_NORMALIZE: usize = ColorNormalizationMode::all_values().len();
    let mut normalize_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
    let normalize_buffers_ptrs = normalize_buffers.get_pointer_slice();

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

    // Normalize blocks into all possible modes.
    let any_normalized = normalize_blocks_all_modes(input_ptr, &normalize_buffers_ptrs, len);

    // Now we got all blocks normalized and split, and have to test all the different possibilities.
    // We can repurpose the normalize_buffers
    let mut best_transform_details = Bc1TransformDetailsWithNormalization::default();
    let mut best_size = usize::MAX;

    if any_normalized {
        // At least 1 block was normalized, so we have to test all options.
        // Allocate split blocks buffers only when needed
        let mut split_blocks_buffers = FixedRawAllocArray::<NUM_NORMALIZE>::new(len)?;
        let split_blocks_buffers_ptrs = split_blocks_buffers.get_pointer_slice();

        // split_blocks_buffers_ptrs: buffer_a
        // result_pointers: buffer_b (output)
        let result_pointers = normalize_buffers.get_pointer_slice();

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
                        comp_buffer_ptr,
                        comp_buffer_len,
                    );
                }
            }
        }

        Ok(best_transform_details)
    } else {
        // No blocks were normalized, we can skip testing normalize steps
        // Drop buffers first for memory usage
        drop(normalize_buffers);

        // Call the regular function since no normalization is needed
        let regular_result =
            determine_best_transform_details(input_ptr, len, null_mut(), transform_options)?;

        // Convert regular result to normalization result using From trait
        Ok(regular_result.into())
    }
}

#[allow(clippy::too_many_arguments)]
#[inline]
unsafe fn test_normalize_variant_with_normalization<T>(
    input: *mut u8,
    output: *mut u8,
    len: usize,
    transform_options: &Bc1EstimateOptions<T>,
    best_transform_details: &mut Bc1TransformDetailsWithNormalization,
    best_size: &mut usize,
    current_mode: Bc1TransformDetailsWithNormalization,
    comp_buffer_ptr: *mut u8,
    comp_buffer_len: usize,
) where
    T: SizeEstimationOperations,
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
    let transform_details = Bc1TransformDetails {
        decorrelation_mode: current_mode.decorrelation_mode,
        split_colour_endpoints: current_mode.split_colour_endpoints,
    };

    let result_size = match transform_options.size_estimator.estimate_compressed_size(
        output,
        len / 2,
        transform_details.to_data_type(),
        comp_buffer_ptr,
        comp_buffer_len,
    ) {
        Ok(size) => size,
        Err(_) => return, // Skip this variant if estimation fails
    };

    if result_size < *best_size {
        *best_size = result_size;
        *best_transform_details = current_mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::estimate::DataType;

    /// Test that determine_best_transform_details_with_normalization doesn't crash with minimal BC1 data
    #[test]
    fn determine_best_transform_details_with_normalization_does_not_crash() {
        // Create minimal BC1 block data (8 bytes per block)
        // This is a simple red block
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];

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

        // This should not crash
        let result = unsafe {
            determine_best_transform_details_with_normalization(
                bc1_data.as_ptr(),
                bc1_data.len(),
                transform_options,
            )
        };

        // Just verify it returns Ok, we don't care about the specific transform details
        assert!(
            result.is_ok(),
            "Function should not crash with valid BC1 data"
        );
    }
}
