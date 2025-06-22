//! BC1 estimate settings builder for C API.
//!
//! This module provides ABI-stable functions for configuring BC1 estimate settings
//! in a convenient builder pattern.

use crate::c_api::Dltbc1TransformSettings;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform::manual_transform_builder::{
    Dltbc1TransformSettingsBuilder, get_settings_builder_mut,
};
use crate::transform::Bc1ManualTransformBuilder;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
use dxt_lossless_transform_bc1::c_api::transform_auto::{
    Dltbc1AutoTransformSettings as CoreAutoTransformSettings, Dltbc1ErrorCode as CoreErrorCode,
    Dltbc1Result as CoreResult, Dltbc1TransformSettings as CoreTransformSettings,
};

// Conversion from core's Dltbc1Result to API's Dltbc1Result
impl From<CoreResult> for Dltbc1Result {
    fn from(core_result: CoreResult) -> Self {
        let api_error_code = match core_result.error_code {
            CoreErrorCode::Success => Dltbc1ErrorCode::Success,
            CoreErrorCode::NullDataPointer => Dltbc1ErrorCode::NullDataPointer,
            CoreErrorCode::NullOutputBufferPointer => Dltbc1ErrorCode::NullOutputBufferPointer,
            CoreErrorCode::NullEstimatorPointer => Dltbc1ErrorCode::NullEstimatorPointer,
            CoreErrorCode::NullTransformSettingsPointer => {
                Dltbc1ErrorCode::NullTransformSettingsPointer
            }
            CoreErrorCode::InvalidDataLength => Dltbc1ErrorCode::InvalidLength,
            CoreErrorCode::OutputBufferTooSmall => Dltbc1ErrorCode::OutputBufferTooSmall,
            CoreErrorCode::SizeEstimationError => Dltbc1ErrorCode::SizeEstimationFailed,
            CoreErrorCode::TransformationError => Dltbc1ErrorCode::AllocationFailed, // Map to closest available
        };

        Self::from_error_code(api_error_code)
    }
}

/// Internal structure holding the actual builder data.
pub struct Dltbc1EstimateSettingsBuilderImpl {
    /// Whether to use all decorrelation modes during optimization
    pub use_all_decorrelation_modes: bool,
}

/// Opaque handle for BC1 estimate settings builder.
///
/// This builder allows configuring options for BC1 transformation with automatic optimization.
/// Use the provided functions to configure the builder and then call
/// [`dltbc1_EstimateSettingsBuilder_BuildAndTransform`] to execute the transformation.
///
/// The builder can be reused multiple times and must be explicitly freed with
/// [`dltbc1_free_EstimateSettingsBuilder`].
///
/// The internal structure of this builder is completely hidden from C callers.
#[repr(C)]
pub struct Dltbc1EstimateSettingsBuilder {
    _private: [u8; 0],
}

/// Create a new BC1 estimate settings builder with default settings.
///
/// The builder starts with conservative defaults that prioritize speed over compression.
/// Use [`dltbc1_EstimateSettingsBuilder_SetUseAllDecorrelationModes`] to configure
/// whether to test all decorrelation modes.
///
/// The returned builder must be freed with [`dltbc1_free_EstimateSettingsBuilder`].
///
/// # Returns
/// A pointer to a new builder, or null if allocation fails.
#[unsafe(no_mangle)]
pub extern "C" fn dltbc1_new_EstimateSettingsBuilder() -> *mut Dltbc1EstimateSettingsBuilder {
    let builder_impl = Box::new(Dltbc1EstimateSettingsBuilderImpl {
        use_all_decorrelation_modes: false,
    });

    Box::into_raw(builder_impl) as *mut Dltbc1EstimateSettingsBuilder
}

