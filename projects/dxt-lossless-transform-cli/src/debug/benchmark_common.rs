//! Common functionality for benchmarking performance across BC1, BC2, BC3, and BC7 formats.
//!
//! This module provides shared data structures, utilities, and benchmarking functions that can be
//! reused across different BC format implementations for performance analysis.

use bytesize::ByteSize;

use super::{
    calculate_content_hash, compressed_data_cache::CompressedDataCache,
    compression_size_cache::CompressionSizeCache,
};
use crate::{
    debug::{calc_compression_stats_common::get_filename, zstd},
    error::TransformError,
};
use core::{fmt::Debug, slice};
use std::{collections::HashMap, sync::Mutex, time::Instant};

/// References to the caches used during benchmarking
pub struct CacheRefs<'a> {
    pub compressed_size_cache: &'a Mutex<CompressionSizeCache>,
    pub compressed_data_cache: &'a CompressedDataCache,
}

/// Result of benchmarking a specific transform scenario.
#[derive(Debug, Clone)]
pub struct BenchmarkScenarioResult {
    pub scenario_name: String,
    pub decompress_time_ms: f64,
    pub detransform_time_ms: f64,
    pub combined_time_ms: f64,
    pub decompress_throughput: ByteSize,
    pub detransform_throughput: ByteSize,
    pub combined_throughput: ByteSize,
}

impl BenchmarkScenarioResult {
    pub fn new(
        scenario_name: String,
        file_size_bytes: usize,
        decompress_time_ms: f64,
        detransform_time_ms: f64,
    ) -> Self {
        let combined_time_ms = decompress_time_ms + detransform_time_ms;

        // Calculate throughput in GiB/s
        let decompress_time_s = decompress_time_ms / 1000.0;
        let detransform_time_s = detransform_time_ms / 1000.0;
        let combined_time_s = combined_time_ms / 1000.0;

        let decompress_throughput_bytes_per_sec = if decompress_time_s > 0.0 {
            file_size_bytes as f64 / decompress_time_s
        } else {
            0.0
        } as u64;

        let detransform_throughput_bytes_per_sec = if detransform_time_s > 0.0 {
            file_size_bytes as f64 / detransform_time_s
        } else {
            0.0
        } as u64;

        let combined_throughput_bytes_per_sec = if combined_time_s > 0.0 {
            file_size_bytes as f64 / combined_time_s
        } else {
            0.0
        } as u64;

        Self {
            scenario_name,
            decompress_time_ms,
            detransform_time_ms,
            combined_time_ms,
            decompress_throughput: ByteSize(decompress_throughput_bytes_per_sec),
            detransform_throughput: ByteSize(detransform_throughput_bytes_per_sec),
            combined_throughput: ByteSize(combined_throughput_bytes_per_sec),
        }
    }
}

/// Result of benchmarking a single file with multiple scenarios.
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub file_path: String,
    pub file_size_bytes: usize,
    pub scenarios: Vec<BenchmarkScenarioResult>,
}

impl BenchmarkResult {
    pub fn new(file_path: String, file_size_bytes: usize) -> Self {
        Self {
            file_path,
            file_size_bytes,
            scenarios: Vec::new(),
        }
    }

    pub fn add_scenario(&mut self, scenario: BenchmarkScenarioResult) {
        self.scenarios.push(scenario);
    }
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

/// Measures the time taken to execute a function and converts it to milliseconds.
pub fn measure_time<F, R>(func: F) -> (R, f64)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = func();
    let duration = start.elapsed();
    (result, duration.as_secs_f64() * 1000.0)
}

/// Prints the results for a single file's benchmark.
pub fn print_file_result(result: &BenchmarkResult) {
    let filename = get_filename(&result.file_path);

    println!(
        "✓ Benchmarked {}: size: {:.3} MiB",
        filename,
        result.file_size_bytes as f64 / (1024.0 * 1024.0)
    );

    for scenario in &result.scenarios {
        println!(
            "  {}: decompress: {:.2} ms ({:.1}/s), detransform: {:.2} ms ({:.1}/s), combined: {:.2} ms ({:.1}/s)",
            scenario.scenario_name,
            scenario.decompress_time_ms,
            scenario.decompress_throughput,
            scenario.detransform_time_ms,
            scenario.detransform_throughput,
            scenario.combined_time_ms,
            scenario.combined_throughput
        );
    }
}

