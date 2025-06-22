use super::{determine_best_transform_details_with_estimator_cached, BenchmarkCmd};
use crate::{
    debug::{
        benchmark_common::{
            self, print_file_result, print_overall_statistics, BenchmarkResult,
            BenchmarkScenarioResult,
        },
        compressed_data_cache::CompressedDataCache,
        compression::{
            helpers::{
                compress_data_cached, decompress_data, validate_compression_algorithm, CacheRefs,
            },
            CompressionAlgorithm,
        },
        compression_size_cache::CompressionSizeCache,
        extract_blocks_from_dds,
    },
    error::TransformError,
    util::find_all_files,
    DdsFilter,
};
use core::time::Duration;
use dxt_lossless_transform_bc1::{
    transform_bc1_with_settings, untransform_bc1_with_settings, Bc1TransformSettings,
};
use dxt_lossless_transform_common::{allocate::allocate_align_64, color_565::YCoCgVariant};
use dxt_lossless_transform_dds::dds::DdsFormat;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{fs, sync::Mutex};

/// Configuration for benchmark execution
struct BenchmarkConfig {
    iterations: u32,
    warmup_iterations: u32,
    compression_level: i32,
    estimate_compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    estimate_compression_algorithm: CompressionAlgorithm,
    dry_run: bool,
    experimental_normalize: bool,
    use_all_decorrelation_modes: bool,
}

pub(crate) fn handle_benchmark_command(cmd: BenchmarkCmd) -> Result<(), TransformError> {
    validate_compression_algorithm(cmd.compression_algorithm)?;

    let input_path = &cmd.input_directory;
    println!(
        "Benchmarking BC1 decompress+detransform performance for files in: {} (recursive)",
        input_path.display()
    );
    println!("Iterations per file: {}", cmd.iterations);
    println!("Warmup iterations: {}", cmd.warmup_iterations);
    println!(
        "Compression algorithm: {} , level: {}",
        cmd.compression_algorithm.name(),
        cmd.get_compression_level()
    );
    println!(
        "Estimate compression algorithm: {} , level: {}",
        cmd.get_estimate_compression_algorithm().name(),
        cmd.get_estimate_compression_level()
    );

    // Initialize and load cache for determining API recommendations
    let mut cache = CompressionSizeCache::new();
    if let Err(e) = cache.load_from_disk() {
        println!("Warning: Failed to load cache: {e}");
    } else {
        println!("Loaded compression size cache with {} entries", cache.len());
    }
    let cache = Mutex::new(cache);

    // Initialize compressed data cache for benchmarking
    let compressed_cache = CompressedDataCache::new();
    println!(
        "Initialized compressed data cache with {} entries",
        compressed_cache.cache_count()
    );

    // Collect all files recursively
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to benchmark", entries.len());

    if entries.is_empty() {
        println!("No files found to benchmark.");
        return Ok(());
    }

    // Dry run phase - pre-populate compression data cache in parallel
    println!(
        "Performing dry run (transform + compress only) to populate compression data cache..."
    );
    entries
        .par_iter()
        .with_max_len(1) // compression is expensive, so 1 item at a time per thread is faster.
        .for_each(|entry| {
            let _ = process_file(
                entry,
                &BenchmarkConfig {
                    iterations: 0,        // No iterations for dry run
                    warmup_iterations: 0, // No warmup for dry run
                    compression_level: cmd.get_compression_level(),
                    estimate_compression_level: cmd.get_estimate_compression_level(),
                    compression_algorithm: cmd.compression_algorithm,
                    estimate_compression_algorithm: cmd.get_estimate_compression_algorithm(),
                    dry_run: true, // This is a dry run
                    experimental_normalize: cmd.experimental_normalize,
                    use_all_decorrelation_modes: cmd.use_all_decorrelation_modes,
                },
                &CacheRefs {
                    compressed_size_cache: &cache,
                    compressed_data_cache: &compressed_cache,
                },
            );
        });

    println!("Dry run completed. Starting actual benchmarks...\n");
    let mut results = Vec::new();

    // Process files (not in parallel!! we want clean results!)
    for entry in entries {
        match process_file(
            &entry,
            &BenchmarkConfig {
                iterations: cmd.iterations,
                warmup_iterations: cmd.warmup_iterations,
                compression_level: cmd.get_compression_level(),
                estimate_compression_level: cmd.get_estimate_compression_level(),
                compression_algorithm: cmd.compression_algorithm,
                estimate_compression_algorithm: cmd.get_estimate_compression_algorithm(),
                dry_run: false, // This is not a dry run
                experimental_normalize: cmd.experimental_normalize,
                use_all_decorrelation_modes: cmd.use_all_decorrelation_modes,
            },
            &CacheRefs {
                compressed_size_cache: &cache,
                compressed_data_cache: &compressed_cache,
            },
        ) {
            Ok(Some(file_result)) => {
                print_file_result(&file_result);
                results.push(file_result);
            }
            Ok(None) => {
                // Dry run. Unreachable.
            }
            Err(e) => {
                println!("✗ Error benchmarking {}: {}", entry.path().display(), e);
            }
        }
    }

    // Save cache
    let cache = cache.into_inner().unwrap();
    println!("Saving compression size cache with {} entries", cache.len());
    if let Err(e) = cache.save_to_disk() {
        println!("Warning: Failed to save cache: {e}");
    }

    // Print overall statistics
    print_overall_statistics(&results);

    Ok(())
}

