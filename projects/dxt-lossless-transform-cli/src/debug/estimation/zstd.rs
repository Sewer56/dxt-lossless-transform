//! ZStandard implementation of size estimation operations.

use crate::debug::compression::zstd as zstd_raw;
use crate::debug::estimation::SizeEstimationOperations;
use crate::error::TransformError;
use core::slice;

/// ZStandard implementation of [`SizeEstimationOperations`].
/// This implementation does actual compression to estimate size.
pub struct ZStandardSizeEstimation;

impl SizeEstimationOperations for ZStandardSizeEstimation {
    fn estimate_compressed_size(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        compression_level: i32,
    ) -> Result<usize, TransformError> {
        // For now, we compress and return the size
        // This could be optimized with size-only compression in the future
        let max_compressed_size = zstd_raw::max_alloc_for_compress_size(len_bytes);
        let mut compressed_buffer =
            unsafe { Box::<[u8]>::new_uninit_slice(max_compressed_size).assume_init() };

        unsafe {
            let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
            match zstd_raw::compress(compression_level, original_slice, &mut compressed_buffer) {
                Ok(size) => Ok(size),
                Err(_) => Err(TransformError::Debug(
                    "ZStandard compression size estimation failed".to_owned(),
                )),
            }
        }
    }
}
