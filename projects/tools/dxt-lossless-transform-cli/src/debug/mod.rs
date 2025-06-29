#![cfg(not(tarpaulin_include))]

pub mod benchmark_common;
pub mod calc_compression_stats_common;
pub mod compressed_data_cache;
pub mod compression;
pub mod compression_size_cache;
pub mod estimation;

use crate::error::TransformError;
use core::{ops::Sub, slice};
use dxt_lossless_transform_dds::dds::{parse_dds, DdsFormat, DdsInfo};
use lightweight_mmap::handles::*;
use lightweight_mmap::mmap::*;
use std::{fs, path::*};

/// Debug filter for DDS formats
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DdsFilter {
    BC1,
    BC2,
    BC3,
    BC7,
    All,
}

impl std::str::FromStr for DdsFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bc1" => Ok(DdsFilter::BC1),
            "bc2" => Ok(DdsFilter::BC2),
            "bc3" => Ok(DdsFilter::BC3),
            "bc7" => Ok(DdsFilter::BC7),
            "all" => Ok(DdsFilter::All),
            _ => Err(format!(
                "Invalid DDS type: {s}. Valid types are: bc1, bc2, bc3, bc7, all"
            )),
        }
    }
}

/// Extracts BC1-BC7 blocks from a DDS file in memory.
/// Raw block data from found DDS files is passed to the `test_fn` parameter for processing.
///
/// # Safety
///
/// This function is unsafe because it uses raw pointers from memory mappings,
/// but in our case we know they're valid.
pub(crate) unsafe fn extract_blocks_from_dds<TFunction>(
    dir_entry: &fs::DirEntry,
    filter: DdsFilter,
    mut test_fn: TFunction,
) -> Result<(), TransformError>
where
    TFunction: FnMut(*const u8, usize, DdsFormat) -> Result<(), TransformError>,
{
    let path = dir_entry.path();

    let source_handle = open_read_handle(path)?;
    let source_size = get_file_size(&source_handle)? as usize;
    let source_mapping = open_readonly_mmap(&source_handle, source_size)?;

    let dds_info = parse_dds(source_mapping.as_slice());
    let (info, format) = check_dds_format(dds_info, filter, &dir_entry.path())?;

    let len_bytes = source_size.sub(info.data_offset as usize);
    let data_ptr = source_mapping.data().add(info.data_offset as usize);

    // Call the roundtrip function with the BC1 data
    test_fn(data_ptr, len_bytes, format)
}

/// Calculates XXH3-128 hash of data for use as a cache key.
pub fn calculate_content_hash(data_ptr: *const u8, len_bytes: usize) -> u128 {
    let data_slice = unsafe { slice::from_raw_parts(data_ptr, len_bytes) };
    xxhash_rust::xxh3::xxh3_128(data_slice)
}

/// Opens a file in read-only mode and returns a handle (debug-only).
#[inline(always)]
pub fn open_read_handle(path: PathBuf) -> Result<ReadOnlyFileHandle, TransformError> {
    ReadOnlyFileHandle::open(path.to_str().unwrap())
        .map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Opens a file with a given handle and creates a memory mapping for reading (debug-only).
#[inline(always)]
pub fn open_readonly_mmap(
    handle: &ReadOnlyFileHandle,
    len: usize,
) -> Result<ReadOnlyMmap<'_>, TransformError> {
    // SAFETY: We keep the handle alive in the returned struct
    ReadOnlyMmap::new(handle, 0, len).map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Retrieves the size of the file for a given handle (debug-only).
#[inline(always)]
pub fn get_file_size(handle: &ReadOnlyFileHandle) -> Result<i64, TransformError> {
    handle
        .size()
        .map_err(|e| TransformError::MmapError(e.to_string()))
}

/// Validates a DDS format against a filter (for debug commands only).
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
