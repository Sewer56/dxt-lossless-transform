use crate::error::TransformError;
use crate::DdsFilter;
use dxt_lossless_transform_api::*;
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
        _ => Err(TransformError::UnsupportedFormat(
            target_path.to_string_lossy().to_string(),
        )),
    }
}
