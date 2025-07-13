//! BC3 Transform Operations
//!
//! This module provides the core transformation functionality for BC3 (DXT5) compressed
//! texture data to achieve optimal compression ratios.
//!
//! ## Overview
//!
//! BC3 compression can be further optimized by applying various transformations before
//! final compression. This module provides both manual transform operations and automatic
//! optimization to determine the best transformation parameters.
//!
//! ## Performance Characteristics
//!
//! This module provides two categories of functions with **very different performance characteristics**:
//!
//! ### Manual Transform Functions (High Speed)
//!
//! Functions like [`transform_bc3_with_settings`] and [`untransform_bc3_with_settings`] that use
//! predetermined settings achieve:
//! - **High-speed** transformation on single thread
//! - Minimal memory overhead
//! - Optimal for production use when settings are known
//!
//! ### Automatic Optimization Functions (Slower but Convenient)  
//!
//! Functions like [`transform_bc3_auto`] perform brute force testing of different transformations:
//!
//! 1. Transform the data into multiple different formats
//! 2. Estimate the compressed size using a provided file size estimator function  
//! 3. Compare the estimated sizes to find the best transformation
//!
//! **Performance is bottlenecked by the estimator speed:**
//! - Additional memory usage: compression buffer needed by estimator (depends on the estimator)
//!
//! The automatic functions optimize further for size at the expense of speed.

// Module structure
pub(crate) mod settings;
pub(crate) mod transform_auto;
pub(crate) mod transform_with_settings;

// Transform module implementations - standard public to crate for bench access
pub(crate) mod standard;
pub(crate) mod with_recorrelate;
pub(crate) mod with_split_alphas;
pub(crate) mod with_split_alphas_and_colour;
pub(crate) mod with_split_alphas_and_recorr;
pub(crate) mod with_split_alphas_colour_and_recorr;
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
    transform_bc3_auto_safe, transform_bc3_with_settings_safe, untransform_bc3_with_settings_safe,
    Bc3AutoTransformError, Bc3ValidationError,
};
