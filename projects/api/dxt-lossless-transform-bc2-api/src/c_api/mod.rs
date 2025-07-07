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
//! // Your BC2 texture data (16 bytes per BC2 block)
//! uint8_t bc2_data[] = {
//!     0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,  // Alpha data (8 bytes)
//!     0x12, 0x34,                                      // Color0 (RGB565)
//!     0x56, 0x78,                                      // Color1 (RGB565)
//!     0x9A, 0xBC, 0xDE, 0xF0                           // Color indices (4 bytes)
//! };
//! uint8_t transformed_data[16];
//!
//! // Create and configure manual transform builder
//! Dltbc2ManualTransformBuilder* builder = dltbc2_new_ManualTransformBuilder();
//! dltbc2_ManualTransformBuilder_decorrelation_mode(builder, 1); // Variant1
//! dltbc2_ManualTransformBuilder_split_colour_endpoints(builder, true);
//!
//! // Transform the data
//! Dltbc2Error result = dltbc2_ManualTransformBuilder_build_and_transform(
//!     builder, bc2_data, transformed_data, sizeof(bc2_data));
//!
//! if (result == DLTBC2_SUCCESS) {
//!     printf("Transform successful!\n");
//!     // Now compress 'transformed_data' with your compressor...
//! } else {
//!     printf("Transform failed\n");
//! }
//!
//! // Clean up
//! dltbc2_free_ManualTransformBuilder(builder);
//! ```
//!
//! ### Untransform Operation (After Decompression)
//!
//! ```c
//! #include <stdio.h>
//! #include <stdlib.h>
//!
//! // Your transformed BC2 data (after decompression)
//! uint8_t transformed_data[] = {
//!     0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,  // Alpha data (8 bytes)
//!     0x12, 0x34,                                      // Color0 (RGB565)
//!     0x56, 0x78,                                      // Color1 (RGB565)
//!     0x9A, 0xBC, 0xDE, 0xF0                           // Color indices (4 bytes)
//! };
//! uint8_t restored_data[16];
//!
//! // Create builder with SAME settings used for original transform
//! Dltbc2ManualTransformBuilder* builder = dltbc2_new_ManualTransformBuilder();
//! dltbc2_ManualTransformBuilder_decorrelation_mode(builder, 1); // Variant1
//! dltbc2_ManualTransformBuilder_split_colour_endpoints(builder, true);
//!
//! // Restore original BC2 data
//! Dltbc2Error result = dltbc2_ManualTransformBuilder_build_and_untransform(
//!     builder, transformed_data, restored_data, sizeof(transformed_data));
//!
//! if (result == DLTBC2_SUCCESS) {
//!     printf("Untransform successful!\n");
//!     // 'restored_data' now contains original BC2 data
//! } else {
//!     printf("Untransform failed\n");
//! }
//!
//! // Clean up
//! dltbc2_free_ManualTransformBuilder(builder);
//! ```
//!
//! ### Determine Best Transform Options (Automatic)
//!
//! ```c
//! #include <stdio.h>
//! #include <stdlib.h>
//!
//! // Your BC2 texture data
//! uint8_t bc2_data[] = {
//!     0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77,  // Alpha data (8 bytes)
//!     0x12, 0x34,                                      // Color0 (RGB565)
//!     0x56, 0x78,                                      // Color1 (RGB565)
//!     0x9A, 0xBC, 0xDE, 0xF0                           // Color indices (4 bytes)
//! };
//! uint8_t transformed_data[16];
//!
//! // Create manual transform builder to receive the selected settings
//! Dltbc2ManualTransformBuilder* settings_builder = dltbc2_new_ManualTransformBuilder();
//! // Create auto transform builder for optimization
//! Dltbc2AutoTransformBuilder* builder = dltbc2_new_AutoTransformBuilder();
//!
//! // Configure analysis (optional - false is default for faster analysis)
//! dltbc2_AutoTransformBuilder_use_all_decorrelation_modes(builder, false);
//!
//! // Analyze data and determine best transform options using ZStd estimator
//! Dltbc2Error result = dltbc2_AutoTransformBuilder_build_and_transform_with_zstd_estimator(
//!     builder, bc2_data, transformed_data, sizeof(bc2_data), 6, settings_builder);
//!
//! if (result == DLTBC2_SUCCESS) {
//!     printf("Transform with best settings successful!\n");
//!     // 'transformed_data' now contains the transformed data
//!     // 'settings_builder' contains the transform details for later untransform
//!     // Now compress 'transformed_data' with your compressor...
//! } else {
//!     printf("Analysis failed\n");
//! }
//!
//! // Clean up
//! dltbc2_free_AutoTransformBuilder(builder);
//! dltbc2_free_ManualTransformBuilder(settings_builder);
//! ```
//!
//! ## ABI-Stable Functions (Recommended)
//!
//! These functions use opaque contexts and builder patterns that maintain ABI stability across versions, making them safe for production use.
//!
//! ### Manual Transform Builder Functions
//!
//! The manual transform builder is an opaque object that stores BC2 transform configuration. Use these functions for safe, ABI-stable transform operations:
//!
//! - **`dltbc2_new_ManualTransformBuilder()`** - Create a new manual transform builder with default settings
//! - **`dltbc2_free_ManualTransformBuilder(builder)`** - Free a manual transform builder (required to avoid memory leaks)
//! - **`dltbc2_clone_ManualTransformBuilder(builder)`** - Create a copy of an existing builder
//!
//! ### Manual Builder Configuration Functions
//!
//! Configure the manual transform builder before performing operations:
//!
//! - **`dltbc2_ManualTransformBuilder_decorrelation_mode(builder, mode)`** - Set color decorrelation mode (YCoCg variants)
//! - **`dltbc2_ManualTransformBuilder_split_colour_endpoints(builder, split)`** - Enable/disable color endpoint splitting
//! - **`dltbc2_ManualTransformBuilder_reset(builder)`** - Reset builder to default values
//!
//! ### Transform Operations
//!
//! Perform the actual BC2 data transformation:
//!
//! - **`dltbc2_ManualTransformBuilder_build_and_transform(builder, input, output, input_len)`** - Transform BC2 data for better compression
//! - **`dltbc2_ManualTransformBuilder_build_and_untransform(builder, input, output, input_len)`** - Restore original BC2 data after decompression
//!
//! ### Automatic Transform Optimization Functions
//!
//! Analyze your data to determine the best transform settings automatically:
//!
//! - **`dltbc2_new_AutoTransformBuilder()`** - Create a builder for automatic optimization
//! - **`dltbc2_AutoTransformBuilder_use_all_decorrelation_modes(builder, use_all)`** - Configure analysis thoroughness  
//! - **`dltbc2_AutoTransformBuilder_build_and_transform_with_zstd_estimator(builder, input, output, input_len, compression_level, settings_builder)`** - Analyze data, determine best settings, and apply transformation in one operation
//! - **`dltbc2_free_AutoTransformBuilder(builder)`** - Free the builder
//!
//! ## ABI-Unstable Functions (Advanced Users)
//!
//! ⚠️ **For advanced users only**: Functions prefixed with `dltbc2_unstable_*` accept transform details directly. These provide maximum performance by avoiding builder overhead but may break between versions if structures change. **Production code should use the ABI-stable builder patterns above.**
//!
//! The unstable functions have been moved to the core crate and are available in `dxt_lossless_transform_bc2::c_api` when the `c_api` feature is enabled. They include:
//!
//! - **`dltbc2_unstable_transform(...)`** - Transform BC2 data with explicit settings (ABI-unstable)
//! - **`dltbc2_unstable_untransform(...)`** - Restore BC2 data with explicit settings (ABI-unstable)  
//! - **`dltbc2_unstable_transform_auto(...)`** - Analyze data, determine best settings, and apply transformation in one operation (ABI-unstable)
//!
//! See the core crate documentation for details and migration guidance.
//!
//! ## Error Handling
//!
//! All functions return `Dltbc2Error` which contains error codes:
//! - `DLTBC2_SUCCESS` (0) for success, non-zero for various error conditions
//! - Use error handling appropriate for your application
//!
//! For detailed documentation of all C API functions, see the [C API documentation](https://docs.rs/dxt-lossless-transform-bc2-api/latest/dxt_lossless_transform_bc2_api/c_api/index.html) (requires `c-exports` feature).

