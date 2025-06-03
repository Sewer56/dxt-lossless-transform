//! Common test imports and utilities for BC1 tests
//!
//! This module provides a common prelude for test modules to avoid
//! duplicate imports across the codebase.

// External crates commonly used in tests
pub use rstest::rstest;

// Core functionality from this crate
pub use crate::{transform_bc1, Bc1TransformDetails};

// Experimental features commonly tested
pub use crate::experimental::normalize_blocks::*;

// Test utilities from transform module
pub(crate) use crate::transforms::standard::transform::tests::{
    assert_implementation_matches_reference, generate_bc1_test_data,
    transform_with_reference_implementation,
};

// Common types from dxt_lossless_transform_common
pub use dxt_lossless_transform_common::allocate::allocate_align_64;
pub use dxt_lossless_transform_common::color_565::YCoCgVariant;
pub use dxt_lossless_transform_common::color_8888::Color8888;
pub use dxt_lossless_transform_common::cpu_detect::*;
pub use dxt_lossless_transform_common::decoded_4x4_block::Decoded4x4Block;

// Standard library imports commonly used in tests
pub use core::ptr::{copy_nonoverlapping, write_bytes};
pub use safe_allocator_api::RawAlloc;

// Common untransform functions that are frequently tested
pub use crate::with_split_colour_and_recorr::untransform_with_split_colour_and_recorr;

// Re-export super for convenience in test modules
pub use super::*;
