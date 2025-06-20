#![doc = include_str!(concat!("../", core::env!("CARGO_PKG_README")))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

extern crate alloc;

use core::{ffi::c_void, slice};
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};
use dxt_lossless_transform_common::allocate::AllocateError;
use thiserror::Error;
use zstd_sys::ZSTD_cParameter::*;
use zstd_sys::ZSTD_format_e::*;
use zstd_sys::*;

/// Errors that can occur during ZStandard size estimation.
#[derive(Debug, Error)]
pub enum ZStandardError {
    /// ZStandard compression failed
    #[error("ZStandard compression failed: {0}")]
    CompressionFailed(String),

    /// Invalid compression level
    #[error("Invalid compression level: {0}")]
    InvalidLevel(i32),

    /// Memory allocation failed
    #[error("Memory allocation failed")]
    AllocationFailed,

    /// ZStandard internal error
    #[error("ZStandard internal error: {0:?}")]
    ZStandardInternal(ZSTD_ErrorCode),

    /// Error during allocation
    #[error(transparent)]
    AllocateError(#[from] AllocateError),
}

/// ZStandard implementation of [`SizeEstimationOperations`].
///
/// This implementation performs actual compression to estimate size.
/// While this is more accurate than approximation methods, it's also
/// slower as it performs the full compression operation.
///
/// The compression level is configured when creating the estimator instance.
pub struct ZStandardSizeEstimation {
    compression_level: i32,
}

impl ZStandardSizeEstimation {
    /// Creates a new ZStandard size estimator with the specified compression level.
    ///
    /// # Parameters
    /// * `compression_level` - Compression level (1-22, where 1 is fastest and 22 is best compression)
    pub fn new(compression_level: i32) -> Result<Self, ZStandardError> {
        // Validate compression level
        if !(1..=22).contains(&compression_level) {
            return Err(ZStandardError::InvalidLevel(compression_level));
        }

        Ok(Self { compression_level })
    }

    /// Creates a new ZStandard size estimator with compression level 1 (fastest).
    pub fn new_fast() -> Self {
        Self {
            compression_level: 1,
        }
    }

    /// Creates a new ZStandard size estimator with compression level 3 (default).
    pub fn new_default() -> Self {
        Self {
            compression_level: 3,
        }
    }

    /// Creates a new ZStandard size estimator with compression level 22 (best compression).
    pub fn new_best() -> Self {
        Self {
            compression_level: 22,
        }
    }
}

impl Default for ZStandardSizeEstimation {
    fn default() -> Self {
        Self::new_default()
    }
}

impl SizeEstimationOperations for ZStandardSizeEstimation {
    type Error = ZStandardError;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        if len_bytes == 0 {
            return Ok(0);
        }

        // Calculate maximum compressed size using ZStandard bounds
        let max_size = unsafe { ZSTD_compressBound(len_bytes) };
        Ok(max_size)
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        _data_type: DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        if input_ptr.is_null() {
            return Ok(0);
        }

        if len_bytes == 0 {
            return Ok(0);
        }

        // Output buffer is guaranteed to be non-null and sufficient size
        let output_buffer = slice::from_raw_parts_mut(output_ptr, output_len);
        let input_data = slice::from_raw_parts(input_ptr, len_bytes);

        // Perform compression using the provided buffer
        let compressed_size = compress(self.compression_level, input_data, output_buffer)?;

        Ok(compressed_size)
    }
}

/// Compresses data with ZStandard using the same settings as the CLI.
/// Does not use fallback to 'copy' if compression is ineffective.
///
/// # Parameters
///
/// * `level`: Level at which we are compressing.
/// * `source`: Source data to compress.
/// * `destination`: Destination buffer.
fn compress(level: i32, source: &[u8], destination: &mut [u8]) -> Result<usize, ZStandardError> {
    // Create a compression context
    let cctx = unsafe { ZSTD_createCCtx() };
    if cctx.is_null() {
        return Err(ZStandardError::ZStandardInternal(
            ZSTD_ErrorCode::ZSTD_error_GENERIC,
        ));
    }

    // Set compression parameters (magicless format, no extra headers)
    zstd_setcommoncompressparams(cctx, Some(level));

    // Perform compression
    let result = unsafe {
        ZSTD_compress2(
            cctx,
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
        )
    };

    // Free the context
    unsafe {
        ZSTD_freeCCtx(cctx);
    }

    if unsafe { ZSTD_isError(result) } == 0 {
        return Ok(result);
    }

    Err(ZStandardError::ZStandardInternal(unsafe {
        ZSTD_getErrorCode(result)
    }))
}

