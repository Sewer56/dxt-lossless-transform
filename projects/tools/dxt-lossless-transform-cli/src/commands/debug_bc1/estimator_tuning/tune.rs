use super::{EstimatorDataPoint, LtuParameters};
use crate::{error::TransformError, util::find_all_files};
use argh::FromArgs;
use core::sync::atomic::{AtomicUsize, Ordering};
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};
use dxt_lossless_transform_bc1::{transform_bc1_with_settings, Bc1TransformSettings};
use dxt_lossless_transform_common::{allocate::allocate_align_64, color_565::YCoCgVariant};
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::embed::TransformFormat;
use dxt_lossless_transform_file_formats_debug::{
    extract_blocks_from_file_format, TransformFormatFilter,
};
use dxt_lossless_transform_zstd::ZStandardSizeEstimation;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(FromArgs, Debug)]
/// gather data for tuning the lossless-transform-utils estimator coefficients
#[argh(subcommand, name = "tune-estimator")]
pub struct TuneEstimatorCmd {
    /// input directory path to analyze (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,

    /// filter for file type (bc1, bc2, bc3, bc7, bc6h, all)
    #[argh(option)]
    pub filter: TransformFormatFilter,

    /// output file path (without extension, will create .json and .csv)
    #[argh(option)]
    pub output: PathBuf,

    /// ZStandard compression levels to test (comma-separated, e.g., "1,6,22" or "all" for 1-22)
    #[argh(option, default = "String::from(\"all\")")]
    pub zstd_levels: String,
}

pub fn handle_tune_estimator_command(cmd: TuneEstimatorCmd) -> Result<(), TransformError> {
    println!("Gathering data for estimator tuning...");
    println!("Input directory: {:?}", cmd.input_directory);
    println!("Filter: {:?}", cmd.filter);

    // Parse ZStandard levels
    let zstd_levels = super::parse_zstd_levels(&cmd.zstd_levels)?;
    println!("ZStandard levels: {zstd_levels:?}");

    let transform_filter = cmd.filter;

    // Collect all files using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(&cmd.input_directory, &mut entries)?;
    println!("Found {} files to analyze", entries.len());

    if entries.is_empty() {
        return Err(TransformError::Debug("No files found".to_string()));
    }

    // Process each file in parallel
    let files_processed = AtomicUsize::new(0);
    let all_data_points = Mutex::new(Vec::<EstimatorDataPoint>::new());

    entries
        .par_iter()
        // 1 item at once per thread. Our items are big generally, and take time to process
        // so 'max work stealing' is preferred.
        .with_max_len(1)
        .for_each(|entry| {
            let file_path = entry.path();

            match process_file(&file_path, transform_filter, &zstd_levels) {
                Some(data_points) => {
                    let current_count = files_processed.fetch_add(1, Ordering::Relaxed) + 1;
                    println!(
                        "[{}/{}] Processed: {}",
                        current_count,
                        entries.len(),
                        file_path.display()
                    );
                    all_data_points.lock().unwrap().extend(data_points);
                }
                None => {
                    // File doesn't match filter - skip silently
                    let current_count = files_processed.fetch_add(1, Ordering::Relaxed) + 1;
                    println!(
                        "[{}/{}] [SKIP - Wrong Format]: {}",
                        current_count,
                        entries.len(),
                        file_path.display()
                    );
                }
            }
        });

    let all_data_points = all_data_points.into_inner().unwrap();

    println!("\nTotal data points collected: {}", all_data_points.len());

    // Write output files
    write_output_files(&cmd.output, &all_data_points, &zstd_levels)?;

    println!("Output written to:");
    println!("  JSON: {}.json", cmd.output.display());
    println!("  CSV: {}.csv", cmd.output.display());

    Ok(())
}

/// Process a single file and collect data for all transform modes
/// Returns None if the file doesn't match the filter (should be skipped)
fn process_file(
    file_path: &Path,
    transform_filter: TransformFormatFilter,
    zstd_levels: &[i32],
) -> Option<Vec<EstimatorDataPoint>> {
    let handlers = [DdsHandler];
    let mut all_data_points = Vec::new();

    let file_name = file_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Use the file-formats-api to extract blocks
    let result = extract_blocks_from_file_format(
        file_path,
        &handlers,
        transform_filter,
        |data: &[u8], _format: TransformFormat| {
            let data_size = data.len(); // Use actual block data size, not file size

            // Process each transform mode (callback only called for filtered format)
            let transform_modes = [
                (DataType::Bc1Colours, false, false),
                (DataType::Bc1DecorrelatedColours, false, true),
                (DataType::Bc1SplitColours, true, false),
                (DataType::Bc1SplitDecorrelatedColours, true, true),
            ];

            for (data_type, split_endpoints, decorrelate) in transform_modes {
                match process_bc1_transform_mode(
                    data,
                    &file_name,
                    data_size,
                    data_type,
                    split_endpoints,
                    decorrelate,
                    zstd_levels,
                ) {
                    Ok(data_point) => {
                        all_data_points.push(data_point);
                    }
                    Err(e) => {
                        eprintln!("    Error processing transform mode {data_type:?}: {e}");
                    }
                }
            }

            Ok(())
        },
    );

    match result {
        Ok(()) => Some(all_data_points),
        Err(_e) => {
            // File doesn't match filter - skip completely
            None
        }
    }
}

