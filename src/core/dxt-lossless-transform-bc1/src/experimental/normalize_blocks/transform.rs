//! Experimental BC1 Transform Operations with Normalization Support
//!
//! This module provides experimental transformation functionality for BC1 (DXT1) compressed
//! texture data that includes block normalization support for improved compression ratios.
//!
//! ## Block Normalization
//!
//! Block normalization is a preprocessing step that standardizes the representation of
//! common block patterns (solid colors, fully transparent blocks) to improve compression.
//! This is done before the standard BC1 transformations.
//!
//! ## Performance Characteristics
//!
//! The experimental functions test normalization combined with transformation options:
//! - 1/12th of the compression speed in fast mode (3 [`ColorNormalizationMode`] * 2 [`YCoCgVariant`] * 2 (split_colours))
//! - 1/24th of the compression speed in comprehensive mode (3 [`ColorNormalizationMode`] * 4 [`YCoCgVariant`] * 2 (split_colours))
//! - Uses 3x the memory of input size when no normalization is needed, 6x when normalization is required
//!
//! [`YCoCgVariant`]: dxt_lossless_transform_common::color_565::YCoCgVariant

use crate::transform::settings::{COMPREHENSIVE_TEST_ORDER, FAST_TEST_ORDER};
use crate::transform::standard::{transform, transform_with_separate_pointers};
use crate::transform::{transform_bc1_auto, Bc1EstimateSettings, DetermineBestTransformError};
use core::mem::size_of;
use core::slice;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_common::{
    allocate::{allocate_align_64, FixedRawAllocArray},
    color_565::Color565,
    transforms::split_565_color_endpoints::split_color_endpoints,
};

use super::{normalize_blocks_all_modes, normalize_split_blocks_in_place, ColorNormalizationMode};
use crate::experimental::Bc1TransformDetailsWithNormalization;

