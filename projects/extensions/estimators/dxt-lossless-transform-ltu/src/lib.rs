#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![no_std]
#![warn(missing_docs)]

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(test)]
pub mod test_prelude;

#[cfg(feature = "c-exports")]
pub mod c_api;

use core::slice;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;
use dxt_lossless_transform_common::allocate::AllocateError;
use lossless_transform_utils::match_estimator::*;
use thiserror::Error;

/// Errors that can occur during lossless-transform-utils size estimation.
#[derive(Debug, Error)]
pub enum LosslessTransformUtilsError {
    /// Error during allocation
    #[error(transparent)]
    AllocateError(#[from] AllocateError),

    /// Invalid input data
    #[error("Invalid input data")]
    InvalidInput,
}

/// Lossless Transform Utils implementation of [`SizeEstimationOperations`].
///
/// This implementation uses the [`lossless_transform_utils`] library to provide
/// fast size estimation based on LZ match analysis.
///
/// It's significantly faster than performing actual compression while still
/// providing reasonable accuracy for optimization purposes.
///
/// # Important: Relative Comparison Only
///
/// This estimator is designed for relative comparison between different transforms
/// of the same data. The absolute values returned are not meaningful - only the
/// relative ordering matters for determining which transform compresses better.
///
/// The estimation is based on a simple formula: `data.len().saturating_sub(num_lz_matches)`.
/// More LZ matches indicate better compressibility, resulting in a lower estimated size.
pub struct LosslessTransformUtilsSizeEstimation;

impl LosslessTransformUtilsSizeEstimation {
    /// Creates a new lossless-transform-utils size estimator.
    pub fn new() -> Self {
        Self
    }

    /// Estimates the compressed size of data.
    ///
    /// Returns a value where lower numbers indicate better compression potential.
    /// The absolute value is not meaningful - only relative ordering matters.
    ///
    /// # Arguments
    /// * `data` - The input data to analyze
    ///
    /// # Returns
    /// Estimated relative size (lower = better compression)
    fn estimate_size(&self, data: &[u8]) -> usize {
        if data.is_empty() {
            return 0;
        }

        // Estimate number of LZ matches
        let lz_matches = estimate_num_lz_matches_fast(data);

        // Return data length minus LZ matches
        // More matches = lower returned value = "better compression"
        data.len().saturating_sub(lz_matches)
    }
}

impl Default for LosslessTransformUtilsSizeEstimation {
    fn default() -> Self {
        Self::new()
    }
}

impl SizeEstimationOperations for LosslessTransformUtilsSizeEstimation {
    type Error = LosslessTransformUtilsError;

    fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
        // For estimation purposes, we don't need a compression buffer
        // since we're not actually compressing data
        Ok(0)
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        _output_ptr: *mut u8,
        _output_len: usize,
    ) -> Result<usize, Self::Error> {
        if input_ptr.is_null() {
            return Ok(0);
        }

        if len_bytes == 0 {
            return Ok(0);
        }

        // Safety: We've checked that input_ptr is not null and len_bytes > 0
        let data = slice::from_raw_parts(input_ptr, len_bytes);

        // Perform size estimation
        let estimated_size = self.estimate_size(data);

        Ok(estimated_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_estimation_empty_data() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let result = unsafe {
            estimator.estimate_compressed_size(core::ptr::null(), 0, core::ptr::null_mut(), 0)
        };
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_size_estimation_with_data() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let test_data = [0u8; 64]; // 64 bytes of zeros
        let result = unsafe {
            estimator.estimate_compressed_size(
                test_data.as_ptr(),
                test_data.len(),
                core::ptr::null_mut(),
                0,
            )
        };

        // Should return a reasonable estimate
        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size < test_data.len()); // Should be smaller than input for repetitive data
    }

    #[test]
    fn test_max_compressed_size() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let result = estimator.max_compressed_size(1024);
        assert_eq!(result.unwrap(), 0); // No buffer needed for estimation
    }

    #[test]
    fn test_estimation_consistency() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();
        let test_data = [0u8; 32];

        // Test that estimation is consistent across multiple calls
        let result1 = unsafe {
            estimator.estimate_compressed_size(
                test_data.as_ptr(),
                test_data.len(),
                core::ptr::null_mut(),
                0,
            )
        };

        let result2 = unsafe {
            estimator.estimate_compressed_size(
                test_data.as_ptr(),
                test_data.len(),
                core::ptr::null_mut(),
                0,
            )
        };

        assert!(result1.is_ok());
        assert!(result2.is_ok());
        let size1 = result1.unwrap();
        let size2 = result2.unwrap();
        assert_eq!(size1, size2);
    }

    #[test]
    fn test_different_data_gives_different_estimates() {
        let estimator = LosslessTransformUtilsSizeEstimation::new();

        // Repetitive data should compress better (lower estimate)
        let repetitive_data = [0u8; 64];
        let repetitive_result = unsafe {
            estimator.estimate_compressed_size(
                repetitive_data.as_ptr(),
                repetitive_data.len(),
                core::ptr::null_mut(),
                0,
            )
        };

        // Random-ish data should compress worse (higher estimate)
        let random_data: alloc::vec::Vec<u8> = (0..64).map(|i| (i * 37) as u8).collect();
        let random_result = unsafe {
            estimator.estimate_compressed_size(
                random_data.as_ptr(),
                random_data.len(),
                core::ptr::null_mut(),
                0,
            )
        };

        assert!(repetitive_result.is_ok());
        assert!(random_result.is_ok());

        let repetitive_size = repetitive_result.unwrap();
        let random_size = random_result.unwrap();

        // Repetitive data should have better (lower) estimate
        assert!(repetitive_size <= random_size);
    }
}
