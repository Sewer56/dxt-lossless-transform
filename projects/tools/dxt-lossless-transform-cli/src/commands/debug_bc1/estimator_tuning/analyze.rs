use super::EstimatorDataPoint;
use crate::error::TransformError;
use argh::FromArgs;
use bytesize::ByteSize;
use dxt_lossless_transform_ltu::{size_estimate, EstimationSettings, SizeEstimationParameters};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[derive(FromArgs, Debug)]
/// analyze estimator tuning data to find optimal coefficients for each data type, power of 2 size group, and ZStandard level
#[argh(subcommand, name = "analyze-estimator")]
pub struct AnalyzeEstimatorCmd {
    /// input JSON file from tune-estimator command
    #[argh(positional)]
    pub input_file: PathBuf,

    /// ZStandard compression levels to optimize for (comma-separated, e.g., "1,6,22" or "all" for 1-22)
    #[argh(option, default = "String::from(\"all\")")]
    pub zstd_levels: String,

    /// minimum number of files required in a group to include it in output (default: 50)
    #[argh(option, default = "50")]
    pub min_files: usize,
}

/// Configuration for brute force search
#[derive(Debug, Clone)]
struct BruteForceConfig {
    /// Range for lz_match_multiplier
    min_lz_multiplier: f64,
    max_lz_multiplier: f64,
    lz_step_size: f64,

    /// Range for entropy_multiplier  
    min_entropy_multiplier: f64,
    max_entropy_multiplier: f64,
    entropy_step_size: f64,
}

impl Default for BruteForceConfig {
    fn default() -> Self {
        Self {
            min_lz_multiplier: 0.5,
            max_lz_multiplier: 1.3,
            lz_step_size: 0.001,
            min_entropy_multiplier: 0.85,
            max_entropy_multiplier: 1.40,
            entropy_step_size: 0.001,
        }
    }
}

/// Results for a specific data type, power of 2 size group, and ZStandard level
#[derive(Debug)]
struct OptimizationResult {
    data_type: String,
    power: u32,
    zstd_level: i32,
    optimal_settings: EstimationSettings,
    mean_absolute_error: f64,
    mean_percentage_error: f64,
    average_compressed_size: f64,
    average_estimated_size: f64,
}

pub fn handle_analyze_estimator_command(cmd: AnalyzeEstimatorCmd) -> Result<(), TransformError> {
    println!("Analyzing estimator tuning data...");
    println!("Input file: {:?}", cmd.input_file);
    println!("Minimum files per group: {}", cmd.min_files);

    // Parse ZStandard levels
    let zstd_levels = super::parse_zstd_levels(&cmd.zstd_levels)?;
    println!("ZStandard levels: {zstd_levels:?}");

    // Load the data points
    let data_points = load_data_points(&cmd.input_file)?;
    println!("Loaded {} data points", data_points.len());

    // Group by data type first, then by power of 2
    let grouped_data = group_by_data_type_and_power_of_2(&data_points);
    println!("Grouped into {} data type groups", grouped_data.len());

    // Calculate and display data point distribution by size (same for all data types)
    let first_data_type = grouped_data.values().next().unwrap();
    let mut power_list: Vec<_> = first_data_type.iter().collect();
    power_list.sort_by_key(|(power, _)| **power);

    let power_info: Vec<String> = power_list
        .iter()
        .map(|(&power, group_data)| {
            let bytes = 1u64 << power;
            let size_str = ByteSize::b(bytes).to_string();
            format!("{}({})", size_str, group_data.len())
        })
        .collect();

    println!("Data point distribution by size: {}", power_info.join(", "));
    println!();

    // Find optimal coefficients for each group and ZStandard level combination
    let config = BruteForceConfig::default();
    let mut all_results = Vec::new();

    let total_data_types = grouped_data.len();
    for (data_type_index, (data_type, power_groups_for_data_type)) in
        grouped_data.into_iter().enumerate()
    {
        println!(
            "Processing {}/{} {}",
            data_type_index + 1,
            total_data_types,
            data_type
        );

        for (power, data_type_grouped_data) in power_groups_for_data_type {
            for &zstd_level in &zstd_levels {
                match find_optimal_coefficients(
                    &data_type_grouped_data,
                    &data_type,
                    power,
                    zstd_level,
                    &config,
                ) {
                    Ok(result) => all_results.push(result),
                    Err(_e) => {
                        // Silently skip failed optimizations
                    }
                }
            }
        }
    }

    // Print comprehensive summary
    print_comprehensive_summary(&all_results, &zstd_levels, &data_points, cmd.min_files);

    Ok(())
}

