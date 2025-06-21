//! C API builder for BC1 determine optimal transform options.

use crate::c_api::{determine_optimal_transform::Dltbc1TransformContext, error::Dltbc1Result};
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;

/// Opaque handle for BC1 estimate options builder.
///
/// This builder allows configuring options for determining optimal BC1 transform settings.
/// Use the provided functions to configure the builder and then call
/// [`dltbc1_estimate_options_build_and_determine_optimal`] to execute the optimization.
#[repr(C)]
pub struct Dltbc1EstimateOptionsBuilder {
    use_all_decorrelation_modes: bool,
}

/// Create a new BC1 estimate options builder with default settings.
///
/// The builder is initialized with:
/// - `use_all_decorrelation_modes`: false (tests only Variant1 and None for faster optimization)
///
/// # Returns
/// A new builder instance that must be freed with [`dltbc1_estimate_options_builder_free`].
///
/// # Safety
/// - This function is unsafe because it returns a raw pointer that must be freed with [`dltbc1_estimate_options_builder_free`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_estimate_options_builder_new() -> *mut Dltbc1EstimateOptionsBuilder
{
    let builder = Box::new(Dltbc1EstimateOptionsBuilder {
        use_all_decorrelation_modes: false,
    });

    Box::into_raw(builder)
}

/// Free a BC1 estimate options builder.
///
/// # Parameters
/// - `builder`: The builder to free (can be null)
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc1_estimate_options_builder_new`] or null
/// - The builder must not be used after calling this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_estimate_options_builder_free(
    builder: *mut Dltbc1EstimateOptionsBuilder,
) {
    if !builder.is_null() {
        let _ = unsafe { Box::from_raw(builder) };
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
pub unsafe extern "C" fn dltbc1_estimate_options_set_use_all_decorrelation_modes(
    builder: *mut Dltbc1EstimateOptionsBuilder,
    use_all: bool,
) {
    if builder.is_null() {
        return;
    }

    let builder_ref = unsafe { &mut *builder };
    builder_ref.use_all_decorrelation_modes = use_all;
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
pub unsafe extern "C" fn dltbc1_estimate_options_build_and_determine_optimal(
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
        let builder_box = unsafe { Box::from_raw(builder) };
        builder_box.use_all_decorrelation_modes
    };

    // Call the existing determine optimal transform function
    unsafe {
        super::dltbc1_determine_optimal_transform(data, data_len, estimator, use_all_modes, context)
    }
}
