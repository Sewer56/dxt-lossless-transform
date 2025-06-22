//! BC1 transform operations and builders.
//!
//! This module provides comprehensive BC1 transform functionality with a focus
//! on ABI-stable builder patterns. For maximum compatibility and future-proofing,
//! **use the builder patterns** rather than direct transform functions.
//!
//! ## Recommended API (ABI-Stable)
//!
//! - [`Bc1EstimateSettingsBuilder`] - Configure automatic optimization settings
//! - [`Bc1TransformSettingsBuilder`] - Configure manual transform settings
//!
//! ## Advanced API (ABI-Unstable)
//!
//! For performance-critical scenarios where you can accept potential breaking changes,
//! see the [`unstable`] module.

// Builder modules (stable, recommended)
pub mod estimate_settings_builder;
pub mod transform_settings_builder;

// Utility functions (stable)
pub mod util;

// ABI-unstable functions (advanced users only)
pub mod unstable;

// Re-export the BUILDERS FIRST at the module level for prominence
pub use estimate_settings_builder::Bc1EstimateSettingsBuilder;
pub use transform_settings_builder::Bc1TransformSettingsBuilder;

// Re-export utility functions
pub use util::{Decoded4x4Block, decode_bc1_block, decode_bc1_block_from_slice};

// Re-export core types for convenience
pub use dxt_lossless_transform_bc1::{
    Bc1DetransformSettings, Bc1EstimateSettings, Bc1TransformSettings, DetermineBestTransformError,
};
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;

// Re-export unstable functions for backwards compatibility (but not prominently)
pub use unstable::{
    transform_bc1_auto, transform_bc1_with_settings, untransform_bc1_with_settings,
};
