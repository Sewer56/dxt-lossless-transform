#![cfg(not(tarpaulin_include))]

use crate::{error::TransformError, util::*, DdsFilter};
use core::{ops::Sub, slice};
use dxt_lossless_transform_api::*;
use std::fs;

pub mod benchmark_common;
pub mod calc_compression_stats_common;
pub mod compressed_data_cache;
pub mod compression_size_cache;
pub mod throughput;
pub mod zstd;

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

    let dds_info = parse_dds(source_mapping.data(), source_mapping.len());
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
