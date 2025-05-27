use super::CompressionStatsCmd;
use crate::{
    debug::{extract_blocks_from_dds, zstd},
    error::TransformError,
    util::find_all_files,
    DdsFilter,
};
use bytesize::ByteSize;
use core::{
    fmt::Debug,
    slice,
    sync::atomic::{AtomicUsize, Ordering},
};
use dxt_lossless_transform_api::DdsFormat;
use dxt_lossless_transform_bc1::{
    determine_optimal_transform::{determine_best_transform_details, Bc1TransformOptions},
    transform_bc1, Bc1TransformDetails,
};
use dxt_lossless_transform_common::allocate::allocate_align_64;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};
use std::{collections::HashMap, fs, path::Path, sync::Mutex};

#[derive(Debug, Clone, PartialEq, Hash, Default)]
struct CompressionStatsResult {
    file_path: String,
    original_uncompressed_size: usize,
    original_compressed_size: usize,
    all_results: Vec<TransformResult>,
    api_recommended_result: TransformResult,
}

impl CompressionStatsResult {
    fn find_best_result(&self) -> TransformResult {
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

    fn find_default_result(&self) -> TransformResult {
        let default_transform = Bc1TransformDetails::default();
        self.all_results
            .iter()
            .find(|result| result.transform_options == default_transform)
            .copied()
            .expect("Default transform should always be present in all_combinations()")
    }
}

#[derive(Debug, Clone, PartialEq, Hash, Default, Copy)]
struct TransformResult {
    transform_options: Bc1TransformDetails,
    compressed_size: usize,
}

impl TransformResult {
    fn compression_ratio(&self, original_size: usize) -> f64 {
        self.compressed_size as f64 / original_size as f64
    }
}

pub(crate) fn handle_compression_stats_command(
    cmd: CompressionStatsCmd,
) -> Result<(), TransformError> {
    let input_path = &cmd.input_directory;
    println!(
        "Analyzing BC1 compression statistics for files in: {} (recursive)",
        input_path.display()
    );
    println!("Compression level: {}", cmd.compression_level);
    println!("API compression level: {}", cmd.estimate_compression_level);

    // Initialize and load cache
    let mut cache = CompressionCache::new();
    if let Err(e) = cache.load_from_disk() {
        println!("Warning: Failed to load cache: {e}");
    } else {
        println!("Loaded compression cache with {} entries", cache.len());
    }
    let cache = Mutex::new(cache);

    // Collect all files recursively using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to analyze", entries.len());

    let files_analyzed = AtomicUsize::new(0);
    let results = Mutex::new(Vec::<CompressionStatsResult>::new());

    // Process files in parallel
    entries
        .par_iter()
        // 1 item at once per thread. Our items are big generally, and take time to process
        // so 'max work stealing' is preferred.
        .with_max_len(1)
        .for_each(|entry| {
            match analyze_bc1_compression_file(
                entry,
                cmd.compression_level,
                cmd.estimate_compression_level,
                &cache,
            ) {
                Ok(file_result) => {
                    files_analyzed.fetch_add(1, Ordering::Relaxed);
                    print_analyzed_file(&file_result);
                    results.lock().unwrap().push(file_result);
                }
                Err(e) => {
                    println!("✗ Error analyzing {}: {}", entry.path().display(), e);
                }
            }
        });

    // Save cache
    let cache = cache.into_inner().unwrap();
    println!("Saving compression cache with {} entries", cache.len());
    if let Err(e) = cache.save_to_disk() {
        println!("Warning: Failed to save cache: {e}");
    }

    // Print overall statistics
    let results = results.into_inner().unwrap();
    print_overall_statistics(&results);

    Ok(())
}

unsafe fn analyze_bc1_api_recommendation(
    data_ptr: *const u8,
    len_bytes: usize,
    estimate_compression_level: i32,
    final_compression_level: i32,
    cache: &Mutex<CompressionCache>,
) -> Result<TransformResult, TransformError> {
    // Create the zstandard file size estimator with cache clone for static lifetime
    let estimator = {
        let cache_clone = cache as *const Mutex<CompressionCache>;
        move |data_ptr: *const u8, len: usize| -> usize {
            let cache_ref = unsafe { &*cache_clone };
            match zstd_calc_size_with_cache(data_ptr, len, estimate_compression_level, cache_ref) {
                Ok(size) => size,
                Err(_) => usize::MAX, // Return max size on error to make this option less favorable
            }
        }
    };

    // Create transform options
    let transform_options = Bc1TransformOptions {
        file_size_estimator: Box::new(estimator),
    };

    // Determine the best transform details using the API
    let best_details = determine_best_transform_details(data_ptr, len_bytes, transform_options)
        .map_err(|e| TransformError::Debug(format!("API recommendation failed: {e}")))?;

    // Transform the data using the recommended details and measure the size
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut work_buffer = allocate_align_64(len_bytes)?;

    transform_bc1(
        data_ptr,
        transformed_data.as_mut_ptr(),
        work_buffer.as_mut_ptr(),
        len_bytes,
        best_details,
    );

    // Compress the transformed data (API recommendation, final level)
    let compressed_size = zstd_calc_size_with_cache(
        transformed_data.as_ptr(),
        len_bytes,
        final_compression_level,
        cache,
    )?;

    Ok(TransformResult {
        transform_options: best_details,
        compressed_size,
    })
}

fn print_analyzed_file(result: &CompressionStatsResult) {
    let best_result = result.find_best_result();
    let default_result = result.find_default_result();
    let api_result = &result.api_recommended_result;

    let ratio_old =
        result.original_compressed_size as f64 / result.original_uncompressed_size as f64;
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

fn analyze_bc1_compression_file(
    entry: &fs::DirEntry,
    compression_level: i32,
    estimate_compression_level: i32,
    cache: &Mutex<CompressionCache>,
) -> Result<CompressionStatsResult, TransformError> {
    let mut file_result: CompressionStatsResult = CompressionStatsResult::default();

    unsafe {
        extract_blocks_from_dds(
            entry,
            DdsFilter::BC1,
            |data_ptr: *const u8,
             len_bytes: usize,
             format: DdsFormat|
             -> Result<(), TransformError> {
                // Only analyze BC1 blocks
                if format != DdsFormat::BC1 {
                    return Ok(()); // Skip non-BC1 data
                }

                file_result = CompressionStatsResult {
                    file_path: entry.path().display().to_string(),
                    original_uncompressed_size: len_bytes,
                    all_results: analyze_bc1_compression_transforms(
                        data_ptr,
                        len_bytes,
                        compression_level,
                        cache,
                    )?,
                    original_compressed_size: zstd_calc_size_with_cache(
                        data_ptr,
                        len_bytes,
                        compression_level,
                        cache,
                    )?,
                    api_recommended_result: analyze_bc1_api_recommendation(
                        data_ptr,
                        len_bytes,
                        estimate_compression_level,
                        compression_level,
                        cache,
                    )?,
                };

                Ok(())
            },
        )?;
    }

    Ok(file_result)
}

fn analyze_bc1_compression_transforms(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    cache: &Mutex<CompressionCache>,
) -> Result<Vec<TransformResult>, TransformError> {
    // Allocate aligned buffers for transformations
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut work_buffer = allocate_align_64(len_bytes)?;

    let mut results = Vec::new();
    unsafe {
        // Test all transform combinations
        for transform_options in Bc1TransformDetails::all_combinations() {
            // Transform the data
            transform_bc1(
                data_ptr,
                transformed_data.as_mut_ptr(),
                work_buffer.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Compress the transformed data
            results.push(TransformResult {
                transform_options,
                compressed_size: zstd_calc_size_with_cache(
                    transformed_data.as_ptr(),
                    len_bytes,
                    compression_level,
                    cache,
                )?,
            });
        }
    }

    Ok(results)
}

fn zstd_calc_size_with_cache(
    data_ptr: *const u8,
    len_bytes: usize,
    compression_level: i32,
    cache: &Mutex<CompressionCache>,
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

/// Formats a byte count as a human-readable string
fn format_bytes(bytes: usize) -> String {
    ByteSize::b(bytes as u64).to_string()
}

/// Extracts the filename from a full path
fn get_filename(full_path: &str) -> String {
    Path::new(full_path)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| full_path.to_string())
}

/// Formats [`Bc1TransformDetails`] as a human-readable string  
fn format_transform_details(details: Bc1TransformDetails) -> String {
    let norm_mode = match details.color_normalization_mode {
        dxt_lossless_transform_bc1::normalize_blocks::ColorNormalizationMode::None => "None",
        dxt_lossless_transform_bc1::normalize_blocks::ColorNormalizationMode::Color0Only => {
            "C0Only"
        }
        dxt_lossless_transform_bc1::normalize_blocks::ColorNormalizationMode::ReplicateColor => {
            "Replicate"
        }
    };

    let decorr_mode = match details.decorrelation_mode {
        dxt_lossless_transform_common::color_565::YCoCgVariant::None => "None",
        dxt_lossless_transform_common::color_565::YCoCgVariant::Variant1 => "YCoCg1",
        dxt_lossless_transform_common::color_565::YCoCgVariant::Variant2 => "YCoCg2",
        dxt_lossless_transform_common::color_565::YCoCgVariant::Variant3 => "YCoCg3",
    };

    let split_endpoints = if details.split_colour_endpoints {
        "Split"
    } else {
        "NoSplit"
    };

    format!("{norm_mode}/{decorr_mode}/{split_endpoints}")
}

fn print_overall_statistics(results: &[CompressionStatsResult]) {
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
    let original_ratio = total_original_compressed as f64 / total_original_uncompressed as f64;
    let best_ratio = total_best_compressed as f64 / total_original_uncompressed as f64;
    let default_ratio = total_default_compressed as f64 / total_original_uncompressed as f64;
    let api_ratio = total_api_compressed as f64 / total_original_uncompressed as f64;
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

    let most_common_method = method_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&method, &count)| (method, count));

    let most_common_api_method = api_method_counts
        .iter()
        .max_by_key(|(_, &count)| count)
        .map(|(&method, &count)| (method, count));

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
        (default_space_saved as f64 / total_original_compressed as f64) * 100.0
    );
    println!(
        "Total space saved with API: {} ({:.1}% reduction)",
        format_bytes(api_space_saved),
        (api_space_saved as f64 / total_original_compressed as f64) * 100.0
    );
    println!(
        "Total space saved with best: {} ({:.1}% reduction)",
        format_bytes(space_saved),
        (space_saved as f64 / total_original_compressed as f64) * 100.0
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

    if let Some((method, count)) = most_common_method {
        let percentage = (count as f64 / results.len() as f64) * 100.0;
        println!(
            "  Most common best method: {} ({count} files, {percentage:.1}%)",
            format_transform_details(method)
        );
    }

    if let Some((method, count)) = most_common_api_method {
        let percentage = (count as f64 / results.len() as f64) * 100.0;
        println!(
            "  Most common API method: {} ({count} files, {percentage:.1}%)",
            format_transform_details(method)
        );
    }

    println!("═══════════════════════════════════════════════════════════════");
}

/// Simple compression cache that stores compressed sizes for specific inputs and compression levels
struct CompressionCache {
    /// Map from (content_hash, compression_level) -> compressed_size
    cache: BTreeMap<(u128, i32), usize>,
    /// Path to the cache file
    cache_file_path: PathBuf,
}

impl CompressionCache {
    fn new() -> Self {
        // Create cache directory in user's cache dir or fallback to current dir (Windows, etc.)
        let cache_dir = std::env::var("HOME")
            .map(|home| {
                PathBuf::from(home)
                    .join(".cache")
                    .join("dxt-lossless-transform-cli")
            })
            .unwrap_or_else(|_| PathBuf::from(".cache").join("dxt-lossless-transform-cli"));

        let cache_file_path = cache_dir.join("compression_cache.bin");

        Self {
            cache: BTreeMap::new(),
            cache_file_path,
        }
    }

