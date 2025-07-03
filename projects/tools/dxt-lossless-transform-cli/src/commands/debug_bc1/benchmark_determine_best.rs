use super::{determine_best_transform_details_with_estimator, BenchmarkDetermineBestCmd};
use crate::{
    debug::{
        benchmark_common::{
            self, print_overall_statistics, BenchmarkResult, BenchmarkScenarioResult,
        },
        compression::CompressionAlgorithm,
        extract_blocks_from_file, handle_debug_error,
    },
    error::TransformError,
    util::find_all_files,
};
use core::time::Duration;
use dxt_lossless_transform_bc1::Bc1TransformSettings;
use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
use dxt_lossless_transform_file_formats_debug::TransformFormatFilter;
use std::fs;

/// Configuration for benchmark execution
struct BenchmarkConfig {
    iterations: u32,
    warmup_iterations: u32,
    compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    experimental_normalize: bool,
    use_all_decorrelation_modes: bool,
}

pub(crate) fn handle_benchmark_determine_best_command(
    cmd: BenchmarkDetermineBestCmd,
) -> Result<(), TransformError> {
    let input_path = &cmd.input_directory;
    println!(
        "Benchmarking BC1 determine_best_transform_details performance for files in: {} (recursive)",
        input_path.display()
    );
    println!("Iterations per file: {}", cmd.iterations);
    println!("Warmup iterations: {}", cmd.warmup_iterations);
    println!(
        "Estimate compression algorithm: {} , level: {}",
        cmd.estimate_compression_algorithm.name(),
        cmd.get_estimate_compression_level()
    );

    // Collect all files recursively
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to benchmark", entries.len());

    if entries.is_empty() {
        println!("No files found to benchmark.");
        return Ok(());
    }

    println!("Starting benchmarks...\n");
    let mut results = Vec::new();

    // Process files (not in parallel!! we want clean results!)
    for entry in entries {
        match process_file(
            &entry,
            &BenchmarkConfig {
                iterations: cmd.iterations,
                warmup_iterations: cmd.warmup_iterations,
                compression_level: cmd.get_estimate_compression_level(),
                compression_algorithm: cmd.estimate_compression_algorithm,
                experimental_normalize: cmd.experimental_normalize,
                use_all_decorrelation_modes: cmd.use_all_decorrelation_modes,
            },
        ) {
            Ok(Some(file_result)) => {
                print_file_result_throughput(&file_result);
                results.push(file_result);
            }
            Ok(None) => {
                // No result to process
            }
            Err(e) => {
                handle_debug_error(&entry.path(), "benchmarking", Err(e));
            }
        }
    }

    // Print overall statistics
    print_overall_statistics(&results);

    Ok(())
}

fn process_file(
    entry: &fs::DirEntry,
    config: &BenchmarkConfig,
) -> Result<Option<BenchmarkResult>, TransformError> {
    let mut file_result = Some(BenchmarkResult::new(entry.path().display().to_string(), 0));

    extract_blocks_from_file(
        &entry.path(),
        TransformFormatFilter::Bc1,
        |data: &[u8], _format: TransformFormat| -> Result<(), TransformError> {
            if let Some(ref mut result) = file_result {
                result.file_size_bytes = data.len();
            }

            // Benchmark determine_best_transform_details function
            if let Some(scenario_result) =
                process_determine_best_scenario(data, "determine_best_transform_details", config)?
            {
                if let Some(ref mut result) = file_result {
                    result.add_scenario(scenario_result);
                }
            }

            Ok(())
        },
    )?;

    Ok(file_result)
}

fn process_determine_best_scenario(
    data: &[u8],
    scenario_name: &str,
    config: &BenchmarkConfig,
) -> Result<Option<BenchmarkScenarioResult>, TransformError> {
    // Warmup phase
    for _ in 0..config.warmup_iterations {
        let _ = run_determine_best_once(
            data,
            config.compression_level,
            config.compression_algorithm,
            config.experimental_normalize,
            config.use_all_decorrelation_modes,
        )?;
    }

    // Benchmark determine_best_transform_details
    let (_, execution_time) =
        benchmark_common::measure_time_result(|| -> Result<(), TransformError> {
            for _ in 0..config.iterations {
                let _ = run_determine_best_once(
                    data,
                    config.compression_level,
                    config.compression_algorithm,
                    config.experimental_normalize,
                    config.use_all_decorrelation_modes,
                )?;
            }
            Ok(())
        })?;

    // Average the time over iterations
    let avg_execution_time = execution_time / config.iterations;

    // For this benchmark, we consider the entire determine_best_transform_details as "untransform"
    // and set decompress time to 0, as we're only measuring the algorithm performance
    Ok(Some(BenchmarkScenarioResult::new(
        scenario_name.to_string(),
        data.len(),
        Duration::ZERO,     // No decompress time for this specific benchmark
        avg_execution_time, // All time is considered "untransform" time
    )))
}

fn run_determine_best_once(
    data: &[u8],
    estimate_compression_level: i32,
    compression_algorithm: CompressionAlgorithm,
    experimental_normalize: bool,
    use_all_decorrelation_modes: bool,
) -> Result<Bc1TransformSettings, TransformError> {
    determine_best_transform_details_with_estimator(
        data,
        estimate_compression_level,
        compression_algorithm,
        experimental_normalize,
        use_all_decorrelation_modes,
    )
}

/// Print file result with throughput measured in MiB/s for determine_best_transform_details benchmark
fn print_file_result_throughput(result: &BenchmarkResult) {
    let file_size_mib = result.file_size_bytes as f64 / (1024.0 * 1024.0);

    println!("📁 {}", result.file_path);
    println!("   📊 File size: {file_size_mib:.2} MiB");

    for scenario in &result.scenarios {
        let execution_time_ms = scenario.untransform_time.as_secs_f64() * 1000.0;
        let throughput = scenario.untransform_throughput;
        println!(
            "   ⚡ {}: {execution_time_ms:.3} ms ({throughput:.2})",
            scenario.scenario_name
        );
    }
    println!();
}
