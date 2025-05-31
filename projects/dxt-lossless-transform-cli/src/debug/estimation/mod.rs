//! Size estimation operations for compression algorithms.

use crate::debug::compression::CompressionAlgorithm;
use crate::error::TransformError;
use zstd::ZStandardSizeEstimation;

pub mod zstd;

/// Trait for size estimation operations (can be fast approximations or actual compression).
pub trait SizeEstimationOperations {
    /// Calculates the estimated compressed size.
    ///
    /// # Parameters
    /// * `data_ptr` - Pointer to the data to estimate
    /// * `len_bytes` - Length of the data in bytes
    /// * `compression_level` - Compression level (algorithm-specific)
    ///
    /// # Returns
    /// The estimated compressed size in bytes
    fn estimate_compressed_size(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        compression_level: i32,
    ) -> Result<usize, TransformError>;
}

/// Estimates compressed size using the specified algorithm.
///
/// This function dispatches the size estimation task to the appropriate
/// implementation based on the [`CompressionAlgorithm`].
///
/// # Parameters
/// * `data_ptr` - Pointer to the data to estimate.
/// * `len_bytes` - Length of the data in bytes.
/// * `algorithm` - The estimation algorithm to use.
/// * `compression_level` - Compression level (algorithm-specific).
///
/// # Returns
/// A [`Result`] containing the estimated compressed size in bytes,
/// or a [`TransformError`] if estimation is not supported or fails.
pub fn estimate_compressed_size_with_algorithm(
    data_ptr: *const u8,
    len_bytes: usize,
    algorithm: CompressionAlgorithm,
    compression_level: i32,
) -> Result<usize, TransformError> {
    match algorithm {
        CompressionAlgorithm::ZStandard => {
            ZStandardSizeEstimation.estimate_compressed_size(data_ptr, len_bytes, compression_level)
        }
        CompressionAlgorithm::LosslessTransformUtils => todo!(),
    }
}
