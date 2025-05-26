use crate::error::TransformError;
use crate::util::find_all_files;
use crate::{debug::extract_blocks_from_dds, DdsFilter};
use argh::FromArgs;
use dxt_lossless_transform_api::*;
use dxt_lossless_transform_bc1::{
    transform_bc1, untransform_bc1, util::decode_bc1_block, Bc1TransformDetails,
};
use dxt_lossless_transform_common::allocate::allocate_align_64;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(FromArgs, Debug)]
/// Debug commands for analyzing BC1 files
#[argh(subcommand, name = "debug-bc1")]
pub struct DebugCmd {
    #[argh(subcommand)]
    pub command: DebugCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum DebugCommands {
    Roundtrip(RoundtripCmd),
}

#[derive(FromArgs, Debug)]
/// Test BC1 transform/untransform roundtrip on files in a directory
#[argh(subcommand, name = "test-roundtrip")]
pub struct RoundtripCmd {
    /// input directory path to test (recursively)
    #[argh(positional)]
    pub input_directory: PathBuf,
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::Roundtrip(roundtrip_cmd) => handle_roundtrip_command(roundtrip_cmd),
    }
}

fn handle_roundtrip_command(cmd: RoundtripCmd) -> Result<(), TransformError> {
    let input_path = &cmd.input_directory;
    println!(
        "Testing BC1 transform/untransform roundtrip on files in: {} (recursive)",
        input_path.display()
    );

    // Collect all files recursively using existing infrastructure
    let mut entries = Vec::new();
    find_all_files(input_path, &mut entries)?;
    println!("Found {} files to test", entries.len());

    let files_tested = AtomicUsize::new(0);
    let files_passed = AtomicUsize::new(0);

    // Process files in parallel similar to main CLI
    entries.iter().for_each(|entry| {
        files_tested.fetch_add(1, Ordering::Relaxed);
        println!("Testing file: {}", entry.path().display());

        match test_bc1_roundtrip_file(entry) {
            Ok(()) => {
                println!("  ✓ PASSED");
                files_passed.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                println!("  ✗ FAILED: {e}");
            }
        }
    });

    let total_tested = files_tested.load(Ordering::Relaxed);
    let total_passed = files_passed.load(Ordering::Relaxed);

    println!("\nSummary: {total_passed}/{total_tested} files passed");
    if total_passed != total_tested {
        return Err(TransformError::Debug(
            "Some roundtrip tests failed".to_string(),
        ));
    }

    Ok(())
}

fn test_bc1_roundtrip(data_ptr: *const u8, len_bytes: usize) -> Result<(), TransformError> {
    // Allocate aligned buffers
    let mut transformed_data = allocate_align_64(len_bytes)?;
    let mut work_buffer = allocate_align_64(len_bytes)?;
    let mut roundtrip_data = allocate_align_64(len_bytes)?;

    unsafe {
        // Use default transform options
        let transform_options = Bc1TransformDetails::default();

        // Transform the data
        transform_bc1(
            data_ptr,
            transformed_data.as_mut_ptr(),
            work_buffer.as_mut_ptr(),
            len_bytes,
            transform_options,
        );

        // Untransform the data back
        untransform_bc1(
            transformed_data.as_ptr(),
            roundtrip_data.as_mut_ptr(),
            work_buffer.as_mut_ptr(),
            len_bytes,
            &transform_options,
        );

        // Compare all pixels by decoding each block
        let num_blocks = len_bytes / 8;
        for block_idx in 0..num_blocks {
            let block_offset = block_idx * 8;

            // Decode original block
            let original_block_ptr = data_ptr.add(block_offset);
            let original_decoded = decode_bc1_block(original_block_ptr);

            // Decode roundtrip block
            let roundtrip_block_ptr = roundtrip_data.as_ptr().add(block_offset);
            let roundtrip_decoded = decode_bc1_block(roundtrip_block_ptr);

            // Compare all 16 pixels in the block
            if original_decoded != roundtrip_decoded {
                return Err(TransformError::Debug(format!(
                    "Pixel mismatch in block {block_idx} (byte offset {block_offset}). Transform/untransform is not lossless!"
                )));
            }
        }
    }

    Ok(())
}

fn test_bc1_roundtrip_file(entry: &fs::DirEntry) -> Result<(), TransformError> {
    unsafe {
        extract_blocks_from_dds(
            entry,
            DdsFilter::BC1,
            |data_ptr: *const u8,
             len_bytes: usize,
             format: DdsFormat|
             -> Result<(), TransformError> {
                // Only test BC1 blocks
                if format != DdsFormat::BC1 {
                    return Ok(()); // Skip non-BC1 data
                }

                test_bc1_roundtrip(data_ptr, len_bytes)
            },
        )
    }
}
