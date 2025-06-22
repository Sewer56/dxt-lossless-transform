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
//! // Create and configure transform settings builder
//! Dltbc1TransformSettingsBuilder* builder = dltbc1_new_TransformSettingsBuilder();
//! dltbc1_TransformSettingsBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
//! dltbc1_TransformSettingsBuilder_SetSplitColourEndpoints(builder, true);
//!
//! // Transform the data
//! Dltbc1Result result = dltbc1_TransformSettingsBuilder_Transform(
//!     bc1_data, sizeof(bc1_data),
//!     transformed_data, sizeof(transformed_data),
//!     builder);
//!
//! if (result.error_code == 0) {
//!     printf("Transform successful!\n");
//!     // Now compress 'transformed_data' with your compressor...
//! } else {
//!     printf("Transform failed: %s\n", dltbc1_error_message(result.error_code));
//! }
//!
//! // Clean up
//! dltbc1_free_TransformSettingsBuilder(builder);
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
//! Dltbc1TransformSettingsBuilder* builder = dltbc1_new_TransformSettingsBuilder();
//! dltbc1_TransformSettingsBuilder_SetDecorrelationMode(builder, YCOCG_VARIANT_1);
//! dltbc1_TransformSettingsBuilder_SetSplitColourEndpoints(builder, true);
//!
//! // Restore original BC1 data
//! Dltbc1Result result = dltbc1_TransformSettingsBuilder_Untransform(
//!     transformed_data, sizeof(transformed_data),
//!     restored_data, sizeof(restored_data),
//!     builder);
//!
//! if (result.error_code == 0) {
//!     printf("Untransform successful!\n");
//!     // 'restored_data' now contains original BC1 data
//! } else {
//!     printf("Untransform failed: %s\n", dltbc1_error_message(result.error_code));
//! }
//!
//! // Clean up
//! dltbc1_free_TransformSettingsBuilder(builder);
//! ```
//!
//! ### Determine Best Transform Options
//!
//! ```c
//! #include <stdio.h>
//! #include <stdlib.h>
//!
//! // Your BC1 texture data
//! uint8_t bc1_data[] = {0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0};
//! uint8_t transformed_data[8];
//!
//! // Create transform settings builder and options builder
//! Dltbc1TransformSettingsBuilder* settings_builder = dltbc1_new_TransformSettingsBuilder();
//! Dltbc1EstimateSettingsBuilder* builder = dltbc1_new_EstimateSettingsBuilder();
//!
//! // Configure analysis (optional - false is default for faster analysis)
//! dltbc1_EstimateSettingsBuilder_SetUseAllDecorrelationModes(builder, false);
//!
//! // Your size estimator (implementation depends on your compression library)
//! DltSizeEstimator estimator = {
//!     .context = your_compressor_context,
//!     .max_compressed_size = your_max_size_function,
//!     .estimate_compressed_size = your_estimate_function,
//!     .supports_data_type_differentiation = false
//! };
//!
//! // Analyze data and determine best transform options
//! Dltbc1Result result = dltbc1_EstimateSettingsBuilder_BuildAndTransform(
//!     builder, bc1_data, sizeof(bc1_data), transformed_data, sizeof(transformed_data), &estimator, settings_builder);
//!
//! if (result.error_code == 0) {
//!     printf("Transform with best settings successful!\n");
//!     // 'transformed_data' now contains the transformed data
//!     // 'settings_builder' contains the transform details for later untransform
//!     // Now compress 'transformed_data' with your compressor...
//! } else {
//!     printf("Analysis failed: %s\n", dltbc1_error_message(result.error_code));
//! }
//!
//! // Clean up
//! dltbc1_free_EstimateSettingsBuilder(builder);
//! dltbc1_free_TransformSettingsBuilder(settings_builder);
//! ```
//!
//! ## ABI-Stable Functions (Recommended)
//!
//! These functions use opaque contexts and builder patterns that maintain ABI stability across versions, making them safe for production use.
//!
//! ### Transform Settings Builder Functions
//!
//! The transform settings builder is an opaque object that stores BC1 transform configuration. Use these functions for safe, ABI-stable transform operations:
//!
//! - **`dltbc1_new_TransformSettingsBuilder()`** - Create a new transform settings builder with default settings
//! - **`dltbc1_free_TransformSettingsBuilder(builder)`** - Free a transform settings builder (required to avoid memory leaks)
//! - **`dltbc1_clone_TransformSettingsBuilder(builder)`** - Create a copy of an existing builder
//!
//! ### Settings Builder Configuration Functions
//!
//! Configure the transform settings builder before performing operations:
//!
//! - **`dltbc1_TransformSettingsBuilder_SetDecorrelationMode(builder, mode)`** - Set color decorrelation mode (YCoCg variants)
//! - **`dltbc1_TransformSettingsBuilder_SetSplitColourEndpoints(builder, split)`** - Enable/disable color endpoint splitting
//! - **`dltbc1_TransformSettingsBuilder_ResetToDefaults(builder)`** - Reset builder to default values
//!
//! ### Transform Operations
//!
//! Perform the actual BC1 data transformation:
//!
//! - **`dltbc1_TransformSettingsBuilder_Transform(input, input_len, output, output_len, builder)`** - Transform BC1 data for better compression
//! - **`dltbc1_TransformSettingsBuilder_Untransform(input, input_len, output, output_len, builder)`** - Restore original BC1 data after decompression
//!
//! ### Transform Options Analysis Functions
//!
//! Analyze your data to determine the best transform settings:
//!
//! - **`dltbc1_new_EstimateSettingsBuilder()`** - Create a builder for analysis settings
//! - **`dltbc1_EstimateSettingsBuilder_SetUseAllDecorrelationModes(builder, use_all)`** - Configure analysis thoroughness
//! - **`dltbc1_EstimateSettingsBuilder_BuildAndTransform(builder, data, data_len, output, output_len, estimator, settings_builder)`** - Analyze data, determine best settings, and apply transformation in one operation
//! - **`dltbc1_free_EstimateSettingsBuilder(builder)`** - Free the builder
//!
//! ## ABI-Unstable Functions
//!
//! Functions prefixed with `dltbc1_unstable_*` that accept transform details directly. These provide maximum performance by avoiding context overhead but may break between versions if structures change.
//!
//! ### Direct Transform Operations
//!
//! - **`dltbc1_unstable_transform(input, input_len, output, output_len, details)`** - Transform BC1 data with explicit settings
//! - **`dltbc1_unstable_untransform(input, input_len, output, output_len, details)`** - Restore BC1 data with explicit settings
//!
//! ### Direct Options Analysis
//!
//! - **`dltbc1_unstable_transform_auto(data, data_len, output, output_len, estimator, settings, out_details)`** - Analyze data, determine best settings, and apply transformation in one operation
//!
//! ## Error Handling
//!
//! All functions return `Dltbc1Result` which contains:
//! - `error_code` - 0 for success, non-zero for various error conditions
//! - Use `dltbc1_error_message(error_code)` to get human-readable error descriptions
//!
//! For detailed documentation of all C API functions, see the [C API documentation](https://docs.rs/dxt-lossless-transform-bc1-api/latest/dxt_lossless_transform_bc1_api/c_api/index.html) (requires `c-exports` feature).

// Module declarations - mirrors the structure of the non-C API
pub mod error;
pub mod transform;

use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
use dxt_lossless_transform_bc1::{Bc1DetransformSettings, Bc1TransformSettings};

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

/// FFI-safe version of [`Bc1DetransformSettings`] for C API.
///
/// This struct mirrors the internal [`Bc1DetransformSettings`] but is guaranteed
/// to have stable ABI layout for C interoperability.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Dltbc1DetransformSettings {
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

impl Default for Dltbc1DetransformSettings {
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

impl From<Bc1DetransformSettings> for Dltbc1DetransformSettings {
    fn from(details: Bc1DetransformSettings) -> Self {
        Self {
            decorrelation_mode: YCoCgVariant::from_internal_variant(details.decorrelation_mode),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}

impl From<Dltbc1DetransformSettings> for Bc1DetransformSettings {
    fn from(details: Dltbc1DetransformSettings) -> Self {
        Self {
            decorrelation_mode: details.decorrelation_mode.to_internal_variant(),
            split_colour_endpoints: details.split_colour_endpoints,
        }
    }
}
