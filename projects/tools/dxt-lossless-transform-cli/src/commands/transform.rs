use crate::util::*;
use argh::FromArgs;
use dxt_lossless_transform_bc1::Bc1TransformDetails;
use dxt_lossless_transform_dds::dds::DdsFormat;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{path::PathBuf, time::Instant};

#[derive(FromArgs, Debug)]
/// Transform DDS files from input directory to output directory
#[argh(subcommand, name = "transform")]
pub struct TransformCmd {
    /// input directory path
    #[argh(option, from_str_fn(crate::util::canonicalize_cli_path))]
    pub input: PathBuf,

    /// output directory path
    #[argh(option, from_str_fn(crate::util::canonicalize_cli_path))]
    pub output: PathBuf,

    /// filter by DDS type (bc1, bc2, bc3, bc7, all) [default: all]
    #[argh(option)]
    pub filter: Option<crate::DdsFilter>,
}

pub fn handle_transform_command(cmd: TransformCmd) -> Result<(), Box<dyn std::error::Error>> {
    let filter = cmd.filter.unwrap_or(crate::DdsFilter::All);

    // Collect all files recursively first
    let mut entries = Vec::new();
    find_all_files(&cmd.input, &mut entries)?;
    println!("Found {} files to transform", entries.len());

    let start = Instant::now();

    // Process files in parallel
    entries.par_iter().for_each(|entry| {
        let process_entry_result = transform_dir_entry(
            entry,
            &cmd.input,
            &cmd.output,
            filter.clone(),
            transform_format,
            &(),
        );
        handle_process_entry_error(process_entry_result);
    });

    println!("Transform completed in {:.2?}", start.elapsed());
    Ok(())
}

/// # Safety
///
/// This function is unsafe because it uses raw pointers and requires a valid length,
/// but in our case we know they're valid.
#[inline]
pub unsafe fn transform_format(
    _param: &(),
    input_ptr: *const u8,
    output_ptr: *mut u8,
    len: usize,
    format: DdsFormat,
) {
    // DDS Integration
    if format == DdsFormat::BC1 {
        dxt_lossless_transform_bc1::transform_bc1(
            input_ptr,
            output_ptr,
            len,
            Bc1TransformDetails::default(),
        );
    } else {
        todo!()
    }
}
