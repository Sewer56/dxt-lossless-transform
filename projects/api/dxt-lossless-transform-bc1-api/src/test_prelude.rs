//! Common test imports and utilities for BC1 API tests
//!
//! This module provides a common prelude for test modules to avoid
//! duplicate imports across the codebase.
#![allow(unused_imports)]

// External crate declaration for no_std compatibility
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

// Re-export commonly used alloc types for tests
pub use alloc::{boxed::Box, format, string::String, vec, vec::Vec};

// Re-export std items for tests that need them
pub use std::{ffi::c_void, is_x86_feature_detected, ptr};

// External crates commonly used in API tests
#[cfg(feature = "c-exports")]
pub use dxt_lossless_transform_api_common::c_api::size_estimation::DltSizeEstimator;
pub use dxt_lossless_transform_api_common::reexports::color_565::YCoCgVariant;
