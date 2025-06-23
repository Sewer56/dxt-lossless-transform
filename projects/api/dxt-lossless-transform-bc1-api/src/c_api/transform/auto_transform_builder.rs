//! BC1 auto transform builder for C API.
//!
//! This module provides ABI-stable functions for configuring BC1 auto transform builder
//! in a convenient builder pattern that mirrors the Rust API structure.

use crate::c_api::Dltbc1TransformSettings;
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform::manual_transform_builder::Dltbc1ManualTransformBuilder;
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
pub struct Dltbc1AutoTransformBuilderImpl {
    /// Whether to use all decorrelation modes during optimization
    pub use_all_decorrelation_modes: bool,
    /// The size estimator to use
    pub estimator: *const DltSizeEstimator,
}

/// Opaque handle for BC1 auto transform builder.
///
/// This builder allows configuring options for BC1 transformation with automatic optimization.
///
/// **Usage Pattern:**
/// 1. Create builder with [`dltbc1_new_AutoTransformBuilder`]
/// 2. Configure with [`dltbc1_AutoTransformBuilder_SetUseAllDecorrelationModes`]  
/// 3. Transform with [`dltbc1_AutoTransformBuilder_Transform`] (returns configured manual builder)
/// 4. Use returned manual builder for untransformation
/// 5. Free both builders when done
///
/// The builder can be reused multiple times and must be explicitly freed with
/// [`dltbc1_free_AutoTransformBuilder`].
///
/// # Remarks
/// This type corresponds to [`crate::Bc1AutoTransformBuilder`] in the Rust API.
#[repr(C)]
pub struct Dltbc1AutoTransformBuilder {
    _private: [u8; 0],
}

/// Create a new BC1 auto transform builder with the provided estimator.
///
/// The estimator should have its compression level and other parameters already configured.
/// This allows for more flexible usage patterns where different estimators can have
/// completely different configuration approaches.
///
/// The returned builder must be freed with [`dltbc1_free_AutoTransformBuilder`].
///
/// # Parameters
/// - `estimator`: The size estimator to use for finding the best possible transform.
///   This will test different transform configurations and choose the one that results
///   in the smallest estimated compressed size according to this estimator.
///
/// # Returns
/// A pointer to a new builder, or null if allocation fails.
///
/// # Safety
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - The estimator's context and functions must remain valid for the lifetime of the builder
///
/// # Remarks
/// This function corresponds to [`crate::Bc1AutoTransformBuilder::new`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_new_AutoTransformBuilder(
    estimator: *const DltSizeEstimator,
) -> *mut Dltbc1AutoTransformBuilder {
    if estimator.is_null() {
        return core::ptr::null_mut();
    }

    let builder_impl = Box::new(Dltbc1AutoTransformBuilderImpl {
        use_all_decorrelation_modes: false,
        estimator,
    });

    Box::into_raw(builder_impl) as *mut Dltbc1AutoTransformBuilder
}

