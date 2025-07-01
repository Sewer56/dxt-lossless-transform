use super::{EstimatorDataPoint, LtuParameters, TuningCheckpoint};
use crate::{error::TransformError, util::find_all_files};
use argh::FromArgs;
use bytesize::ByteSize;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// Context for checkpoint operations
struct CheckpointContext<'a> {
    checkpoint_path: &'a Path,
    zstd_levels: &'a [i32],
    transform_filter: TransformFormatFilter,
    input_directory: &'a Path,
    total_files: usize,
}

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
    let checkpoint_path = get_checkpoint_path(&cmd.output);

    // Set up signal handling for Ctrl+C
    let should_exit = setup_signal_handler()?;

    // Collect all files using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(&cmd.input_directory, &mut entries)?;
    let total_files = entries.len();
    println!("Found {total_files} files to analyze");

    if entries.is_empty() {
        return Err(TransformError::Debug("No files found".to_string()));
    }

    // Load and validate checkpoint
    let (mut processed_files, mut all_data_points) = load_and_validate_checkpoint(
        &checkpoint_path,
        &cmd.input_directory,
        &zstd_levels,
        transform_filter,
    )?;

    // Filter out already processed files
    let remaining_entries = filter_remaining_files(entries, &processed_files);

    println!(
        "Processing {} remaining files ({} already processed)",
        remaining_entries.len(),
        processed_files.len()
    );

    if remaining_entries.is_empty() {
        println!("All files have been processed! Writing final output...");
    } else {
        // Process remaining files with checkpointing
        let context = CheckpointContext {
            checkpoint_path: &checkpoint_path,
            zstd_levels: &zstd_levels,
            transform_filter,
            input_directory: &cmd.input_directory,
            total_files,
        };

        let (final_data_points, final_processed_files) = process_files_with_checkpointing(
            &remaining_entries,
            &should_exit,
            &all_data_points,
            &processed_files,
            &context,
        )?;

        // Merge final results
        all_data_points.extend(final_data_points);
        processed_files.extend(final_processed_files);
    }

    // Write final output
    write_final_output(&cmd.output, &all_data_points, &zstd_levels)?;

    // Clean up checkpoint file on successful completion
    cleanup_checkpoint(&checkpoint_path)?;

    Ok(())
}

/// Generate checkpoint file path from output path
fn get_checkpoint_path(output_path: &Path) -> PathBuf {
    output_path.with_extension("checkpoint.json")
}

/// Save checkpoint data to disk
fn save_checkpoint(
    checkpoint_path: &Path,
    checkpoint: &TuningCheckpoint,
) -> Result<(), TransformError> {
    let checkpoint_file = File::create(checkpoint_path)
        .map_err(|e| TransformError::Debug(format!("Failed to create checkpoint file: {e}")))?;

    serde_json::to_writer(checkpoint_file, checkpoint)
        .map_err(|e| TransformError::Debug(format!("Failed to write checkpoint: {e}")))?;

    println!("Checkpoint saved to: {}", checkpoint_path.display());
    Ok(())
}

/// Load checkpoint data from disk if it exists
fn load_checkpoint(checkpoint_path: &Path) -> Result<Option<TuningCheckpoint>, TransformError> {
    if !checkpoint_path.exists() {
        return Ok(None);
    }

    let checkpoint_data = fs::read_to_string(checkpoint_path)
        .map_err(|e| TransformError::Debug(format!("Failed to read checkpoint file: {e}")))?;

    let checkpoint: TuningCheckpoint = serde_json::from_str(&checkpoint_data)
        .map_err(|e| TransformError::Debug(format!("Failed to parse checkpoint file: {e}")))?;

    println!("Loaded checkpoint from: {}", checkpoint_path.display());
    println!(
        "Previously processed {} files with {} data points",
        checkpoint.processed_files.len(),
        checkpoint.accumulated_data.len()
    );

    Ok(Some(checkpoint))
}

/// Remove checkpoint file
fn cleanup_checkpoint(checkpoint_path: &Path) -> Result<(), TransformError> {
    if checkpoint_path.exists() {
        fs::remove_file(checkpoint_path)
            .map_err(|e| TransformError::Debug(format!("Failed to remove checkpoint file: {e}")))?;
        println!("Checkpoint file cleaned up");
    }
    Ok(())
}