/// Load data points from JSON file
fn load_data_points(input_file: &PathBuf) -> Result<Vec<EstimatorDataPoint>, TransformError> {
    let file = File::open(input_file)
        .map_err(|e| TransformError::Debug(format!("Failed to open input file: {e}")))?;

    let reader = BufReader::new(file);
    let data_points: Vec<EstimatorDataPoint> = serde_json::from_reader(reader)
        .map_err(|e| TransformError::Debug(format!("Failed to parse JSON: {e}")))?;

    Ok(data_points)
}

/// Group data points (file_name + data_type combinations) by data type, then by power of 2 of their size
fn group_by_data_type_and_power_of_2(
    data_points: &[EstimatorDataPoint],
) -> HashMap<String, HashMap<u32, Vec<&EstimatorDataPoint>>> {
    let mut groups: HashMap<String, HashMap<u32, Vec<&EstimatorDataPoint>>> = HashMap::new();

    for point in data_points {
        let power = super::get_power_of_2_for_size(point.data_size);
        groups
            .entry(point.data_type.clone())
            .or_default()
            .entry(power)
            .or_default()
            .push(point);
    }

    groups
}

/// Find optimal coefficients for a group of data points and specific ZStandard level
fn find_optimal_coefficients(
    data_points: &[&EstimatorDataPoint],
    data_type: &str,
    power: u32,
    zstd_level: i32,
    config: &BruteForceConfig,
) -> Result<OptimizationResult, TransformError> {
    if data_points.is_empty() {
        return Err(TransformError::Debug("No data points in group".to_string()));
    }

    // Check if any data points have data for this ZStandard level
    let valid_points: Vec<_> = data_points
        .iter()
        .filter(|point| point.zstd_sizes.contains_key(&zstd_level))
        .cloned()
        .collect();

    if valid_points.is_empty() {
        return Err(TransformError::Debug(format!(
            "No data points have ZStandard level {zstd_level} data"
        )));
    }

    // Generate all parameter combinations
    let mut combinations = Vec::new();
    let mut lz = config.min_lz_multiplier;
    while lz <= config.max_lz_multiplier {
        let mut entropy = config.min_entropy_multiplier;
        while entropy <= config.max_entropy_multiplier {
            combinations.push(EstimationSettings {
                lz_match_multiplier: lz,
                entropy_multiplier: entropy,
            });
            entropy += config.entropy_step_size;
        }
        lz += config.lz_step_size;
    }

    // Test all combinations in parallel
    let results: Vec<(EstimationSettings, f64, f64, f64, f64)> = combinations
        .into_par_iter()
        .map(|settings| {
            let (mean_abs_error, mean_percent_error, avg_compressed, avg_estimated) =
                evaluate_settings(&settings, &valid_points, zstd_level);
            (
                settings,
                mean_abs_error,
                mean_percent_error,
                avg_compressed,
                avg_estimated,
            )
        })
        .collect();

    // Find the best combination (minimize mean absolute error)
    let (optimal_settings, mae, mpe, avg_compressed, avg_estimated) = results
        .into_iter()
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();

    Ok(OptimizationResult {
        data_type: data_type.to_string(),
        power,
        zstd_level,
        optimal_settings,
        mean_absolute_error: mae,
        mean_percentage_error: mpe,
        average_compressed_size: avg_compressed,
        average_estimated_size: avg_estimated,
    })
}

