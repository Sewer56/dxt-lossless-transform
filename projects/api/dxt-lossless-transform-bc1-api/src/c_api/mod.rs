//! # C API (FFI) Documentation
//!
//! *Note: The C API is only available when the `c-exports` feature is enabled.*
//!
//! The `c-exports` feature enables C-compatible FFI exports for using this library from C, C++, or other languages that support C FFI. The C API provides two categories of functions with different trade-offs.
//!
//! ## Example Usage
//!
//! **⚠️ Disclaimer: The following C examples are AI-generated and have not been tested by humans. They are provided for reference only and may require modification for actual use.**
//!
//! **📝 Note: The transform operation should be performed *before* compression, and untransform should be performed *after* decompression.**
//!
//! ### Basic Transform Operation
//!
//! ```c
//! #include <stdio.h>
//! #include <stdlib.h>
//!
//! // Your BC1 texture data (8 bytes per BC1 block)
//! uint8_t bc1_data[] = {0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0};
//! uint8_t transformed_data[8];
//!
//! // Create and configure manual transform builder
//! Dltbc1ManualTransformBuilder* builder = dltbc1_new_ManualTransformBuilder();
//! dltbc1_ManualTransformBuilder_decorrelation_mode(builder, 1); // Variant1
//! dltbc1_ManualTransformBuilder_split_colour_endpoints(builder, true);
//!
//! // Transform the data
//! Dltbc1Error result = dltbc1_ManualTransformBuilder_build_and_transform(
//!     builder, bc1_data, transformed_data, sizeof(bc1_data));
//!
//! if (result == DLTBC1_SUCCESS) {
//!     printf("Transform successful!\n");
//!     // Now compress 'transformed_data' with your compressor...
//! } else {
//!     printf("Transform failed\n");
//! }
//!
//! // Clean up
//! dltbc1_free_ManualTransformBuilder(builder);
//! ```
//!
//! ### Untransform Operation (After Decompression)
//!
//! ```c
//! #include <stdio.h>
//! #include <stdlib.h>
//!
//! // Your transformed BC1 data (after decompression)
//! uint8_t transformed_data[] = {0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0};
//! uint8_t restored_data[8];
//!
//! // Create builder with SAME settings used for original transform
//! Dltbc1ManualTransformBuilder* builder = dltbc1_new_ManualTransformBuilder();
//! dltbc1_ManualTransformBuilder_decorrelation_mode(builder, 1); // Variant1
//! dltbc1_ManualTransformBuilder_split_colour_endpoints(builder, true);
//!
//! // Restore original BC1 data
//! Dltbc1Error result = dltbc1_ManualTransformBuilder_build_and_untransform(
//!     builder, transformed_data, restored_data, sizeof(transformed_data));
//!
//! if (result == DLTBC1_SUCCESS) {
//!     printf("Untransform successful!\n");
//!     // 'restored_data' now contains original BC1 data
//! } else {
//!     printf("Untransform failed\n");
//! }
//!
//! // Clean up
//! dltbc1_free_ManualTransformBuilder(builder);
//! ```
//!
//! ### Determine Best Transform Options (Automatic)
//!
//! ```c
//! #include <stdio.h>
//! #include <stdlib.h>
//!
//! // Your BC1 texture data
//! uint8_t bc1_data[] = {0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0};
//! uint8_t transformed_data[8];
//!
//! // Create manual transform builder to receive the selected settings
//! Dltbc1ManualTransformBuilder* settings_builder = dltbc1_new_ManualTransformBuilder();
//! // Create auto transform builder for optimization
//! Dltbc1AutoTransformBuilder* builder = dltbc1_new_AutoTransformBuilder();
//!
//! // Configure analysis (optional - false is default for faster analysis)
//! dltbc1_AutoTransformBuilder_use_all_decorrelation_modes(builder, false);
//!
//! // Analyze data and determine best transform options using ZStd estimator
//! Dltbc1Error result = dltbc1_AutoTransformBuilder_build_and_transform_with_zstd_estimator(
//!     builder, bc1_data, transformed_data, sizeof(bc1_data), 6, settings_builder);
//!
//! if (result == DLTBC1_SUCCESS) {
//!     printf("Transform with best settings successful!\n");
//!     // 'transformed_data' now contains the transformed data
//!     // 'settings_builder' contains the transform details for later untransform
//!     // Now compress 'transformed_data' with your compressor...
//! } else {
//!     printf("Analysis failed\n");
//! }
//!
//! // Clean up
//! dltbc1_free_AutoTransformBuilder(builder);
//! dltbc1_free_ManualTransformBuilder(settings_builder);
//! ```
//!
//! ## ABI-Stable Functions (Recommended)
//!
//! These functions use opaque contexts and builder patterns that maintain ABI stability across versions, making them safe for production use.
//!
//! ### Manual Transform Builder Functions
//!
//! The manual transform builder is an opaque object that stores BC1 transform configuration. Use these functions for safe, ABI-stable transform operations:
//!
//! - **`dltbc1_new_ManualTransformBuilder()`** - Create a new manual transform builder with default settings
//! - **`dltbc1_free_ManualTransformBuilder(builder)`** - Free a manual transform builder (required to avoid memory leaks)
//! - **`dltbc1_clone_ManualTransformBuilder(builder)`** - Create a copy of an existing builder
//!
//! ### Manual Builder Configuration Functions
//!
//! Configure the manual transform builder before performing operations:
//!
//! - **`dltbc1_ManualTransformBuilder_decorrelation_mode(builder, mode)`** - Set color decorrelation mode (YCoCg variants)
//! - **`dltbc1_ManualTransformBuilder_split_colour_endpoints(builder, split)`** - Enable/disable color endpoint splitting
//! - **`dltbc1_ManualTransformBuilder_reset(builder)`** - Reset builder to default values
//!
//! ### Transform Operations
//!
//! Perform the actual BC1 data transformation:
//!
//! - **`dltbc1_ManualTransformBuilder_build_and_transform(builder, input, output, input_len)`** - Transform BC1 data for better compression
//! - **`dltbc1_ManualTransformBuilder_build_and_untransform(builder, input, output, input_len)`** - Restore original BC1 data after decompression
//!
//! ### Automatic Transform Optimization Functions
//!
//! Analyze your data to determine the best transform settings automatically:
//!
//! - **`dltbc1_new_AutoTransformBuilder()`** - Create a builder for automatic optimization
//! - **`dltbc1_AutoTransformBuilder_use_all_decorrelation_modes(builder, use_all)`** - Configure analysis thoroughness  
//! - **`dltbc1_AutoTransformBuilder_build_and_transform_with_zstd_estimator(builder, input, output, input_len, compression_level, settings_builder)`** - Analyze data, determine best settings, and apply transformation in one operation
//! - **`dltbc1_free_AutoTransformBuilder(builder)`** - Free the builder
//!
//! ## ABI-Unstable Functions (Advanced Users)
//!
//! ⚠️ **For advanced users only**: Functions prefixed with `dltbc1_unstable_*` accept transform details directly. These provide maximum performance by avoiding builder overhead but may break between versions if structures change. **Production code should use the ABI-stable builder patterns above.**
//!
//! The unstable functions have been moved to the core crate and are available in `dxt_lossless_transform_bc1::c_api` when the `c_api` feature is enabled. They include:
//!
//! - **`dltbc1_unstable_transform(...)`** - Transform BC1 data with explicit settings (ABI-unstable)
//! - **`dltbc1_unstable_untransform(...)`** - Restore BC1 data with explicit settings (ABI-unstable)  
//! - **`dltbc1_unstable_transform_auto(...)`** - Analyze data, determine best settings, and apply transformation in one operation (ABI-unstable)
//!
//! See the core crate documentation for details and migration guidance.
//!
//! ## Error Handling
//!
//! All functions return `Dltbc1Error` which contains error codes:
//! - `DLTBC1_SUCCESS` (0) for success, non-zero for various error conditions
//! - Use error handling appropriate for your application
//!
//! For detailed documentation of all C API functions, see the [C API documentation](https://docs.rs/dxt-lossless-transform-bc1-api/latest/dxt_lossless_transform_bc1_api/c_api/index.html) (requires `c-exports` feature).

// Module declarations - mirrors the structure of the non-C API
pub mod error;
pub mod transform;

use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc1::{Bc1TransformSettings, Bc1UntransformSettings};

/// FFI-safe version of [`Bc1TransformSettings`] for C API.
///
/// This struct mirrors the internal [`Bc1TransformSettings`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc1TransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,
    /// Whether color endpoints are split.
    pub split_colour_endpoints: bool,
}

/// FFI-safe version of [`Bc1UntransformSettings`] for C API.
///
/// This struct mirrors the internal [`Bc1UntransformSettings`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc1UntransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,
    /// Whether color endpoints are split.
    pub split_colour_endpoints: bool,
}

impl Default for Dltbc1TransformSettings {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Default for Dltbc1UntransformSettings {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

// Conversion implementations
impl From<Bc1TransformSettings> for Dltbc1TransformSettings {
    fn from(details: Bc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc1TransformSettings> for Bc1TransformSettings {
    fn from(details: Dltbc1TransformSettings) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Bc1UntransformSettings> for Dltbc1UntransformSettings {
    fn from(details: Bc1UntransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc1UntransformSettings> for Bc1UntransformSettings {
    fn from(details: Dltbc1UntransformSettings) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}
