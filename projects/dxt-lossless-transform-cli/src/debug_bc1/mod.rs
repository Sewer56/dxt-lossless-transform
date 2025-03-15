use crate::{error::TransformError, util};
use argh::FromArgs;
use core::alloc::Layout;
use dxt_lossless_transform_bc1::normalize_blocks::normalize_blocks;
use dxt_lossless_transform_bc1::util::decode_bc1_block;
use rayon::prelude::*;
use safe_allocator_api::RawAlloc;
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Debug commands for analyzing BC1 files
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "debug-bc1")]
pub struct DebugCmd {
    #[argh(subcommand)]
    pub command: DebugCommands,
}

/// Commands for debugging BC1 files
#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum DebugCommands {
    ValidateNormalize(ValidateCommand),
}

/// Validate BC1 block normalization code by processing them through normalize_blocks and
/// checking if pixels match
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "validate-normalize")]
pub struct ValidateCommand {
    /// input directory path containing BC1 files to validate
    #[argh(positional)]
    pub directory: PathBuf,
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::ValidateNormalize(cmd) => validate_normalize(cmd),
    }
}

fn validate_normalize(cmd: ValidateCommand) -> Result<(), TransformError> {
    // Collect all files recursively
    let mut entries = Vec::new();
    util::find_all_files(&cmd.directory, &mut entries)?;
    println!("Found {} files to validate", entries.len());

    // Process files in parallel
    let results: Vec<_> = entries
        .par_iter()
        .map(|entry| {
            let path = entry.path();
            match validate_normalize_single(&path) {
                Ok(result) => (path, result, None),
                Err(e) => (path, false, Some(e)),
            }
        })
        .collect();

    // Print summary
    let mut success_count = 0;
    let mut failure_count = 0;

    for (path, success, error) in results {
        if success {
            println!("✅ Validated: {}", path.display());
            success_count += 1;
        } else {
            if let Some(err) = error {
                println!("❌ Failed: {} - Error: {}", path.display(), err);
            } else {
                println!("❌ Failed: {} - Pixels don't match", path.display());
            }
            failure_count += 1;
        }
    }

    println!(
        "Validation complete: {} succeeded, {} failed",
        success_count, failure_count
    );

    Ok(())
}

fn validate_normalize_single(path: &Path) -> Result<bool, TransformError> {
    // Read file contents
    let content = fs::read(path).map_err(TransformError::IoError)?;

    // Skip if file is too small or not a multiple of 8 bytes (BC1 block size)
    if content.len() < 8 || content.len() % 8 != 0 {
        return Err(TransformError::IgnoredByFilter);
    }

    // Allocate buffer for normalized blocks
    let mut normalized =
        unsafe { RawAlloc::new(Layout::from_size_align_unchecked(content.len(), 64)) }.unwrap();

    // Process through normalize_blocks
    unsafe {
        normalize_blocks(content.as_ptr(), normalized.as_mut_ptr(), content.len());
    }

    // Validate each block
    let block_count = content.len() / 8;
    for block_index in 0..block_count {
        let src_offset = block_index * 8;

        // Decode original block
        let original_block = unsafe { decode_bc1_block(content.as_ptr().add(src_offset)) };

        // Decode normalized block
        let normalized_block = unsafe { decode_bc1_block(normalized.as_ptr().add(src_offset)) };

        // Compare all pixels
        for y in 0..4 {
            for x in 0..4 {
                let original_pixel = unsafe { original_block.get_pixel_unchecked(x, y) };
                let normalized_pixel = unsafe { normalized_block.get_pixel_unchecked(x, y) };

                if original_pixel != normalized_pixel {
                    println!(
                        "Pixel mismatch in file {}, block {}, coord ({},{}): Original: {:?}, Normalized: {:?}",
                        path.display(), block_index, x, y, original_pixel, normalized_pixel
                    );

                    // Return early on first mismatch
                    return Ok(false);
                }
            }
        }
    }

    Ok(true)
}
