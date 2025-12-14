use super::{determine_best_transform_details_with_estimator_cached, CompressionStatsCmd};
use crate::{
    debug_format::{
        calc_compression_stats_common,
        compression::{
            helpers::{
                calc_size_with_cache_and_estimation_algorithm, validate_compression_algorithm,
            },
            CompressionAlgorithm,
        },
        compression_size_cache, extract_blocks_from_file, handle_debug_error,
    },
    error::TransformError,
    util::find_all_files,
};
use core::sync::atomic::{AtomicUsize, Ordering};
use dxt_lossless_transform_bc2::{transform_bc2_with_settings, Bc2TransformSettings};
use dxt_lossless_transform_common::{allocate::allocate_align_64, color_565::YCoCgVariant};
use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
use dxt_lossless_transform_file_formats_debug::TransformFormatFilter;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{fs, sync::Mutex};

// Type aliases for BC2-specific compression stats
type Bc2CompressionStatsResult =
    calc_compression_stats_common::CompressionStatsResult<Bc2TransformSettings>;
type Bc2TransformResult = calc_compression_stats_common::TransformResult<Bc2TransformSettings>;
type CompressionCache = compression_size_cache::CompressionSizeCache;

pub(crate) fn handle_compression_stats_command(
    cmd: CompressionStatsCmd,
) -> Result<(), TransformError> {
    validate_compression_algorithm(cmd.compression_algorithm)?;

    let input_path = &cmd.input_directory;
    println!(
        "Analyzing BC2 compression statistics for files in: {} (recursive)",
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

    // Filter by file size if max_size is specified
    if let Some(max_size) = cmd.max_size {
        println!("Filtering files by maximum size: {max_size} bytes");
        let original_count = entries.len();

        entries.retain(|entry| {
            match entry.metadata() {
                Ok(metadata) => {
                    let file_size = metadata.len();
                    file_size <= max_size
                }
                Err(_) => {
                    // If we can't get metadata, skip the file
                    eprintln!(
                        "Warning: Could not get metadata for {}, skipping",
                        entry.path().display()
                    );
                    false
                }
            }
        });

        let filtered_count = entries.len();
        let excluded_count = original_count - filtered_count;
        println!(
            "Filtered {filtered_count} files (excluded {excluded_count} files larger than {max_size} bytes)"
        );
    }

    println!("Found {} files to analyze", entries.len());

    let files_analyzed = AtomicUsize::new(0);
    let results = Mutex::new(Vec::<Bc2CompressionStatsResult>::new());

    // Process files in parallel
    entries
        .par_iter()
        // 1 item at once per thread. Our items are big generally, and take time to process
        // so 'max work stealing' is preferred.
        .with_max_len(1)
        .for_each(|entry| {
            match analyze_bc2_compression_file(
                entry,
                cmd.get_compression_level(),
                cmd.get_estimate_compression_level(),
                cmd.compression_algorithm,
                cmd.get_estimate_compression_algorithm(),
                cmd.use_all_decorrelation_modes,
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
                    handle_debug_error(&entry.path(), "analyzing", Err(e));
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

#[allow(clippy::too_many_arguments)]
fn analyze_bc2_compression_file(
    entry: &fs::DirEntry,
    compression_level: i32,
    estimate_compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    estimate_compression_algorithm: CompressionAlgorithm,
    use_all_decorrelation_modes: bool,
    cache: &Mutex<CompressionCache>,
) -> Result<Bc2CompressionStatsResult, TransformError> {
    let mut file_result: Bc2CompressionStatsResult = Bc2CompressionStatsResult::default();

    extract_blocks_from_file(
        &entry.path(),
        TransformFormatFilter::Bc2,
        |data: &[u8], _format: TransformFormat| -> Result<(), TransformError> {
            file_result = Bc2CompressionStatsResult {
                file_path: entry.path().display().to_string(),
                original_uncompressed_size: data.len(),
                all_results: analyze_bc2_compression_transforms(
                    data,
                    compression_level,
                    compression_algorithm,
                    cache,
                )?,
                original_compressed_size: calc_size_with_cache_and_estimation_algorithm(
                    data,
                    compression_level,
                    compression_algorithm,
                    cache,
                )?,
                api_recommended_result: analyze_bc2_api_recommendation(
                    data,
                    estimate_compression_level,
                    estimate_compression_algorithm,
                    compression_level,
                    compression_algorithm,
                    use_all_decorrelation_modes,
                    cache,
                )?,
            };

            Ok(())
        },
    )?;

    Ok(file_result)
}

fn analyze_bc2_compression_transforms(
    data: &[u8],
    compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    cache: &Mutex<CompressionCache>,
) -> Result<Vec<Bc2TransformResult>, TransformError> {
    // Allocate aligned buffers for transformations
    let mut transformed_data = allocate_align_64(data.len())?;
    let mut results = Vec::new();

    unsafe {
        // Test all transform combinations
        for transform_options in Bc2TransformSettings::all_combinations() {
            // Transform the data
            transform_bc2_with_settings(
                data.as_ptr(),
                transformed_data.as_mut_ptr(),
                data.len(),
                transform_options,
            );

            // Compress the transformed data
            results.push(Bc2TransformResult {
                transform_options,
                compressed_size: calc_size_with_cache_and_estimation_algorithm(
                    transformed_data.as_slice(),
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
fn analyze_bc2_api_recommendation(
    data: &[u8],
    estimate_compression_level: i32,
    estimate_compression_algorithm: CompressionAlgorithm,
    final_compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    use_all_decorrelation_modes: bool,
    cache: &Mutex<CompressionCache>,
) -> Result<Bc2TransformResult, TransformError> {
    let best_details = determine_best_transform_details_with_estimator_cached(
        data,
        estimate_compression_level,
        estimate_compression_algorithm,
        use_all_decorrelation_modes,
        cache,
    )?;

    // Transform the data using the recommended details and measure the size
    let mut transformed_data = allocate_align_64(data.len())?;
    unsafe {
        transform_bc2_with_settings(
            data.as_ptr(),
            transformed_data.as_mut_ptr(),
            data.len(),
            best_details,
        );
    }

    // Compress the transformed data (API recommendation, final level)
    let compressed_size = calc_size_with_cache_and_estimation_algorithm(
        transformed_data.as_slice(),
        final_compression_level,
        compression_algorithm,
        cache,
    )?;

    Ok(Bc2TransformResult {
        transform_options: best_details,
        compressed_size,
    })
}

/// Formats [`Bc2TransformSettings`] as a human-readable string
fn format_transform_details(details: Bc2TransformSettings) -> String {
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

    format!("{decorr_mode}/{split_endpoints}")
}
