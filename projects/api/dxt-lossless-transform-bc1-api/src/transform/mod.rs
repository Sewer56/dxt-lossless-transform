//! BC1 Transform API
//!
//! This module provides high-level builders for BC1 texture transformation:
//!
//! ## Automatic Optimization
//! - [`Bc1AutoTransformBuilder`] - Automatically optimizes transform settings using a size estimator
//!
//! ## Manual Configuration  
//! - [`Bc1ManualTransformBuilder`] - Allows precise control over transform parameters
//!
//! ## Low-level Functions
//! For advanced use cases, the module also provides direct access to the underlying transform functions.

pub(crate) mod auto_transform_builder;
pub(crate) mod manual_transform_builder;

// Re-export the BUILDERS FIRST at the module level for prominence
pub use auto_transform_builder::Bc1AutoTransformBuilder;
pub use manual_transform_builder::Bc1ManualTransformBuilder;

// Re-export core types for convenience
pub use dxt_lossless_transform_bc1::{
    Bc1DetransformSettings, Bc1EstimateSettings, Bc1TransformSettings, DetermineBestTransformError,
};
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;
