//! C-compatible size estimation interface.

use crate::estimate::{DataType, SizeEstimationOperations};
use core::{ffi::c_void, fmt::Display};

/// Function pointer type for [`SizeEstimationOperations::max_compressed_size`] operation.
///
/// # Parameters
/// - `context`: User-provided context (can be null)
/// - `len_bytes`: Length of the input data in bytes
/// - `out_size`: Output parameter for the maximum compressed size
///
/// # Returns
/// 0 on success, non-zero error code on failure
pub type DltMaxCompressedSizeFn =
    unsafe extern "C" fn(context: *mut c_void, len_bytes: usize, out_size: *mut usize) -> u32;

/// Function pointer type for [`SizeEstimationOperations::estimate_compressed_size`] operation.
///
/// # Parameters
/// - `context`: User-provided context (can be null)
/// - `input_ptr`: Pointer to the input data
/// - `len_bytes`: Length of the input data in bytes
/// - `data_type`: The type of data being compressed (see DataType enum values)
/// - `output_ptr`: Pre-allocated output buffer for compression
/// - `output_len`: Length of the pre-allocated output buffer
/// - `out_size`: Output parameter for the estimated compressed size
///
/// # Returns
/// 0 on success, non-zero error code on failure
pub type DltEstimateCompressedSizeFn = unsafe extern "C" fn(
    context: *mut c_void,
    input_ptr: *const u8,
    len_bytes: usize,
    data_type: u8,
    output_ptr: *mut u8,
    output_len: usize,
    out_size: *mut usize,
) -> u32;

/// C-compatible size estimator that wraps function pointers.
#[repr(C)]
pub struct DltSizeEstimator {
    /// User-provided context passed to all callbacks
    pub context: *mut c_void,
    /// Function to get maximum compressed size
    pub max_compressed_size: DltMaxCompressedSizeFn,
    /// Function to estimate compressed size
    pub estimate_compressed_size: DltEstimateCompressedSizeFn,
    /// Whether the estimator supports data type differentiation
    pub supports_data_type_differentiation: bool,
}

// Safety: DltSizeEstimator is Send if the context pointer is Send
unsafe impl Send for DltSizeEstimator {}

// Safety: DltSizeEstimator is Sync if the context pointer is Sync
unsafe impl Sync for DltSizeEstimator {}

/// Error type for C API size estimation
#[derive(Debug)]
pub struct CSizeEstimationError {
    /// The error code returned by the size estimation function
    pub code: u32,
}

impl Display for CSizeEstimationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Size estimation error: code {}", self.code)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CSizeEstimationError {}

impl SizeEstimationOperations for DltSizeEstimator {
    type Error = CSizeEstimationError;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        let mut out_size = 0;
        let result = unsafe { (self.max_compressed_size)(self.context, len_bytes, &mut out_size) };

        if result == 0 {
            Ok(out_size)
        } else {
            Err(CSizeEstimationError { code: result })
        }
    }

    fn supports_data_type_differentiation(&self) -> bool {
        self.supports_data_type_differentiation
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        let mut out_size = 0;
        let result = unsafe {
            (self.estimate_compressed_size)(
                self.context,
                input_ptr,
                len_bytes,
                data_type as u8,
                output_ptr,
                output_len,
                &mut out_size,
            )
        };

        if result == 0 {
            Ok(out_size)
        } else {
            Err(CSizeEstimationError { code: result })
        }
    }
}

/// Implementation for references to allow passing by pointer from C
impl SizeEstimationOperations for &DltSizeEstimator {
    type Error = CSizeEstimationError;

    fn max_compressed_size(&self, len_bytes: usize) -> Result<usize, Self::Error> {
        (*self).max_compressed_size(len_bytes)
    }

    fn supports_data_type_differentiation(&self) -> bool {
        (*self).supports_data_type_differentiation()
    }

    unsafe fn estimate_compressed_size(
        &self,
        input_ptr: *const u8,
        len_bytes: usize,
        data_type: DataType,
        output_ptr: *mut u8,
        output_len: usize,
    ) -> Result<usize, Self::Error> {
        (*self).estimate_compressed_size(input_ptr, len_bytes, data_type, output_ptr, output_len)
    }
}
