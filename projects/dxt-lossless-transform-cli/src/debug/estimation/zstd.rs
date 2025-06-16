//! ZStandard implementation of size estimation operations.

use crate::debug::compression::zstd as zstd_raw;
use crate::debug::estimation::SizeEstimationOperations;
use crate::error::TransformError;
use core::slice;

/// ZStandard implementation of [`SizeEstimationOperations`].
/// This implementation uses optimized streaming compression with a null sink
/// to estimate size without allocating a full output buffer.
pub struct ZStandardSizeEstimation;

impl SizeEstimationOperations for ZStandardSizeEstimation {
    fn estimate_compressed_size(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        compression_level: i32,
    ) -> Result<usize, TransformError> {
        // Use optimized size-only estimation that doesn't allocate a full buffer
        unsafe {
            let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
            match zstd_raw::estimate_compressed_size(compression_level, original_slice) {
                Ok(size) => Ok(size),
                Err(_) => Err(TransformError::Debug(
                    "ZStandard compression size estimation failed".to_owned(),
                )),
            }
        }
    }
}
