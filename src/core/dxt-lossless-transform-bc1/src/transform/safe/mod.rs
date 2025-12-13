//! Safe slice-based BC1 transform wrappers
//!
//! This module provides safe, slice-based wrapper functions around the unsafe
//! pointer-based core transform functions. These functions include input validation
//! and proper error handling.
//!
//! This module is intended for:
//! - Performance-focused users who want safe wrappers
//! - Library implementers building their own stable APIs
//! - Advanced users who can handle API changes between versions
//!
//! Note: For production use with ABI stability, prefer the
//! `dxt-lossless-transform-bc1-api` crate.

pub mod transform_auto;
pub mod transform_with_settings;

// Re-export the main functions with _safe suffix for discoverability
pub use transform_auto::{transform_bc1_auto as transform_bc1_auto_safe, Bc1AutoTransformError};
pub use transform_with_settings::{
    transform_bc1_with_settings as transform_bc1_with_settings_safe,
    untransform_bc1_with_settings as untransform_bc1_with_settings_safe, Bc1ValidationError,
};
