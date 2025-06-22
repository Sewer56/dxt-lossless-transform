#![doc = include_str!("../README.MD")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// Module declarations
pub mod error;
pub mod transform;

#[cfg(feature = "c-exports")]
pub mod c_api;

// Re-export main functionality at crate root
pub use error::Bc1Error;
pub use transform::builder::Bc1EstimateOptionsBuilder;
pub use transform::builder::Bc1TransformOptionsBuilder;
pub use transform::transform_bc1_auto;

// Test utilities
#[cfg(test)]
mod test_prelude;
