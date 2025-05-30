//! Common functionality for compression statistics analysis across BC1, BC2, BC3, and BC7 formats.
//!
//! This module provides shared data structures, utilities, and analysis functions that can be
//! reused across different BC format implementations for compression statistics analysis.

use crate::error::TransformError;
use bytesize::ByteSize;
use core::{fmt::Debug, slice};
use std::{collections::HashMap, hash::Hash, path::Path, sync::Mutex};

use super::{calculate_content_hash, compression_size_cache::CompressionSizeCache, zstd};

/// Generic compression statistics result for a single file analysis.
///
/// Type parameter `T` represents the transform details type (e.g., [`Bc1TransformDetails`]).
///
/// [`Bc1TransformDetails`]: dxt_lossless_transform_bc1::Bc1TransformDetails
#[derive(Debug, Clone, PartialEq, Hash, Default)]
pub struct CompressionStatsResult<T>
where
    T: Copy + Debug + PartialEq + Eq + Hash + Default,
{
    pub file_path: String,
    pub original_uncompressed_size: usize,
    pub original_compressed_size: usize,
    pub all_results: Vec<TransformResult<T>>,
    pub api_recommended_result: TransformResult<T>,
}

impl<T> CompressionStatsResult<T>
where
    T: Copy + Debug + PartialEq + Eq + Hash + Default,
{
    /// Finds the transform result that achieves the best (smallest) compression size.
    /// This is an O(n) operation where n is the number of transform results; but
    /// n tends to be rather small. (e.g. 12 for BC1)
    pub fn find_best_result(&self) -> TransformResult<T> {
        let mut best_result = TransformResult::default();
        let mut best_size = usize::MAX;

        for result in &self.all_results {
            if result.compressed_size < best_size {
                best_size = result.compressed_size;
                best_result = *result;
            }
        }

        best_result
    }

    /// Finds the transform result for the default transform configuration.
    /// For the purposes of fine tuning.
    pub fn find_default_result(&self) -> TransformResult<T> {
        let default_transform = T::default();
        self.all_results
            .iter()
            .find(|result| result.transform_options == default_transform)
            .copied()
            .expect("Default transform should always be present in all_combinations()")
    }
}

/// Result of applying a specific transform and measuring its compressed size.
///
/// Type parameter `T` represents the transform details type (e.g., [`Bc1TransformDetails`]).
///
/// [`Bc1TransformDetails`]: dxt_lossless_transform_bc1::Bc1TransformDetails
#[derive(Debug, Clone, PartialEq, Hash, Default, Copy)]
pub struct TransformResult<T>
where
    T: Copy + Debug + PartialEq + Eq + Hash + Default,
{
    pub transform_options: T,
    pub compressed_size: usize,
}

