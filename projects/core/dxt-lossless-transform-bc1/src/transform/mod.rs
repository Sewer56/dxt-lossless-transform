//! BC1 Transform Operations
//!
//! This module provides the core transformation and optimization functionality for BC1
//! (DXT1) compressed texture data to achieve optimal compression ratios.
//!
//! ## Overview
//!
//! BC1 compression can be further optimized by applying various transformations before
//! final compression. This module provides both manual transform operations and automatic
//! optimization to determine the best transformation parameters.
//!
//! ## Performance Characteristics
//!
//! This module provides two categories of functions with **very different performance characteristics**:
//!
//! ### Manual Transform Functions (High Speed)
//!
//! Functions like [`transform_bc1_with_settings`] and [`untransform_bc1_with_settings`] that use
//! predetermined settings achieve:
//! - **~24GB/s** transformation speed on single thread (Ryzen 9950X3D)
//! - Minimal memory overhead
//! - Optimal for production use when settings are known
//!
//! ### Automatic Optimization Functions (Slower but Convenient)  
//!
//! Functions like [`transform_bc1_auto`] perform brute force testing of different transformations:
//!
//! 1. Transform the data into multiple different formats
//! 2. Estimate the compressed size using a provided file size estimator function  
//! 3. Compare the estimated sizes to find the best transformation
//!
//! **Performance is bottlenecked by the estimator speed (single thread, Ryzen 9950X3D):**
//! - **~265MiB/s** overall throughput with `dxt-lossless-transform-zstd` estimator (level 1)
//! - **~641MiB/s** overall throughput with `lossless-transform-utils` estimator  
//! - Additional memory usage: compression buffer needed by estimator (depends on the estimator)
//!
//! The automatic functions optimize further for size at the expense of speed.
//! As a general rule of thumb, if you're compressing zstd level 3 or below, go manual.
//! If you're compressing zstd level 4 to 9, use `lossless-transform-utils` instead.
//! Otherwise for levels 10 and above, use `dxt-lossless-transform-zstd` instead.

// Module structure
pub(crate) mod operations;
pub(crate) mod optimization;
pub(crate) mod settings;

// Transform module implementations
pub(crate) mod standard;
pub(crate) mod with_recorrelate;
pub(crate) mod with_split_colour;
pub(crate) mod with_split_colour_and_recorr;

// Re-export all public items from submodules
pub use operations::*;
pub use optimization::*;
pub use settings::*;

#[cfg(test)]
mod tests {
    use super::optimization::{
        transform_bc1_auto, Bc1EstimateOptions, DetermineBestTransformError,
    };
    use crate::test_prelude::*;
    use dxt_lossless_transform_api_common::estimate::{DataType, SizeEstimationOperations};

    /// Test that transform_bc1_auto works correctly
    #[rstest]
    fn test_transform_bc1_auto() {
        // Create minimal BC1 block data (8 bytes per block)
        // This is a simple red block
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output_buffer = [0u8; 8];

        // Create a simple dummy estimator
        struct DummyEstimator;

        impl SizeEstimationOperations for DummyEstimator {
            type Error = &'static str;

            fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
                Ok(0) // No buffer needed for dummy estimator
            }

            unsafe fn estimate_compressed_size(
                &self,
                _input_ptr: *const u8,
                len_bytes: usize,
                _data_type: DataType,
                _output_ptr: *mut u8,
                _output_len: usize,
            ) -> Result<usize, Self::Error> {
                Ok(len_bytes) // Just return the input length
            }
        }

        let transform_options = Bc1EstimateOptions {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        // This should not crash and should produce transformed data
        let result = unsafe {
            transform_bc1_auto(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
                bc1_data.len(),
                transform_options,
            )
        };

        assert!(
            result.is_ok(),
            "Function should not crash with valid BC1 data"
        );
    }

    /// Test that transform_bc1_auto handles estimation errors properly
    #[rstest]
    fn test_transform_bc1_auto_handles_errors() {
        // Create minimal BC1 block data
        let bc1_data = [
            0x00, 0xF8, // Color0: Red in RGB565 (0xF800)
            0x00, 0x00, // Color1: Black (0x0000)
            0x00, 0x00, 0x00, 0x00, // Indices: all pointing to Color0
        ];
        let mut output_buffer = [0u8; 8];

        // Create an estimator that always fails
        struct FailingEstimator;

        impl SizeEstimationOperations for FailingEstimator {
            type Error = &'static str;

            fn max_compressed_size(&self, _len_bytes: usize) -> Result<usize, Self::Error> {
                Err("Estimation failed")
            }

            unsafe fn estimate_compressed_size(
                &self,
                _input_ptr: *const u8,
                _len_bytes: usize,
                _data_type: DataType,
                _output_ptr: *mut u8,
                _output_len: usize,
            ) -> Result<usize, Self::Error> {
                Err("Estimation failed")
            }
        }

        let transform_options = Bc1EstimateOptions {
            size_estimator: FailingEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = unsafe {
            transform_bc1_auto(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
                bc1_data.len(),
                transform_options,
            )
        };

        // Should return an error
        assert!(
            result.is_err(),
            "Function should return error when estimator fails"
        );

        // Check that it's specifically a SizeEstimationError
        if let Err(e) = result {
            match e {
                DetermineBestTransformError::SizeEstimationError(_) => {
                    // This is what we expect
                }
                _ => panic!("Expected SizeEstimationError, got {e:?}"),
            }
        }
    }
}
