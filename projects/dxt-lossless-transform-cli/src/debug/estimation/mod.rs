//! Size estimation operations for compression algorithms.

use crate::debug::compression::CompressionAlgorithm;
use crate::error::TransformError;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Creates a size estimator for the specified algorithm.
///
/// This function returns a boxed trait object that implements
/// [`SizeEstimationOperations`] for the given compression algorithm.
///
/// # Parameters
/// * `algorithm` - The compression algorithm to create an estimator for
/// * `compression_level` - The compression level to use for estimation
///
/// # Returns
/// A boxed trait object implementing size estimation operations
pub fn create_size_estimator(
    algorithm: CompressionAlgorithm,
    compression_level: i32,
) -> Result<Box<dyn SizeEstimationOperations<Error = TransformError>>, TransformError> {
    match algorithm {
        CompressionAlgorithm::ZStandard => {
            use dxt_lossless_transform_zstd::ZStandardSizeEstimation;
            let estimator = ZStandardSizeEstimation::new(compression_level).map_err(|e| {
                TransformError::Debug(format!("Failed to create ZStandard estimator: {e}"))
            })?;

            // Create a wrapper that converts the error type
            Ok(Box::new(ZstdEstimatorWrapper(estimator)))
        }
        CompressionAlgorithm::LosslessTransformUtils => {
            use dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation;
            let estimator = LosslessTransformUtilsSizeEstimation::new();

            // Create a wrapper that converts the error type
            Ok(Box::new(LtuEstimatorWrapper(estimator)))
        }
    }
}

/// Wrapper for ZStandard estimator to convert error types
struct ZstdEstimatorWrapper(dxt_lossless_transform_zstd::ZStandardSizeEstimation);

impl SizeEstimationOperations for ZstdEstimatorWrapper {
    type Error = TransformError;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        self.0.max_compressed_size(len_bytes).map_err(|e| {
            TransformError::Debug(format!("ZStandard max compressed size failed: {e}"))
        })
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: dxt_lossless_transform_api_common::estimate::DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        self.0
            .estimate_compressed_size(input_ptr, len_bytes, data_type, output_ptr, output_len)
            .map_err(|e| TransformError::Debug(format!("ZStandard estimation failed: {e}")))
    }
}

/// Wrapper for LosslessTransformUtils estimator to convert error types
struct LtuEstimatorWrapper(dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation);

impl SizeEstimationOperations for LtuEstimatorWrapper {
    type Error = TransformError;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        self.0.max_compressed_size(len_bytes).map_err(|e| {
            TransformError::Debug(format!(
                "LosslessTransformUtils max compressed size failed: {e}"
            ))
        })
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: dxt_lossless_transform_api_common::estimate::DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        self.0
            .estimate_compressed_size(input_ptr, len_bytes, data_type, output_ptr, output_len)
            .map_err(|e| {
                TransformError::Debug(format!("LosslessTransformUtils estimation failed: {e}"))
            })
    }
}
