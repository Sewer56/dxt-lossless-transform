#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// Module declarations
pub mod determine_optimal_transform;
pub mod determine_optimal_transform_options_builder;
pub mod error;
pub mod transform;
pub mod transform_options_builder;

#[cfg(feature = "c-exports")]
pub mod c_api;

// Re-export main functionality at crate root
pub use determine_optimal_transform::{
    determine_optimal_transform, determine_optimal_transform_with_options,
};
pub use determine_optimal_transform_options_builder::Bc1EstimateOptionsBuilder;
pub use error::Bc1Error;
pub use transform::{
    transform_bc1_allocating, transform_bc1_slice, untransform_bc1_allocating,
    untransform_bc1_slice,
};
pub use transform_options_builder::Bc1TransformOptionsBuilder;
