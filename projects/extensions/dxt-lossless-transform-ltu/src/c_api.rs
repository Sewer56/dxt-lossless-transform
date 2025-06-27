//! C API for the Lossless Transform Utils (LTU) size estimator.
//!
//! # Required Headers
//!
//! When using this API from C/C++, you must include the common API header first:
//! ```c
//! #include "dxt-lossless-transform-api-common.h"
//! #include "dxt-lossless-transform-ltu.h"
//! ```
//!
//! This module provides C-compatible exports for using the LTU size estimator
//! from C, C++, or other languages that support C FFI.
//!
//! The LTU estimator provides fast size estimation based on LZ match analysis
//! and entropy calculation, offering a good balance between speed and accuracy.
//!
//! # Important: Texture-Specific Implementation
//!
//! **This estimator is specifically tuned for DXT/BC texture data and may not work well with generic data.**
//! The default parameters have been carefully calibrated for texture compression patterns, each
//! marked with a [`DataType`] value on entry to the API.
//!
//! **Using custom parameters via [`dltltu_new_size_estimator_with_params`] or [`dltltu_new_size_estimator_with_settings`]
//! is discouraged** unless you have conducted thorough testing with your specific data type and understand
//! the estimation model. The default settings via [`dltltu_new_size_estimator`] should be used in most cases.
//!
//! # Usage with Transform APIs (BC1, BC2, BC3, BC7, etc.)
//!
//! The LTU estimator implements the [`DltSizeEstimator`] interface and can be used
//! directly with BCX transform optimization functions like
//! [`dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal`].
//!
//! The LTU estimator is significantly faster than actual compression-based estimators
//! while still providing reasonable accuracy for optimization purposes.
//!
//! # Available Functions
//!
//! - [`dltltu_new_size_estimator`] - Create a new estimator with default settings
//! - [`dltltu_new_size_estimator_with_params`] - Create with custom multipliers
//! - [`dltltu_new_size_estimator_with_settings`] - Create with settings struct
//! - [`dltltu_free_size_estimator`] - Free an estimator
//!
//! [`dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal`]: https://docs.rs/dxt-lossless-transform-bc1-api/latest/dxt_lossless_transform_bc1_api/c_api/determine_optimal_transform/fn.dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal.html

use crate::{EstimationSettings, LosslessTransformUtilsSizeEstimation};
use alloc::boxed::Box;
use core::ffi::c_void;

// Note: DltSizeEstimator is defined in dxt-lossless-transform-api-common
// Users must include that header before this one
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};

/// Creates a new LTU size estimator with default settings.
///
/// The returned estimator uses the default estimation parameters which
/// are optimized for BC1 texture data.
///
/// # Returns
/// A pointer to a heap-allocated [`DltSizeEstimator`] configured to use the LTU implementation.
/// The caller must free this with [`dltltu_free_size_estimator`] when done.
///
/// # Safety
/// - The returned pointer must be freed with [`dltltu_free_size_estimator`]
/// - The returned pointer must not be used after it is freed
#[no_mangle]
pub unsafe extern "C" fn dltltu_new_size_estimator() -> *mut DltSizeEstimator {
    let ltu = Box::new(LosslessTransformUtilsSizeEstimation::new());
    let estimator = create_c_size_estimator(ltu);
    Box::into_raw(Box::new(estimator))
}

/// Creates a new LTU size estimator with custom parameters.
///
/// # Warning
/// **Using custom parameters is discouraged unless you have conducted thorough testing**
/// with your specific data type. The default estimator is tuned for DXT/BC texture data.
/// The profile is inherited from the [`DataType`] field passed in via the API.
/// Consider using [`dltltu_new_size_estimator`] instead.
///
/// # Parameters
/// - `lz_match_multiplier`: Multiplier for LZ match effectiveness (recommended: 0.5-0.7)
/// - `entropy_multiplier`: Multiplier for entropy coding efficiency (recommended: 1.0-1.3)
///
/// # Returns
/// A pointer to a heap-allocated [`DltSizeEstimator`] configured with custom parameters.
/// The caller must free this with [`dltltu_free_size_estimator`] when done.
///
/// # Safety
/// - The returned pointer must be freed with [`dltltu_free_size_estimator`]
/// - The returned pointer must not be used after it is freed
#[no_mangle]
pub unsafe extern "C" fn dltltu_new_size_estimator_with_params(
    lz_match_multiplier: f64,
    entropy_multiplier: f64,
) -> *mut DltSizeEstimator {
    let ltu = Box::new(LosslessTransformUtilsSizeEstimation::new_with_params(
        lz_match_multiplier,
        entropy_multiplier,
    ));
    let estimator = create_c_size_estimator(ltu);
    Box::into_raw(Box::new(estimator))
}

