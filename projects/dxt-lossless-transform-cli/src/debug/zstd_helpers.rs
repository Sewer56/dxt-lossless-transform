//! Wrappers for ZStandard around the `zstd` module, with caching support for benchmarking
//! and 'native' error handling.

use super::{
    calculate_content_hash, compressed_data_cache::CompressedDataCache,
    compression_size_cache::CompressionSizeCache, zstd,
};
use crate::error::TransformError;
use core::slice;
use std::sync::Mutex;

/// References to the caches used during benchmarking
pub struct CacheRefs<'a> {
    pub compressed_size_cache: &'a Mutex<CompressionSizeCache>,
    pub compressed_data_cache: &'a CompressedDataCache,
}

/// Compresses data using ZStandard with caching support.
///
/// This function first checks if compressed data is already cached, and if not,
/// compresses the data and stores the result in the cache for future use.
/// Returns both the compressed data and size, just like [`zstd_compress_data`].
///
/// Also populates a [`CompressionSizeCache`] with the compressed size for future size-only queries.
pub fn zstd_compress_data_cached(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    caches: &CacheRefs,
) -> Result<(Box<[u8]>, usize), TransformError> {
    // Calculate content hash once
    let content_hash = calculate_content_hash(data_ptr, len_bytes);

    // Try to load from cache first
    if let Some((cached_data, cached_size)) = caches
        .compressed_data_cache
        .load_compressed_data(content_hash, compression_level)?
    {
        // Data cache and size cache should be synced, so no need to write to size cache here
        return Ok((cached_data, cached_size));
    }

    // Not in cache, compress the data using the original function
    let (compressed_data, compressed_size) =
        zstd_compress_data(data_ptr, len_bytes, compression_level)?;

    // Save to cache for future use
    if let Err(e) = caches.compressed_data_cache.save_compressed_data(
        content_hash,
        compression_level,
        &compressed_data[..compressed_size],
    ) {
        // Log the error but don't fail the operation
        eprintln!("Warning: Failed to save compressed data to cache: {e}");
    }

    // Also populate the size cache when writing to data cache
    {
        let mut size_cache_guard = caches.compressed_size_cache.lock().unwrap();
        size_cache_guard.insert(content_hash, compression_level, compressed_size);
    }

    Ok((compressed_data, compressed_size))
}

/// Compresses data using ZStandard and returns both the compressed data and size.
///
/// This function allocates a buffer for the compressed data and returns ownership
/// of that buffer along with the actual compressed size.
pub fn zstd_compress_data(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
) -> Result<(Box<[u8]>, usize), TransformError> {
    let max_compressed_size = zstd::max_alloc_for_compress_size(len_bytes);
    let mut compressed_buffer =
        unsafe { Box::<[u8]>::new_uninit_slice(max_compressed_size).assume_init() };

    let compressed_size = unsafe {
        let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
        match zstd::compress(compression_level, original_slice, &mut compressed_buffer) {
            Ok(size) => size,
            Err(_) => {
                return Err(TransformError::Debug(
                    "Benchmark: Compression failed".to_owned(),
                ))
            }
        }
    };

    Ok((compressed_buffer, compressed_size))
}

/// Calculates ZStandard compressed size without caching.
/// This is a simplified version of calc_compression_stats_common::zstd_calc_size_with_cache
/// that doesn't use any caching mechanisms for pure performance measurement.
pub unsafe fn zstd_calc_size(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
) -> Result<usize, TransformError> {
    let max_compressed_size = zstd::max_alloc_for_compress_size(len_bytes);
    let mut compressed_buffer = Box::<[u8]>::new_uninit_slice(max_compressed_size).assume_init();

    let compressed_size = {
        let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
        match zstd::compress(compression_level, original_slice, &mut compressed_buffer) {
            Ok(size) => size,
            Err(_) => {
                return Err(TransformError::Debug(
                    "Benchmark: Compression failed".to_owned(),
                ))
            }
        }
    };

    Ok(compressed_size)
}

/// Calculates ZStandard compressed size with caching support.
///
/// This function first checks if the compressed size is already cached, and if not,
/// compresses the data and stores the result in the cache for future use.
pub fn zstd_calc_size_with_cache(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    cache: &Mutex<CompressionSizeCache>,
) -> Result<usize, TransformError> {
    let content_hash = calculate_content_hash(data_ptr, len_bytes);

    // Try to get from cache
    {
        let cache_guard = cache.lock().unwrap();
        if let Some(cached_size) = cache_guard.get(content_hash, compression_level) {
            return Ok(cached_size);
        }
    }

    // Not in cache, compute it
    let max_compressed_size = zstd::max_alloc_for_compress_size(len_bytes);
    let mut compressed_buffer =
        unsafe { Box::<[u8]>::new_uninit_slice(max_compressed_size).assume_init() };

    let compressed_size = unsafe {
        let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
        match zstd::compress(compression_level, original_slice, &mut compressed_buffer) {
            Ok(size) => size,
            Err(_) => {
                return Err(TransformError::Debug(
                    "Debug: Compression failed".to_owned(),
                ))
            }
        }
    };

    // Store in cache
    {
        let mut cache_guard = cache.lock().unwrap();
        cache_guard.insert(content_hash, compression_level, compressed_size);
    }

    Ok(compressed_size)
}

/// Decompresses ZStandard data into a pre-allocated buffer.
///
/// This function decompresses the provided compressed data into the given buffer.
pub fn zstd_decompress_data(
    compressed_data: &[u8],
    output_buffer: &mut [u8],
) -> Result<usize, TransformError> {
    match zstd::decompress(compressed_data, output_buffer) {
        Ok(decompressed_size) => Ok(decompressed_size),
        Err(_) => Err(TransformError::Debug(
            "Benchmark: Decompression failed".to_owned(),
        )),
    }
}
