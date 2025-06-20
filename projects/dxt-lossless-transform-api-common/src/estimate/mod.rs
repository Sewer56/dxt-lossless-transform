//! Size estimation operations for file compression.
//!
//! This module provides traits and utilities for estimating compressed sizes
//! of data, which can be used for optimization algorithms that need to compare
//! compression ratios without performing full compression.

/// Enum representing the type of data being estimated for compression.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    /// Standard BC1 colour data (interleaved color0/color1 pairs)
    Bc1Colours = 0,
    /// BC1 colour data after decorrelation transforms have been applied
    Bc1DecorrelatedColours = 1,
    /// BC1 colour data that has been split into separate color0 and color1 arrays
    Bc1SplitColours = 2,
    /// BC1 colour data that has been both split and decorrelated
    Bc1SplitDecorrelatedColours = 3,
}

/// Trait for size estimation operations.
///
/// Implementations can provide either fast approximations or perform actual
/// compression to estimate the compressed size of data.
///
/// The trait allows implementations to have their compression levels and other
/// parameters pre-configured, making it more flexible than function-based approaches.
pub trait SizeEstimationOperations {
    /// The error type returned by estimation operations.
    type Error;

    /// Returns the maximum size required for a compression buffer.
    ///
    /// This allows implementations that require pre-allocation to specify
    /// the buffer size needed for compression operations.
    ///
    /// # Parameters
    /// * `len_bytes` - Length of the input data in bytes
    ///
    /// # Returns
    /// The maximum buffer size needed for compression, or 0 if no allocation is needed
    ///
    /// # Contract
    /// The returned size must be consistent regardless of the [`DataType`] that will be passed
    /// to [`estimate_compressed_size`]. This allows callers to allocate once and reuse the
    /// buffer across different data types.
    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error>;

    /// Calculates the estimated compressed size.
    ///
    /// # Parameters
    /// * `input_ptr` - Pointer to the input data to estimate
    /// * `len_bytes` - Length of the input data in bytes
    /// * `data_type` - The type of data being compressed
    /// * `output_ptr` - Pre-allocated output buffer for compression (guaranteed non-null)
    /// * `output_len` - Length of the pre-allocated output buffer
    ///
    /// # Returns
    /// The estimated compressed size in bytes
    ///
    /// # Remarks
    /// The output buffer may be reused across multiple calls with the same input size.
    /// All inputs in a batch will have the same `len_bytes` size.
    ///
    /// # Safety
    /// * `input_ptr` must be valid for reads of `len_bytes` bytes
    /// * `output_ptr` must be valid for writes of `output_len` bytes
    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error>;
}