/// Evaluate how good a set of estimation settings is
/// Returns (mean_absolute_error, mean_percentage_error, average_compressed_size, average_estimated_size)
fn evaluate_settings(
    settings: &EstimationSettings,
    data_points: &[&EstimatorDataPoint],
    zstd_level: i32,
) -> (f64, f64, f64, f64) {
    let mut absolute_errors = Vec::new();
    let mut percentage_errors = Vec::new();
    let mut compressed_sizes = Vec::new();
    let mut estimated_sizes = Vec::new();

    for point in data_points {
        // Get the actual compressed size for this zstd level
        let actual_size = match point.zstd_sizes.get(&zstd_level) {
            Some(&size) => size,
            None => continue, // Skip if no data for this level
        };

        if actual_size == 0 {
            continue; // Skip empty compressions
        }

        // Calculate estimated size using these settings
        let params = SizeEstimationParameters {
            data_len: point.ltu_params.data_len,
            num_lz_matches: point.ltu_params.num_lz_matches,
            entropy: point.ltu_params.entropy,
        };

        let estimated_size = size_estimate(params, *settings);

        // Calculate errors
        let absolute_error = (estimated_size as isize - actual_size as isize).unsigned_abs();
        let percentage_error = absolute_error as f64 / actual_size as f64;

        absolute_errors.push(absolute_error);
        percentage_errors.push(percentage_error);
        compressed_sizes.push(actual_size);
        estimated_sizes.push(estimated_size);
    }

    if absolute_errors.is_empty() {
        return (f64::INFINITY, f64::INFINITY, 0.0, 0.0);
    }

    let mean_absolute_error =
        absolute_errors.iter().sum::<usize>() as f64 / absolute_errors.len() as f64;
    let mean_percentage_error =
        percentage_errors.iter().sum::<f64>() / percentage_errors.len() as f64;
    let average_compressed_size =
        compressed_sizes.iter().sum::<usize>() as f64 / compressed_sizes.len() as f64;
    let average_estimated_size =
        estimated_sizes.iter().sum::<usize>() as f64 / estimated_sizes.len() as f64;

    (
        mean_absolute_error,
        mean_percentage_error,
        average_compressed_size,
        average_estimated_size,
    )
}

/// Print a comprehensive summary of all results
fn print_comprehensive_summary(
    results: &[OptimizationResult],
    zstd_levels: &[i32],
    data_points: &[EstimatorDataPoint],
    min_files: usize,
) {
    println!("\n==========================================");
    println!("      OPTIMAL COEFFICIENTS SUMMARY");
    println!("==========================================");
    println!("ZStandard Levels: {zstd_levels:?}");
    println!("Total Results: {}", results.len());

    // Group results by power first, then by data type
    let mut power_groups: HashMap<u32, HashMap<String, Vec<&OptimizationResult>>> = HashMap::new();
    for result in results {
        power_groups
            .entry(result.power)
            .or_default()
            .entry(result.data_type.clone())
            .or_default()
            .push(result);
    }

    // Sort powers ascending
    let mut sorted_powers: Vec<_> = power_groups.keys().cloned().collect();
    sorted_powers.sort();

    for power in sorted_powers {
        if let Some(data_type_groups) = power_groups.get(&power) {
            // Calculate bytes for this power of 2
            let bytes = 1u64 << power;
            let size_str = ByteSize::b(bytes).to_string();

            // Get all data types for this power level and calculate total data points
            let mut sorted_data_types: Vec<_> = data_type_groups.keys().cloned().collect();
            sorted_data_types.sort();

            // Count data points (file_name + data_type combinations) for this power level
            let total_data_points = data_points
                .iter()
                .filter(|point| super::get_power_of_2_for_size(point.data_size) == power)
                .count();

            // Count unique files for this power level
            let unique_file_count = total_data_points / 4; // each file generates 4 data points for BC1
            if unique_file_count < min_files {
                continue; // Skip entire power level if no data type has enough files
            }

            println!(
                "\n📏 {} ({} data points from {} files) - Data Types: {}",
                size_str,
                total_data_points,
                unique_file_count,
                sorted_data_types.join(", ")
            );

            // Process each data type for this power level
            for data_type in sorted_data_types {
                if let Some(type_results) = data_type_groups.get(&data_type) {
                    // Sort by ZStandard level
                    let mut sorted_results = type_results.to_vec();
                    sorted_results.sort_by_key(|r| r.zstd_level);

                    for result in sorted_results {
                        println!(
                            "  {} ZStd{}: lz={:.4}, entropy={:.4} (AvgErr={:.1}B, AvgErr%={:.1}%) | AvgCompressed={:.1}B, AvgEstimated={:.1}B",
                            result.data_type,
                            result.zstd_level,
                            result.optimal_settings.lz_match_multiplier,
                            result.optimal_settings.entropy_multiplier,
                            result.mean_absolute_error,
                            result.mean_percentage_error * 100.0,
                            result.average_compressed_size,
                            result.average_estimated_size,
                        );
                    }
                }
            }
        }
    }

    println!("==========================================");
}
