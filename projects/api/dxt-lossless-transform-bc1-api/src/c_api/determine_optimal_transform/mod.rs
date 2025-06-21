//! C API for determining optimal BC1 transform settings.

pub mod builder;

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform::transform_context::{Dltbc1TransformContext, get_context_mut};
use crate::determine_optimal_transform::determine_optimal_transform;
use crate::{Bc1Error, transform::Bc1TransformOptionsBuilder};
use core::slice;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

/// Determine the optimal transform settings for BC1 data.
///
/// This function tests different transform configurations and returns the one
/// that produces the best compression ratio estimate using the provided size estimator.
///
/// # Parameters
/// - `data`: Pointer to BC1 data to analyze
/// - `data_len`: Length of data in bytes (must be divisible by 8)
/// - `estimator`: The size estimator to use for compression estimation
/// - `use_all_modes`: If true, tests all decorrelation modes; if false, only tests Variant1
/// - `context`: The BC1 context where optimal options will be stored on success
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
/// - The estimator's context and functions must remain valid for the duration of the call
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_determine_optimal_transform(
    data: *const u8,
    data_len: usize,
    estimator: *const DltSizeEstimator,
    use_all_modes: bool,
    context: *mut Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate pointers
    if data.is_null() || estimator.is_null() || context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::InvalidLength);
    }

    // Create slice from raw pointer
    let data_slice = unsafe { slice::from_raw_parts(data, data_len) };

    // Use the provided estimator - dereference it
    let estimator_ref = unsafe { &*estimator };

    // Determine optimal transform
    match determine_optimal_transform(data_slice, estimator_ref, use_all_modes) {
        Ok(optimal_details) => {
            // Update the context with optimal settings
            let inner = unsafe { get_context_mut(context) };

            // Convert from internal variant to API decorrelation mode variant
            let api_variant =
                YCoCgVariant::from_internal_variant(optimal_details.decorrelation_mode);

            inner.builder = Bc1TransformOptionsBuilder::new()
                .decorrelation_mode(api_variant)
                .split_colour_endpoints(optimal_details.split_colour_endpoints);

            Dltbc1Result::success()
        }
        Err(e) => {
            // Map the error to error codes directly
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
