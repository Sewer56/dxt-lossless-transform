//! C API for transforming BC1 data with automatic optimization.
//!
//! This module provides ABI-stable builder pattern for transforming BC1 data with
//! automatically determined optimal settings. The builder pattern ensures ABI stability
//! while internally using the unstable functions.
//!
//! [`YCoCgVariant::Variant1`]: dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::Variant1
//! [`YCoCgVariant::None`]: dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant::None
//! [`DltSizeEstimator`]: dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator
//! [`Dltbc1TransformContext`]: crate::c_api::transform::transform_context::Dltbc1TransformContext

pub mod transform_context;

#[cfg(test)]
mod tests {
    use crate::c_api::{
        auto_transform::{
            dltbc1_EstimateOptionsBuilder_BuildAndTransform,
            dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes,
            dltbc1_free_EstimateOptionsBuilder, dltbc1_new_EstimateOptionsBuilder,
        },
        error::Dltbc1ErrorCode,
        transform::transform_context::*,
    };
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
    fn test_transform_auto_success() {
        // Create test data (8 bytes = 1 BC1 block)
        let test_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut output = [0u8; 8];

        // Create builder
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        assert!(!builder.is_null());

        // Create context
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        // Create test estimator
        let estimator = create_test_estimator();

        // Transform with automatic optimization
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndTransform(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
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
    fn test_transform_auto_null_context() {
        let test_data = [0u8; 8];
        let mut output = [0u8; 8];
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        let estimator = create_test_estimator();

        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndTransform(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
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
    fn test_transform_auto_null_builder() {
        let test_data = [0u8; 8];
        let mut output = [0u8; 8];
        let context = dltbc1_new_TransformContext();
        let estimator = create_test_estimator();

        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndTransform(
                core::ptr::null_mut(),
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                context,
            );
            // Should work with null builder (uses default settings)
            assert!(result.is_success());

            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_transform_auto_with_use_all_modes() {
        let test_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut output = [0u8; 8];
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        let context = dltbc1_new_TransformContext();
        let estimator = create_test_estimator();

        // Set use_all_decorrelation_modes to true
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(builder, true);
            assert!(result.is_success());

            let result = dltbc1_EstimateOptionsBuilder_BuildAndTransform(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                context,
            );
            assert!(result.is_success());

            dltbc1_free_EstimateOptionsBuilder(builder);
            dltbc1_free_TransformContext(context);
        }
    }

    #[test]
    fn test_transform_auto_invalid_data_length() {
        // Use 7 bytes (not divisible by 8)
        let test_data = [0u8; 7];
        let mut output = [0u8; 8];
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        let context = dltbc1_new_TransformContext();
        let estimator = create_test_estimator();

        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndTransform(
                builder,
                test_data.as_ptr(),
                test_data.len(),
                output.as_mut_ptr(),
                output.len(),
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
    fn test_c_example_transform_auto() {
        // Your BC1 texture data (8 bytes per BC1 block)
        let bc1_data = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
        let mut output = [0u8; 8];

        // Create builder and set options
        let builder = unsafe { dltbc1_new_EstimateOptionsBuilder() };
        assert!(!builder.is_null());

        unsafe {
            // Configure for more thorough optimization
            let result = dltbc1_EstimateOptionsBuilder_SetUseAllDecorrelationModes(builder, true);
            assert!(result.is_success());
        }

        // Create context to receive transform details
        let context = dltbc1_new_TransformContext();
        assert!(!context.is_null());

        // Create estimator
        let estimator = create_test_estimator();

        // Transform with automatic optimization
        unsafe {
            let result = dltbc1_EstimateOptionsBuilder_BuildAndTransform(
                builder,
                bc1_data.as_ptr(),
                bc1_data.len(),
                output.as_mut_ptr(),
                output.len(),
                &estimator,
                context,
            );

            if result.is_success() {
                println!("Transform completed successfully!");
                // Context now contains the transform details
                // Output buffer contains the transformed data
            } else {
                panic!("Failed to transform data: {:?}", result.error_code);
            }
        }

        // Clean up
        unsafe {
            dltbc1_free_EstimateOptionsBuilder(builder);
            dltbc1_free_TransformContext(context);
        }
    }
}
