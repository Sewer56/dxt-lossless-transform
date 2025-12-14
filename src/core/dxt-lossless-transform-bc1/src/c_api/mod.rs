//! C API functions for BC1 transforms.
//!
//! These functions provide C-compatible FFI exports for maximum performance scenarios.
//! They were moved from the API layer to the core to reduce dependencies
//! and improve the architecture.

pub mod transform_auto;
pub mod transform_with_settings;
