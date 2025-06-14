use super::CompressionStatsCmd;
use crate::{
    debug::{
        calc_compression_stats_common,
        compression::{
            helpers::{
                calc_size_with_cache_and_estimation_algorithm, validate_compression_algorithm,
            },
            CompressionAlgorithm,
        },
        compression_size_cache, extract_blocks_from_dds,
    },
    error::TransformError,
    util::find_all_files,
    DdsFilter,
};
use core::sync::atomic::{AtomicUsize, Ordering};
use dxt_lossless_transform_api::DdsFormat;
use dxt_lossless_transform_bc1::{
    determine_optimal_transform::{determine_best_transform_details, Bc1EstimateOptions},
    experimental::normalize_blocks::ColorNormalizationMode,
    transform_bc1, Bc1TransformDetails,
};
use dxt_lossless_transform_common::{allocate::allocate_align_64, color_565::YCoCgVariant};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{fs, sync::Mutex};

// Type aliases for BC1-specific compression stats
type Bc1CompressionStatsResult =
    calc_compression_stats_common::CompressionStatsResult<Bc1TransformDetails>;
type Bc1TransformResult = calc_compression_stats_common::TransformResult<Bc1TransformDetails>;
type CompressionCache = compression_size_cache::CompressionSizeCache;

pub(crate) fn handle_compression_stats_command(
    cmd: CompressionStatsCmd,
) -> Result<(), TransformError> {
    validate_compression_algorithm(cmd.compression_algorithm)?;

    let input_path = &cmd.input_directory;
    println!(
        "Analyzing BC1 compression statistics for files in: {} (recursive)",
        input_path.display()
    );
    println!(
        "Compression algorithm: {} , level: {}",
        cmd.compression_algorithm.name(),
        cmd.get_compression_level()
    );
    println!(
        "Estimate (API) compression algorithm: {} , level: {}",
        cmd.get_estimate_compression_algorithm().name(),
        cmd.get_estimate_compression_level()
    );

    // Initialize and load cache
    let mut cache = CompressionCache::new();
    if let Err(e) = cache.load_from_disk() {
        println!("Warning: Failed to load cache: {e}");
    } else {
        println!("Loaded compression size cache with {} entries", cache.len());
    }
    let cache = Mutex::new(cache);

    // Collect all files recursively using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to analyze", entries.len());

    let files_analyzed = AtomicUsize::new(0);
    let results = Mutex::new(Vec::<Bc1CompressionStatsResult>::new());

    // Process files in parallel
    entries
        .par_iter()
        // 1 item at once per thread. Our items are big generally, and take time to process
        // so 'max work stealing' is preferred.
        .with_max_len(1)
        .for_each(|entry| {
            match analyze_bc1_compression_file(
                entry,
                cmd.get_compression_level(),
                cmd.get_estimate_compression_level(),
                cmd.compression_algorithm,
                cmd.get_estimate_compression_algorithm(),
                cmd.experimental_normalize,
                &cache,
            ) {
                Ok(file_result) => {
                    files_analyzed.fetch_add(1, Ordering::Relaxed);
                    calc_compression_stats_common::print_analyzed_file(
                        &file_result,
                        format_transform_details,
                    );
                    results.lock().unwrap().push(file_result);
                }
                Err(e) => {
                    println!("✗ Error analyzing {}: {}", entry.path().display(), e);
                }
            }
        });

    // Save cache
    let cache = cache.into_inner().unwrap();
    println!("Saving compression size cache with {} entries", cache.len());
    if let Err(e) = cache.save_to_disk() {
        println!("Warning: Failed to save cache: {e}");
    }

    // Print overall statistics
    let results = results.into_inner().unwrap();
    calc_compression_stats_common::print_overall_statistics(&results, format_transform_details);

    Ok(())
}

