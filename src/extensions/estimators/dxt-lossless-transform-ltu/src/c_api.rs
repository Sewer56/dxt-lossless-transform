//! C API for LTU (Lossless Transform Utils) Size Estimation
//!
//! This module provides a C-compatible interface for the LTU size estimation functionality.
//! It exposes the core [`LosslessTransformUtilsSizeEstimation`] type and related operations
//! through C function pointers and structures.
//!
//! ## Usage Pattern
//!
//! 1. Create an estimator instance using [`dltltu_new_size_estimator`]
//! 2. Use the estimator with any API that accepts [`SizeEstimationOperations`]
//! 3. Free the estimator when done using [`dltltu_free_size_estimator`]
//!
//! ## Important Notes
//!
//! This estimator is designed for relative comparison between different transforms
//! of the same data. The absolute values returned are not meaningful - only the
//! relative ordering matters for determining which transform compresses better.
//!
//! ## Thread Safety
//!
//! The LTU estimator is thread-safe and can be used from multiple threads simultaneously.
//! The estimator has no internal state, making it safe for concurrent use.
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
//! The LTU estimator provides fast size estimation based on LZ match analysis,
//! offering a good balance between speed and accuracy for relative comparisons.
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
//! - [`dltltu_new_size_estimator`] - Create a new estimator
//! - [`dltltu_free_size_estimator`] - Free an estimator
//!
//! [`dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal`]: https://docs.rs/dxt-lossless-transform-bc1-api/latest/dxt_lossless_transform_bc1_api/c_api/determine_optimal_transform/fn.dltbc1_EstimateOptionsBuilder_BuildAndDetermineOptimal.html

use crate::LosslessTransformUtilsSizeEstimation;
use alloc::boxed::Box;
use core::ffi::c_void;
use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

/// Create a new LTU size estimator.
///
/// The estimator uses LZ match analysis for fast relative size comparison.
///
/// # Returns
///
/// Pointer to the created estimator, or null on allocation failure
///
/// # Safety
///
/// This function allocates memory and returns a raw pointer. The caller must
/// ensure that [`dltltu_free_size_estimator`] is called to properly deallocate
/// the memory when the estimator is no longer needed.
#[no_mangle]
pub unsafe extern "C" fn dltltu_new_size_estimator() -> *mut DltSizeEstimator {
    let ltu = Box::new(LosslessTransformUtilsSizeEstimation::new());
    let estimator = create_c_size_estimator(ltu);
    Box::into_raw(Box::new(estimator))
}

/// Free an LTU size estimator created by [`dltltu_new_size_estimator`].
///
/// # Parameters
/// * `estimator` - Pointer to the estimator to free (can be null)
///
/// # Safety
/// The estimator pointer must have been returned by [`dltltu_new_size_estimator`],
/// or be null. After calling this function, the pointer becomes invalid.
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
    output_ptr: *mut u8,
    output_len: usize,
    out_size: *mut usize,
) -> u32 {
    if context.is_null() || out_size.is_null() {
        return 1; // Error: null pointer
    }

    let ltu = unsafe { &*(context as *const LosslessTransformUtilsSizeEstimation) };
    match unsafe { ltu.estimate_compressed_size(input_ptr, len_bytes, output_ptr, output_len) } {
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
/// let bc1_data = [
///     0x12, 0x34,                                      // Color0 (RGB565)
///     0x56, 0x78,                                      // Color1 (RGB565)
///     0x9A, 0xBC, 0xDE, 0xF0                           // Color indices (4 bytes)
/// ];
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

    #[test]
    fn test_create_and_free_estimator() {
        unsafe {
            let estimator = dltltu_new_size_estimator();
            assert!(!estimator.is_null());
            dltltu_free_size_estimator(estimator);
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
                core::ptr::null_mut(),
                0,
                &mut estimated_size,
            );
            assert_eq!(result, 0); // Success
            assert!(estimated_size < test_data.len()); // Should be smaller for repetitive data

            dltltu_free_size_estimator(estimator);
        }
    }
}
