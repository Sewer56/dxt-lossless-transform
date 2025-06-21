//! ABI-unstable C API functions for determining optimal BC1 transform settings.
//!
//! ## Warning: ABI Instability
//!
//! Functions in this module are prefixed with `dltbc1_unstable_` and are **NOT ABI-stable**.
//! The parameter structures may change between versions, potentially breaking compatibility.

use crate::Bc1Error;
use crate::c_api::Dltbc1TransformDetails;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::determine_optimal_transform::determine_optimal_transform;
use core::slice;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;

/// Settings for determining optimal BC1 transform configuration (ABI-unstable).
///
/// This struct contains all settings that affect how the optimal transform
/// is determined. Using a struct allows adding new fields without breaking
/// the function signature, though the struct layout itself is still ABI-unstable.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc1DetermineOptimalSettings {
    /// If true, tests all decorrelation modes; if false, only tests Variant1 and None
    pub use_all_modes: bool,
}

/// Determine optimal transform settings for BC1 data (ABI-unstable).
///
/// ## ABI Instability Warning
/// This function accepts and returns ABI-unstable structures which may change between versions.
/// Use `dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal` for ABI stability.
///
/// # Parameters
/// - `data`: Pointer to BC1 data to analyze
/// - `data_len`: Length of data in bytes (must be divisible by 8)
/// - `estimator`: The size estimator to use for compression estimation
/// - `settings`: Settings controlling the optimization process
/// - `out_details`: Pointer where optimal transform details will be written on success
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `out_details` must be a valid pointer for writing [`Dltbc1TransformDetails`]
/// - The estimator's context and functions must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_determine_optimal(
    data: *const u8,
    data_len: usize,
    estimator: *const DltSizeEstimator,
    settings: Dltbc1DetermineOptimalSettings,
    out_details: *mut Dltbc1TransformDetails,
) -> Dltbc1Result {
    // Validate pointers
    if data.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if estimator.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullEstimatorPointer);
    }
    if out_details.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputPointer);
    }

    // Create slice from raw pointer
    let data_slice = unsafe { slice::from_raw_parts(data, data_len) };

    // Use the provided estimator
    let estimator_ref = unsafe { &*estimator };

    // Determine optimal transform
    match determine_optimal_transform(data_slice, estimator_ref, settings.use_all_modes) {
        Ok(optimal_details) => {
            // Write the optimal details to the output pointer
            unsafe {
                *out_details = optimal_details.into();
            }
            Dltbc1Result::success()
        }
        Err(e) => {
            // Map the error to error codes
            match e {
                Bc1Error::SizeEstimationFailed(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::SizeEstimationFailed)
                }
                Bc1Error::InvalidLength(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength)
                }
                Bc1Error::OutputBufferTooSmall { .. } => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::OutputBufferTooSmall)
                }
                Bc1Error::AllocationFailed(_) => {
                    Dltbc1Result::from_error_code(Dltbc1ErrorCode::AllocationFailed)
                }
            }
        }
    }
}