impl<T> TransformResult<T>
where
    T: Copy + Debug + PartialEq + Eq + Hash + Default,
{
    /// Calculates the compression ratio (compressed_size / original_size).
    pub fn compression_ratio(&self, original_size: usize) -> f64 {
        self.compressed_size as f64 / original_size.max(1) as f64
    }
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

/// Formats a byte count as a human-readable string using [`ByteSize`].
pub fn format_bytes(bytes: usize) -> String {
    format!("{:.3}", ByteSize::b(bytes as u64))
}

/// Extracts the filename from a full path string.
pub fn get_filename(full_path: &str) -> String {
    Path::new(full_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| full_path.to_string())
}

/// Prints the analysis results for a single file.
///
/// This function takes a format transform details formatter function to handle
/// format-specific transform detail formatting.
pub fn print_analyzed_file<T, F>(result: &CompressionStatsResult<T>, format_transform_details: F)
where
    T: Copy + Debug + PartialEq + Eq + Hash + Default,
    F: Fn(T) -> String,
{
    let best_result = result.find_best_result();
    let default_result = result.find_default_result();
    let api_result = &result.api_recommended_result;

    let ratio_old =
        result.original_compressed_size as f64 / result.original_uncompressed_size.max(1) as f64;
    let ratio_new = best_result.compression_ratio(result.original_uncompressed_size);
    let ratio_default = default_result.compression_ratio(result.original_uncompressed_size);
    let ratio_api = api_result.compression_ratio(result.original_uncompressed_size);
    let ratio_improvement = ratio_old - ratio_new;
    let default_improvement = ratio_old - ratio_default;
    let api_improvement = ratio_old - ratio_api;

    // Check if API recommendation matches the best result
    let api_matches_best = api_result.transform_options == best_result.transform_options;
    let api_indicator = if api_matches_best { "✓" } else { "✗" };

    println!(
        "✓ Analyzed {}: orig/default/api/best: {}/{}/{}/{}, ratio orig/default/api/best: {:.3}/{:.3}/{:.3}/{:.3} (-{:.3}/-{:.3}/-{:.3}), space saved default/api/best: {}/{}/{}, best method: {}, api method: {} {}",
        get_filename(&result.file_path),               // name
        format_bytes(result.original_compressed_size), // orig
        format_bytes(default_result.compressed_size),  // default
        format_bytes(api_result.compressed_size),      // api
        format_bytes(best_result.compressed_size),     // best
        ratio_old,
        ratio_default,
        ratio_api,
        ratio_new,
        default_improvement,
        api_improvement,
        ratio_improvement,
        format_bytes(
            result
                .original_compressed_size
                .saturating_sub(default_result.compressed_size)
        ), // space saved with default
        format_bytes(
            result
                .original_compressed_size
                .saturating_sub(api_result.compressed_size)
        ), // space saved with api
        format_bytes(
            result
                .original_compressed_size
                .saturating_sub(best_result.compressed_size)
        ), // space saved with best
        format_transform_details(best_result.transform_options), // best method
        format_transform_details(api_result.transform_options),  // api method
        api_indicator // indicates if API recommendation matches best
    );
}

/// Prints comprehensive statistics for a collection of analysis results.
///
/// This function takes a format transform details formatter function to handle
/// format-specific transform detail formatting.
pub fn print_overall_statistics<T, F>(
    results: &[CompressionStatsResult<T>],
    format_transform_details: F,
) where
    T: Copy + Debug + PartialEq + Eq + Hash + Default,
    F: Fn(T) -> String,
{
    if results.is_empty() {
        println!("\n📊 No files analyzed.");
        return;
    }

    println!("\n📊 Overall Statistics:");
    println!("═══════════════════════════════════════════════════════════════");

    // Calculate totals
    let total_original_uncompressed: usize =
        results.iter().map(|r| r.original_uncompressed_size).sum();
    let total_original_compressed: usize = results.iter().map(|r| r.original_compressed_size).sum();
    let total_best_compressed: usize = results
        .iter()
        .map(|r| r.find_best_result().compressed_size)
        .sum();
    let total_default_compressed: usize = results
        .iter()
        .map(|r| r.find_default_result().compressed_size)
        .sum();
    let total_api_compressed: usize = results
        .iter()
        .map(|r| r.api_recommended_result.compressed_size)
        .sum();

    // Calculate ratios
    let original_ratio =
        total_original_compressed as f64 / total_original_uncompressed.max(1) as f64;
    let best_ratio = total_best_compressed as f64 / total_original_uncompressed.max(1) as f64;
    let default_ratio = total_default_compressed as f64 / total_original_uncompressed.max(1) as f64;
    let api_ratio = total_api_compressed as f64 / total_original_uncompressed.max(1) as f64;
    let improvement_ratio = original_ratio - best_ratio;
    let default_improvement_ratio = original_ratio - default_ratio;
    let api_improvement_ratio = original_ratio - api_ratio;
    let space_saved = total_original_compressed.saturating_sub(total_best_compressed);
    let default_space_saved = total_original_compressed.saturating_sub(total_default_compressed);
    let api_space_saved = total_original_compressed.saturating_sub(total_api_compressed);

    // Count most common methods
    let mut method_counts = HashMap::new();
    let mut api_method_counts = HashMap::new();
    for result in results {
        let best_result = result.find_best_result();
        *method_counts
            .entry(best_result.transform_options)
            .or_insert(0) += 1;
        *api_method_counts
            .entry(result.api_recommended_result.transform_options)
            .or_insert(0) += 1;
    }

    let mut most_common_methods = method_counts
        .iter()
        .map(|(&method, &count)| (method, count))
        .collect::<Vec<_>>();
    most_common_methods.sort_by_key(|&(_, count)| -count);

    let mut most_common_api_method = api_method_counts
        .iter()
        .map(|(&method, &count)| (method, count))
        .collect::<Vec<_>>();
    most_common_api_method.sort_by_key(|&(_, count)| -count);

    // Calculate API recommendation accuracy
    let api_matches_best_count = results
        .iter()
        .filter(|r| {
            r.api_recommended_result.transform_options == r.find_best_result().transform_options
        })
        .count();
    let api_accuracy = (api_matches_best_count as f64 / results.len() as f64) * 100.0;

    // Calculate how close API recommendations are to best results
    let mut total_api_vs_best_diff = 0.0;
    for result in results {
        let api_ratio = result
            .api_recommended_result
            .compression_ratio(result.original_uncompressed_size);
        let best_ratio = result
            .find_best_result()
            .compression_ratio(result.original_uncompressed_size);
        total_api_vs_best_diff += (api_ratio - best_ratio).abs();
    }
    let avg_api_vs_best_diff = total_api_vs_best_diff / results.len() as f64;

    // Print statistics
    println!("Files analyzed: {}", results.len());
    println!(
        "Total original uncompressed size: {}",
        format_bytes(total_original_uncompressed)
    );
    println!(
        "Total original compressed size: {}",
        format_bytes(total_original_compressed)
    );
    println!(
        "Total default compressed size: {}",
        format_bytes(total_default_compressed)
    );
    println!(
        "Total API recommended compressed size: {}",
        format_bytes(total_api_compressed)
    );
    println!(
        "Total best compressed size: {}",
        format_bytes(total_best_compressed)
    );
    println!();
    println!("Compression ratios:");
    println!("  Original (no transform): {original_ratio:.3}");
    println!("  Default (None/YCoCg1/Split): {default_ratio:.3}");
    println!("  API recommended: {api_ratio:.3}");
    println!("  Best (brute force): {best_ratio:.3}");
    println!("  Default improvement: -{default_improvement_ratio:.3}");
    println!("  API improvement: -{api_improvement_ratio:.3}");
    println!("  Best improvement: -{improvement_ratio:.3}");
    println!();
    println!(
        "Total space saved with default: {} ({:.1}% reduction)",
        format_bytes(default_space_saved),
        (default_space_saved as f64 / total_original_compressed.max(1) as f64) * 100.0
    );
    println!(
        "Total space saved with API: {} ({:.1}% reduction)",
        format_bytes(api_space_saved),
        (api_space_saved as f64 / total_original_compressed.max(1) as f64) * 100.0
    );
    println!(
        "Total space saved with best: {} ({:.1}% reduction)",
        format_bytes(space_saved),
        (space_saved as f64 / total_original_compressed.max(1) as f64) * 100.0
    );
    println!();
    println!("📈 API Recommendation Analysis:");
    println!(
        "  API accuracy (matches best): {}/{} files ({:.1}%)",
        api_matches_best_count,
        results.len(),
        api_accuracy
    );
    println!("  Average ratio difference (API vs best): {avg_api_vs_best_diff:.6}");

    println!("  Most common best methods: ");
    for (method, count) in most_common_methods.iter().take(3) {
        let percentage = (*count as f64 / results.len() as f64) * 100.0;
        println!(
            "    - {} ({count} files, {percentage:.1}%)",
            format_transform_details(*method)
        );
    }

    println!("  Most common API methods: ");
    for (method, count) in most_common_api_method.iter().take(3) {
        let percentage = (*count as f64 / results.len() as f64) * 100.0;
        println!(
            "    - {} ({count} files, {percentage:.1}%)",
            format_transform_details(*method)
        );
    }

    println!("═══════════════════════════════════════════════════════════════");
}