fn process_file(
    entry: &fs::DirEntry,
    config: &BenchmarkConfig,
    caches: &CacheRefs,
) -> Result<Option<BenchmarkResult>, TransformError> {
    let mut file_result = if config.dry_run {
        None
    } else {
        Some(BenchmarkResult::new(entry.path().display().to_string(), 0))
    };

    unsafe {
        extract_blocks_from_dds(
            entry,
            DdsFilter::BC1,
            |data_ptr: *const u8,
             len_bytes: usize,
             format: DdsFormat|
             -> Result<(), TransformError> {
                // Only benchmark BC1 blocks
                if format != DdsFormat::BC1 {
                    return Ok(()); // Skip non-BC1 data
                }

                if let Some(ref mut result) = file_result {
                    result.file_size_bytes = len_bytes;
                }

                // Define the scenarios to benchmark
                let scenarios = vec![
                    (
                        "API Recommended",
                        determine_best_transform_details_with_estimator_cached(
                            data_ptr,
                            len_bytes,
                            config.estimate_compression_level,
                            config.estimate_compression_algorithm,
                            config.experimental_normalize,
                            config.use_all_decorrelation_modes,
                            caches.compressed_size_cache,
                        )?,
                    ),
                    (
                        "NoSplit/None",
                        Bc1TransformSettings {
                            decorrelation_mode: YCoCgVariant::None,
                            split_colour_endpoints: false,
                        },
                    ),
                    (
                        "NoSplit/YCoCg1",
                        Bc1TransformSettings {
                            decorrelation_mode: YCoCgVariant::Variant1,
                            split_colour_endpoints: false,
                        },
                    ),
                    (
                        "Split/None",
                        Bc1TransformSettings {
                            decorrelation_mode: YCoCgVariant::None,
                            split_colour_endpoints: true,
                        },
                    ),
                    (
                        "Split/YCoCg1",
                        Bc1TransformSettings {
                            decorrelation_mode: YCoCgVariant::Variant1,
                            split_colour_endpoints: true,
                        },
                    ),
                ];

                // Process each scenario
                for (scenario_name, transform_details) in scenarios {
                    if let Some(scenario_result) = process_scenario(
                        data_ptr,
                        len_bytes,
                        scenario_name,
                        transform_details,
                        config,
                        caches,
                    )? {
                        if let Some(ref mut result) = file_result {
                            result.add_scenario(scenario_result);
                        }
                    }
                }

                // Process untransformed data (no transformation applied)
                if let Some(untransformed_result) = process_untransformed_scenario(
                    data_ptr,
                    len_bytes,
                    "Untransformed",
                    config,
                    caches,
                )? {
                    if let Some(ref mut result) = file_result {
                        result.add_scenario(untransformed_result);
                    }
                }

                Ok(())
            },
        )?;
    }

    Ok(file_result)
}

