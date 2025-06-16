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
}

/// Determine the best transform details for the given BC1 blocks.
///
/// # Parameters
///
/// - `input_ptr`: A pointer to the input data (input BC1 blocks)
/// - `len`: The length of the input data in bytes
/// - `transform_options`: Options for the estimation including the file size estimator
///
/// # Returns
///
/// The best (smallest size) format for the given data.
///
/// # Remarks
///
/// This function is a brute force approach that tests all standard transform options:
/// - 1/8th of the compression speed ([`YCoCgVariant`] * 2 (split_colours))
/// - Uses 1x the memory of input size (for temporary copy of the input data)
///
/// For experimental normalization support, use the functions in the experimental module instead.
///
/// # Safety
///
/// Function is unsafe because it deals with raw pointers which must be correct.
pub unsafe fn determine_best_transform_details<F>(
    input_ptr: *const u8,
    len: usize,
    transform_options: Bc1EstimateOptions<F>,
) -> Result<Bc1TransformDetails, DetermineBestTransformError>
where
    F: Fn(*const u8, usize) -> usize,
{
    let mut result_buffer = allocate_align_64(len)?;

    let mut best_transform_details = Bc1TransformDetails::default();
    let mut best_size = usize::MAX;

    for decorrelation_mode in YCoCgVariant::all_values() {
        for split_colours in [true, false] {
            // Get the current mode we're testing.
            let current_mode = Bc1TransformDetails {
                decorrelation_mode: *decorrelation_mode,
                split_colour_endpoints: split_colours,
            };

            // Apply a full transformation (~24GB/s on 1 thread, Ryzen 9950X3D)
            crate::transform_bc1(input_ptr, result_buffer.as_mut_ptr(), len, current_mode);

            // Note(sewer): The indices are very poorly compressible (entropy == ~7.0 , no lz matches).
            // Excluding them from the estimation has negligible effect on results, with a doubling of
            // speed.

            // Test the current mode by measuring the compressed size
            let result_size =
                (transform_options.file_size_estimator)(result_buffer.as_ptr(), len / 2);
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
        };

        // This should not crash
        let result = unsafe {
            determine_best_transform_details(bc1_data.as_ptr(), bc1_data.len(), transform_options)
        };

        // Just verify it returns Ok, we don't care about the specific transform details
        assert!(
            result.is_ok(),
            "Function should not crash with valid BC1 data"
        );
    }
}
