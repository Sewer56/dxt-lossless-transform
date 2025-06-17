use super::{is_dds as internal_is_dds, parse_dds as internal_parse_dds, DdsFormat};
use crate::dds::DdsInfo;

/// Determines if the given file represents a DDS texture.
/// This is done by checking the 'MAGIC' header, 'DDS ' at offset 0.
///
/// # Safety
///
/// - `ptr` must be valid for reads of `len` bytes
#[no_mangle]
pub unsafe extern "C" fn is_dds(ptr: *const u8, len: usize) -> bool {
    internal_is_dds(ptr, len)
}

/// Attempts to parse the data format of a DDS file from the given pointer and length.
///
/// # Safety
///
/// Any input which passes [`is_dds`] check should be a valid input;
/// but you do not need to explicitly call [`is_dds`], this function will return [`DdsFormat::NotADds`]
/// if the file is not a DDS.
///
/// - `ptr` must be valid for reads of `len` bytes
/// - `len` must accurately represent the length of the file
///
/// # Return
///
/// A [`DdsInfo`] structure. If the file is not a DDS then [`DdsFormat`] will be [`DdsFormat::NotADds`].
/// If the format is an unsupported one, then [`DdsFormat`] will be [`DdsFormat::Unknown`].
#[no_mangle]
pub unsafe extern "C" fn parse_dds(ptr: *const u8, len: usize) -> DdsInfo {
    if let Some(info) = internal_parse_dds(ptr, len) {
        DdsInfo {
            format: info.format,
            data_offset: info.data_offset,
        }
    } else {
        DdsInfo {
            format: DdsFormat::NotADds,
            data_offset: 0,
        }
    }
}
