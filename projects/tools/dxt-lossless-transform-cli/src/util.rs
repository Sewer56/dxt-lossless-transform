#![cfg(not(tarpaulin_include))]

use crate::error::TransformError;
use crate::DdsFilter;
use core::{ops::Sub, ptr::copy_nonoverlapping};
use dxt_lossless_transform_dds::dds::*;
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::fs;
use std::path::*;

/// Recursively visits directories and collects entries.
///
/// This function traverses the directory tree rooted at `dir`, collecting all
/// directory entries into a vector. If an error occurs while reading a directory
/// or an individual entry, the error is handled gracefully and the function
/// continues with the remaining entries.
///
/// # Arguments
///
/// * `dir`: The directory to start the traversal from.
/// * `entries`: A mutable reference to the vector of entries to populate.
///
/// # Returns
///
/// A `Result` indicating whether the traversal was successful.
pub fn find_all_files(dir: &Path, entries: &mut Vec<fs::DirEntry>) -> std::io::Result<()> {
    // Gracefully handle cases where the directory cannot be read
    let dir_entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()), // Silently return if directory can't be read
    };

    for entry in dir_entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue, // Skip problematic entries, e.g. those without access.
        };

        let path = entry.path();
        if path.is_dir() {
            // Recursively collect files.
            find_all_files(&path, entries)?;
        } else {
            entries.push(entry);
        }
    }
    Ok(())
}

/// Opens a file in read-only mode and returns a handle.
///
/// # Arguments
///
/// * `path` - The path to the file to open.
///
/// # Returns
///
/// A read-only file handle on success, or a [`TransformError`] if the file cannot be opened.
#[inline(always)]
pub fn open_read_handle(path: PathBuf) -> Result<ReadOnlyFileHandle, TransformError> {
    ReadOnlyFileHandle::open(path.to_str().unwrap())
        .map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Opens a file with a given handle and creates a memory mapping for reading.
///
/// # Arguments
///
/// * `handle` - Handle to the file to mmap
/// * `len` - Length of the file/mapping
///
/// # Returns
///
/// A memory mapping on success, or a [`TransformError`] on failure.
#[inline(always)]
pub fn open_readonly_mmap(
    handle: &ReadOnlyFileHandle,
    len: usize,
) -> Result<ReadOnlyMmap<'_>, TransformError> {
    // SAFETY: We keep the handle alive in the returned struct
    ReadOnlyMmap::new(handle, 0, len).map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Retrieves the size of the file for a given handle.
///
/// # Arguments
///
/// * `handle` - Handle to the file to mmap
///
/// # Returns
///
/// The size of the file in bytes, or a [`TransformError`] on failure.
#[inline(always)]
pub fn get_file_size(handle: &ReadOnlyFileHandle) -> Result<i64, TransformError> {
    handle
        .size()
        .map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Creates a file with a given path for writing.
/// Size
///
/// # Arguments
///
/// * `handle` - Handle to the file to mmap
///
/// # Returns
///
/// A memory mapping on success, or a [`TransformError`] on failure.
#[inline(always)]
pub fn open_write_handle(
    source_mapping: &ReadOnlyMmap,
    target_path_str: &str,
) -> Result<ReadWriteFileHandle, TransformError> {
    ReadWriteFileHandle::create_preallocated(target_path_str, source_mapping.len() as i64)
        .map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Creates a new file and memory mapping for writing.
///
/// # Arguments
///
/// * `handle` - Handle to the output file to be mapped.
/// * `size` - Size to preallocate for the file
///
/// # Returns
///
/// A memory mapping on success, or a [`TransformError`] on failure.
#[inline(always)]
pub fn create_output_mapping(
    handle: &ReadWriteFileHandle,
    size: u64,
) -> Result<ReadWriteMmap<'_>, TransformError> {
    ReadWriteMmap::new(handle, 0, size as usize)
        .map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Validates a DDS format against a filter.
///
/// # Arguments
///
/// * `dds_info` - Optional DDS file information
/// * `filter` - Filter specifying which formats to accept
///
/// # Returns
///
/// The validated DdsFormat on success, or a TransformError if the format is invalid or unsupported.
#[inline(always)]
pub fn check_dds_format(
    dds_info: Option<DdsInfo>,
    filter: DdsFilter,
    target_path: &Path,
) -> Result<(DdsInfo, DdsFormat), TransformError> {
    let info = dds_info.ok_or(TransformError::InvalidDdsFile)?;

    if info.format == DdsFormat::Unknown {
        return Err(TransformError::UnsupportedFormat(
            target_path.to_string_lossy().to_string(),
        ));
    }

    match (info.format, filter) {
        (DdsFormat::BC1, DdsFilter::BC1 | DdsFilter::All) => Ok((info, DdsFormat::BC1)),
        (DdsFormat::BC2, DdsFilter::BC2 | DdsFilter::All) => Ok((info, DdsFormat::BC2)),
        (DdsFormat::BC3, DdsFilter::BC3 | DdsFilter::All) => Ok((info, DdsFormat::BC3)),
        (DdsFormat::BC7, DdsFilter::BC7 | DdsFilter::All) => Ok((info, DdsFormat::BC7)),
        _ => Err(TransformError::IgnoredByFilter),
    }
}

/// Handles errors from process_dir_entry function by printing to stderr
/// (except for IgnoredByFilter which is silently ignored).
pub fn handle_process_entry_error(result: Result<(), TransformError>) {
    if let Err(e) = result {
        match e {
            TransformError::IgnoredByFilter => (),
            _ => eprintln!("{e}"),
        }
    }
}

/// Processes a single directory entry (DDS file) for transformation or detransformation.
///
/// # Arguments
///
/// * `dir_entry` - The directory entry to process
/// * `input` - Input directory path
/// * `output` - Output directory path  
/// * `filter` - DDS format filter
/// * `transform_fn` - The transformation function to apply
/// * `param` - Additional parameter for the transform function
///
/// # Returns
///
/// Result indicating success or a TransformError on failure.
pub fn transform_dir_entry<TParam>(
    dir_entry: &fs::DirEntry,
    input: &Path,
    output: &Path,
    filter: DdsFilter,
    transform_fn: unsafe fn(&TParam, *const u8, *mut u8, usize, DdsFormat),
    param: &TParam,
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
        copy_nonoverlapping(
            source_mapping.data(),
            target_mapping.data(),
            info.data_offset as usize,
        );
    }

    unsafe {
        transform_fn(
            param,
            source_mapping.data().add(info.data_offset as usize),
            target_mapping.data().add(info.data_offset as usize),
            source_size.sub(info.data_offset as usize),
            format,
        );
    }

    Ok(())
}
