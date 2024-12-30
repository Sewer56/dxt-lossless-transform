use crate::error::TransformError;
use crate::*;
use argh::FromArgs;
use core::ptr::copy_nonoverlapping;
use dxt_lossless_transform_api::{parse_dds, DdsFormat};
use rayon::*;
use std::collections::HashMap;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(FromArgs, Debug)]
/// Debug commands for analyzing DDS files
#[argh(subcommand, name = "debug")]
pub struct DebugCmd {
    #[argh(subcommand)]
    pub command: DebugCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub enum DebugCommands {
    AnalyzeBC7(AnalyzeBC7Cmd),
    SplitByBlockTypeCmd(SplitByBlockTypeCmd),
}

#[derive(FromArgs, Debug)]
/// Analyze BC7 block types in DDS files
#[argh(subcommand, name = "analyze-bc7")]
pub struct AnalyzeBC7Cmd {
    /// input directory path
    #[argh(option)]
    pub input: PathBuf,
}

#[derive(FromArgs, Debug)]
/// Transforms BC7 files by splitting the blocks into one section with first byte, and one section with the rest
#[argh(subcommand, name = "bc7-split-by-block-type")]
pub struct SplitByBlockTypeCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,
}

pub fn handle_debug_command(cmd: DebugCmd) -> Result<(), TransformError> {
    match cmd.command {
        DebugCommands::AnalyzeBC7(analyze_cmd) => analyze_bc7_blocks(&analyze_cmd.input),
        DebugCommands::SplitByBlockTypeCmd(cmd) => {
            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;
            println!("Found {} files to transform", entries.len());

            // Process files in parallel
            entries.par_iter().for_each(|entry| {
                let process_entry_result = process_dir_entry(
                    entry,
                    &cmd.input,
                    &cmd.output,
                    crate::DdsFilter::All,
                    transform_bc7_split_block_type,
                );
                handle_process_entry_error(process_entry_result);
            });

            Ok(())
        }
    }
}

fn analyze_bc7_blocks(input: &Path) -> Result<(), TransformError> {
    let mut total_blocks = 0;
    let mut mode_counts = HashMap::new();
    let mut first_byte_counts = HashMap::new();

    // Find all file paths.
    let mut entries = Vec::new();
    find_all_files(input, &mut entries)?;

    for entry in entries {
        let data = fs::read(entry.path())?;
        let info = unsafe { parse_dds(data.as_ptr(), data.len()) };
        let info = info.ok_or(TransformError::InvalidDdsFile)?;

        // Filter out non-BC7 files.
        if info.format != DdsFormat::BC7 {
            continue;
        }

        // Skip the DDS header and any additional headers
        let data_offset = info.data_offset;
        let data = &data[data_offset as usize..];

        // BC7 blocks are 16 bytes each
        for block in data.chunks_exact(16) {
            // The first byte contains the mode in its lowest bits
            // Mode is determined by the position of the first 1 bit
            let mode_byte = block[0];
            *first_byte_counts.entry(mode_byte).or_insert(0) += 1;
            let mode = if mode_byte == 0 {
                8 // Invalid mode
            } else {
                mode_byte.leading_zeros() as u8
            };

            *mode_counts.entry(mode).or_insert(0) += 1;
            total_blocks += 1;
        }

        println!("File: {}", entry.path().display());
    }

    if total_blocks > 0 {
        println!("\nBC7 Block Type Analysis:");
        println!("Total blocks analyzed: {}", total_blocks);
        println!("\nMode distribution:");

        let mut modes: Vec<_> = mode_counts.iter().collect();
        modes.sort_by_key(|&(mode, _)| mode);

        for (mode, count) in modes {
            let percentage = (*count as f64 / total_blocks as f64) * 100.0;
            println!("Mode {}: {} blocks ({:.2}%)", mode, count, percentage);
        }

        // Print first byte distribution
        println!("\nMost common first bytes:");
        let mut first_bytes: Vec<_> = first_byte_counts.iter().collect();
        first_bytes.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

        for (byte, count) in first_bytes {
            let percentage = ((*count) as f64 / total_blocks as f64) * 100.0;
            println!("0x{:02X}: {} blocks ({:.2}%)", byte, count, percentage);
        }
    } else {
        println!("No BC7 blocks found in the directory");
    }

    Ok(())
}

#[inline]
pub unsafe fn transform_bc7_split_block_type(
    mut input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    _format: DdsFormat,
) {
    let mut block_type_ptr = output_ptr;
    let block_type_end = block_type_ptr.add(len / 16); // each type is 1 byte
    let mut block_data_ptr = block_type_end;

    while block_type_ptr < block_type_end {
        let block_type = *input_ptr;

        *block_type_ptr = block_type;
        copy_nonoverlapping(input_ptr.add(1), block_data_ptr, 15); // remaining block bytes

        block_type_ptr = block_type_ptr.add(1);
        block_data_ptr = block_data_ptr.add(15);
        input_ptr = input_ptr.add(16);
    }
}