/// Free a BC1 estimate settings builder.
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc1_new_EstimateSettingsBuilder`]
/// - `builder` must not have been freed already
/// - After calling this function, `builder` becomes invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_free_EstimateSettingsBuilder(
    builder: *mut Dltbc1EstimateSettingsBuilder,
) {
    if !builder.is_null() {
        unsafe {
            drop(Box::from_raw(
                builder as *mut Dltbc1EstimateSettingsBuilderImpl,
            ));
        }
    }
}

/// Set whether to use all decorrelation modes during optimization.
///
/// When `false` (default), only tests common configurations for faster optimization.
/// When `true`, tests all decorrelation modes for potentially better compression
/// at the cost of twice as long optimization time.
///
/// **Note**: The typical improvement from testing all decorrelation modes is <0.1% in practice.
/// For better compression gains, it's recommended to use a compression level on the
/// estimator (e.g., ZStandard estimator) closer to your final compression level instead.
///
/// # Parameters
/// - `builder`: The builder to configure
/// - `use_all`: Whether to test all decorrelation modes
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1EstimateSettingsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_EstimateSettingsBuilder_SetUseAllDecorrelationModes(
    builder: *mut Dltbc1EstimateSettingsBuilder,
    use_all: bool,
) -> Dltbc1Result {
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullBuilderPointer);
    }

    let builder_impl = unsafe { &mut *(builder as *mut Dltbc1EstimateSettingsBuilderImpl) };
    builder_impl.use_all_decorrelation_modes = use_all;
    Dltbc1Result::success()
}

/// Transform BC1 data using automatically determined optimal settings (ABI-stable).
///
/// This function uses the configured options to determine the optimal
/// transform settings for the given BC1 data, applies the transformation,
/// and stores the transform details in the provided context.
/// The builder remains valid after this call and can be reused.
///
/// # Parameters
/// - `builder`: The configured builder (can be null to use default settings)
/// - `data`: Pointer to BC1 data to transform
/// - `data_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `data_len`)
/// - `estimator`: The size estimator to use for compression estimation
/// - `settings_builder`: The transform settings builder where transform details will be stored
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `settings_builder` must be a valid pointer to a [`Dltbc1TransformSettingsBuilder`]
/// - The estimator's context and functions must remain valid for the duration of the call
/// - The builder (if not null) must be a valid pointer to a [`Dltbc1EstimateSettingsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_EstimateSettingsBuilder_BuildAndTransform(
    builder: *mut Dltbc1EstimateSettingsBuilder,
    data: *const u8,
    data_len: usize,
    output: *mut u8,
    output_len: usize,
    estimator: *const DltSizeEstimator,
    settings_builder: *mut Dltbc1TransformSettingsBuilder,
) -> Dltbc1Result {
    // Validate settings builder
    if settings_builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformSettingsBuilderPointer);
    }

    // Get settings from builder without freeing it
    let use_all_modes = if builder.is_null() {
        false // Default value if builder is null
    } else {
        let builder_impl = unsafe { &*(builder as *const Dltbc1EstimateSettingsBuilderImpl) };
        builder_impl.use_all_decorrelation_modes
    };

    // Create settings struct
    let settings = CoreAutoTransformSettings { use_all_modes };

    // Allocate space for the transform details
    let mut transform_details = CoreTransformSettings::default();

    // Call the core transform auto function
    let result = unsafe {
        dxt_lossless_transform_bc1::c_api::transform_auto::dltbc1core_transform_auto(
            data,
            data_len,
            output,
            output_len,
            estimator,
            settings,
            &mut transform_details as *mut CoreTransformSettings,
        )
    };

    // If successful, update the settings builder
    if result.is_success() {
        let builder_inner = unsafe { get_settings_builder_mut(settings_builder) };

        // Convert core transform details to API transform details
        let api_transform_details = Dltbc1TransformSettings {
            decorrelation_mode: match transform_details.decorrelation_mode {
                0 => dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::None,
                1 => {
                    dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant1
                }
                2 => {
                    dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant2
                }
                3 => {
                    dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant3
                }
                _ => {
                    dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant1
                } // Default fallback
            },
            split_colour_endpoints: transform_details.split_colour_endpoints,
        };

        // Update the builder with the transform details
        builder_inner.builder = Bc1ManualTransformBuilder::new()
            .decorrelation_mode(api_transform_details.decorrelation_mode)
            .split_colour_endpoints(api_transform_details.split_colour_endpoints);
    }

    result.into()
}
