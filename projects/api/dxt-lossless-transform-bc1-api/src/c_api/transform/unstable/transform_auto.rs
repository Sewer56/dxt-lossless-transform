//! BC1 automatic transform operations for C API (ABI-unstable).
//!
//! **⚠️ ABI Instability Warning**: All functions in this module accept ABI-unstable
//! structures which may change between versions without major version bumps.
//! For production use, prefer the ABI-stable builder patterns in
//! [`super::super::auto_transform_builder`].
//!
//! This module provides ABI-unstable functions for transforming BC1 data
//! using automatically determined optimal settings.

use crate::c_api::Dltbc1TransformSettings;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use core::slice;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
use dxt_lossless_transform_bc1::{Bc1EstimateSettings, transform_bc1_auto_safe};

/// Settings for automatic BC1 transform configuration (ABI-unstable).
///
/// **⚠️ ABI Instability Warning**: This struct layout may change between versions.
/// For ABI-stable alternatives, use the estimate settings builder pattern.
///
/// This struct contains all settings that affect how the optimal transform
/// is determined and applied. Using a struct allows adding new fields without breaking
/// the function signature, though the struct layout itself is still ABI-unstable.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Dltbc1AutoTransformSettings {
    /// If true, tests all decorrelation modes; if false, only tests Variant1 and None
    ///
    /// Note: The typical improvement from testing all decorrelation modes is <0.1% in practice.
    /// For better compression gains, consider using a compression level on the estimator
    /// (e.g., ZStandard estimator) closer to your final compression level instead.
    pub use_all_modes: bool,
}

// =============================================================================
// ABI-Unstable Functions
// =============================================================================

/// Transform BC1 data using automatically determined optimal settings (ABI-unstable).
///
/// This unstable function is provided for maximum performance scenarios where the caller
/// can accept the risk of potential breaking changes between library versions.
///
/// # Parameters
/// - `data`: Pointer to BC1 data to transform
/// - `data_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `data_len`)
/// - `estimator`: The size estimator to use for compression estimation
/// - `settings`: Settings controlling the optimization process
/// - `out_details`: Pointer where transform details will be written on success
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `out_details` must be a valid pointer for writing [`Dltbc1TransformSettings`]
/// - The estimator's context and functions must remain valid for the duration of the call
///
/// **⚠️ ABI Instability Warning**: This function accepts and returns ABI-unstable structures
/// which may change between library versions. For production use,
/// [`super::super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform`]
/// is strongly recommended as it guarantees better ABI stability across library versions.
///
/// **Prefer the ABI-stable builder:** This function provides direct access but may have
/// breaking changes. Consider using
/// [`super::super::auto_transform_builder::dltbc1_EstimateSettingsBuilder_BuildAndTransform`]
/// which provides ABI stability and the same functionality.
///
/// # Recommended Alternative
///
/// For production use:
/// ```c
/// // Create builders
/// Dltbc1EstimateSettingsBuilder* estimate_builder = dltbc1_new_EstimateSettingsBuilder();
/// Dltbc1TransformSettingsBuilder* settings_builder = dltbc1_new_TransformSettingsBuilder();
///
/// // Configure estimation (optional)
/// dltbc1_EstimateSettingsBuilder_SetUseAllDecorrelationModes(estimate_builder, false);
///
/// // Analyze and transform in one operation (ABI-stable)
/// Dltbc1Result result = dltbc1_EstimateSettingsBuilder_BuildAndTransform(
///     estimate_builder, data, data_len, output, output_len, &estimator, settings_builder);
///
/// // Clean up
/// dltbc1_free_EstimateSettingsBuilder(estimate_builder);
/// dltbc1_free_TransformSettingsBuilder(settings_builder);
/// ```
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_unstable_transform_auto(
    data: *const u8,
    data_len: usize,
    output: *mut u8,
    output_len: usize,
    estimator: *const DltSizeEstimator,
    settings: Dltbc1AutoTransformSettings,
    out_details: *mut Dltbc1TransformSettings,
) -> Dltbc1Result {
    // Validate pointers
    if data.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if estimator.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullEstimatorPointer);
    }
    if out_details.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformSettingsPointer);
    }

    // Create slices from raw pointers
    let data_slice = unsafe { slice::from_raw_parts(data, data_len) };
    let output_slice = unsafe { slice::from_raw_parts_mut(output, output_len) };

    // Use the provided estimator
    let estimator_ref = unsafe { &*estimator };

    // Create options struct
    let options = Bc1EstimateSettings {
        size_estimator: estimator_ref,
        use_all_decorrelation_modes: settings.use_all_modes,
    };

    // Transform with automatic optimization using core crate's safe function
    match transform_bc1_auto_safe(data_slice, output_slice, options) {
        Ok(transform_details) => {
            // Write the transform details to the output pointer
            unsafe {
                *out_details = transform_details.into();
            }
            Dltbc1Result::success()
        }
        Err(e) => e.into(),
    }
}