/// Creates a new LTU size estimator with custom settings structure.
///
/// # Warning
/// **Using custom settings is discouraged unless you have conducted thorough testing**
/// with your specific data type. The default estimator is tuned for DXT/BC texture data, profile
/// is inherited from the [`DataType`] field passed in via the API.
/// Consider using [`dltltu_new_size_estimator`] instead.
///
/// # Parameters
/// - `settings`: Pointer to estimation settings structure
///
/// # Returns
/// A pointer to a heap-allocated [`DltSizeEstimator`] configured with the provided settings.
/// Returns null if `settings` is null.
/// The caller must free this with [`dltltu_free_size_estimator`] when done.
///
/// # Safety
/// - `settings` must be a valid pointer to an [`EstimationSettings`] structure or null
/// - The returned pointer must be freed with [`dltltu_free_size_estimator`]
/// - The returned pointer must not be used after it is freed
#[no_mangle]
pub unsafe extern "C" fn dltltu_new_size_estimator_with_settings(
    settings: *const EstimationSettings,
) -> *mut DltSizeEstimator {
    if settings.is_null() {
        return core::ptr::null_mut();
    }

    let settings = unsafe { *settings };
    let ltu = Box::new(LosslessTransformUtilsSizeEstimation::new_with_settings(
        settings,
    ));
    let estimator = create_c_size_estimator(ltu);
    Box::into_raw(Box::new(estimator))
}

/// Frees an LTU size estimator.
///
/// # Parameters
/// - `estimator`: The estimator to free (can be null)
///
/// # Safety
/// - `estimator` must be a valid pointer returned by one of the `dltltu_new_*` functions or null
/// - The estimator must not be used after calling this function
#[no_mangle]
pub unsafe extern "C" fn dltltu_free_size_estimator(estimator: *mut DltSizeEstimator) {
    if !estimator.is_null() {
        // First free the boxed LTU implementation
        let estimator_ref = unsafe { &*estimator };
        if !estimator_ref.context.is_null() {
            let _ = unsafe {
                Box::from_raw(estimator_ref.context as *mut LosslessTransformUtilsSizeEstimation)
            };
        }
        // Then free the DltSizeEstimator itself
        let _ = unsafe { Box::from_raw(estimator) };
    }
}

/// C-compatible callback for [`DltSizeEstimator::max_compressed_size`].
unsafe extern "C" fn ltu_max_compressed_size(
    context: *mut c_void,
    len_bytes: usize,
    out_size: *mut usize,
) -> u32 {
    if context.is_null() || out_size.is_null() {
        return 1; // Error: null pointer
    }

    let ltu = unsafe { &*(context as *const LosslessTransformUtilsSizeEstimation) };

    match ltu.max_compressed_size(len_bytes) {
        Ok(size) => {
            unsafe { *out_size = size };
            0 // Success
        }
        Err(_) => 2, // Error: max_compressed_size failed
    }
}

/// C-compatible callback for [`DltSizeEstimator::estimate_compressed_size`].
unsafe extern "C" fn ltu_estimate_compressed_size(
    context: *mut c_void,
    input_ptr: *const u8,
    len_bytes: usize,
    data_type: u8,
    output_ptr: *mut u8,
    output_len: usize,
    out_size: *mut usize,
) -> u32 {
    if context.is_null() || out_size.is_null() {
        return 1; // Error: null pointer
    }

    let ltu = unsafe { &*(context as *const LosslessTransformUtilsSizeEstimation) };
    let data_type = match data_type {
        // 1:1 mapping at time of writing, so no-op after compilation
        0 => DataType::Unknown,
        1 => DataType::Bc1Colours,
        2 => DataType::Bc1DecorrelatedColours,
        3 => DataType::Bc1SplitColours,
        4 => DataType::Bc1SplitDecorrelatedColours,
        _ => DataType::Unknown,
    };

    match unsafe {
        ltu.estimate_compressed_size(input_ptr, len_bytes, data_type, output_ptr, output_len)
    } {
        Ok(size) => {
            unsafe { *out_size = size };
            0 // Success
        }
        Err(_) => 3, // Error: estimate_compressed_size failed
    }
}

