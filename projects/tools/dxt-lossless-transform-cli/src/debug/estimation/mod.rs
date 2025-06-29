//! Size estimation operations for compression algorithms.

use crate::debug::compression_size_cache::CompressionSizeCache;
use crate::debug::{calculate_content_hash, compression::CompressionAlgorithm};
use crate::error::TransformError;
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};
use std::sync::Mutex;

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

    fn supports_data_type_differentiation(&self) -> bool {
        self.0.supports_data_type_differentiation()
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

    fn supports_data_type_differentiation(&self) -> bool {
        self.0.supports_data_type_differentiation()
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

/// Caching wrapper for SizeEstimationOperations that uses [`CompressionSizeCache`]
///
/// This wrapper adds caching functionality on top of any [`SizeEstimationOperations`]
/// implementation, reducing redundant size estimations for the same data.
pub struct CachedSizeEstimator<'a, T> {
    inner: T,
    cache: &'a Mutex<CompressionSizeCache>,
    algorithm: CompressionAlgorithm,
    compression_level: i32,
}

impl<'a> CachedSizeEstimator<'a, Box<dyn SizeEstimationOperations<Error = TransformError>>> {
    /// Creates a new cached size estimator for the specified algorithm.
    ///
    /// # Parameters
    /// * `algorithm` - The compression algorithm to create an estimator for
    /// * `compression_level` - The compression level to use for estimation
    /// * `cache` - Shared cache for storing size estimation results
    ///
    /// # Returns
    /// A cached size estimator that wraps the specified algorithm
    pub fn new(
        algorithm: CompressionAlgorithm,
        compression_level: i32,
        cache: &'a Mutex<CompressionSizeCache>,
    ) -> Result<Self, TransformError> {
        let inner = create_size_estimator(algorithm, compression_level)?;
        Ok(Self {
            inner,
            cache,
            algorithm,
            compression_level,
        })
    }
}

impl<'a, T> SizeEstimationOperations for CachedSizeEstimator<'a, T>
where
    T: SizeEstimationOperations<Error = TransformError>,
{
    type Error = TransformError;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        // Delegate to inner estimator - max size doesn't need caching
        self.inner.max_compressed_size(len_bytes)
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        // Calculate content hash for cache key
        let input_slice = unsafe { core::slice::from_raw_parts(input_ptr, len_bytes) };
        let cache_hash = calculate_content_hash(input_slice);

        // Determine the data type to use for caching
        // If the estimator doesn't support data type differentiation, use Unknown
        let cache_data_type = if self.inner.supports_data_type_differentiation() {
            data_type
        } else {
            DataType::Unknown
        };

        // Try to get from cache first
        {
            let cache_guard = self.cache.lock().unwrap();
            if let Some(cached_size) = cache_guard.get(
                cache_hash,
                self.compression_level,
                self.algorithm,
                cache_data_type,
            ) {
                return Ok(cached_size);
            }
        }

        // Not in cache, delegate to inner estimator
        let estimated_size = self
            .inner
            .estimate_compressed_size(input_ptr, len_bytes, data_type, output_ptr, output_len)?;

        // Store result in cache
        {
            let mut cache_guard = self.cache.lock().unwrap();
            cache_guard.insert(
                cache_hash,
                self.compression_level,
                self.algorithm,
                cache_data_type,
                estimated_size,
            );
        }

        Ok(estimated_size)
    }
}
