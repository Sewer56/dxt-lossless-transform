//! ZStandard compression implementation module.

pub mod zstd_raw;
use super::CompressionOperations;
use crate::error::TransformError;
use core::slice;
pub use zstd_raw::*;

/// ZStandard implementation of [`CompressionOperations`].
pub struct ZStandardCompression;

impl CompressionOperations for ZStandardCompression {
    fn compress_data(
        &self,
        data_ptr: *const u8,
        len_bytes: usize,
        compression_level: i32,
    ) -> Result<(Box<[u8]>, usize), TransformError> {
        let max_compressed_size = zstd_raw::max_alloc_for_compress_size(len_bytes);
        let mut compressed_buffer =
            unsafe { Box::<[u8]>::new_uninit_slice(max_compressed_size).assume_init() };

        let compressed_size = unsafe {
            let original_slice = slice::from_raw_parts(data_ptr, len_bytes);
            match zstd_raw::compress(compression_level, original_slice, &mut compressed_buffer) {
                Ok(size) => size,
                Err(_) => {
                    return Err(TransformError::Debug(
                        "ZStandard compression failed".to_owned(),
                    ))
                }
            }
        };

        Ok((compressed_buffer, compressed_size))
    }

    fn decompress_data(
        &self,
        compressed_data: &[u8],
        output_buffer: &mut [u8],
    ) -> Result<usize, TransformError> {
        match zstd_raw::decompress(compressed_data, output_buffer) {
            Ok(decompressed_size) => Ok(decompressed_size),
            Err(_) => Err(TransformError::Debug(
                "ZStandard decompression failed".to_owned(),
            )),
        }
    }
}
