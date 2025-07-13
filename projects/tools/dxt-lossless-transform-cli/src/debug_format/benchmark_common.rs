//! Common functionality for benchmarking performance across BC1, BC2, BC3, and BC7 formats.
//!
//! This module provides shared data structures, utilities, and benchmarking functions that can be
//! reused across different BC format implementations for performance analysis.

use crate::debug_format::calc_compression_stats_common::get_filename;
use crate::util::Throughput;
use core::{f64, fmt::Debug};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// Result of benchmarking a specific transform scenario.
#[derive(Debug, Clone)]
pub struct BenchmarkScenarioResult {
    pub scenario_name: String,
    pub decompress_time: Duration,
    pub untransform_time: Duration,
    pub combined_time: Duration,
    pub decompress_throughput: Throughput,
    pub untransform_throughput: Throughput,
    pub combined_throughput: Throughput,
}

impl BenchmarkScenarioResult {
    pub fn new(
        scenario_name: String,
        file_size_bytes: usize,
        decompress_time: Duration,
        untransform_time: Duration,
    ) -> Self {
        let combined_time = decompress_time + untransform_time;

        // Calculate throughput
        let decompress_throughput_bytes_per_sec = if decompress_time.is_zero() {
            0.0
        } else {
            file_size_bytes as f64 / decompress_time.as_secs_f64()
        };

        let untransform_throughput_bytes_per_sec = if untransform_time.is_zero() {
            0.0
        } else {
            file_size_bytes as f64 / untransform_time.as_secs_f64()
        };

        let combined_throughput_bytes_per_sec = if combined_time.is_zero() {
            0.0
        } else {
            file_size_bytes as f64 / combined_time.as_secs_f64()
        };

        Self {
            scenario_name,
            decompress_time,
            untransform_time,
            combined_time,
            decompress_throughput: Throughput::from_bytes_per_sec(
                decompress_throughput_bytes_per_sec.round() as u64,
            ),
            untransform_throughput: Throughput::from_bytes_per_sec(
                untransform_throughput_bytes_per_sec.round() as u64,
            ),
            combined_throughput: Throughput::from_bytes_per_sec(
                combined_throughput_bytes_per_sec.round() as u64,
            ),
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

/// Measures the time taken to execute a function and returns the duration.
pub fn measure_time<F, R>(func: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = func();
    let duration = start.elapsed();
    (result, duration)
}

/// Measures the time taken to execute a function that returns a Result and propagates errors.
/// Returns the duration only on success, or the error if the function fails.
pub fn measure_time_result<F, T, E>(func: F) -> Result<(T, Duration), E>
where
    F: FnOnce() -> Result<T, E>,
{
    let start = Instant::now();
    let result = func()?;
    let duration = start.elapsed();
    Ok((result, duration))
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
            "  {}: decompress: {:.2} ms ({:.1}), untransform: {:.2} ms ({:.1}), combined: {:.2} ms ({:.1})",
            scenario.scenario_name,
            scenario.decompress_time.as_secs_f64() * 1000.0,
            scenario.decompress_throughput,
            scenario.untransform_time.as_secs_f64() * 1000.0,
            scenario.untransform_throughput,
            scenario.combined_time.as_secs_f64() * 1000.0,
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
        avg_decompress_throughput: Throughput,
        avg_untransform_throughput: Throughput,
        avg_combined_throughput: Throughput,
        total_decompress_time: Duration,
        total_untransform_time: Duration,
        total_combined_time: Duration,
    }

    // Calculate statistics for each scenario type
    let mut scenario_stats: Vec<ScenarioStats> = Vec::new();
    for (scenario_name, scenario_data) in scenario_groups {
        if scenario_data.is_empty() {
            continue;
        }

        // Calculate weighted average throughput based on total data and total time
        let total_size_bytes: usize = scenario_data.iter().map(|(_, file_size)| *file_size).sum();

        let total_decompress_time: Duration = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.decompress_time)
            .sum();
        let total_untransform_time: Duration = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.untransform_time)
            .sum();
        let total_combined_time: Duration = scenario_data
            .iter()
            .map(|(scenario, _)| scenario.combined_time)
            .sum();

        // Calculate weighted average throughput
        let total_decompress_time_s = total_decompress_time.as_secs_f64();
        let total_untransform_time_s = total_untransform_time.as_secs_f64();
        let total_combined_time_s = total_combined_time.as_secs_f64();

        let avg_decompress_throughput = if total_decompress_time_s > 0.0 {
            Throughput::from_bytes_per_sec(
                (total_size_bytes as f64 / total_decompress_time_s).round() as u64,
            )
        } else {
            Throughput::from_bytes_per_sec(0)
        };
        let avg_untransform_throughput = if total_untransform_time_s > 0.0 {
            Throughput::from_bytes_per_sec(
                (total_size_bytes as f64 / total_untransform_time_s).round() as u64,
            )
        } else {
            Throughput::from_bytes_per_sec(0)
        };
        let avg_combined_throughput = if total_combined_time_s > 0.0 {
            Throughput::from_bytes_per_sec(
                (total_size_bytes as f64 / total_combined_time_s).round() as u64,
            )
        } else {
            Throughput::from_bytes_per_sec(0)
        };

        scenario_stats.push(ScenarioStats {
            scenario_name: scenario_name.to_string(),
            avg_decompress_throughput,
            avg_untransform_throughput,
            avg_combined_throughput,
            total_decompress_time,
            total_untransform_time,
            total_combined_time,
        });
    }

    // Sort by combined throughput (descending - fastest first)
    scenario_stats.sort_by(|a, b| {
        b.avg_combined_throughput
            .bytes_per_sec()
            .cmp(&a.avg_combined_throughput.bytes_per_sec())
    });

    // Print statistics for each scenario type
    for stats in scenario_stats {
        println!("📈 {}:", stats.scenario_name);
        println!(
            "  Decompress: avg {:.2}, total {:.2} ms",
            stats.avg_decompress_throughput,
            stats.total_decompress_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Untransform: avg {:.2}, total {:.2} ms",
            stats.avg_untransform_throughput,
            stats.total_untransform_time.as_secs_f64() * 1000.0
        );
        println!(
            "  Combined: avg {:.2}, total {:.2} ms",
            stats.avg_combined_throughput,
            stats.total_combined_time.as_secs_f64() * 1000.0
        );
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════");
}
