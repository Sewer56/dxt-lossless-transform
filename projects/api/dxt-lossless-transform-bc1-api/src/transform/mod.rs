//! BC1 transform operations and builders.
//!
//! This module provides comprehensive BC1 transform functionality including:
//! - Automatic transform optimization
//! - Explicit transform and untransform operations
//! - Builder patterns for configuration
//! - Utility functions

// Individual modules for different functionality
pub mod auto_transform;
pub mod detransform_settings_builder;
pub mod estimate_options_builder;
pub mod transform_settings_builder;
pub mod transform_with_settings;
pub mod untransform_with_settings;
pub mod util;

// Re-export the main functions at the module level for convenience
pub use auto_transform::transform_bc1_auto;
pub use detransform_settings_builder::Bc1DetransformSettingsBuilder;
pub use estimate_options_builder::Bc1EstimateOptionsBuilder;
pub use transform_settings_builder::Bc1TransformSettingsBuilder;
pub use transform_with_settings::transform_bc1_with_settings;
pub use untransform_with_settings::untransform_bc1_with_settings;

// Re-export utility functions
pub use util::{Decoded4x4Block, decode_bc1_block, decode_bc1_block_from_slice};

// Re-export core types for convenience
pub use dxt_lossless_transform_bc1::{
    Bc1DetransformSettings, Bc1EstimateOptions, Bc1TransformSettings, DetermineBestTransformError,
};
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;
