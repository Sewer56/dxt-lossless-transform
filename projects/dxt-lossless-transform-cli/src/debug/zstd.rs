//! Copied from sewer56.archives.nx crate.

use core::ffi::c_void;
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
