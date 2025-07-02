//! Generic compression helpers with caching support for benchmarking
//! and 'native' error handling. Supports multiple compression algorithms.

use dxt_lossless_transform_common::allocate::allocate_align_64;

use super::super::{
    calculate_content_hash, compressed_data_cache::CompressedDataCache,
    compression_size_cache::CompressionSizeCache,
};
use super::compress_with_algorithm;
use super::decompress_with_algorithm;
use super::CompressionAlgorithm;
use crate::debug::estimation::create_size_estimator;
use crate::error::TransformError;
use std::sync::Mutex;

/// Validates that a compression algorithm supports actual compression
pub fn validate_compression_algorithm(
    algorithm: CompressionAlgorithm,
) -> Result<(), TransformError> {
    if !algorithm.supports_compress() {
        return Err(TransformError::Debug(format!(
            "Compression algorithm '{algorithm}' does not support actual compression.\
Use a compression algorithm like ZStandard for operations that require real compression.",
        )));
    }
    Ok(())
}

/// References to the caches used during benchmarking
pub struct CacheRefs<'a> {
    pub compressed_size_cache: &'a Mutex<CompressionSizeCache>,
    pub compressed_data_cache: &'a CompressedDataCache,
}

/// Compresses data using the specified compression algorithm with caching support.
///
/// This function first checks if compressed data is already cached, and if not,
/// compresses the data and stores the result in the cache for future use.
/// Returns both the compressed data and size.
///
/// Also populates a [`CompressionSizeCache`] with the compressed size for future size-only queries.
pub fn compress_data_cached(
    data: &[u8],
    compression_level: i32,
    algorithm: CompressionAlgorithm,
    caches: &CacheRefs,
) -> Result<(Box<[u8]>, usize), TransformError> {
    // Calculate content hash once
    let content_hash = calculate_content_hash(data);

    // Try to load from cache first
    if let Some((cached_data, cached_size)) = caches.compressed_data_cache.load_compressed_data(
        content_hash,
        compression_level,
        algorithm,
    )? {
        // Data cache and size cache should be synced, so no need to write to size cache here
        return Ok((cached_data, cached_size));
    }

    // Not in cache, compress the data using the compression operations
    let (compressed_data, compressed_size) =
        compress_with_algorithm(data.as_ptr(), data.len(), algorithm, compression_level)?;

    // Save to cache for future use
    if let Err(e) = caches.compressed_data_cache.save_compressed_data(
        content_hash,
        compression_level,
        algorithm,
        &compressed_data[..compressed_size],
    ) {
        // Log the error but don't fail the operation
        eprintln!("Warning: Failed to save compressed data to cache: {e}");
    }

    // Also populate the size cache when writing to data cache
    {
        let mut size_cache_guard = caches.compressed_size_cache.lock().unwrap();
        size_cache_guard.insert(content_hash, compression_level, algorithm, compressed_size);
    }

    Ok((compressed_data, compressed_size))
}

/// Calculates compressed size using a separate estimation algorithm without caching.
/// This is a simplified version that doesn't use any caching mechanisms for pure performance measurement.
pub fn calc_size_with_estimation_algorithm(
    data: &[u8],
    compression_level: i32,
    estimation_algorithm: CompressionAlgorithm,
) -> Result<usize, TransformError> {
    let estimator = create_size_estimator(estimation_algorithm, compression_level)?;

    // Get max buffer size (may be 0 if no allocation needed)
    let max_comp_size = estimator.max_compressed_size(data.len())?;

    // Allocate buffer if needed
    let (comp_buffer_ptr, comp_buffer_len, _comp_buffer) = if max_comp_size == 0 {
        (core::ptr::null_mut(), 0, None)
    } else {
        let mut comp_buffer = allocate_align_64(max_comp_size)?;
        let ptr = comp_buffer.as_mut_ptr();
        (ptr, max_comp_size, Some(comp_buffer))
    };

    unsafe {
        estimator.estimate_compressed_size(
            data.as_ptr(),
            data.len(),
            comp_buffer_ptr,
            comp_buffer_len,
        )
    }
}

/// Calculates compressed size using a separate estimation algorithm with caching support.
///
/// This function first checks if the compressed size is already cached, and if not,
/// estimates the compressed size using the specified estimation algorithm and stores the result in the cache for future use.
pub fn calc_size_with_cache_and_estimation_algorithm(
    data: &[u8],
    compression_level: i32,
    estimation_algorithm: CompressionAlgorithm,
    cache: &Mutex<CompressionSizeCache>,
) -> Result<usize, TransformError> {
    let content_hash = calculate_content_hash(data);

    // Try to get from cache
    {
        let cache_guard = cache.lock().unwrap();
        if let Some(cached_size) =
            cache_guard.get(content_hash, compression_level, estimation_algorithm)
        {
            return Ok(cached_size);
        }
    }

    // Not in cache, compute it
    let compressed_size =
        calc_size_with_estimation_algorithm(data, compression_level, estimation_algorithm)?;

    // Store in cache
    {
        let mut cache_guard = cache.lock().unwrap();
        cache_guard.insert(
            content_hash,
            compression_level,
            estimation_algorithm,
            compressed_size,
        );
    }

    Ok(compressed_size)
}

/// Decompresses data into a pre-allocated buffer using the specified algorithm.
///
/// This function decompresses the provided compressed data into the given buffer.
pub fn decompress_data(
    compressed_data: &[u8],
    output_buffer: &mut [u8],
    algorithm: CompressionAlgorithm,
) -> Result<usize, TransformError> {
    decompress_with_algorithm(compressed_data, output_buffer, algorithm)
}
