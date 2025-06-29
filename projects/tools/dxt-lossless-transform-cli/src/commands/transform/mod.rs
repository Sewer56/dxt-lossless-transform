use crate::util::Throughput;
use crate::util::{canonicalize_cli_path, find_all_files};
use argh::FromArgs;
use bytesize::ByteSize;
use dxt_lossless_transform_api_common::estimate::NoEstimation;
use dxt_lossless_transform_bc1_api::{Bc1AutoTransformBuilder, Bc1ManualTransformBuilder};
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::{file_io, TransformBundle};

use std::{
    path::{Path, PathBuf},
    time::Instant,
};

#[derive(FromArgs, Debug)]
/// Transform DDS files using lossless compression optimization (Demo CLI - use API for production)
#[argh(subcommand, name = "transform")]
pub struct TransformCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,

    /// compression preset: low, medium, optimal, max [default: optimal]
    #[argh(option, default = "CompressionPreset::Optimal")]
    pub preset: CompressionPreset,
}

/// Compression preset options for DXT lossless transform
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CompressionPreset {
    /// Default manual settings for fast processing
    Low,
    /// Automatic optimization using LTU estimator
    Medium,
    /// Automatic optimization using ZStandard level 1
    Optimal,
    /// Automatic optimization with brute force settings using ZStandard
    Max,
}

impl std::str::FromStr for CompressionPreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "optimal" => Ok(Self::Optimal),
            "max" => Ok(Self::Max),
            _ => Err(format!(
                "Unknown preset: {s}. Valid options: low, medium, optimal, max"
            )),
        }
    }
}

pub fn handle_transform_command(cmd: TransformCmd) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DXT Lossless Transform CLI Demo ===");
    println!("Note: This CLI is for demonstration purposes only.");
    println!("For production use, integrate the API directly into your application.\n");

    // Collect all files
    let mut entries = Vec::new();
    find_all_files(&cmd.input, &mut entries)?;

    if entries.is_empty() {
        println!("No files found in input directory.");
        return Ok(());
    }

    println!("Found {} files to process\n", entries.len());

    let start = Instant::now();

    let total_bytes = match cmd.preset {
        CompressionPreset::Low => {
            let bundle = create_low_preset_bundle()?;
            process_files_with_bundle(&entries, &cmd.input, &cmd.output, &bundle)
        }
        CompressionPreset::Medium => {
            let bundle = create_medium_preset_bundle()?;
            process_files_with_bundle(&entries, &cmd.input, &cmd.output, &bundle)
        }
        CompressionPreset::Optimal => {
            let bundle = create_optimal_preset_bundle()?;
            process_files_with_bundle(&entries, &cmd.input, &cmd.output, &bundle)
        }
        CompressionPreset::Max => {
            let bundle = create_max_preset_bundle()?;
            process_files_with_bundle(&entries, &cmd.input, &cmd.output, &bundle)
        }
    };

    let elapsed = start.elapsed();
    let data_size = ByteSize(total_bytes);
    let throughput = if elapsed.as_secs_f64() > 0.0 {
        Throughput::from_bytes_per_sec((total_bytes as f64 / elapsed.as_secs_f64()) as u64)
    } else {
        Throughput::from_bytes_per_sec(0)
    };

    println!("\n=== Transform Complete ===");
    println!("Time taken: {elapsed:.2?}");
    println!("Data processed: {data_size}");
    println!("Throughput: {throughput}");

    Ok(())
}

/// Create transform bundle for low preset (manual settings)
fn create_low_preset_bundle() -> Result<TransformBundle<NoEstimation>, Box<dyn std::error::Error>> {
    // For Low preset, Bc1ManualTransformBuilder::new() already gives default settings
    let bundle =
        TransformBundle::<NoEstimation>::new().with_bc1_manual(Bc1ManualTransformBuilder::new());
    Ok(bundle)
}

/// Create transform bundle for medium preset (LTU estimator)
fn create_medium_preset_bundle() -> Result<
    TransformBundle<dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation>,
    Box<dyn std::error::Error>,
