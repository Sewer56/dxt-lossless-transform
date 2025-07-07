//! BC2 Transform API
//!
//! This module provides high-level builders for BC2 texture transformation:
//!
//! ## Automatic Optimization
//! - [`Bc2AutoTransformBuilder`] - Automatically finds the best transform settings by testing different configurations and choosing the one that results in the smallest estimated compressed size
//!
//! ## Manual Configuration  
//! - [`Bc2ManualTransformBuilder`] - Allows precise control over transform parameters
//!
//! ## Clean API Design
//! The API uses builders that provide a clean interface while using internal types from the core crate directly.
//!
//! ## Block Size Note
//! BC2 blocks are 16 bytes (8 bytes alpha + 8 bytes color data), unlike BC1 which uses 8 bytes per block.

pub(crate) mod auto_transform_builder;
pub(crate) mod manual_transform_builder;

// Re-export the builders
pub use auto_transform_builder::Bc2AutoTransformBuilder;
pub use manual_transform_builder::Bc2ManualTransformBuilder;

// Re-export stable API types for configuration
pub use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