/// Process a BC1 transform mode
fn process_bc1_transform_mode(
    data: &[u8],
    file_name: &str,
    data_size: usize,
    data_type: DataType,
    split_endpoints: bool,
    decorrelate: bool,
    zstd_levels: &[i32],
) -> Result<EstimatorDataPoint, TransformError> {
    // Transform the data
    let transformed_data = transform_bc1_data_from_slice(data, split_endpoints, decorrelate)?;

    // Calculate LTU parameters
    let ltu_params = calculate_ltu_parameters(&transformed_data)?;

    // Compress with different ZStandard levels
    let zstd_sizes = compress_with_all_levels(&transformed_data, zstd_levels)?;

    Ok(EstimatorDataPoint {
        file_name: file_name.to_string(),
        data_size,
        data_type: format!("{data_type:?}"),
        zstd_sizes,
        ltu_params,
    })
}

/// Transform BC1 data with the specified settings
fn transform_bc1_data_from_slice(
    data: &[u8],
    split_endpoints: bool,
    decorrelate: bool,
) -> Result<Vec<u8>, TransformError> {
    let settings = Bc1TransformSettings {
        split_colour_endpoints: split_endpoints,
        decorrelation_mode: if decorrelate {
            YCoCgVariant::Variant1
        } else {
            YCoCgVariant::None
        },
    };

    // Allocate output buffer
    let mut output = allocate_align_64(data.len())
        .map_err(|e| TransformError::Debug(format!("Failed to allocate output buffer: {e:?}")))?;

    // Transform the data
    unsafe {
        transform_bc1_with_settings(data.as_ptr(), output.as_mut_ptr(), data.len(), settings);
    }

    Ok(output.as_slice().to_vec())
}

/// Calculate LTU parameters from the data
pub fn calculate_ltu_parameters(data: &[u8]) -> Result<LtuParameters, TransformError> {
    use lossless_transform_utils::{
        entropy::code_length_of_histogram32,
        histogram::{histogram32_from_bytes, Histogram32},
        match_estimator::estimate_num_lz_matches_fast,
    };

    let num_lz_matches = estimate_num_lz_matches_fast(data);

    let mut histogram = Histogram32::default();
    histogram32_from_bytes(data, &mut histogram);
    let entropy = code_length_of_histogram32(&histogram, data.len() as u64);

    Ok(LtuParameters {
        data_len: data.len(),
        num_lz_matches,
        entropy,
    })
}

/// Compress data with specified ZStandard levels and return sizes
fn compress_with_all_levels(
    data: &[u8],
    zstd_levels: &[i32],
) -> Result<HashMap<i32, usize>, TransformError> {
    let mut sizes = HashMap::new();

    // Test specified ZStandard levels
    for &level in zstd_levels {
        let estimator = ZStandardSizeEstimation::new(level).map_err(|e| {
            TransformError::Debug(format!("Failed to create ZStandard estimator: {e:?}"))
        })?;

        let max_size = estimator.max_compressed_size(data.len()).map_err(|e| {
            TransformError::Debug(format!("Failed to get max compressed size: {e:?}"))
        })?;

        let mut output = allocate_align_64(max_size).map_err(|e| {
            TransformError::Debug(format!("Failed to allocate compression buffer: {e:?}"))
        })?;

        let compressed_size = unsafe {
            estimator
                .estimate_compressed_size(
                    data.as_ptr(),
                    data.len(),
                    DataType::Unknown, // ZStandard doesn't use data type
                    output.as_mut_ptr(),
                    output.len(),
                )
                .map_err(|e| {
                    TransformError::Debug(format!("Failed to compress at level {level}: {e:?}"))
                })?
        };

        sizes.insert(level, compressed_size);
    }

    Ok(sizes)
}

/// Write output files in JSON and CSV formats
fn write_output_files(
    output_path: &Path,
    data_points: &[EstimatorDataPoint],
    zstd_levels: &[i32],
) -> Result<(), TransformError> {
    // Write JSON file
    let json_path = output_path.with_extension("json");
    let json_file = File::create(&json_path)
        .map_err(|e| TransformError::Debug(format!("Failed to create JSON file: {e}")))?;
    serde_json::to_writer_pretty(json_file, data_points)
        .map_err(|e| TransformError::Debug(format!("Failed to write JSON: {e}")))?;

    // Write CSV file
    let csv_path = output_path.with_extension("csv");
    let mut csv_file = File::create(&csv_path)
        .map_err(|e| TransformError::Debug(format!("Failed to create CSV file: {e}")))?;

    // Write CSV header with dynamic ZStandard level columns
    let zstd_columns: Vec<String> = zstd_levels
        .iter()
        .map(|level| format!("zstd_{level}"))
        .collect();
    let header = format!(
        "file_name,data_size,data_type,num_lz_matches,entropy,{}",
        zstd_columns.join(",")
    );
    writeln!(csv_file, "{header}")
        .map_err(|e| TransformError::Debug(format!("Failed to write CSV header: {e}")))?;

    // Write data rows
    for point in data_points {
        write!(
            csv_file,
            "{},{},{},{},{:.6}",
            point.file_name,
            point.data_size,
            point.data_type,
            point.ltu_params.num_lz_matches,
            point.ltu_params.entropy,
        )
        .map_err(|e| TransformError::Debug(format!("Failed to write CSV row: {e}")))?;

        // Write ZStandard sizes for specified levels
        for &level in zstd_levels {
            let size = point.zstd_sizes.get(&level).unwrap_or(&0);
            write!(csv_file, ",{size}")
                .map_err(|e| TransformError::Debug(format!("Failed to write CSV value: {e}")))?;
        }

        writeln!(csv_file)
            .map_err(|e| TransformError::Debug(format!("Failed to write CSV line ending: {e}")))?;
    }

    Ok(())
}