/// Free a BC1 auto transform builder.
///
/// # Safety
/// - `builder` must be a valid pointer returned by [`dltbc1_new_AutoTransformBuilder`]
/// - `builder` must not have been freed already
/// - After calling this function, `builder` becomes invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_free_AutoTransformBuilder(
    builder: *mut Dltbc1AutoTransformBuilder,
) {
    if !builder.is_null() {
        unsafe {
            drop(Box::from_raw(
                builder as *mut Dltbc1AutoTransformBuilderImpl,
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
/// - `builder` must be a valid pointer to a [`Dltbc1AutoTransformBuilder`]
///
/// # Remarks
/// This function corresponds to [`crate::Bc1AutoTransformBuilder::use_all_decorrelation_modes`] in the Rust API.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_AutoTransformBuilder_SetUseAllDecorrelationModes(
    builder: *mut Dltbc1AutoTransformBuilder,
    use_all: bool,
) -> Dltbc1Result {
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullBuilderPointer);
    }

    let builder_impl = unsafe { &mut *(builder as *mut Dltbc1AutoTransformBuilderImpl) };
    builder_impl.use_all_decorrelation_modes = use_all;
    Dltbc1Result::success()
}

/// Transform BC1 data using automatically determined optimal settings and return a configured manual builder.
///
/// This function determines optimal transform settings using the configured estimator,
/// applies the transformation to the input data, and outputs a pre-configured
/// manual transform builder for untransformation.
///
/// # Parameters
/// - `builder`: The configured auto builder
/// - `data`: Pointer to BC1 data to transform
/// - `data_len`: Length of input data in bytes (must be divisible by 8)
/// - `output`: Pointer to output buffer where transformed data will be written
/// - `output_len`: Length of output buffer in bytes (must be at least `data_len`)
/// - `out_manual_builder`: Output pointer where the configured manual builder will be written.
///   The returned builder must be freed with [`dltbc1_free_ManualTransformBuilder`].
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error code.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1AutoTransformBuilder`]
/// - `data` must be valid for reads of `data_len` bytes
/// - `output` must be valid for writes of `output_len` bytes
/// - `out_manual_builder` must be a valid pointer to write the result
/// - The estimator associated with the builder must remain valid for the duration of the call
///
/// # Examples
///
/// ```c
/// // Create auto transform builder with estimator
/// Dltbc1AutoTransformBuilder* auto_builder = dltbc1_new_AutoTransformBuilder(estimator);
/// dltbc1_AutoTransformBuilder_SetUseAllDecorrelationModes(auto_builder, false);
///
/// // Transform and get configured manual builder
/// Dltbc1ManualTransformBuilder* manual_builder = NULL;
/// Dltbc1Result result = dltbc1_AutoTransformBuilder_Transform(
///     auto_builder, bc1_data, sizeof(bc1_data),
///     transformed_data, sizeof(transformed_data), &manual_builder);
///
/// if (result.error_code == DLTBC1_SUCCESS) {
///     // Later, untransform using the returned manual builder
///     Dltbc1Result untransform_result = dltbc1_ManualTransformBuilder_Untransform(
///         transformed_data, sizeof(transformed_data),
///         restored_data, sizeof(restored_data), manual_builder);
///
///     // Clean up
///     dltbc1_free_ManualTransformBuilder(manual_builder);
/// }
/// dltbc1_free_AutoTransformBuilder(auto_builder);
/// ```
///
/// # Remarks
/// This function corresponds to [`crate::Bc1AutoTransformBuilder::transform`] in the Rust API.
///
/// [`dltbc1_free_ManualTransformBuilder`]: crate::c_api::transform::manual_transform_builder::dltbc1_free_ManualTransformBuilder
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_AutoTransformBuilder_Transform(
    builder: *mut Dltbc1AutoTransformBuilder,
    data: *const u8,
    data_len: usize,
    output: *mut u8,
    output_len: usize,
    out_manual_builder: *mut *mut Dltbc1ManualTransformBuilder,
) -> Dltbc1Result {
    // Validate required pointers
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullBuilderPointer);
    }
    if data.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullDataPointer);
    }
    if output.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }
    if out_manual_builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullOutputBufferPointer);
    }

    // Get settings from builder
    let builder_impl = unsafe { &*(builder as *const Dltbc1AutoTransformBuilderImpl) };
    let estimator = builder_impl.estimator;
    let use_all_modes = builder_impl.use_all_decorrelation_modes;

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

    // Convert the core result to API result
    let api_result = Dltbc1Result::from(result);

    // If successful, create and return a configured manual builder
    if api_result.is_success() {
        // Convert core transform details to API transform details
        let api_transform_details = Dltbc1TransformSettings {
            decorrelation_mode: transform_details.decorrelation_mode.into(),
            split_colour_endpoints: transform_details.split_colour_endpoints,
        };

        // Create a new manual builder with the optimal settings
        let manual_builder = Bc1ManualTransformBuilder::new()
            .decorrelation_mode(api_transform_details.decorrelation_mode)
            .split_colour_endpoints(api_transform_details.split_colour_endpoints);

        // Create the C API wrapper
        let inner = Box::new(
            crate::c_api::transform::manual_transform_builder::Dltbc1ManualTransformBuilderInner {
                builder: manual_builder,
            },
        );

        // Write the result to the output pointer
        unsafe {
            *out_manual_builder = Box::into_raw(inner) as *mut Dltbc1ManualTransformBuilder;
        }
    } else {
        // On failure, ensure the output pointer is null
        unsafe {
            *out_manual_builder = core::ptr::null_mut();
        }
    }

    api_result
}
