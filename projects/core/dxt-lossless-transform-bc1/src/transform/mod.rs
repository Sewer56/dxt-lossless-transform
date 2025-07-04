//! BC1 Transform Operations
//!
//! This module provides the core transformation functionality for BC1 (DXT1) compressed
//! texture data to achieve optimal compression ratios.
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
//! - **~1018MiB/s** overall throughput with `lossless-transform-utils` estimator  
//! - Additional memory usage: compression buffer needed by estimator (depends on the estimator)
//!
//! The automatic functions optimize further for size at the expense of speed.
//! As a general rule of thumb, use `lossless-transform-utils` for zstd levels 1-3,
//! and `dxt-lossless-transform-zstd` level 1 for zstd level 4 and above.

// Module structure
pub(crate) mod settings;
pub(crate) mod transform_auto;
pub(crate) mod transform_with_settings;

// Transform module implementations
pub(crate) mod standard;
pub(crate) mod with_recorrelate;
pub(crate) mod with_split_colour;
pub(crate) mod with_split_colour_and_recorr;

// Safe slice-based wrapper functions
pub mod safe;

// Re-export all public items from submodules
pub use settings::*;
pub use transform_auto::*;
pub use transform_with_settings::*;

// Re-export safe module functions
pub use safe::{
    transform_bc1_auto_safe, transform_bc1_with_settings_safe, untransform_bc1_with_settings_safe,
    Bc1AutoTransformError, Bc1ValidationError,
};

#[cfg(test)]
mod tests {
    use super::transform_auto::{
        transform_bc1_auto, Bc1EstimateSettings, DetermineBestTransformError,
    };
    use crate::test_prelude::*;
    use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

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

        // Use the dummy estimator from test_prelude
        let transform_options = Bc1EstimateSettings {
            size_estimator: DummyEstimator,
            use_all_decorrelation_modes: false,
        };

        // This should not crash and should produce transformed data
        let result = unsafe {
            transform_bc1_auto(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
                bc1_data.len(),
                &transform_options,
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
                _output_ptr: *mut u8,
                _output_len: usize,
            ) -> Result<usize, Self::Error> {
                Err("Estimation failed")
            }
        }

        let transform_options = Bc1EstimateSettings {
            size_estimator: FailingEstimator,
            use_all_decorrelation_modes: false,
        };

        let result = unsafe {
            transform_bc1_auto(
                bc1_data.as_ptr(),
                output_buffer.as_mut_ptr(),
                bc1_data.len(),
                &transform_options,
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
