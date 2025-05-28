use super::BenchmarkCmd;
use crate::{
    debug::{
        benchmark_common::{
            self, print_file_result, print_overall_statistics, zstd_decompress_data,
            BenchmarkResult, BenchmarkScenarioResult,
        },
        compressed_data_cache::CompressedDataCache,
        compression_size_cache, extract_blocks_from_dds,
    },
    error::TransformError,
    util::find_all_files,
    DdsFilter,
};
use dxt_lossless_transform_api::DdsFormat;
use dxt_lossless_transform_bc1::{
    determine_optimal_transform::{determine_best_transform_details, Bc1TransformOptions},
    normalize_blocks::ColorNormalizationMode,
    transform_bc1, untransform_bc1, Bc1TransformDetails,
};
use dxt_lossless_transform_common::{allocate::allocate_align_64, color_565::YCoCgVariant};
use std::{fs, sync::Mutex};

type CompressionCache = compression_size_cache::CompressionCache;

pub(crate) fn handle_benchmark_command(cmd: BenchmarkCmd) -> Result<(), TransformError> {
    let input_path = &cmd.input_directory;
    println!(
        "Benchmarking BC1 decompress+detransform performance for files in: {} (recursive)",
        input_path.display()
    );
    println!("Iterations per file: {}", cmd.iterations);
    println!("Warmup iterations: {}", cmd.warmup_iterations);
    println!("Compression level: {}", cmd.compression_level);
    println!(
        "Estimate compression level: {}",
        cmd.estimate_compression_level
    );

    // Initialize and load cache for determining API recommendations
    let mut cache = CompressionCache::new();
    if let Err(e) = cache.load_from_disk() {
        println!("Warning: Failed to load cache: {e}");
    } else {
        println!("Loaded compression cache with {} entries", cache.len());
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

    let results = std::sync::Mutex::new(Vec::<benchmark_common::BenchmarkResult>::new());

    // Process files (not in parallel!! we want clean results!)
    for entry in entries {
        match benchmark_file(
            &entry,
            cmd.iterations,
            cmd.warmup_iterations,
            cmd.compression_level,
            cmd.estimate_compression_level,
            &cache,
            &compressed_cache,
        ) {
            Ok(file_result) => {
                print_file_result(&file_result);
                results.lock().unwrap().push(file_result);
            }
            Err(e) => {
                println!("✗ Error benchmarking {}: {}", entry.path().display(), e);
            }
        }
    }

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

fn benchmark_file(
    entry: &fs::DirEntry,
    iterations: u32,
    warmup_iterations: u32,
    compression_level: i32,
    estimate_compression_level: i32,
    cache: &Mutex<CompressionCache>,
    compressed_cache: &CompressedDataCache,
) -> Result<BenchmarkResult, TransformError> {
    let mut file_result = BenchmarkResult::new(entry.path().display().to_string(), 0);

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

                file_result.file_size_bytes = len_bytes;

                // Define the scenarios to benchmark
                let scenarios = vec![
                    (
                        "API Recommended",
                        get_api_recommended_details(
                            data_ptr,
                            len_bytes,
                            estimate_compression_level,
                            cache,
                        )?,
                    ),
                    (
                        "NoSplit/None",
                        Bc1TransformDetails {
                            color_normalization_mode: ColorNormalizationMode::None,
                            decorrelation_mode: YCoCgVariant::None,
                            split_colour_endpoints: false,
                        },
                    ),
                    (
                        "Split/None",
                        Bc1TransformDetails {
                            color_normalization_mode: ColorNormalizationMode::None,
                            decorrelation_mode: YCoCgVariant::None,
                            split_colour_endpoints: true,
                        },
                    ),
                    (
                        "Split/YCoCg1",
                        Bc1TransformDetails {
                            color_normalization_mode: ColorNormalizationMode::None,
                            decorrelation_mode: YCoCgVariant::Variant1,
                            split_colour_endpoints: true,
                        },
                    ),
                ];

                // Benchmark each scenario
                for (scenario_name, transform_details) in scenarios {
                    let scenario_result = benchmark_scenario(
                        data_ptr,
                        len_bytes,
                        scenario_name,
                        transform_details,
                        compression_level,
                        iterations,
                        warmup_iterations,
                        compressed_cache,
                    )?;
                    file_result.add_scenario(scenario_result);
                }

                // Benchmark untransformed data (no transformation applied)
                let untransformed_result = benchmark_untransformed_scenario(
                    data_ptr,
                    len_bytes,
                    "Untransformed",
                    compression_level,
                    iterations,
                    warmup_iterations,
                    compressed_cache,
                )?;
                file_result.add_scenario(untransformed_result);

                Ok(())
            },
        )?;
    }

    Ok(file_result)
}

