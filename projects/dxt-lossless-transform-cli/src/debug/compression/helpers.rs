//! Wrappers for ZStandard around the `zstd` module, with caching support for benchmarking
//! and 'native' error handling.

use super::super::{
    calculate_content_hash, compressed_data_cache::CompressedDataCache,
    compression_size_cache::CompressionSizeCache,
};
use super::{create_compression_operations, CompressionAlgorithm};
use crate::debug::estimation::create_size_estimation_operations;
use crate::error::TransformError;
use std::sync::Mutex;

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
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    algorithm: CompressionAlgorithm,
    caches: &CacheRefs,
) -> Result<(Box<[u8]>, usize), TransformError> {
    // Calculate content hash once
    let content_hash = calculate_content_hash(data_ptr, len_bytes);

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
    let compression_ops = create_compression_operations(algorithm);
    let (compressed_data, compressed_size) =
        compression_ops.compress_data(data_ptr, len_bytes, compression_level)?;

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
pub unsafe fn calc_size_with_estimation_algorithm(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    estimation_algorithm: CompressionAlgorithm,
) -> Result<usize, TransformError> {
    let estimation_ops = create_size_estimation_operations(estimation_algorithm);
    estimation_ops.estimate_compressed_size(data_ptr, len_bytes, compression_level)
}

/// Calculates compressed size using a separate estimation algorithm with caching support.
///
/// This function first checks if the compressed size is already cached, and if not,
/// estimates the compressed size using the specified estimation algorithm and stores the result in the cache for future use.
pub fn calc_size_with_cache_and_estimation_algorithm(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    estimation_algorithm: CompressionAlgorithm,
    cache: &Mutex<CompressionSizeCache>,
) -> Result<usize, TransformError> {
    let content_hash = calculate_content_hash(data_ptr, len_bytes);

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
    let estimation_ops = create_size_estimation_operations(estimation_algorithm);
    let compressed_size =
        estimation_ops.estimate_compressed_size(data_ptr, len_bytes, compression_level)?;

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
    let compression_ops = create_compression_operations(algorithm);
    compression_ops.decompress_data(compressed_data, output_buffer)
}