/// Creates a C-compatible [`DltSizeEstimator`] from an LTU implementation.
fn create_c_size_estimator(ltu: Box<LosslessTransformUtilsSizeEstimation>) -> DltSizeEstimator {
    DltSizeEstimator {
        context: Box::into_raw(ltu) as *mut c_void,
        max_compressed_size: ltu_max_compressed_size,
        estimate_compressed_size: ltu_estimate_compressed_size,
        supports_data_type_differentiation: true,
    }
}

/// Example using LTU with BC1 transform API.
///
/// This example demonstrates how to use the LTU size estimator with the BC1
/// determine optimal transform function from a C perspective.
///
/// ```ignore
/// // Create an LTU size estimator
/// let ltu_estimator = unsafe { dltltu_new_size_estimator() };
///
/// // Use it with BC1 API
/// let bc1_settings_builder = unsafe { dltbc1_new_ManualTransformBuilder() };
/// let bc1_estimate_builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
///
/// // Your BC1 texture data (8 bytes per BC1 block)
/// let bc1_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
///
/// // Determine optimal settings using LTU for fast estimation
/// let result = unsafe {
///     dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal(
///         bc1_estimate_builder,
///         bc1_data.as_ptr(),
///         bc1_data.len(),
///         ltu_estimator, // Use LTU estimator here
///         bc1_settings_builder,
///     )
/// };
///
/// if result.is_success() {
///     println!("Optimal settings determined using LTU!");
///     // Settings builder now contains the optimal transform settings
///     // Use it for transform operations...
/// }
///
/// // Clean up
/// unsafe {
///     dltltu_free_size_estimator(ltu_estimator);
///     dltbc1_free_EstimateOptionsBuilder(bc1_estimate_builder);
///     dltbc1_free_ManualTransformBuilder(bc1_settings_builder);
/// }
/// ```
#[cfg(test)]
mod tests {
    use super::*;
    use dxt_lossless_transform_api_common::estimate::DataType;

    #[test]
    fn test_create_and_free_default_estimator() {
        unsafe {
            let estimator = dltltu_new_size_estimator();
            assert!(!estimator.is_null());
            dltltu_free_size_estimator(estimator);
        }
    }

    #[test]
    fn test_create_and_free_with_params() {
        unsafe {
            let estimator = dltltu_new_size_estimator_with_params(0.6, 1.2);
            assert!(!estimator.is_null());
            dltltu_free_size_estimator(estimator);
        }
    }

    #[test]
    fn test_create_and_free_with_settings() {
        let settings = EstimationSettings {
            lz_match_multiplier: 0.6,
            entropy_multiplier: 1.2,
        };

        unsafe {
            let estimator = dltltu_new_size_estimator_with_settings(&settings);
            assert!(!estimator.is_null());
            dltltu_free_size_estimator(estimator);
        }
    }

    #[test]
    fn test_null_settings_returns_null() {
        unsafe {
            let estimator = dltltu_new_size_estimator_with_settings(core::ptr::null());
            assert!(estimator.is_null());
        }
    }

    #[test]
    fn test_free_null_estimator() {
        // Should not crash
        unsafe {
            dltltu_free_size_estimator(core::ptr::null_mut());
        }
    }

    #[test]
    fn test_estimator_functionality() {
        unsafe {
            let estimator = dltltu_new_size_estimator();
            assert!(!estimator.is_null());

            let estimator_ref = &*estimator;

            // Test max_compressed_size
            let mut max_size = 0;
            let result =
                (estimator_ref.max_compressed_size)(estimator_ref.context, 1024, &mut max_size);
            assert_eq!(result, 0); // Success
            assert_eq!(max_size, 0); // LTU returns 0 for max_compressed_size

            // Test estimate_compressed_size
            let test_data = [0u8; 64];
            let mut estimated_size = 0;
            let result = (estimator_ref.estimate_compressed_size)(
                estimator_ref.context,
                test_data.as_ptr(),
                test_data.len(),
                DataType::Bc1Colours as u8,
                core::ptr::null_mut(),
                0,
                &mut estimated_size,
            );
            assert_eq!(result, 0); // Success
            assert!(estimated_size > 0);
            assert!(estimated_size < test_data.len());

            dltltu_free_size_estimator(estimator);
        }
    }
}
