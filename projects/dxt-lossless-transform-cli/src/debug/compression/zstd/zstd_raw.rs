//! Copied from sewer56.archives.nx crate and stripped down.

use core::cmp::min;
use core::ffi::c_void;
use derive_more::derive::{Deref, DerefMut};
use dxt_lossless_transform_common::allocate::allocate_align_64;
use thiserror_no_std::Error;
use zstd_sys::ZSTD_cParameter::*;
use zstd_sys::ZSTD_dParameter::*;
use zstd_sys::ZSTD_format_e::*;
use zstd_sys::*;

/// Determines maximum file size for output needed to alloc to compress data with ZStandard.
///
/// # Parameters
///
/// * `source_length`: Number of bytes at source.
pub fn max_alloc_for_compress_size(source_length: usize) -> usize {
    unsafe { ZSTD_compressBound(source_length) }
}

/// Decompresses data with ZStandard
///
/// # Parameters
///
/// * `source`: Source data to decompress.
/// * `destination`: Destination buffer for decompressed data.
pub fn decompress(source: &[u8], destination: &mut [u8]) -> DecompressionResult {
    // Create decompression context
    let dctx = unsafe { ZSTD_createDCtx() };
    if dctx.is_null() {
        return Err(NxDecompressionError::ZStandard(
            ZSTD_ErrorCode::ZSTD_error_GENERIC,
        ));
    }

    // Set decompression parameters to match compression
    zstd_setcommondecompressionparams(dctx);

    // Perform decompression
    let result = unsafe {
        ZSTD_decompressDCtx(
            dctx,
            destination.as_mut_ptr() as *mut c_void,
            destination.len(),
            source.as_ptr() as *const c_void,
            source.len(),
        )
    };

    // Free the context
    unsafe {
        ZSTD_freeDCtx(dctx);
    }

    if unsafe { ZSTD_isError(result) } != 0 {
        let errcode = unsafe { ZSTD_getErrorCode(result) };
        return Err(NxDecompressionError::ZStandard(errcode));
    }

    Ok(result)
}

/// Compresses data with ZStandard.
/// Does not use fallback to 'copy' if compression is ineffective.
///
/// # Parameters
///
/// * `level`: Level at which we are compressing.
/// * `source`: Length of the source in bytes.
/// * `destination`: Pointer to destination.
pub fn compress(level: i32, source: &[u8], destination: &mut [u8]) -> CompressionResult {
    // Create a compression context
    let cctx = unsafe { ZSTD_createCCtx() };
    if cctx.is_null() {
        return Err(NxCompressionError::ZStandard(
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

    Err(NxCompressionError::ZStandard(unsafe {
        ZSTD_getErrorCode(result)
    }))
}

/// Estimates the compressed size of data using ZStandard without allocating a full output buffer.
/// This function uses streaming compression with optimal buffer sizes to get accurate size estimates
/// while minimizing memory allocation.
///
/// # Parameters
///
/// * `level`: Compression level to use.
/// * `source`: Source data to compress.
///
/// # Returns
///
/// * `Ok(usize)`: The estimated compressed size in bytes.
/// * `Err(NxCompressionError)`: If size estimation fails.
pub fn estimate_compressed_size(level: i32, source: &[u8]) -> CompressionResult {
    unsafe {
        // Create compression stream
        let cstream = SafeCStream::new(ZSTD_createCStream());
        if cstream.is_null() {
            return Err(NxCompressionError::ZStandard(
                ZSTD_ErrorCode::ZSTD_error_memory_allocation,
            ));
        }

        // Set compression parameters (magicless format, no extra headers)
        zstd_setcommoncompressparams(*cstream, Some(level));

        // Initialize the stream
        let init_result = ZSTD_initCStream(*cstream, level);
        if ZSTD_isError(init_result) != 0 {
            return Err(NxCompressionError::ZStandard(ZSTD_getErrorCode(
                init_result,
            )));
        }

        // Use optimal buffer sizes for streaming compression
        let in_buffer_size = ZSTD_CStreamInSize();
        let out_buffer_size = ZSTD_CStreamOutSize();
        let mut temp_buffer = allocate_align_64(out_buffer_size).unwrap();
        let mut total_read: usize = 0;
        let mut total_compressed: usize = 0;
        let source_len = source.len();

        while total_read < source_len {
            let to_read = min(in_buffer_size, source_len - total_read);
            let last_chunk = to_read < in_buffer_size;
            let mode = if last_chunk {
                ZSTD_EndDirective::ZSTD_e_end
            } else {
                ZSTD_EndDirective::ZSTD_e_continue
            };

            let mut input = ZSTD_inBuffer {
                // SAFETY: total_read is guaranteed under < source_len by while condition above.
                src: source.as_ptr().add(total_read) as *const c_void,
                size: to_read,
                pos: 0,
            };

            let mut finished = false;
            while !finished {
                let mut output = ZSTD_outBuffer {
                    dst: temp_buffer.as_mut_ptr() as *mut c_void,
                    size: temp_buffer.len(),
                    pos: 0,
                };

                let result = ZSTD_compressStream2(*cstream, &mut output, &mut input, mode);

                // Check if zstd returned an error
                if ZSTD_isError(result) != 0 {
                    return Err(NxCompressionError::ZStandard(ZSTD_getErrorCode(result)));
                }

                // Add the compressed bytes to our total
                total_compressed += output.pos;

                // If we're on the last chunk we're finished when zstd returns 0,
                // which means its consumed all the input AND finished the frame.
                // Otherwise, we're finished when we've consumed all the input.
                finished = if last_chunk {
                    result == 0
                } else {
                    input.pos == input.size
                };
            }

            total_read += input.pos;

            if last_chunk {
                break;
            }
        }

        Ok(total_compressed)
    }
}

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

pub(crate) fn zstd_setcommondecompressionparams(dctx: *mut ZSTD_DCtx_s) {
    unsafe {
        ZSTD_DCtx_setParameter(
            dctx,
            ZSTD_d_experimentalParam1, // zstd_d_format
            ZSTD_f_zstd1_magicless as i32,
        );
    };
}

/// A result type around compression functions..
/// Either a success code (number of bytes decompressed), or an error code.
pub type DecompressionResult = Result<usize, NxDecompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum NxDecompressionError {
    #[error("ZStandard Error: {0:?}")]
    ZStandard(#[from] ZSTD_ErrorCode),
}

/// A result type around compression functions..
/// Either a success code (number of bytes written), or an error code.
pub type CompressionResult = Result<usize, NxCompressionError>;

/// Represents an error returned from the Nx compression APIs.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Error)]
pub enum NxCompressionError {
    #[error("ZStandard Error: {0:?}")]
    ZStandard(#[from] ZSTD_ErrorCode),
}

#[derive(Deref, DerefMut)]
pub struct SafeCStream(*mut ZSTD_CStream);

impl SafeCStream {
    pub fn new(stream: *mut ZSTD_CStream) -> Self {
        Self(stream)
    }
}

impl Drop for SafeCStream {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { ZSTD_freeCStream(self.0) };
        }
    }
}
