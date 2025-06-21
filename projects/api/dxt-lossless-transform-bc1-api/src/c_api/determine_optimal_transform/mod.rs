//! C API for determining optimal BC1 transform settings.
//!
//! This module contains the private implementation of the determine optimal transform functionality.

use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform::transform_context::{Dltbc1TransformContext, get_context_mut};
use crate::determine_optimal_transform::determine_optimal_transform;
use crate::{Bc1Error, transform::Bc1TransformOptionsBuilder};
use core::slice;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;

/// Internal structure holding the actual builder data.
struct Dltbc1EstimateOptionsBuilderImpl {
    use_all_decorrelation_modes: bool,
}

/// Opaque handle for BC1 estimate options builder.
///
/// This builder allows configuring options for determining optimal BC1 transform settings.
/// Use the provided functions to configure the builder and then call
/// [`dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal`] to execute the optimization.
///
/// The internal structure of this builder is completely hidden from C callers.
#[repr(C)]
pub struct Dltbc1EstimateOptionsBuilder {
    _private: [u8; 0],
}

/// Create a new BC1 estimate options builder with default settings.
///
/// The builder is initialized with:
/// - `use_all_decorrelation_modes`: false (tests only Variant1 and None for faster optimization)
///
/// # Returns
/// A new builder instance that must be freed with [`dltbc1_free_EstimateOptionsBuilder`].
///
/// # Safety
/// - This function is unsafe because it returns a raw pointer that must be freed with [`dltbc1_free_EstimateOptionsBuilder`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_new_EstimateOptionsBuilder() -> *mut Dltbc1EstimateOptionsBuilder {
    let builder_impl = Box::new(Dltbc1EstimateOptionsBuilderImpl {
        use_all_decorrelation_modes: false,
    });

    Box::into_raw(builder_impl) as *mut Dltbc1EstimateOptionsBuilder
}

/// Free a BC1 estimate options builder.
///
/// # Parameters
/// - `builder`: The builder to free (can be null)
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc1_new_EstimateOptionsBuilder`] or null
/// - The builder must not be used after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_free_EstimateOptionsBuilder(
    builder: *mut Dltbc1EstimateOptionsBuilder,
) {
    if !builder.is_null() {
        let _ = unsafe { Box::from_raw(builder as *mut Dltbc1EstimateOptionsBuilderImpl) };
    }
}

/// Set whether to use all decorrelation modes.
///
/// When `false` (default), only tests `YCoCgVariant::Variant1` and `YCoCgVariant::None`
/// for faster optimization with good results.
///
/// When `true`, tests all available decorrelation modes for potentially better
/// compression at the cost of longer optimization time.
///
/// # Parameters
/// - `builder`: The builder to configure
/// - `use_all`: Whether to use all decorrelation modes
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1EstimateOptionsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(
    builder: *mut Dltbc1EstimateOptionsBuilder,
    use_all: bool,
) {
    if builder.is_null() {
        return;
    }

    let builder_impl = unsafe { &mut *(builder as *mut Dltbc1EstimateOptionsBuilderImpl) };
    builder_impl.use_all_decorrelation_modes = use_all;
}

/// Build the estimate options and determine optimal transform settings for BC1 data.
///
/// This function consumes the builder, uses the configured options to determine the optimal
/// transform settings, and stores the results in the provided context.
///
/// # Parameters
/// - `builder`: The builder to use (will be freed automatically)
/// - `data`: Pointer to BC1 data to analyze
/// - `data_len`: Length of data in bytes (must be divisible by 8)
/// - `estimator`: The size estimator to use for compression estimation
/// - `context`: The BC1 context where optimal options will be stored on success
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error that must be freed.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1EstimateOptionsBuilder`] or null
/// - `data` must be valid for reads of `data_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
/// - The estimator's context and functions must remain valid for the duration of the call
/// - The builder will be automatically freed, regardless of success or failure
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
    builder: *mut Dltbc1EstimateOptionsBuilder,
    data: *const u8,
    data_len: usize,
    estimator: *const DltSizeEstimator,
    context: *mut Dltbc1TransformContext,
) -> Dltbc1Result {
    // Always free the builder, even on early return
    let use_all_modes = if builder.is_null() {
        false // Default value if builder is null
    } else {
        let builder_box =
            unsafe { Box::from_raw(builder as *mut Dltbc1EstimateOptionsBuilderImpl) };
        builder_box.use_all_decorrelation_modes
    };

    // Call the private determine optimal transform function
    unsafe { dltbc1_determine_optimal_transform(data, data_len, estimator, use_all_modes, context) }
}

/// Private function to determine the optimal transform settings for BC1 data.
///
/// This function tests different transform configurations and returns the one
/// that produces the best compression ratio estimate using the provided size estimator.
///
/// This function is internal and should only be called through the builder API.
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
pub(crate) unsafe fn dltbc1_determine_optimal_transform(
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