unsafe fn process_scenario(
    data_ptr: *const u8,
    len_bytes: usize,
    scenario_name: &str,
    transform_details: Bc1TransformSettings,
    config: &BenchmarkConfig,
    caches: &CacheRefs,
) -> Result<Option<BenchmarkScenarioResult>, TransformError> {
    // Allocate buffers
    let mut transformed_data = allocate_align_64(len_bytes)?;

    // Transform the original data
    transform_bc1_with_settings(
        data_ptr,
        transformed_data.as_mut_ptr(),
        len_bytes,
        transform_details,
    );

    // Compress the transformed data (this populates the cache for both dry run and benchmark)
    let (compressed_data, compressed_size) = compress_data_cached(
        transformed_data.as_ptr(),
        len_bytes,
        config.compression_level,
        config.compression_algorithm,
        caches,
    )?;

    // For dry run, we only need to populate the cache
    if config.dry_run {
        return Ok(None);
    }

    drop(transformed_data);

    // Create decompression buffer (we know the exact size)
    let mut decompressed_data = allocate_align_64(len_bytes)?;
    let mut final_output = allocate_align_64(len_bytes)?;

    // Warmup phase
    for _ in 0..config.warmup_iterations {
        // Decompress
        decompress_data(
            &compressed_data[..compressed_size],
            decompressed_data.as_mut_slice(),
            config.compression_algorithm,
        )?;

        // Detransform
        untransform_bc1_with_settings(
            decompressed_data.as_ptr(),
            final_output.as_mut_ptr(),
            len_bytes,
            transform_details.into(),
        );
    }

    // Benchmark decompression
    let (_, decompress_time) = benchmark_common::measure_time(|| {
        for _ in 0..config.iterations {
            decompress_data(
                &compressed_data[..compressed_size],
                decompressed_data.as_mut_slice(),
                config.compression_algorithm,
            )
            .unwrap();
        }
    });

    // Benchmark detransform
    let (_, detransform_time) = benchmark_common::measure_time(|| {
        for _ in 0..config.iterations {
            untransform_bc1_with_settings(
                decompressed_data.as_ptr(),
                final_output.as_mut_ptr(),
                len_bytes,
                transform_details.into(),
            );
        }
    });

    // Average the times over iterations
    let avg_decompress_time = decompress_time / config.iterations;
    let avg_detransform_time = detransform_time / config.iterations;

    Ok(Some(BenchmarkScenarioResult::new(
        scenario_name.to_string(),
        len_bytes,
        avg_decompress_time,
        avg_detransform_time,
    )))
}

unsafe fn process_untransformed_scenario(
    data_ptr: *const u8,
    len_bytes: usize,
    scenario_name: &str,
    config: &BenchmarkConfig,
    caches: &CacheRefs,
) -> Result<Option<BenchmarkScenarioResult>, TransformError> {
    // Compress the original data directly (bypassing transformation)
    let (compressed_data_ptr, compressed_size) = compress_data_cached(
        data_ptr,
        len_bytes,
        config.compression_level,
        config.compression_algorithm,
        caches,
    )?;

    // For dry run, we only need to populate the cache
    if config.dry_run {
        return Ok(None);
    }

    // Allocate decompression buffer
    let mut decompressed_data = allocate_align_64(len_bytes)?;

    // Warmup phase
    for _ in 0..config.warmup_iterations {
        // Decompress
        decompress_data(
            &compressed_data_ptr[..compressed_size],
            decompressed_data.as_mut_slice(),
            config.compression_algorithm,
        )?;
    }

    // Benchmark decompression
    let (_, decompress_time) = benchmark_common::measure_time(|| {
        for _ in 0..config.iterations {
            decompress_data(
                &compressed_data_ptr[..compressed_size],
                decompressed_data.as_mut_slice(),
                config.compression_algorithm,
            )
            .unwrap();
        }
    });

    // Average the time over iterations
    let avg_decompress_time = decompress_time / config.iterations;

    Ok(Some(BenchmarkScenarioResult::new(
        scenario_name.to_string(),
        len_bytes,
        avg_decompress_time,
        Duration::ZERO, // No detransform time for untransformed scenario
    )))
}
