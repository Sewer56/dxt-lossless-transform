#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

//! Safe, high-level API for BC1 texture data lossless transforms.
//!
//! This crate provides a safe wrapper around the low-level BC1 transform operations,
//! with automatic validation, error handling, and convenient APIs for both in-place
//! and allocating transformations.
//!
//! # Examples
//!
//! Note: Using `vec` here is not recommended, as it zeroes the buffer, leaving
//! performance on the table. These examples use `vec` only to make it easier
//! to demonstrate the API.
//!
//! ## Basic Transform and Untransform
//!
//! ```ignore
//! use dxt_lossless_transform_bc1_api::{transform_bc1_slice, untransform_bc1_slice};
//! use dxt_lossless_transform_bc1::Bc1TransformDetails;
//!
//! let bc1_data = vec![0u8; 8 * 100]; // 100 BC1 blocks
//! let mut output = vec![0u8; bc1_data.len()];
//!
//! // Transform with default options
//! let options = Bc1TransformDetails::default();
//! transform_bc1_slice(&bc1_data, &mut output, options)?;
//!
//! // Untransform back
//! let mut restored = vec![0u8; bc1_data.len()];
//! untransform_bc1_slice(&output, &mut restored, options.into())?;
//! ```

// Module declarations
pub mod determine_optimal_transform;
pub mod error;
pub mod transform;
pub mod transform_options_builder;

// Re-export main functionality at crate root
pub use determine_optimal_transform::determine_optimal_transform;
pub use error::Bc1Error;
pub use transform::{
    transform_bc1_allocating, transform_bc1_slice, untransform_bc1_allocating,
    untransform_bc1_slice,
};
pub use transform_options_builder::Bc1TransformOptionsBuilder;
