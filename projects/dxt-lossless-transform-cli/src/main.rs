#![allow(unexpected_cfgs)]
#![cfg(not(tarpaulin_include))]

#[cfg(feature = "debug")]
mod debug;

mod error;
mod util;
use argh::FromArgs;
use core::{error::Error, ops::Sub};
use dxt_lossless_transform_api::*;
use error::TransformError;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::Instant,
};
use util::*;

#[derive(Debug, Clone)]
enum DdsFilter {
    BC1,
    BC2,
    BC3,
    BC7,
    All,
}

// Implement FromStr to allow parsing from command line arguments
impl FromStr for DdsFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bc1" => Ok(DdsFilter::BC1),
            "bc2" => Ok(DdsFilter::BC2),
            "bc3" => Ok(DdsFilter::BC3),
            "bc7" => Ok(DdsFilter::BC7),
            "all" => Ok(DdsFilter::All),
            _ => Err(format!(
                "Invalid DDS type: {}. Valid types are: bc1, bc2, bc3, bc7, all",
                s
            )),
        }
    }
}

#[derive(FromArgs, Debug)]
/// File transformation tool for DDS files
struct TopLevel {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    Transform(TransformCmd),
    Detransform(DetransformCmd),
    #[cfg(feature = "debug")]
    Debug(debug::DebugCmd),
}

#[derive(FromArgs, Debug)]
/// Transform DDS files from input directory to output directory
#[argh(subcommand, name = "transform")]
struct TransformCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,

    /// filter by DDS type (bc1, bc2, bc3, bc7, all) [default: all]
    #[argh(option)]
    pub filter: Option<DdsFilter>,
}

#[derive(FromArgs, Debug)]
/// Detransform DDS files from input directory to output directory
#[argh(subcommand, name = "detransform")]
struct DetransformCmd {
    /// input directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(canonicalize_cli_path))]
    pub output: PathBuf,

    /// filter by DDS type (bc1, bc2, bc3, bc7, all) [default: all]
    #[argh(option)]
    pub filter: Option<DdsFilter>,
}

fn canonicalize_cli_path(value: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);

    // If path doesn't exist, create it
    if !path.exists() {
        fs::create_dir_all(path).map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    // Now we can canonicalize it
    fs::canonicalize(path).map_err(|e| format!("Invalid path: {}", e))
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli: TopLevel = argh::from_env();

    let start = Instant::now();
    match cli.command {
        Commands::Transform(cmd) => {
            let filter = cmd.filter.unwrap_or(DdsFilter::All);

            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;
            println!("Found {} files to transform", entries.len());

            // Process files in parallel
            entries.par_iter().for_each(|entry| {
                if let Err(e) = process_dir_entry(
                    entry,
                    &cmd.input,
                    &cmd.output,
                    filter.clone(),
                    transform_format,
                ) {
                    eprintln!("{}", e);
                }
            });
        }
        Commands::Detransform(cmd) => {
            let filter = cmd.filter.unwrap_or(DdsFilter::All);

            // Collect all files recursively first
            let mut entries = Vec::new();
            find_all_files(&cmd.input, &mut entries)?;
            println!("Found {} files to detransform", entries.len());

            // Process files in parallel
            entries.par_iter().for_each(|entry| {
                if let Err(e) = process_dir_entry(
                    entry,
                    &cmd.input,
                    &cmd.output,
                    filter.clone(),
                    untransform_format,
                ) {
                    eprintln!("{}", e);
                }
            });
        }
        #[cfg(feature = "debug")]
        Commands::Debug(cmd) => {
            debug::handle_debug_command(cmd)?;
        }
    }

    println!("Transform completed in {:.2?}", start.elapsed());
    Ok(())
}

fn process_dir_entry(
    dir_entry: &fs::DirEntry,
    input: &Path,
    output: &Path,
    filter: DdsFilter,
    transform_fn: unsafe fn(*const u8, *mut u8, usize, DdsFormat),
) -> Result<(), TransformError> {
    let path = dir_entry.path();
    let relative = path.strip_prefix(input).unwrap();
    let target_path = output.join(relative);

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let source_handle = open_read_handle(path)?;
    let source_size = get_file_size(&source_handle)? as usize;
    let source_mapping = open_readonly_mmap(&source_handle, source_size)?;

    let dds_info = unsafe { parse_dds(source_mapping.data(), source_mapping.len()) };
    let (info, format) = check_dds_format(dds_info, filter, &dir_entry.path())?;

    let target_path_str = target_path.to_str().unwrap();
    let target_handle = open_write_handle(&source_mapping, target_path_str)?;
    let target_mapping = create_output_mapping(&target_handle, source_size as u64)?;

    // Copy DDS headers.
    unsafe {
        std::ptr::copy_nonoverlapping(
            source_mapping.data(),
            target_mapping.data(),
            info.data_offset as usize,
        );
    }

    unsafe {
        transform_fn(
            source_mapping.data().add(info.data_offset as usize),
            target_mapping.data().add(info.data_offset as usize),
            source_size.sub(info.data_offset as usize),
            format,
        );
    }

    Ok(())
}