// Module declarations - mirrors the structure of the non-C API
pub mod error;
pub mod transform;

use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc2::{Bc2TransformSettings, Bc2UntransformSettings};

/// FFI-safe version of [`Bc2TransformSettings`] for C API.
///
/// This struct mirrors the internal [`Bc2TransformSettings`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc2TransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,
    /// Whether color endpoints are split.
    pub split_colour_endpoints: bool,
}

/// FFI-safe version of [`Bc2UntransformSettings`] for C API.
///
/// This struct mirrors the internal [`Bc2UntransformSettings`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc2UntransformSettings {
    /// The decorrelation mode that was used to decorrelate the colors.
    pub decorrelation_mode: YCoCgVariant,
    /// Whether color endpoints are split.
    pub split_colour_endpoints: bool,
}

impl Default for Dltbc2TransformSettings {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

impl Default for Dltbc2UntransformSettings {
    fn default() -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::Variant1,
            split_colour_endpoints: true,
        }
    }
}

// Conversion implementations
impl From<Bc2TransformSettings> for Dltbc2TransformSettings {
    fn from(details: Bc2TransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc2TransformSettings> for Bc2TransformSettings {
    fn from(details: Dltbc2TransformSettings) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Bc2UntransformSettings> for Dltbc2UntransformSettings {
    fn from(details: Bc2UntransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc2UntransformSettings> for Bc2UntransformSettings {
    fn from(details: Dltbc2UntransformSettings) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}