/// Sets common compression parameters matching the CLI's behavior.
#[inline(always)]
fn zstd_setcommoncompressparams(cctx: *mut ZSTD_CCtx_s, level: Option<i32>) {
    unsafe {
        if let Some(lv) = level {
            ZSTD_CCtx_setParameter(cctx, ZSTD_c_compressionLevel, lv);
        }
        ZSTD_CCtx_setParameter(
            cctx,
            ZSTD_c_experimentalParam2, // zstd_c_format
            ZSTD_f_zstd1_magicless as i32,
        );
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_contentSizeFlag, 0);
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_checksumFlag, 0);
        ZSTD_CCtx_setParameter(cctx, ZSTD_c_dictIDFlag, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_empty_data() {
        let estimator = ZStandardSizeEstimation::new_default();
        let result = unsafe {
            estimator.estimate_compressed_size(
                core::ptr::null(),
                0,
                DataType::Bc1Colours,
                core::ptr::null_mut(),
                0,
            )
        };
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn estimate_simple_data() {
        let estimator = ZStandardSizeEstimation::new_default();
        let data =
            b"Hello, world! This is a test string for compression. test test test test test test!!";

        // Get max compressed size and allocate buffer
        let max_size = estimator.max_compressed_size(data.len()).unwrap();
        let mut output_buffer = vec![0u8; max_size];

        let result = unsafe {
            estimator.estimate_compressed_size(
                data.as_ptr(),
                data.len(),
                DataType::Bc1Colours,
                output_buffer.as_mut_ptr(),
                max_size,
            )
        };

        assert!(result.is_ok());
        let size = result.unwrap();
        assert!(size > 0);
        assert!(size < data.len()); // Should be smaller than input for this test case
    }

    #[test]
    fn test_invalid_compression_level() {
        let result = ZStandardSizeEstimation::new(0);
        assert!(matches!(result, Err(ZStandardError::InvalidLevel(0))));

        let result = ZStandardSizeEstimation::new(23);
        assert!(matches!(result, Err(ZStandardError::InvalidLevel(23))));
    }

    #[test]
    fn verify_different_compression_levels() {
        let estimator1 = ZStandardSizeEstimation::new_fast();
        let estimator10 = ZStandardSizeEstimation::new(10).unwrap();
        let data = b"This is a longer test string that should compress well with different levels.";

        // Get max compressed size and allocate buffers
        let max_size1 = estimator1.max_compressed_size(data.len()).unwrap();
        let max_size10 = estimator10.max_compressed_size(data.len()).unwrap();
        let mut output_buffer1 = vec![0u8; max_size1];
        let mut output_buffer10 = vec![0u8; max_size10];

        let level1_result = unsafe {
            estimator1.estimate_compressed_size(
                data.as_ptr(),
                data.len(),
                DataType::Bc1Colours,
                output_buffer1.as_mut_ptr(),
                max_size1,
            )
        };

        let level10_result = unsafe {
            estimator10.estimate_compressed_size(
                data.as_ptr(),
                data.len(),
                DataType::Bc1Colours,
                output_buffer10.as_mut_ptr(),
                max_size10,
            )
        };

        assert!(level1_result.is_ok());
        assert!(level10_result.is_ok());

        // Higher compression level should generally produce smaller size
        // (though not guaranteed for all inputs)
        let level1_size = level1_result.unwrap();
        let level10_size = level10_result.unwrap();

        assert!(level1_size > 0);
        assert!(level10_size > 0);
    }
}
