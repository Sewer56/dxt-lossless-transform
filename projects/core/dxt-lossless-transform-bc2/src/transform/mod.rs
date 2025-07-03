//! BC2 Transform Operations
//!
//! This module provides the core transformation functionality for BC2 (DXT3) compressed
//! texture data to achieve optimal compression ratios.
//!
//! ## Overview
//!
//! BC2 compression can be further optimized by applying various transformations before
//! final compression. This module provides both manual transform operations and automatic
//! optimization to determine the best transformation parameters.
//!
//! ## Performance Characteristics
//!
//! This module provides two categories of functions with **very different performance characteristics**:
//!
//! ### Manual Transform Functions (High Speed)
//!
//! Functions like [`transform_bc2_with_settings`] and [`untransform_bc2_with_settings`] that use
//! predetermined settings achieve:
//! - **~24GB/s** transformation speed on single thread (Ryzen 9950X3D)
//! - Minimal memory overhead
//! - Optimal for production use when settings are known
//!
//! ### Automatic Optimization Functions (Slower but Convenient)  
//!
//! Functions like [`transform_bc2_auto`] perform brute force testing of different transformations:
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
mod standard;
mod with_recorrelate;
mod with_split_colour;
mod with_split_colour_and_recorr;

// Safe slice-based wrapper functions
pub mod safe;

// Re-export all public items from submodules
pub use settings::*;
pub use transform_auto::*;
pub use transform_with_settings::*;

// Re-export safe module functions
pub use safe::{
    transform_bc2_auto_safe, transform_bc2_with_settings_safe, untransform_bc2_with_settings_safe,
    Bc2AutoTransformError, Bc2ValidationError,
};
