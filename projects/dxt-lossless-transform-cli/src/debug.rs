use crate::{error::TransformError, util::*, DdsFilter};
use core::ops::Sub;
use dxt_lossless_transform_api::*;
use std::fs;

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
    test_fn: TFunction,
) -> Result<(), TransformError>
where
    TFunction: Fn(*const u8, usize, DdsFormat) -> Result<(), TransformError>,
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
