//! Test prelude for LTU (Lossless Transform Utils) size estimation tests.
//!
//! This module provides common types and functions used across LTU tests.

// Re-export main types for tests
pub use crate::{LosslessTransformUtilsError, LosslessTransformUtilsSizeEstimation};

// Re-export common trait for tests
pub use dxt_lossless_transform_api_common::estimate::SizeEstimationOperations;

// Re-export standard testing utilities
#[cfg(feature = "std")]
pub use std::{vec, vec::Vec};

#[cfg(not(feature = "std"))]
pub use alloc::{vec, vec::Vec};