/// Prints overall benchmark statistics across all files and scenarios.
pub fn print_overall_statistics(results: &[BenchmarkResult]) {
    if results.is_empty() {
        println!("\n📊 No files benchmarked.");
        return;
    }

    println!("\n📊 Overall Benchmark Statistics:");
    println!("═══════════════════════════════════════════════════════════════");

    let total_files = results.len();
    let total_size_bytes: usize = results.iter().map(|r| r.file_size_bytes).sum();
    let total_size_mib = total_size_bytes as f64 / (1024.0 * 1024.0);

    println!("Files benchmarked: {total_files}");
    println!("Total data processed: {total_size_mib:.3} MiB");
    println!();

    // Collect all scenarios from all files with their file sizes
    let mut all_scenarios: Vec<(&str, &BenchmarkScenarioResult, usize)> = Vec::new();
    for result in results {
        for scenario in &result.scenarios {
            all_scenarios.push((&scenario.scenario_name, scenario, result.file_size_bytes));
        }
    }

    // Group scenarios by name and calculate averages
    let mut scenario_groups: HashMap<&str, Vec<(&BenchmarkScenarioResult, usize)>> = HashMap::new();
    for (name, scenario, file_size) in all_scenarios {
        scenario_groups
            .entry(name)
            .or_default()
            .push((scenario, file_size));
    }

    // Struct to hold scenario statistics for sorting
    struct ScenarioStats {
        scenario_name: String,
        avg_decompress_throughput: f64,
        avg_detransform_throughput: f64,
        avg_combined_throughput: f64,
        total_decompress_time_ms: f64,
        total_detransform_time_ms: f64,
        total_combined_time_ms: f64,
    }

    // Calculate statistics for each scenario type
    let mut scenario_stats: Vec<ScenarioStats> = Vec::new();
    for (scenario_name, scenario_data) in scenario_groups {
        if scenario_data.is_empty() {
            continue;
        }

        // Calculate weighted average throughput based on total data and total time
        let total_size_bytes: usize = scenario_data.iter().map(|(_, file_size)| *file_size).sum();
        let total_size_gib = total_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        let total_decompress_time_s: f64 = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.decompress_time_ms / 1000.0)
            .sum();
        let total_detransform_time_s: f64 = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.detransform_time_ms / 1000.0)
            .sum();
        let total_combined_time_s: f64 = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.combined_time_ms / 1000.0)
            .sum();

        // Calculate weighted average throughput
        let avg_decompress_throughput = if total_decompress_time_s > 0.0 {
            total_size_gib / total_decompress_time_s
        } else {
            0.0
        };
        let avg_detransform_throughput = if total_detransform_time_s > 0.0 {
            total_size_gib / total_detransform_time_s
        } else {
            0.0
        };
        let avg_combined_throughput = if total_combined_time_s > 0.0 {
            total_size_gib / total_combined_time_s
        } else {
            0.0
        };

        // Calculate total times in milliseconds
        let total_decompress_time_ms: f64 = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.decompress_time_ms)
            .sum();
        let total_detransform_time_ms: f64 = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.detransform_time_ms)
            .sum();
        let total_combined_time_ms: f64 = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.combined_time_ms)
            .sum();

        scenario_stats.push(ScenarioStats {
            scenario_name: scenario_name.to_string(),
            avg_decompress_throughput,
            avg_detransform_throughput,
            avg_combined_throughput,
            total_decompress_time_ms,
            total_detransform_time_ms,
            total_combined_time_ms,
        });
    }

    // Sort by combined throughput (descending - fastest first)
    scenario_stats.sort_by(|a, b| {
        b.avg_combined_throughput
            .partial_cmp(&a.avg_combined_throughput)
            .unwrap()
    });

    // Print statistics for each scenario type
    for stats in scenario_stats {
        println!("📈 {}:", stats.scenario_name);
        println!(
            "  Decompress: avg {:.2} GiB/s, total {:.2} ms",
            stats.avg_decompress_throughput, stats.total_decompress_time_ms
        );
        println!(
            "  Detransform: avg {:.2} GiB/s, total {:.2} ms",
            stats.avg_detransform_throughput, stats.total_detransform_time_ms
        );
        println!(
            "  Combined: avg {:.2} GiB/s, total {:.2} ms",
            stats.avg_combined_throughput, stats.total_combined_time_ms
        );
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════");
}