unsafe fn get_api_recommended_details(
    data_ptr: *const u8,
    len_bytes: usize,
    estimate_compression_level: i32,
    cache: &Mutex<CompressionCache>,
) -> Result<Bc1TransformDetails, TransformError> {
    // Create the zstandard file size estimator with cache clone for static lifetime
    let estimator = move |data_ptr: *const u8, len: usize| -> usize {
        match crate::debug::calc_compression_stats_common::zstd_calc_size_with_cache(
            data_ptr,
            len,
            estimate_compression_level,
            cache,
        ) {
            Ok(size) => size,
            Err(_) => usize::MAX, // Return max size on error to make this option less favorable
        }
    };

    // Create transform options
    let transform_options = Bc1TransformOptions {
        file_size_estimator: Box::new(estimator),
    };

    // Determine the best transform details using the API
    determine_best_transform_details(data_ptr, len_bytes, transform_options)
        .map_err(|e| TransformError::Debug(format!("API recommendation failed: {e}")))
}

#[allow(clippy::too_many_arguments)]
unsafe fn benchmark_scenario(
    data_ptr: *const u8,
    len_bytes: usize,
    scenario_name: &str,
    transform_details: Bc1TransformDetails,
    compression_level: i32,
    iterations: u32,
    warmup_iterations: u32,
    compressed_cache: &CompressedDataCache,
) -> Result<BenchmarkScenarioResult, TransformError> {
    // Allocate buffers
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut work_buffer = allocate_align_64(len_bytes)?;
    let mut final_output = allocate_align_64(len_bytes)?;

    // Transform the original data
    transform_bc1(
        data_ptr,
        transformed_data.as_mut_ptr(),
        work_buffer.as_mut_ptr(),
        len_bytes,
        transform_details,
    );

    // Compress the transformed data
    let (compressed_data, compressed_size) = benchmark_common::zstd_compress_data_cached(
        transformed_data.as_ptr(),
        len_bytes,
        compression_level,
        compressed_cache,
    )?;

    drop(transformed_data);

    // Create decompression buffer (we know the exact size)
    let mut decompressed_data = allocate_align_64(len_bytes)?;

    // Warmup phase
    for _ in 0..warmup_iterations {
        // Decompress
        zstd_decompress_data(
            &compressed_data[..compressed_size],
            decompressed_data.as_mut_slice(),
        )?;

        // Detransform
        untransform_bc1(
            decompressed_data.as_ptr(),
            final_output.as_mut_ptr(),
            work_buffer.as_mut_ptr(),
            len_bytes,
            transform_details,
        );
    }

    // Benchmark decompression
    let (_, decompress_time_ms) = benchmark_common::measure_time(|| {
        for _ in 0..iterations {
            zstd_decompress_data(
                &compressed_data[..compressed_size],
                decompressed_data.as_mut_slice(),
            )
            .unwrap();
        }
    });

    // Benchmark detransform
    let (_, detransform_time_ms) = benchmark_common::measure_time(|| {
        for _ in 0..iterations {
            untransform_bc1(
                decompressed_data.as_ptr(),
                final_output.as_mut_ptr(),
                work_buffer.as_mut_ptr(),
                len_bytes,
                transform_details,
            );
        }
    });

    // Average the times over iterations
    let avg_decompress_time = decompress_time_ms / iterations as f64;
    let avg_detransform_time = detransform_time_ms / iterations as f64;

    Ok(BenchmarkScenarioResult::new(
        scenario_name.to_string(),
        len_bytes,
        avg_decompress_time,
        avg_detransform_time,
    ))
}

unsafe fn benchmark_untransformed_scenario(
    data_ptr: *const u8,
    len_bytes: usize,
    scenario_name: &str,
    compression_level: i32,
    iterations: u32,
    warmup_iterations: u32,
    compressed_cache: &CompressedDataCache,
) -> Result<BenchmarkScenarioResult, TransformError> {
    // Allocate decompression buffer
    let mut decompressed_data = allocate_align_64(len_bytes)?;

    // Compress the original data directly (bypassing transformation)
    let (compressed_data_ptr, compressed_size) = benchmark_common::zstd_compress_data_cached(
        data_ptr,
        len_bytes,
        compression_level,
        compressed_cache,
    )?;

    // Warmup phase
    for _ in 0..warmup_iterations {
        // Decompress
        zstd_decompress_data(
            &compressed_data_ptr[..compressed_size],
            decompressed_data.as_mut_slice(),
        )?;
    }

    // Benchmark decompression
    let (_, decompress_time_ms) = benchmark_common::measure_time(|| {
        for _ in 0..iterations {
            zstd_decompress_data(
                &compressed_data_ptr[..compressed_size],
                decompressed_data.as_mut_slice(),
            )
            .unwrap();
        }
    });

    // Average the time over iterations
    let avg_decompress_time = decompress_time_ms / iterations as f64;

    Ok(BenchmarkScenarioResult::new(
        scenario_name.to_string(),
        len_bytes,
        avg_decompress_time,
        0.0, // No detransform time for untransformed scenario
    ))
}
