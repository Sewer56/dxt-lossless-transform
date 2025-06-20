//! Size estimation operations for file compression.
//!
//! This module provides traits and utilities for estimating compressed sizes
//! of data, which can be used for optimization algorithms that need to compare
//! compression ratios without performing full compression.

/// Enum representing the type of data being estimated for compressed size.
///
/// # Remarks
///
/// In CLI modules, and similar, this is supported for estimation only, not compression.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, bincode::Encode, bincode::Decode)]
pub enum DataType {
    /// Unknown data type
    Unknown = 0,
    /// Standard BC1 colour data (interleaved color0/color1 pairs)
    Bc1Colours = 1,
    /// BC1 colour data after decorrelation transforms have been applied
    Bc1DecorrelatedColours = 2,
    /// BC1 colour data that has been split into separate color0 and color1 arrays
    Bc1SplitColours = 3,
    /// BC1 colour data that has been both split and decorrelated
    Bc1SplitDecorrelatedColours = 4,
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

    /// Indicates whether this implementation produces different results for different [`DataType`] values.
    ///
    /// This is used by caching systems to determine whether to include the data type
    /// in cache keys or treat all data types the same.
    ///
    /// # Returns
    /// `true` if the implementation produces different results for different data types,
    /// `false` if it treats all data types the same way.
    ///
    /// # Default Implementation
    /// The default implementation returns `false`, meaning implementations that don't
    /// differentiate between data types don't need to override this method.
    fn supports_data_type_differentiation(&self) -> bool {
        false
    }

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

/// Blanket implementation of [`SizeEstimationOperations`] for any boxed variant of it.
impl<T: SizeEstimationOperations + ?Sized> SizeEstimationOperations for Box<T> {
    type Error = T::Error;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        (**self).max_compressed_size(len_bytes)
    }

    fn supports_data_type_differentiation(&self) -> bool {
        (**self).supports_data_type_differentiation()
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        (**self).estimate_compressed_size(input_ptr, len_bytes, data_type, output_ptr, output_len)
    }
}
