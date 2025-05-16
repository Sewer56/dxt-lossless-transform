use crate::Bc1TransformDetails;

/// The options for [`determine_best_transform_details`], regarding how the estimation is done,
/// and other related factors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub file_size_estimator: fn(*const u8, usize) -> usize,
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
/// This function is a brute force, the throughput of this function is:
///
/// - 1/24th of the compression speed ([`ColorNormalizationMode`] * [`YCoCgVariant`] * [`split_blocks`])
/// - Uses 3x the memory of input size on average (6x peak)
pub fn determine_best_transform_details(
    _input_ptr: *const u8,
    _len: usize,
    _transform_options: Bc1TransformOptions,
) -> Bc1TransformDetails {
    /*
    let mode_count = ColorNormalizationMode::all_values().len();
    let mut output_buffers = Vec::with_capacity(mode_count);

    for _ in 0..mode_count {
        output_buffers.push(allocate_align_64(file_size));
    }

    // Create a fresh stack array of pointers for each iteration (else it segfaults because pointers
    // are not reset back to starting pos across iterations)
    const NUM_MODES: usize = ColorNormalizationMode::all_values().len();
    let mut output_ptrs_array: [*mut u8; NUM_MODES] = [null_mut(); NUM_MODES];
    for x in 0..NUM_MODES {
        output_ptrs_array[x] = output_buffers[x].as_mut_ptr();
    }

    let mut normalized = RawAlloc::new(Layout::from_size_align_unchecked(len, 64)).unwrap();
    normalize_blocks_all_modes(input_ptr, normalized.as_mut_ptr(), len);
    */
    Bc1TransformDetails::default()
}
