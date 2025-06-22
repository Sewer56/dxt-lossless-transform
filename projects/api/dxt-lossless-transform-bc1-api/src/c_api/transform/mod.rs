//! C API bindings for BC1 transform operations.
//!
//! This module provides C-compatible FFI exports for BC1 transform functionality with a focus
//! on ABI stability and ease of use from C/C++ code.
//!
//! ## Builder Modules (ABI-Stable)
//!
//! - [`auto_transform_builder`] - Builder pattern for automatic optimization settings
//! - [`manual_transform_builder`] - Builder pattern for manual transform configuration
//!
//! ## Unstable Functions (ABI-Unstable)
//!
//! For advanced users requiring maximum performance, see the [`unstable`] module.
//! These functions may have breaking changes between versions without major version bumps.
//!
//! **Production code should use the ABI-stable builder patterns above.**

// Builder modules (stable, recommended)
pub mod auto_transform_builder;
pub mod manual_transform_builder;

// Unstable direct functions (advanced users only)
pub mod unstable;