> {
    let estimator = dxt_lossless_transform_ltu::LosslessTransformUtilsSizeEstimation::new();
    let bundle = TransformBundle::new().with_bc1_auto(Bc1AutoTransformBuilder::new(estimator));
    Ok(bundle)
}

/// Create transform bundle for optimal preset (ZStandard level 1)
fn create_optimal_preset_bundle() -> Result<
    TransformBundle<dxt_lossless_transform_zstd::ZStandardSizeEstimation>,
    Box<dyn std::error::Error>,
> {
    // Create ZStandard level 1 estimator
    let estimator = dxt_lossless_transform_zstd::ZStandardSizeEstimation::new(1)?;
    let bundle = TransformBundle::new().with_bc1_auto(Bc1AutoTransformBuilder::new(estimator));
    Ok(bundle)
}

/// Create transform bundle for max preset (ZStandard with brute force)
fn create_max_preset_bundle() -> Result<
    TransformBundle<dxt_lossless_transform_zstd::ZStandardSizeEstimation>,
    Box<dyn std::error::Error>,
> {
    // Create ZStandard level 1 estimator
    let estimator = dxt_lossless_transform_zstd::ZStandardSizeEstimation::new(1)?;
    let bundle =
        TransformBundle::new().with_bc1_auto(Bc1AutoTransformBuilder::new_ultra(estimator));
    Ok(bundle)
}

/// Process all files using the provided bundle and return total bytes processed
fn process_files_with_bundle<T>(
    entries: &[std::fs::DirEntry],
    input_dir: &Path,
    output_dir: &Path,
    bundle: &TransformBundle<T>,
) -> u64
where
    T: dxt_lossless_transform_api_common::estimate::SizeEstimationOperations + Sync,
    T::Error: std::fmt::Debug,
{
    use crate::util::handle_process_entry_error;

    #[cfg(feature = "multithreaded")]
    {
        use rayon::prelude::*;
        use std::sync::atomic::{AtomicU64, Ordering};

        let bytes_processed = AtomicU64::new(0);
        entries
            .par_iter()
            // 1 item at once per thread. Our items are big generally, and take time to process
            // so 'max work stealing' is preferred.
            .with_max_len(1)
            .for_each(
                |entry| match process_file_transform(entry, input_dir, output_dir, bundle) {
                    Ok(bytes) => {
                        bytes_processed.fetch_add(bytes, Ordering::Relaxed);
                    }
                    Err(e) => handle_process_entry_error(Err(e)),
                },
            );
        bytes_processed.load(Ordering::Relaxed)
    }

    #[cfg(not(feature = "multithreaded"))]
    {
        let mut bytes_processed = 0u64;
        for entry in entries {
            match process_file_transform(entry, input_dir, output_dir, bundle) {
                Ok(bytes) => bytes_processed += bytes,
                Err(e) => handle_process_entry_error(Err(e)),
            }
        }
        bytes_processed
    }
}

/// Process a single file transform - returns bytes processed
fn process_file_transform<T>(
    entry: &std::fs::DirEntry,
    input_dir: &Path,
    output_dir: &Path,
    bundle: &TransformBundle<T>,
) -> Result<u64, crate::error::TransformError>
where
    T: dxt_lossless_transform_api_common::estimate::SizeEstimationOperations,
    T::Error: std::fmt::Debug,
{
    let path = entry.path();
    let relative = path.strip_prefix(input_dir).unwrap();
    let target_path = output_dir.join(relative);

    // Create output directory if needed
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Get file size for throughput calculation
    let bytes = if let Ok(metadata) = std::fs::metadata(&path) {
        metadata.len()
    } else {
        0
    };

    // Try different file format handlers in sequence using detection
    // Use the new wrapper API that handles multiple handlers automatically
    let handlers = [DdsHandler];

    file_io::transform_file_with_multiple_handlers(handlers, &path, &target_path, bundle)?;

    Ok(bytes)
}
