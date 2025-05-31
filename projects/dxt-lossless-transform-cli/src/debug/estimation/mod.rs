//! Size estimation operations for compression algorithms.

use crate::debug::compression::CompressionAlgorithm;
use crate::error::TransformError;

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

/// Factory for creating size estimation operation instances.
pub fn create_size_estimation_operations(
    algorithm: CompressionAlgorithm,
) -> Box<dyn SizeEstimationOperations> {
    match algorithm {
        CompressionAlgorithm::ZStandard => Box::new(zstd::ZStandardSizeEstimation),
        CompressionAlgorithm::LosslessTransformUtils => panic!("Not yet supported"),
    }
}