/// Set up signal handling for Ctrl+C
fn setup_signal_handler() -> Result<Arc<AtomicBool>, TransformError> {
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_clone = should_exit.clone();

    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C signal. Saving checkpoint and exiting gracefully...");
        should_exit_clone.store(true, Ordering::Relaxed);
    })
    .map_err(|e| TransformError::Debug(format!("Failed to set Ctrl+C handler: {e}")))?;

    Ok(should_exit)
}

/// Load and validate checkpoint compatibility with current parameters
fn load_and_validate_checkpoint(
    checkpoint_path: &Path,
    input_directory: &Path,
    zstd_levels: &[i32],
    transform_filter: TransformFormatFilter,
) -> Result<(HashSet<String>, Vec<EstimatorDataPoint>), TransformError> {
    let checkpoint = load_checkpoint(checkpoint_path)?;

    match checkpoint {
        Some(chkpt) => {
            // Validate checkpoint compatibility
            if chkpt.input_directory != input_directory.to_string_lossy()
                || chkpt.zstd_levels != zstd_levels
                || chkpt.filter != format!("{transform_filter:?}")
            {
                println!(
                    "Warning: Checkpoint parameters don't match current command. Starting fresh."
                );
                Ok((HashSet::new(), Vec::new()))
            } else {
                println!("Resuming from checkpoint...");
                Ok((chkpt.processed_files, chkpt.accumulated_data))
            }
        }
        None => {
            println!("No checkpoint found. Starting fresh.");
            Ok((HashSet::new(), Vec::new()))
        }
    }
}

/// Filter out already processed files from the file list
fn filter_remaining_files(
    entries: Vec<std::fs::DirEntry>,
    processed_files: &HashSet<String>,
) -> Vec<std::fs::DirEntry> {
    entries
        .into_iter()
        .filter(|entry| {
            let file_path_str = entry.path().to_string_lossy().to_string();
            !processed_files.contains(&file_path_str)
        })
        .collect()
}

/// Save a periodic checkpoint during processing
fn save_periodic_checkpoint(
    context: &CheckpointContext,
    all_data_points: &[EstimatorDataPoint],
    processed_files: &HashSet<String>,
    current_data_points: &[EstimatorDataPoint],
    current_processed_files: &HashSet<String>,
) -> Result<(), TransformError> {
    // Merge with previous data
    let mut combined_data = all_data_points.to_vec();
    combined_data.extend_from_slice(current_data_points);

    let mut combined_processed = processed_files.clone();
    combined_processed.extend(current_processed_files.iter().cloned());

    let checkpoint = TuningCheckpoint {
        processed_files: combined_processed,
        accumulated_data: combined_data,
        zstd_levels: context.zstd_levels.to_vec(),
        filter: format!("{:?}", context.transform_filter),
        input_directory: context.input_directory.to_string_lossy().to_string(),
        total_files: context.total_files,
    };

    save_checkpoint(context.checkpoint_path, &checkpoint)
}

/// Handle processing interruption by saving final checkpoint
fn handle_processing_interruption(
    context: &CheckpointContext,
    all_data_points: &mut Vec<EstimatorDataPoint>,
    processed_files: &mut HashSet<String>,
    final_data_points: Vec<EstimatorDataPoint>,
    final_processed_files: HashSet<String>,
) -> TransformError {
    // Merge final results
    all_data_points.extend(final_data_points);
    processed_files.extend(final_processed_files);

    let final_checkpoint = TuningCheckpoint {
        processed_files: processed_files.clone(),
        accumulated_data: all_data_points.clone(),
        zstd_levels: context.zstd_levels.to_vec(),
        filter: format!("{:?}", context.transform_filter),
        input_directory: context.input_directory.to_string_lossy().to_string(),
        total_files: context.total_files,
    };

    if let Err(save_err) = save_checkpoint(context.checkpoint_path, &final_checkpoint) {
        eprintln!("Warning: Failed to save final checkpoint: {save_err}");
    } else {
        println!("Progress saved. You can resume by running the same command again.");
    }

    TransformError::Debug("Interrupted by user".to_string())
}