/// Transform BC1 data into a more compressible format with experimental normalization support.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `work_ptr`: A pointer to a work buffer (used by function)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr` and half size of `work_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`transform_bc1_auto_with_normalization`] or
///   [`Bc1TransformDetailsWithNormalization::default`] for less optimal result(s).
///
/// # Remarks
///
/// The transform is lossless, in the sense that each pixel will produce an identical value upon
/// decode, however, it is not guaranteed that after decode, the file will produce an identical hash.
///
/// `output_ptr` will be written to twice if normalization is used (it normally is).
/// This may have performance implications if `output_ptr` is a pointer to a memory mapped file
/// and amount of available memory is scarce. Outside of that, memory should be fairly unaffected.
///
/// # Safety
///
/// - input_ptr must be valid for reads of len bytes
/// - output_ptr must be valid for writes of len bytes
/// - work_ptr must be valid for writes of len/2 bytes
/// - len must be divisible by 8
/// - It is recommended that input_ptr and output_ptr are at least 16-byte aligned (recommended 32-byte align)
#[inline]
pub unsafe fn transform_bc1_with_normalize_blocks(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    work_ptr: *mut u8,
    len: usize,
    transform_options: Bc1TransformDetailsWithNormalization,
) {
    debug_assert!(len.is_multiple_of(8));

    let has_normalization =
        transform_options.color_normalization_mode != ColorNormalizationMode::None;
    let has_split_colours = transform_options.split_colour_endpoints;

    // Both normalization and split colours. 11
    if has_normalization && has_split_colours {
        // Split the blocks, colours to work area, indices to final destination.
        transform_with_separate_pointers(
            input_ptr,                           // from our input
            work_ptr as *mut u32,                // colours to go our work area
            output_ptr.add(len / 2) as *mut u32, // but the indices go to their final destination
            len,
        );

        // Now normalize the blocks in place. In place is faster because it avoids copying the data unnecessarily.
        normalize_split_blocks_in_place(
            work_ptr,                // colours are in first half of the work buffer
            output_ptr.add(len / 2), // indices are in second half of the output buffer
            len / 8,                 // 8 bytes per block, so len / 8 blocks
            transform_options.color_normalization_mode,
        );

        // Split the colour endpoints, writing them to the output buffer alongside the indices for final result
        split_color_endpoints(
            work_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Decorrelate the colours in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
    // Only normalization. 10
    else if has_normalization {
        // Split the blocks into the output area.
        transform(input_ptr, output_ptr, len);

        // Now normalize them in place. In place is faster because it avoids copying the data unnecessarily.
        normalize_split_blocks_in_place(
            output_ptr,              // colours are in first half of the output buffer
            output_ptr.add(len / 2), // indices are in second half of the output buffer
            len / 8,                 // 8 bytes per block, so len / 8 blocks
            transform_options.color_normalization_mode,
        );

        // Decorrelate the colours in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
    // Only split colours. 01
    else if has_split_colours {
        // Split the blocks, colours to work area, indices to final destination.
        transform_with_separate_pointers(
            input_ptr,                           // from our input
            work_ptr as *mut u32,                // colours to go our work area
            output_ptr.add(len / 2) as *mut u32, // but the indices go to their final destination
            len,
        );

        // Split the colour endpoints, writing them to the final output buffer.
        split_color_endpoints(
            work_ptr as *const Color565,
            output_ptr as *mut Color565,
            len / 2,
        );

        // Decorrelate the colours in output buffer in-place (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
    // None. 00
    else {
        // Split the blocks directly into expected output.
        transform(input_ptr, output_ptr, len);

        // And if there's colour decorrelation, do it right now (if needed, no-ops if mode is 'none')
        Color565::decorrelate_ycocg_r_ptr(
            output_ptr as *const Color565,
            output_ptr as *mut Color565,
            (len / 2) / size_of::<Color565>(), // (len / 2): Length of colour endpoints in bytes
            transform_options.decorrelation_mode,
        );
    }
}

/// Transform BC1 data with the best normalization and transform settings.
///
/// This function tests various normalization modes combined with transform configurations  
/// and applies the combination that produces the smallest compressed size according to
/// the provided estimator.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output buffer where transformed data will be written
/// - `len`: The length of the input data in bytes
/// - `transform_options`: Options for the estimation including the file size estimator and normalization settings
///
/// # Returns
///
/// The best (smallest size) format for the given data.
///
/// # Remarks
///
/// This function combines normalization with the brute force transform approach,
/// testing all normalization options with all transform configurations:
///
/// - 1/12th of the compression speed in fast mode (3 [`ColorNormalizationMode`] * 2 [`YCoCgVariant`] * 2 (split_colours))
/// - 1/24th of the compression speed in comprehensive mode (3 [`ColorNormalizationMode`] * 4 [`YCoCgVariant`] * 2 (split_colours))
/// - Uses 3x the memory of input size when no normalization is needed, 6x when normalization is required
///
/// [`YCoCgVariant`]: dxt_lossless_transform_common::color_565::YCoCgVariant
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
/// This ordering is applied within each normalization mode to minimize redundant
/// final transforms.
///
/// # Safety
///
/// - `input_ptr` must be valid for reads of `len` bytes
/// - `output_ptr` must be valid for writes of `len` bytes
/// - `len` must be divisible by 8
/// - It is recommended that `input_ptr` and `output_ptr` are at least 16-byte aligned (recommended 32-byte align)
pub unsafe fn transform_bc1_auto_with_normalization<T>(
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    transform_options: Bc1EstimateSettings<T>,
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

        // Now split all blocks.
        for x in 0..NUM_NORMALIZE {
            transform(normalize_buffers_ptrs[x], split_blocks_buffers_ptrs[x], len);
        }

        // We'll also need a working buffer for transformation
        let mut work_buffer = allocate_align_64(len / 2)?;
        let work_ptr = work_buffer.as_mut_ptr();

        // Use optimal test order within each normalization mode to minimize redundant transforms
        let test_order = if transform_options.use_all_decorrelation_modes {
            COMPREHENSIVE_TEST_ORDER
        } else {
            FAST_TEST_ORDER
        };

        #[allow(clippy::needless_range_loop)]
        for norm_idx in 0..NUM_NORMALIZE {
            for &(decorrelation_mode, split_colours) in test_order {
                // Get the current mode we're testing.
                let current_mode = Bc1TransformDetailsWithNormalization {
                    color_normalization_mode: ColorNormalizationMode::all_values()[norm_idx],
                    decorrelation_mode,
                    split_colour_endpoints: split_colours,
                };

                // Get input/output buffers.
                let input = split_blocks_buffers_ptrs[norm_idx];
                let output = work_ptr;

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

        // Now transform with the best settings to the output buffer
        transform_bc1_with_normalize_blocks(
            input_ptr,
            output_ptr,
            work_ptr,
            len,
            best_transform_details,
        );

        Ok(best_transform_details)
    } else {
        // No blocks were normalized, we can skip testing normalize steps
        // Drop buffers first for memory usage
        drop(normalize_buffers);

        // Use the regular transform function since no normalization is needed
        let regular_result = transform_bc1_auto(input_ptr, output_ptr, len, &transform_options)?;

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
    transform_options: &Bc1EstimateSettings<T>,
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
    let result_size = match transform_options.size_estimator.estimate_compressed_size(
        output,
        len / 2,
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
    use crate::test_prelude::*;
    use crate::{untransform_bc1_with_settings, Bc1UntransformSettings};
    use dxt_lossless_transform_common::allocate::allocate_align_64;

    /// Test roundtrip transform→untransform for all combinations of Bc1TransformDetailsWithNormalization
    #[test]
    fn roundtrip_test_all_combinations() {
        const MAX_BLOCKS: usize = 64;

        for num_blocks in 1..=MAX_BLOCKS {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();

            // Test all combinations of Bc1TransformDetailsWithNormalization
            for details in
                crate::experimental::Bc1TransformDetailsWithNormalization::all_combinations()
            {
                let mut transformed = allocate_align_64(len).unwrap();
                let mut work_buffer = allocate_align_64(len / 2).unwrap();
                let mut reconstructed = allocate_align_64(len).unwrap();

                unsafe {
                    // Transform using experimental function
                    transform_bc1_with_normalize_blocks(
                        input.as_ptr(),
                        transformed.as_mut_ptr(),
                        work_buffer.as_mut_ptr(),
                        len,
                        details,
                    );

                    // Untransform using standard function (normalization doesn't need to be reversed)
                    let untransform_details: Bc1UntransformSettings = details.into();
                    untransform_bc1_with_settings(
                        transformed.as_ptr(),
                        reconstructed.as_mut_ptr(),
                        len,
                        untransform_details,
                    );
                }

                assert_eq!(
                    reconstructed.as_slice(),
                    input.as_slice(),
                    "Roundtrip failed for {num_blocks} blocks with details: {details:?}",
                );
            }
        }
    }

    /// Test that transform_bc1_auto_with_normalization doesn't crash with minimal BC1 data
    #[test]
    fn test_transform_bc1_auto_with_normalization_does_not_crash() {
        // Create minimal BC1 block data (8 bytes per block)
        // This is a simple red block
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output_buffer = [0u8; 8];

        // Use the dummy estimator from test_prelude
        let transform_options = Bc1EstimateSettings {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        // This should not crash
        let result = unsafe {
            transform_bc1_auto_with_normalization(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
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
