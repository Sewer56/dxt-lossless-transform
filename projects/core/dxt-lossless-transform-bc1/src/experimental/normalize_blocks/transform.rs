//! Transform functions with experimental normalization support.

use crate::transforms::standard::{transform, transform_with_separate_pointers};
use core::mem::size_of;
use dxt_lossless_transform_common::{
    color_565::Color565, transforms::split_565_color_endpoints::split_color_endpoints,
};

use super::{
    normalize_split_blocks_in_place, Bc1TransformDetailsWithNormalization, ColorNormalizationMode,
};

/// Transform BC1 data into a more compressible format.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `output_ptr`: A pointer to the output data (output BC1 blocks)
/// - `work_ptr`: A pointer to a work buffer (used by function)
/// - `len`: The length of the input data in bytes (size of `input_ptr`, `output_ptr` and half size of `work_ptr`)
/// - `transform_options`: The transform options to use.
///   Obtained from [`super::determine_best_transform_details_with_normalization`] or
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
    debug_assert!(len % 8 == 0);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_prelude::*;
    use dxt_lossless_transform_common::allocate::allocate_align_64;

    /// Test roundtrip transform→untransform for all combinations of Bc1TransformDetailsWithNormalization
    #[test]
    fn roundtrip_test_all_combinations() {
        use crate::{untransform_bc1, Bc1DetransformSettings};

        const MAX_BLOCKS: usize = 64;

        for num_blocks in 1..=MAX_BLOCKS {
            let input = generate_bc1_test_data(num_blocks);
            let len = input.len();

            // Test all combinations of Bc1TransformDetailsWithNormalization
            for details in Bc1TransformDetailsWithNormalization::all_combinations() {
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
                    let detransform_details: Bc1DetransformSettings = details.into();
                    untransform_bc1(
                        transformed.as_ptr(),
                        reconstructed.as_mut_ptr(),
                        len,
                        detransform_details,
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
}
