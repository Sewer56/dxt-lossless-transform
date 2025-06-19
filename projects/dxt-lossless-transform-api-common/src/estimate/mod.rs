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

    /// Calculates the estimated compressed size.
    ///
    /// # Parameters
    /// * `data_ptr` - Pointer to the data to estimate
    /// * `len_bytes` - Length of the data in bytes
    /// * `data_type` - The type of data being compressed
    ///
    /// # Returns
    /// The estimated compressed size in bytes
    ///
    /// # Safety
    /// `data_ptr` must be valid for reads of `len_bytes` bytes.
    unsafe fn estimate_compressed_size(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
    ) -> Result<usize, Self::Error>;
}

/// A function-based size estimator that can be used where a simple closure is needed.
///
/// This wrapper allows converting [`SizeEstimationOperations`] implementations into
/// function pointers that can be used in APIs like [`Bc1EstimateOptions`].
///
/// # Example
/// ```rust,ignore
/// use dxt_lossless_transform_api_common::estimate::FunctionSizeEstimator;
///
/// let estimator = FunctionSizeEstimator::new(my_size_estimation_impl);
/// let size_fn = estimator.as_function();
/// ```
pub struct FunctionSizeEstimator<T: SizeEstimationOperations> {
    inner: T,
}

impl<T: SizeEstimationOperations> FunctionSizeEstimator<T> {
    /// Creates a new function size estimator from an implementation.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Returns a closure that can be used as a size estimation function.
    ///
    /// The returned closure has the signature `Fn(*const u8, usize, DataType) -> usize`
    /// and returns 0 if estimation fails.
    pub fn as_function(&self) -> impl Fn(*const u8, usize, DataType) -> usize + '_
    where
        T::Error: core::fmt::Debug,
    {
        move |data_ptr: *const u8, len_bytes: usize, data_type: DataType| -> usize {
            unsafe {
                match self
                    .inner
                    .estimate_compressed_size(data_ptr, len_bytes, data_type)
                {
                    Ok(size) => size,
                    Err(e) => {
                        #[cfg(feature = "std")]
                        eprintln!("Size estimation failed: {e:?}");
                        0 // Return 0 on error as a fallback
                    }
                }
            }
        }
    }
}
