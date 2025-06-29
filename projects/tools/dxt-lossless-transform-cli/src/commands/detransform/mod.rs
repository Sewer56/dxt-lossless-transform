use crate::util::Throughput;
use crate::util::{canonicalize_cli_path, find_all_files};
use argh::FromArgs;
use bytesize::ByteSize;
use dxt_lossless_transform_dds::DdsHandler;
use dxt_lossless_transform_file_formats_api::file_io;

use std::{
    path::{Path, PathBuf},
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

    // Process files using file format handler pipeline
    let total_bytes = process_files_untransform(&entries, &cmd.input, &cmd.output);

    let elapsed = start.elapsed();
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

/// Process all files for untransform and return total bytes processed
fn process_files_untransform(
    entries: &[std::fs::DirEntry],
    input_dir: &Path,
    output_dir: &Path,
) -> u64 {
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
                |entry| match process_file_untransform(entry, input_dir, output_dir) {
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
            match process_file_untransform(entry, input_dir, output_dir) {
                Ok(bytes) => bytes_processed += bytes,
                Err(e) => handle_process_entry_error(Err(e)),
            }
        }
        bytes_processed
    }
}

/// Process a single file untransform - returns bytes processed
fn process_file_untransform(
    entry: &std::fs::DirEntry,
    input_dir: &Path,
    output_dir: &Path,
) -> Result<u64, crate::error::TransformError> {
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

    file_io::untransform_file_with_multiple_handlers(handlers, &path, &target_path)?;

    Ok(bytes)
}
