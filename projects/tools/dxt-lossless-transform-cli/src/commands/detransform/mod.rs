use crate::util::Throughput;
use crate::util::{canonicalize_cli_path, find_all_files, handle_process_entry_error};
use argh::FromArgs;
use bytesize::ByteSize;
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::{file_io, traits::FileFormatUntransformDetection};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

#[derive(FromArgs, Debug)]
/// Detransform DDS files (Demo CLI - use API for production)
#[argh(subcommand, name = "detransform")]
pub struct DetransformCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

pub fn handle_detransform_command(cmd: DetransformCmd) -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DXT Lossless Detransform CLI Demo ===");
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
    let bytes_processed = AtomicU64::new(0);

    // Process files in parallel using file format handler pipeline
    entries.par_iter().for_each(|entry| {
        let result = process_file_untransform(entry, &cmd.input, &cmd.output, &bytes_processed);
        handle_process_entry_error(result);
    });

    let elapsed = start.elapsed();
    let total_bytes = bytes_processed.load(Ordering::Relaxed);
    let data_size = ByteSize(total_bytes);
    let throughput = if elapsed.as_secs_f64() > 0.0 {
        Throughput::from_bytes_per_sec((total_bytes as f64 / elapsed.as_secs_f64()) as u64)
    } else {
        Throughput::from_bytes_per_sec(0)
    };

    println!("\n=== Detransform Complete ===");
    println!("Time taken: {elapsed:.2?}");
    println!("Data processed: {data_size}");
    println!("Throughput: {throughput}");

    Ok(())
}

fn process_file_untransform(
    entry: &std::fs::DirEntry,
    input_dir: &Path,
    output_dir: &Path,
    bytes_processed: &AtomicU64,
) -> Result<(), crate::error::TransformError> {
    let path = entry.path();
    let relative = path.strip_prefix(input_dir).unwrap();
    let target_path = output_dir.join(relative);

    // Create output directory if needed
    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Get file size for throughput calculation
    if let Ok(metadata) = std::fs::metadata(&path) {
        bytes_processed.fetch_add(metadata.len(), Ordering::Relaxed);
    }

    // Try to detect and untransform using format detection
    try_untransform_with_detection(&path, &target_path)
}

/// Try to untransform a file using format detection
fn try_untransform_with_detection(
    path: &Path,
    target_path: &Path,
) -> Result<(), crate::error::TransformError> {
    // Read a small portion of the file to check headers for detection
    let file_data = std::fs::read(path)?;

    // Try different file format handlers in sequence using detection
    // Currently only DDS handler is available, but structured for easy expansion

    let dds_handler = DdsHandler;
    if dds_handler.can_handle_untransform(&file_data) {
        return file_io::untransform_file_with(&dds_handler, path, target_path).map_err(Into::into);
    }

    // TODO: Add more file format handlers here when available
    // let tga_handler = TgaHandler;
    // if tga_handler.can_handle_untransform(&file_data) {
    //     return file_io::untransform_file_with(&tga_handler, path, target_path)
    //         .map_err(Into::into);
    // }

    Err(crate::error::TransformError::UnsupportedFormat(
        path.to_string_lossy().to_string(),
    ))
}
