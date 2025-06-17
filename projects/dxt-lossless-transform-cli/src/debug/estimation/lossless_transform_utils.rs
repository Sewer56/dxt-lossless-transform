//! Size estimation using lossless-transform-utils.

use crate::debug::estimation::SizeEstimationOperations;
use crate::error::TransformError;
use core::slice;
use lossless_transform_utils::{entropy::*, histogram::*, match_estimator::*};

/// Parameters for size estimation calculation.
#[derive(Debug, Clone)]
pub struct SizeEstimationParameters {
    /// Length of the original data in bytes
    pub data_len: usize,
    /// Number of LZ matches found in the data
    pub num_lz_matches: usize,
    /// Entropy value (bits per byte) from histogram analysis
    pub entropy: f64,
    /// Multiplier for LZ match effectiveness (default: 0.5809)
    pub lz_match_multiplier: f64,
    /// Multiplier for entropy coding efficiency (default: 1.1670)
    pub entropy_multiplier: f64,
}

impl Default for SizeEstimationParameters {
    fn default() -> Self {
        Self {
            data_len: 0,
            num_lz_matches: 0,
            entropy: 0.0,
            lz_match_multiplier: 0.5809,
            entropy_multiplier: 1.1670,
        }
    }
}

/// Size estimator using lossless-transform-utils library.
pub struct LosslessTransformUtilsSizeEstimation;

impl SizeEstimationOperations for LosslessTransformUtilsSizeEstimation {
    /// Estimates compressed size using lossless-transform-utils.
    ///
    /// This implementation uses LZ match estimation and entropy calculation
    /// to provide a fast approximation of the compressed size.
    ///
    /// # Parameters
    /// * `data_ptr` - Pointer to the data to estimate
    /// * `len_bytes` - Length of the data in bytes
    /// * `compression_level` - Not used by this estimator
    ///
    /// # Returns
    /// The estimated compressed size in bytes
    fn estimate_compressed_size(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        _compression_level: i32,
    ) -> Result<usize, TransformError> {
        if data_ptr.is_null() || len_bytes == 0 {
            return Ok(0);
        }

        // Safety: We've checked that data_ptr is not null and len_bytes > 0
        let data = unsafe { slice::from_raw_parts(data_ptr, len_bytes) };

        // Estimate number of LZ matches
        let num_matches = estimate_num_lz_matches_fast(data);

        // Calculate entropy using histogram
        let mut histogram = Histogram32::default();
        histogram32_from_bytes(data, &mut histogram);
        let entropy = code_length_of_histogram32(&histogram, data.len() as u64);

        // Create parameters and calculate estimate
        let params = SizeEstimationParameters {
            data_len: len_bytes,
            num_lz_matches: num_matches,
            entropy,
            ..Default::default()
        };

        Ok(size_estimate(params))
    }
}

/// Estimates the compressed size of data in bytes using the provided parameters.
///
/// # Arguments
///
/// * `params` - The size estimation parameters containing data length, LZ matches, and entropy
///
/// # Returns
///
/// Estimated size in bytes after compression
pub fn size_estimate(params: SizeEstimationParameters) -> usize {
    // Calculate expected bytes after LZ
    let bytes_after_lz =
        params.data_len - (params.num_lz_matches as f64 * params.lz_match_multiplier) as usize;

    // Calculate expected bits and convert to bytes
    (bytes_after_lz as f64 * params.entropy * params.entropy_multiplier).ceil() as usize / 8
}
