//! Size estimation operations for file compression.
//!
//! This module provides traits and utilities for estimating compressed sizes
//! of data, which can be used for optimization algorithms that need to compare
//! compression ratios without performing full compression.

use alloc::boxed::Box;

/// Trait for size estimation operations.
///
/// This trait is used to test the most optimal transform by comparing compressed sizes
/// between different data transformations. Implementations can provide either fast
/// approximations or perform actual compression to estimate the compressed size of data.
///
/// The trait allows implementations to have their compression levels and other
/// parameters pre-configured, making it more flexible than function-based approaches.
///
/// # Important Notes
///
/// The results of [`SizeEstimationOperations::estimate_compressed_size`] will only ever be compared against calls to
/// [`SizeEstimationOperations::estimate_compressed_size`] for the same estimator instance. The most important thing
/// is to be able to correctly assert whether one piece of data will compress to a smaller
/// size than another, rather than providing absolute accuracy of compressed sizes.
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
    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error>;

    /// Calculates the estimated compressed size.
    ///
    /// # Parameters
    /// * `input_ptr` - Pointer to the input data to estimate
    /// * `len_bytes` - Length of the input data in bytes
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
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error>;
}

/// Blanket implementation of [`SizeEstimationOperations`] for any boxed variant of it.
impl<T: SizeEstimationOperations + ?Sized> SizeEstimationOperations for Box<T> {
    type Error = T::Error;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        (**self).max_compressed_size(len_bytes)
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        (**self).estimate_compressed_size(input_ptr, len_bytes, output_ptr, output_len)
    }
}

/// Dummy size estimation implementation that provides no actual estimation.
///
/// This is primarily intended for testing and manual transform operations where
/// size estimation is not required. Only manual configuration is supported when
/// using this estimator - automatic optimization features that rely on size
/// estimation will not function.
///
/// # Usage
///
/// Use this when you need to provide an estimator type but don't actually need
/// size estimation functionality:
///
/// ```rust
/// use dxt_lossless_transform_api_common::estimate::NoEstimation;
/// // For types that require an estimator but you're only doing manual operations
/// ```
#[derive(Debug, Clone, Copy)]
pub struct NoEstimation;

impl SizeEstimationOperations for NoEstimation {
    type Error = ();

    fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    unsafe fn estimate_compressed_size(
        &self,
        _input_ptr: *const u8,
        _len_bytes: usize,
        _output_ptr: *mut u8,
        _output_len: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}
