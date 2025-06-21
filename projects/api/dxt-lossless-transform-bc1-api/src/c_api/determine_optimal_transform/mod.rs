//! C API for determining optimal BC1 transform settings.
//!
//! This module provides ABI-stable builder pattern for determining optimal transform settings.
//! The builder pattern ensures ABI stability while internally using the unstable functions.
//!
//! [`YCoCgVariant::Variant1`]: dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant1
//! [`YCoCgVariant::None`]: dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::None
//! [`DltSizeEstimator`]: dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator
//! [`Dltbc1TransformContext`]: crate::c_api::transform::transform_context::Dltbc1TransformContext

pub mod unstable;

use crate::c_api::Dltbc1TransformDetails;
use crate::c_api::determine_optimal_transform::unstable::{
    Dltbc1DetermineOptimalSettings, dltbc1_unstable_determine_optimal,
};
use crate::c_api::error::{Dltbc1ErrorCode, Dltbc1Result};
use crate::c_api::transform::transform_context::{Dltbc1TransformContext, get_context_mut};
use crate::transform::Bc1TransformOptionsBuilder;
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
/// The builder can be reused multiple times and must be explicitly freed with
/// [`dltbc1_free_EstimateOptionsBuilder`].
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
/// When `false` (default), only tests [`YCoCgVariant::Variant1`] and [`YCoCgVariant::None`]
/// for faster optimization with good results.
///
/// When `true`, tests all available decorrelation modes for potentially better
/// compression at the cost of twice as long optimization time (tests 4 options
/// instead of 2) for negligible gains (typically <0.1% extra savings).
///
/// # Parameters
/// - `builder`: The builder to configure
/// - `use_all`: Whether to use all decorrelation modes
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `builder` must be a valid pointer to a [`Dltbc1EstimateOptionsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(
    builder: *mut Dltbc1EstimateOptionsBuilder,
    use_all: bool,
) -> Dltbc1Result {
    if builder.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullBuilderPointer);
    }

    let builder_impl = unsafe { &mut *(builder as *mut Dltbc1EstimateOptionsBuilderImpl) };
    builder_impl.use_all_decorrelation_modes = use_all;
    Dltbc1Result::success()
}

/// Build the estimate options and determine optimal transform settings for BC1 data (ABI-stable).
///
/// This function uses the configured options to determine the optimal
/// transform settings for the given BC1 data and stores them in the provided context.
/// The builder remains valid after this call and can be reused.
///
/// # Parameters
/// - `builder`: The configured builder (can be null to use default settings)
/// - `data`: Pointer to BC1 data to analyze
/// - `data_len`: Length of data in bytes (must be divisible by 8)
/// - `estimator`: The size estimator to use for compression estimation
/// - `context`: The transform context where optimal settings will be stored
///
/// # Returns
/// A [`Dltbc1Result`] indicating success or containing an error.
///
/// # Safety
/// - `data` must be valid for reads of `data_len` bytes
/// - `estimator` must be a valid pointer to a [`DltSizeEstimator`] with valid function pointers
/// - `context` must be a valid pointer to a [`Dltbc1TransformContext`]
/// - The estimator's context and functions must remain valid for the duration of the call
/// - The builder (if not null) must be a valid pointer to a [`Dltbc1EstimateOptionsBuilder`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
    builder: *mut Dltbc1EstimateOptionsBuilder,
    data: *const u8,
    data_len: usize,
    estimator: *const DltSizeEstimator,
    context: *mut Dltbc1TransformContext,
) -> Dltbc1Result {
    // Validate context
    if context.is_null() {
        return Dltbc1Result::from_error_code(Dltbc1ErrorCode::NullTransformContextPointer);
    }

    // Get settings from builder without freeing it
    let use_all_modes = if builder.is_null() {
        false // Default value if builder is null
    } else {
        let builder_impl = unsafe { &*(builder as *const Dltbc1EstimateOptionsBuilderImpl) };
        builder_impl.use_all_decorrelation_modes
    };

    // Create settings struct
    let settings = Dltbc1DetermineOptimalSettings { use_all_modes };

    // Allocate space for the optimal details
    let mut optimal_details = Dltbc1TransformDetails::default();

    // Call the unstable determine optimal function
    let result = unsafe {
        dltbc1_unstable_determine_optimal(
            data,
            data_len,
            estimator,
            settings,
            &mut optimal_details as *mut Dltbc1TransformDetails,
        )
    };

    // If successful, update the context
    if result.is_success() {
        // Update the context with optimal settings
        let inner = unsafe { get_context_mut(context) };

        // Convert from internal variant to API decorrelation mode variant
        let api_variant = YCoCgVariant::from_internal_variant(
            optimal_details.decorrelation_mode.to_internal_variant(),
        );

        inner.builder = Bc1TransformOptionsBuilder::new()
            .decorrelation_mode(api_variant)
            .split_colour_endpoints(optimal_details.split_colour_endpoints);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::c_api::transform::transform_context::*;
    use core::ffi::c_void;
    use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;

    /// Create a test size estimator that returns predictable results
    fn create_test_estimator() -> DltSizeEstimator {
        // Simple estimator functions that return predictable results
        unsafe extern "C" fn test_max_compressed_size(
            _context: *mut c_void,
            _len_bytes: usize,
            out_size: *mut usize,
        ) -> u32 {
            unsafe {
                *out_size = 0;
            } // No buffer needed
            0 // Success
        }

        unsafe extern "C" fn test_estimate_compressed_size(
            _context: *mut c_void,
            _input_ptr: *const u8,
            len_bytes: usize,
            _data_type: u8,
            _output_ptr: *mut u8,
            _output_len: usize,
            out_size: *mut usize,
        ) -> u32 {
            unsafe {
                *out_size = len_bytes / 2;
            } // Return half the input size as estimate
            0 // Success
        }

        DltSizeEstimator {
            context: core::ptr::null_mut(),
            max_compressed_size: test_max_compressed_size,
            estimate_compressed_size: test_estimate_compressed_size,
            supports_data_type_differentiation: false,
        }
    }

    #[test]
    fn test_builder_creation_and_destruction() {
        // Create builder
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        assert!(!builder.is_null());

        // Free builder
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(builder);
        }

        // Test freeing null builder (should not crash)
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(core::ptr::null_mut());
        }
    }

    #[test]
    fn test_builder_use_all_decorrelation_modes() {
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        assert!(!builder.is_null());

        // Test setting use_all_decorrelation_modes to true
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(builder, true);
            assert!(result.is_success());
        }

        // Test setting use_all_decorrelation_modes to false
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(builder, false);
            assert!(result.is_success());
        }

        // Test with null builder (should return error)
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(
                core::ptr::null_mut(),
                true,
            );
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::NullBuilderPointer);
        }

        // Clean up
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(builder);
        }
    }

    #[test]
    fn test_determine_optimal_success() {
        // Create test data (8 bytes = 1 BC1 block)
        let test_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];

        // Create builder
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        assert!(!builder.is_null());

        // Create context
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        // Create test estimator
        let estimator = create_test_estimator();

        // Determine optimal transform
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                &estimator,
                context,
            );
            assert!(
                result.is_success(),
                "Expected success, got error code: {:?}",
                result.error_code
            );
        }

        // Clean up builder and context
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(builder);
            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_determine_optimal_null_context() {
        let test_data = [0u8; 8];
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        let estimator = create_test_estimator();

        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                &estimator,
                core::ptr::null_mut(),
            );
            assert!(!result.is_success());
            assert_eq!(
                result.error_code,
                Dltbc1ErrorCode::NullTransformContextPointer
            );
        }

        // Clean up builder
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(builder);
        }
    }

    #[test]
    fn test_determine_optimal_null_builder() {
        let test_data = [0u8; 8];
        let context = dltbc1_new_TransformContext();
        let estimator = create_test_estimator();

        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
                core::ptr::null_mut(),
                test_data.as_ptr(),
                test_data.len(),
                &estimator,
                context,
            );
            // Should work with null builder (uses default settings)
            assert!(result.is_success());

            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_determine_optimal_with_use_all_modes() {
        let test_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        let context = dltbc1_new_TransformContext();
        let estimator = create_test_estimator();

        // Set use_all_decorrelation_modes to true
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(builder, true);
            assert!(result.is_success());

            let result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                &estimator,
                context,
            );
            assert!(result.is_success());

            dltbc1_free_EstimateOptionsBuilder(builder);
            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_determine_optimal_invalid_data_length() {
        // Use 7 bytes (not divisible by 8)
        let test_data = [0u8; 7];
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        let context = dltbc1_new_TransformContext();
        let estimator = create_test_estimator();

        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                &estimator,
                context,
            );
            // Should fail with invalid length
            assert!(!result.is_success());
            assert_eq!(result.error_code, Dltbc1ErrorCode::InvalidLength);

            dltbc1_free_EstimateOptionsBuilder(builder);
            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_c_example_determine_optimal() {
        // Your BC1 texture data (8 bytes per BC1 block)
        let bc1_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];

        // Create builder and set options
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        assert!(!builder.is_null());

        unsafe {
            // Configure for more thorough optimization
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(builder, true);
            assert!(result.is_success());
        }

        // Create context to receive optimal settings
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        // Create estimator
        let estimator = create_test_estimator();

        // Determine optimal settings
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
                builder,
                bc1_data.as_ptr(),
                bc1_data.len(),
                &estimator,
                context,
            );

            if result.is_success() {
                println!("Optimal settings determined successfully!");
                // Context now contains the optimal transform settings
                // Use it for transform operations...
            } else {
                panic!(
                    "Failed to determine optimal settings: {:?}",
                    result.error_code
                );
            }
        }

        // Clean up
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(builder);
            dltbc1_free_TransformContext(context);
        }
    }
}