fn analyze_bc1_compression_file(
    entry: &fs::DirEntry,
    compression_level: i32,
    estimate_compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    estimate_compression_algorithm: CompressionAlgorithm,
    experimental_normalize: bool,
    cache: &Mutex<CompressionCache>,
) -> Result<Bc1CompressionStatsResult, TransformError> {
    let mut file_result: Bc1CompressionStatsResult = Bc1CompressionStatsResult::default();

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

                file_result = Bc1CompressionStatsResult {
                    file_path: entry.path().display().to_string(),
                    original_uncompressed_size: len_bytes,
                    all_results: analyze_bc1_compression_transforms(
                        data_ptr,
                        len_bytes,
                        compression_level,
                        compression_algorithm,
                        cache,
                    )?,
                    original_compressed_size: calc_size_with_cache_and_estimation_algorithm(
                        data_ptr,
                        len_bytes,
                        compression_level,
                        compression_algorithm,
                        cache,
                    )?,
                    api_recommended_result: analyze_bc1_api_recommendation(
                        data_ptr,
                        len_bytes,
                        estimate_compression_level,
                        estimate_compression_algorithm,
                        compression_level,
                        compression_algorithm,
                        experimental_normalize,
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
    compression_algorithm: CompressionAlgorithm,
    cache: &Mutex<CompressionCache>,
) -> Result<Vec<Bc1TransformResult>, TransformError> {
    // Allocate aligned buffers for transformations
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut results = Vec::new();
    unsafe {
        // Test all transform combinations
        for transform_options in Bc1TransformDetails::all_combinations() {
            // Transform the data
            transform_bc1(
                data_ptr,
                transformed_data.as_mut_ptr(),
                len_bytes,
                transform_options,
            );

            // Compress the transformed data
            results.push(Bc1TransformResult {
                transform_options,
                compressed_size: calc_size_with_cache_and_estimation_algorithm(
                    transformed_data.as_ptr(),
                    len_bytes,
                    compression_level,
                    compression_algorithm,
                    cache,
                )?,
            });
        }
    }

    Ok(results)
}

#[allow(clippy::too_many_arguments)]
unsafe fn analyze_bc1_api_recommendation(
    data_ptr: *const u8,
    len_bytes: usize,
    estimate_compression_level: i32,
    estimate_compression_algorithm: CompressionAlgorithm,
    final_compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    experimental_normalize: bool,
    cache: &Mutex<CompressionCache>,
) -> Result<Bc1TransformResult, TransformError> {
    // Create the file size estimator with cache clone for static lifetime
    let estimator = move |data_ptr: *const u8, len: usize| -> usize {
        match calc_size_with_cache_and_estimation_algorithm(
            data_ptr,
            len,
            estimate_compression_level,
            estimate_compression_algorithm,
            cache,
        ) {
            Ok(size) => size,
            Err(e) => {
                eprintln!("Warning: Compression estimation failed: {e}");
                usize::MAX // Return max size on error to make this option less favorable
            }
        }
    };

    // Create transform options
    let transform_options = Bc1EstimateOptions {
        file_size_estimator: estimator,
        test_normalize_options: experimental_normalize,
    };

    // Determine the best transform details using the API
    let best_details = determine_best_transform_details(data_ptr, len_bytes, transform_options)
        .map_err(|e| TransformError::Debug(format!("API recommendation failed: {e}")))?;

    // Transform the data using the recommended details and measure the size
    let mut transformed_data = allocate_align_64(len_bytes)?;
    transform_bc1(
        data_ptr,
        transformed_data.as_mut_ptr(),
        len_bytes,
        best_details,
    );

    // Compress the transformed data (API recommendation, final level)
    let compressed_size = calc_size_with_cache_and_estimation_algorithm(
        transformed_data.as_ptr(),
        len_bytes,
        final_compression_level,
        compression_algorithm,
        cache,
    )?;

    Ok(Bc1TransformResult {
        transform_options: best_details,
        compressed_size,
    })
}

/// Formats [`Bc1TransformDetails`] as a human-readable string  
fn format_transform_details(details: Bc1TransformDetails) -> String {
    let norm_mode = match details.color_normalization_mode {
        ColorNormalizationMode::None => "None",
        ColorNormalizationMode::Color0Only => "C0Only",
        ColorNormalizationMode::ReplicateColor => "Replicate",
    };

    let decorr_mode = match details.decorrelation_mode {
        YCoCgVariant::None => "None",
        YCoCgVariant::Variant1 => "YCoCg1",
        YCoCgVariant::Variant2 => "YCoCg2",
        YCoCgVariant::Variant3 => "YCoCg3",
    };

    let split_endpoints = if details.split_colour_endpoints {
        "Split"
    } else {
        "NoSplit"
    };

    format!("{norm_mode}/{decorr_mode}/{split_endpoints}")
}