/// Process files with periodic checkpointing
fn process_files_with_checkpointing(
    remaining_entries: &[std::fs::DirEntry],
    should_exit: &Arc<AtomicBool>,
    checkpoint_data_points: &[EstimatorDataPoint],
    checkpoint_processed_files: &HashSet<String>,
    context: &CheckpointContext,
) -> Result<(Vec<EstimatorDataPoint>, HashSet<String>), TransformError> {
    let files_processed = AtomicUsize::new(0);
    let files_with_data_processed = AtomicUsize::new(0);
    let accumulated_data_size = AtomicUsize::new(0);
    let data_points_mutex = Mutex::new(Vec::<EstimatorDataPoint>::new());
    let processed_files_mutex = Mutex::new(HashSet::<String>::new());

    // Checkpoint every 256MB of data processed
    const CHECKPOINT_INTERVAL_BYTES: usize = 256 * 1024 * 1024; // 256MB

    let processing_result = remaining_entries.par_iter().with_max_len(1).try_for_each(
        |entry| -> Result<(), TransformError> {
            // Check if we should exit early
            if should_exit.load(Ordering::Relaxed) {
                return Err(TransformError::Debug("Interrupted by user".to_string()));
            }

            let file_path = entry.path();
            let file_path_str = file_path.to_string_lossy().to_string();

            match process_file(&file_path, context.transform_filter, context.zstd_levels) {
                Some(data_points) => {
                    let current_count = files_processed.fetch_add(1, Ordering::Relaxed) + 1;
                    let _data_processed_count =
                        files_with_data_processed.fetch_add(1, Ordering::Relaxed) + 1;

                    // Calculate total data size for this file
                    let file_data_size: usize = data_points.first().unwrap().data_size;
                    let total_data_processed = accumulated_data_size
                        .fetch_add(file_data_size, Ordering::Relaxed)
                        + file_data_size;

                    println!(
                        "[{}/{}] Processed: {}",
                        checkpoint_processed_files.len() + current_count,
                        context.total_files,
                        file_path.display()
                    );

                    // Add to local collections
                    data_points_mutex.lock().unwrap().extend(data_points);
                    processed_files_mutex.lock().unwrap().insert(file_path_str);

                    // Save checkpoint periodically based on data size (every 1GB)
                    if total_data_processed / CHECKPOINT_INTERVAL_BYTES
                        > (total_data_processed - file_data_size) / CHECKPOINT_INTERVAL_BYTES
                    {
                        let current_data_points = data_points_mutex.lock().unwrap().clone();
                        let current_processed_files = processed_files_mutex.lock().unwrap().clone();

                        println!(
                            "Checkpoint triggered after {} of data processed",
                            ByteSize::b(CHECKPOINT_INTERVAL_BYTES as u64)
                        );

                        if let Err(e) = save_periodic_checkpoint(
                            context,
                            checkpoint_data_points,
                            checkpoint_processed_files,
                            &current_data_points,
                            &current_processed_files,
                        ) {
                            eprintln!("Warning: Failed to save checkpoint: {e}");
                        }
                    }
                }
                None => {
                    let current_count = files_processed.fetch_add(1, Ordering::Relaxed) + 1;
                    println!(
                        "[{}/{}] [SKIP - Wrong Format]: {}",
                        checkpoint_processed_files.len() + current_count,
                        context.total_files,
                        file_path.display()
                    );

                    // Still mark as processed to avoid reprocessing
                    processed_files_mutex.lock().unwrap().insert(file_path_str);
                }
            }

            Ok(())
        },
    );

    // Handle result
    match processing_result {
        Ok(()) => {
            let final_data_points = data_points_mutex.into_inner().unwrap();
            let final_processed_files = processed_files_mutex.into_inner().unwrap();
            Ok((final_data_points, final_processed_files))
        }
        Err(_) => {
            let final_data_points = data_points_mutex.into_inner().unwrap();
            let final_processed_files = processed_files_mutex.into_inner().unwrap();

            Err(handle_processing_interruption(
                context,
                &mut checkpoint_data_points.to_vec(),
                &mut checkpoint_processed_files.clone(),
                final_data_points,
                final_processed_files,
            ))
        }
    }
}

/// Write final output files and display completion message
fn write_final_output(
    output_path: &Path,
    all_data_points: &[EstimatorDataPoint],
    zstd_levels: &[i32],
) -> Result<(), TransformError> {
    println!("\nTotal data points collected: {}", all_data_points.len());

    write_output_files(output_path, all_data_points, zstd_levels)?;

    println!("Output written to:");
    println!("  JSON: {}.json", output_path.display());
    println!("  CSV: {}.csv", output_path.display());

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
    serde_json::to_writer(json_file, data_points)
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