    fn load_from_disk(&mut self) -> Result<(), TransformError> {
        if !self.cache_file_path.exists() {
            return Ok(()); // No cache file yet
        }

        let mut file = File::open(&self.cache_file_path)
            .map_err(|e| TransformError::Debug(format!("Failed to open cache file: {e}")))?;

        self.cache = bincode::decode_from_std_read(&mut file, bincode::config::standard())
            .map_err(|e| TransformError::Debug(format!("Failed to deserialize cache: {e}")))?;

        Ok(())
    }

    fn save_to_disk(&self) -> Result<(), TransformError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = self.cache_file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TransformError::Debug(format!("Failed to create cache directory: {e}"))
            })?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cache_file_path)
            .map_err(|e| TransformError::Debug(format!("Failed to create cache file: {e}")))?;

        bincode::encode_into_std_write(&self.cache, &mut file, bincode::config::standard())
            .map_err(|e| TransformError::Debug(format!("Failed to serialize cache: {e}")))?;

        Ok(())
    }

    fn get(&self, content_hash: u128, compression_level: i32) -> Option<usize> {
        self.cache.get(&(content_hash, compression_level)).copied()
    }

    fn insert(&mut self, content_hash: u128, compression_level: i32, compressed_size: usize) {
        self.cache
            .insert((content_hash, compression_level), compressed_size);
    }

    fn len(&self) -> usize {
        self.cache.len()
    }
}

/// Calculates XXH3-128 hash of data
fn calculate_content_hash(data_ptr: *const u8, len_bytes: usize) -> u128 {
    let data_slice = unsafe { slice::from_raw_parts(data_ptr, len_bytes) };
    xxhash_rust::xxh3::xxh3_128(data_slice)
}
