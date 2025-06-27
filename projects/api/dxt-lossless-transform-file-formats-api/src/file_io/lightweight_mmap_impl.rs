//! File I/O implementation using lightweight-mmap.

use crate::bundle::TransformBundle;
use crate::error::FileFormatResult;
use crate::traits::FileFormatHandler;
use core::slice;
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::path::Path;

/// Transform a file using a specific handler and transform bundle.
///
/// This function memory-maps the input file, transforms it using the provided handler
/// and bundle, and writes the result to the output file.
///
/// # Arguments
///
/// * `handler` - The file format handler to use
/// * `input_path` - Path to the input file
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
/// * `bundle` - The transform bundle containing BCx builders
///
/// # Returns
///
/// Result indicating success or error
pub fn transform_file_bundle<H: FileFormatHandler>(
    handler: &H,
    input_path: &Path,
    output_path: &Path,
    bundle: &TransformBundle,
) -> FileFormatResult<()> {
    // Open input file
    let input_handle = ReadOnlyFileHandle::open(input_path)?;
    let input_size = input_handle.size()? as usize;
    let input_mapping = ReadOnlyMmap::new(&input_handle, 0, input_size)?;

    let output_handle = ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;

    let output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

    // Transform directly into the memory-mapped output
    crate::api::transform_slice_bundle(
        handler,
        unsafe { slice::from_raw_parts(input_mapping.data(), input_mapping.len()) },
        unsafe { slice::from_raw_parts_mut(output_mapping.data(), output_mapping.len()) },
        bundle,
    )?;

    Ok(())
}

/// Untransform a file using a specific handler.
///
/// This function memory-maps the input file, untransforms it using the provided handler,
/// and writes the result to the output file.
///
/// # Arguments
///
/// * `handler` - The file format handler to use
/// * `input_path` - Path to the input file
/// * `output_path` - Path to the output file (will be created). The output directory must exist.
///
/// # Returns
///
/// Result indicating success or error
pub fn untransform_file_with<H: FileFormatHandler>(
    handler: &H,
    input_path: &Path,
    output_path: &Path,
) -> FileFormatResult<()> {
    // Open input file
    let input_handle = ReadOnlyFileHandle::open(input_path)?;
    let input_size = input_handle.size()? as usize;
    let input_mapping = ReadOnlyMmap::new(&input_handle, 0, input_size)?;

    let output_handle = ReadWriteFileHandle::create_preallocated(output_path, input_size as i64)?;

    let output_mapping = ReadWriteMmap::new(&output_handle, 0, input_size)?;

    // Untransform directly into the memory-mapped output
    crate::api::untransform_slice_with(
        handler,
        unsafe { slice::from_raw_parts(input_mapping.data(), input_mapping.len()) },
        unsafe { slice::from_raw_parts_mut(output_mapping.data(), output_mapping.len()) },
    )?;

    Ok(())
}
